// SPDX-License-Identifier: Apache-2.0

//! Readings for health warnings (docs/01-product/health-warnings.md) that no other collector
//! takes. [`Health`] reads files only, every minute; [`HealthSlow`] runs a few programs with a
//! time limit, every 5 minutes (rule facts stay fresh for 10).

use std::collections::{HashMap, VecDeque};
use std::io;
use std::time::Duration;

use deecure_core::{Timestamp, Unit};

use super::{Collector, Sample, SystemRoot, metric, read_trimmed};
use crate::inventory::{command, security_detail};

/// Kernel taint bits: `D` (a kernel crash: oops or BUG) and `M` (a machine-check error).
const TAINT_DIED: u64 = 1 << 7;
const TAINT_MACHINE_CHECK: u64 = 1 << 4;
/// Drivers that only show a picture, without graphics acceleration.
const NO_ACCELERATION: &[&str] = &["simpledrm", "simple-framebuffer", "efifb", "vesafb"];
/// Filesystems on a local disk that should be writable when `/etc/fstab` says so.
const LOCAL_FS: &[&str] = &[
    "ext4", "ext3", "ext2", "btrfs", "xfs", "f2fs", "vfat", "exfat",
];
/// Wi-Fi under this is weak (dBm).
const WEAK_WIFI: f64 = -75.0;
/// Network error share counts only with at least this many packets in the window.
const MIN_PACKETS: f64 = 1000.0;
const NET_WINDOW: usize = 10;

/// Readings from files, every minute.
#[derive(Debug, Default)]
pub struct Health {
    weak_wifi: f64,
    /// The last 10 readings of (errors, packets) per network interface.
    net: HashMap<String, VecDeque<(f64, f64)>>,
}

/// The sum of `TOTAL_ERR_*` lines in a PCIe AER counter file.
pub fn aer_total(text: &str) -> f64 {
    text.lines()
        .filter(|l| l.starts_with("TOTAL_ERR"))
        .filter_map(|l| l.split_whitespace().nth(1)?.parse::<f64>().ok())
        .sum()
}

/// Mounts that are read-only now although `/etc/fstab` mounts them writable.
pub fn unexpected_read_only(mounts: &str, fstab: &str) -> Vec<String> {
    let writable: Vec<&str> = fstab
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .filter_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            let (mount, options) = (*f.get(1)?, f.get(3).copied().unwrap_or("defaults"));
            (!options.split(',').any(|o| o == "ro")).then_some(mount)
        })
        .collect();
    mounts
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            let (mount, kind, options) = (*f.get(1)?, *f.get(2)?, *f.get(3)?);
            (LOCAL_FS.contains(&kind)
                && options.split(',').next() == Some("ro")
                && writable.contains(&mount))
            .then(|| mount.to_owned())
        })
        .collect()
}

/// The Wi-Fi signal in dBm from `/proc/net/wireless`.
pub fn wifi_dbm(text: &str) -> Option<f64> {
    text.lines().skip(2).find_map(|l| {
        l.split_whitespace()
            .nth(3)?
            .trim_end_matches('.')
            .parse()
            .ok()
    })
}

fn number(root: &SystemRoot, path: &str) -> Option<f64> {
    read_trimmed(&root.path(path))?.parse().ok()
}

fn names(root: &SystemRoot, dir: &str) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(root.path(dir))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

impl Collector for Health {
    fn name(&self) -> &'static str {
        "health"
    }

    fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let mut sample = Sample::default();
        let mut push = |name: &str, subject: Option<&str>, value: f64| {
            sample
                .metrics
                .push(metric(name, subject, value, Unit::Count, now));
        };
        let flag = |on: bool| f64::from(u8::from(on));

        if let Some(taint) = number(root, "/proc/sys/kernel/tainted").map(|t| t as u64) {
            push("kernel.died", None, flag(taint & TAINT_DIED != 0));
            push(
                "kernel.machine_check",
                None,
                flag(taint & TAINT_MACHINE_CHECK != 0),
            );
        }

        let pci = names(root, "/sys/bus/pci/devices");
        let aer = |file: &str| -> Option<f64> {
            let found: Vec<f64> = pci
                .iter()
                .filter_map(|d| root.read(&format!("/sys/bus/pci/devices/{d}/{file}")).ok())
                .map(|t| aer_total(&t))
                .collect();
            (!found.is_empty()).then(|| found.iter().sum())
        };
        for (file, name) in [
            ("aer_dev_fatal", "pcie.errors_fatal"),
            ("aer_dev_nonfatal", "pcie.errors_nonfatal"),
            ("aer_dev_correctable", "pcie.errors_corrected"),
        ] {
            if let Some(n) = aer(file) {
                push(name, None, n);
            }
        }

        let mcs: Vec<String> = names(root, "/sys/devices/system/edac/mc")
            .into_iter()
            .filter(|m| m.starts_with("mc"))
            .collect();
        if !mcs.is_empty() {
            let sum = |f: &str| -> f64 {
                mcs.iter()
                    .filter_map(|m| number(root, &format!("/sys/devices/system/edac/mc/{m}/{f}")))
                    .sum()
            };
            push("memory.errors_corrected", None, sum("ce_count"));
            push("memory.errors_uncorrected", None, sum("ue_count"));
        }

        // Filesystem errors (ext4 keeps a count) and disks that turned read-only.
        let mounts = root.read("/proc/mounts").unwrap_or_default();
        for line in mounts.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            let (Some(dev), Some(mount), Some(&"ext4")) = (f.first(), f.get(1), f.get(2)) else {
                continue;
            };
            let real = std::fs::canonicalize(root.path(dev)).ok();
            let Some(name) = real
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
            else {
                continue;
            };
            if let Some(n) = number(root, &format!("/sys/fs/ext4/{name}/errors_count")) {
                push("disk.fs_errors", Some(mount), n);
            }
        }
        let read_only = unexpected_read_only(&mounts, &root.read("/etc/fstab").unwrap_or_default());
        for line in mounts.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            if let (Some(mount), Some(kind)) = (f.get(1), f.get(2))
                && LOCAL_FS.contains(kind)
            {
                push(
                    "disk.read_only",
                    Some(mount),
                    flag(read_only.iter().any(|m| m == mount)),
                );
            }
        }

        let cards: Vec<String> = names(root, "/sys/class/drm")
            .into_iter()
            .filter(|c| c.starts_with("card") && !c.contains('-'))
            .collect();
        if !cards.is_empty() {
            let accelerated = cards.iter().any(|c| {
                std::fs::canonicalize(root.path(&format!("/sys/class/drm/{c}/device/driver")))
                    .ok()
                    .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                    .is_some_and(|d| !NO_ACCELERATION.contains(&d.as_str()))
            });
            push("graphics.accelerated", None, flag(accelerated));
        }

        push(
            "security.apparmor",
            None,
            flag(
                read_trimmed(&root.path("/sys/module/apparmor/parameters/enabled")).as_deref()
                    == Some("Y"),
            ),
        );
        if root.path("/sys/firmware/efi").exists() {
            let var = root
                .path("/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c");
            if let Ok(bytes) = std::fs::read(var) {
                push("security.secure_boot", None, flag(bytes.last() == Some(&1)));
            }
        }
        let flaws = names(root, "/sys/devices/system/cpu/vulnerabilities");
        if !flaws.is_empty() {
            let open = flaws
                .iter()
                .filter(|f| {
                    read_trimmed(
                        &root.path(&format!("/sys/devices/system/cpu/vulnerabilities/{f}")),
                    )
                    .is_some_and(|t| t.starts_with("Vulnerable"))
                })
                .count();
            push("cpu.flaws_vulnerable", None, open as f64);
        }

        if let Some(dbm) = root
            .read("/proc/net/wireless")
            .ok()
            .and_then(|t| wifi_dbm(&t))
        {
            push("wifi.signal_dbm", None, dbm);
            self.weak_wifi = if dbm < WEAK_WIFI {
                self.weak_wifi + 1.0
            } else {
                0.0
            };
            push("wifi.weak_minutes", None, self.weak_wifi);
        } else {
            self.weak_wifi = 0.0;
        }

        // Physical interfaces only (they have a device); the share over the last 10 minutes.
        let ifaces: Vec<String> = names(root, "/sys/class/net")
            .into_iter()
            .filter(|i| root.path(&format!("/sys/class/net/{i}/device")).exists())
            .collect();
        self.net.retain(|k, _| ifaces.contains(k));
        for i in &ifaces {
            let stat = |f: &str| {
                number(root, &format!("/sys/class/net/{i}/statistics/{f}")).unwrap_or(0.0)
            };
            let reading = (
                stat("rx_errors") + stat("tx_errors"),
                stat("rx_packets") + stat("tx_packets"),
            );
            let ring = self.net.entry(i.clone()).or_default();
            ring.push_back(reading);
            if ring.len() > NET_WINDOW + 1 {
                ring.pop_front();
            }
            if ring.len() == NET_WINDOW + 1
                && let (Some(first), Some(last)) = (ring.front(), ring.back())
            {
                let (errors, packets) = (last.0 - first.0, last.1 - first.1);
                if packets >= MIN_PACKETS && errors >= 0.0 {
                    push("net.error_percent", Some(i), errors * 100.0 / packets);
                }
            }
        }
        Ok(sample)
    }
}

/// Days from 1970-01-01 to a `YYYY-MM-DD` date (the civil calendar).
pub fn days_from_date(date: &str) -> Option<i64> {
    let mut p = date.get(..10)?.split('-');
    let (y, m, d): (i64, i64, i64) = (
        p.next()?.parse().ok()?,
        p.next()?.parse().ok()?,
        p.next()?.parse().ok()?,
    );
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

/// Readings that run a program, every 5 minutes, each with a time limit.
#[derive(Debug, Default)]
pub struct HealthSlow {
    unsynced: f64,
    limited: f64,
}

impl HealthSlow {
    /// The name used in logs.
    pub const NAME: &'static str = "health (slow)";
    /// It runs every this many collector runs (minutes).
    pub const EVERY_NTH_RUN: u64 = 5;
    const LIMIT: Duration = Duration::from_secs(10);

    /// Time sync, half-installed packages, internet reachability, and automatic updates.
    pub async fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let mut sample = Sample::default();
        let mut push = |name: &str, value: f64| {
            sample
                .metrics
                .push(metric(name, None, value, Unit::Count, now));
        };
        let minutes = Self::EVERY_NTH_RUN as f64;

        if let Some(out) = command::output(
            "timedatectl",
            &["show", "-p", "NTP", "-p", "NTPSynchronized"],
            Self::LIMIT,
        )
        .await
        {
            let get = |k: &str| {
                out.lines()
                    .find_map(|l| l.strip_prefix(k))
                    .map(|v| v == "yes")
            };
            let (ntp, synced) = (
                get("NTP=").unwrap_or(false),
                get("NTPSynchronized=").unwrap_or(false),
            );
            self.unsynced = if synced { 0.0 } else { self.unsynced + minutes };
            push("time.ntp", f64::from(u8::from(ntp)));
            push("time.synchronized", f64::from(u8::from(synced)));
            push("time.unsynced_minutes", self.unsynced);
        }

        // It exits non-zero when it finds problems; each package is an indented line.
        if let Some(out) = command::output_any("dpkg", &["--audit"], Self::LIMIT).await {
            let broken = out
                .lines()
                .filter(|l| l.starts_with(' ') && !l.trim().is_empty())
                .count();
            push("packages.broken", broken as f64);
        }

        if let Some(state) =
            command::output("nmcli", &["networking", "connectivity"], Self::LIMIT).await
        {
            // "limited": connected, but the internet does not answer. A captive portal
            // ("portal") is a sign-in page, not a fault.
            self.limited = if state.trim() == "limited" {
                self.limited + minutes
            } else {
                0.0
            };
            push("net.limited_minutes", self.limited);
        }

        let detail = security_detail::read(root);
        let enabled = detail.unattended_installed
            && detail
                .periodic
                .iter()
                .any(|(k, v)| k == "Unattended-Upgrade" && v != "0");
        push("updates.auto_enabled", f64::from(u8::from(enabled)));
        if let Some(runs) = detail.upgrade_runs.filter(|r| !r.is_empty()) {
            let ok = |r: &security_detail::UpgradeRun| {
                r.message.contains("All upgrades installed")
                    || r.message.contains("No packages found")
            };
            let today = now.as_millis().div_euclid(86_400_000);
            let last_ok = runs
                .iter()
                .find(|r| ok(r))
                .and_then(|r| days_from_date(&r.at));
            push(
                "updates.auto_last_failed",
                f64::from(u8::from(!ok(&runs[0]))),
            );
            // Never worked in the log: count from the oldest run there.
            let since = last_ok.or_else(|| runs.last().and_then(|r| days_from_date(&r.at)));
            if let Some(day) = since {
                push("updates.auto_last_ok_days", (today - day).max(0) as f64);
            }
        }
        Ok(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::{fixture_root, value};

    #[test]
    fn reads_kernel_pcie_memory_and_security_flags() {
        let root = fixture_root(
            "health",
            &[
                ("/proc/sys/kernel/tainted", "4224\n"),
                (
                    "/sys/bus/pci/devices/0000:00:1c.0/aer_dev_correctable",
                    "RxErr 3\nBadTLP 1\nTOTAL_ERR_COR 4\n",
                ),
                (
                    "/sys/bus/pci/devices/0000:00:1c.0/aer_dev_fatal",
                    "TOTAL_ERR_FATAL 0\n",
                ),
                ("/sys/devices/system/edac/mc/mc0/ce_count", "2\n"),
                ("/sys/devices/system/edac/mc/mc0/ue_count", "0\n"),
                ("/sys/module/apparmor/parameters/enabled", "Y\n"),
                (
                    "/sys/devices/system/cpu/vulnerabilities/spectre_v1",
                    "Mitigation: usercopy\n",
                ),
                (
                    "/sys/devices/system/cpu/vulnerabilities/gds",
                    "Vulnerable: No microcode\n",
                ),
                (
                    "/proc/net/wireless",
                    "Inter-| sta-|   Quality        |\n face | tus | link level noise |\nwlp45s0: 0000   40.  -80.  -256        0      0      0      0     12        0\n",
                ),
            ],
        );
        let mut h = Health::default();
        let s = h.collect(&root, Timestamp(0)).unwrap();
        // 4224 = 4096 + 128: the kernel died once; no machine check.
        assert_eq!(value(&s, "kernel.died", None), Some(1.0));
        assert_eq!(value(&s, "kernel.machine_check", None), Some(0.0));
        assert_eq!(value(&s, "pcie.errors_corrected", None), Some(4.0));
        assert_eq!(value(&s, "pcie.errors_fatal", None), Some(0.0));
        assert_eq!(value(&s, "memory.errors_corrected", None), Some(2.0));
        assert_eq!(value(&s, "security.apparmor", None), Some(1.0));
        assert_eq!(value(&s, "cpu.flaws_vulnerable", None), Some(1.0));
        assert_eq!(value(&s, "wifi.signal_dbm", None), Some(-80.0));
        let s = h.collect(&root, Timestamp(0)).unwrap();
        assert_eq!(value(&s, "wifi.weak_minutes", None), Some(2.0));
    }

    #[test]
    fn a_disk_read_only_against_fstab_is_found() {
        let mounts = "/dev/nvme0n1p2 / ext4 ro,relatime 0 0\n/dev/nvme0n1p1 /boot/efi vfat rw 0 0\n/dev/loop1 /snap/core22/1 squashfs ro 0 0\n/dev/sdb1 /mnt/old ext4 ro 0 0\n";
        let fstab = "UUID=1 / ext4 errors=remount-ro 0 1\nUUID=2 /boot/efi vfat umask=0077 0 1\nUUID=3 /mnt/old ext4 ro 0 2\n";
        assert_eq!(unexpected_read_only(mounts, fstab), ["/"]);
    }

    #[test]
    fn counters_and_dates_parse() {
        assert_eq!(aer_total("A 1\nTOTAL_ERR_NONFATAL 7\n"), 7.0);
        assert_eq!(days_from_date("1970-01-02 00:00:00"), Some(1));
        assert_eq!(days_from_date("2026-10-24"), Some(20_750));
        assert_eq!(wifi_dbm("x\ny\n"), None);
    }
}
