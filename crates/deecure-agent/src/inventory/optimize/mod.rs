// SPDX-License-Identifier: Apache-2.0

//! The Optimize page (docs/01-product/optimize-catalog.md): every improvement a measurement on
//! this computer shows is needed, and nothing "just in case". Each check says what it measured,
//! also when nothing is needed, so the page can show what was checked.
//!
//! A fix opens the existing fix preview (`action` and `subject`); advice is a measurement and what
//! to do, for things software cannot change.

use std::time::Duration;

use deecure_core::Timestamp;
use serde::Serialize;

use super::storage_detail;
mod cleanup;
mod drivers;
mod history;

use super::{
    command, cpu_detail, kernel, memory_detail, pci_detail, power_detail, services_detail,
};
use crate::collect::SystemRoot;
use crate::context::Agent;

/// One check and its result.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Check {
    /// Stable, such as `boot.wait_online` or `power.device_saving:<device>`.
    pub id: String,
    /// `fix` (opens a preview) or `advice` (what to do by hand).
    pub kind: &'static str,
    /// Such as "Stop \"wait for network\" from delaying boot".
    pub title: String,
    /// The measurement, in plain words with its numbers.
    pub measured: String,
    /// True when the improvement is needed now.
    pub offered: bool,
    /// The fix action, such as `services.disable`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// `you` (no password) or `admin`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access: Option<&'static str>,
    /// What to do, for advice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advice: Option<String>,
}

/// "1 update", "2 updates".
pub(super) fn count(n: usize, word: &str) -> String {
    format!("{n} {word}{}", if n == 1 { "" } else { "s" })
}

pub(super) fn fix(
    id: &str,
    title: &str,
    measured: String,
    offered: bool,
    action: &str,
    subject: Option<String>,
    access: &'static str,
) -> Check {
    Check {
        id: id.to_owned(),
        kind: "fix",
        title: title.to_owned(),
        measured,
        offered,
        action: offered.then(|| action.to_owned()),
        subject: subject.filter(|_| offered),
        access: Some(access),
        advice: None,
    }
}

pub(super) fn advice(id: &str, title: &str, measured: String, offered: bool, what: &str) -> Check {
    Check {
        id: id.to_owned(),
        kind: "advice",
        title: title.to_owned(),
        measured,
        offered,
        action: None,
        subject: None,
        access: None,
        advice: offered.then(|| what.to_owned()),
    }
}

const WAIT_ONLINE: &str = "NetworkManager-wait-online.service";
/// Filesystems that need the network before they can mount.
const NETWORK_FS: [&str; 8] = [
    "nfs",
    "nfs4",
    "cifs",
    "smb3",
    "fuse.sshfs",
    "glusterfs",
    "ceph",
    "9p",
];

/// Mounts that need the network: by type, or marked `_netdev` in fstab.
pub fn network_mounts(mounts: &str, fstab: &str) -> Vec<String> {
    let mut out: Vec<String> = mounts
        .lines()
        .chain(fstab.lines().filter(|l| !l.trim_start().starts_with('#')))
        .filter_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            let (mount, kind, options) = (f.get(1)?, f.get(2)?, f.get(3).copied().unwrap_or(""));
            (NETWORK_FS.contains(kind) || options.split(',').any(|o| o == "_netdev"))
                .then(|| (*mount).to_owned())
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Units that want or need `network-online.target`, from
/// `systemctl list-dependencies --reverse --plain`, without the target itself.
pub fn wanting_network(list: &str) -> Vec<String> {
    list.lines()
        .skip(1)
        .map(|l| {
            l.trim_matches(|c: char| c.is_whitespace() || "●○*".contains(c))
                .to_owned()
        })
        .filter(|u| !u.is_empty() && u != WAIT_ONLINE && !u.ends_with(".target"))
        .collect()
}

async fn boot_wait_online(root: &SystemRoot) -> Check {
    let title = "Stop \"wait for network\" from delaying boot";
    let boot = services_detail::read_boot().await;
    let Some(ms) = boot
        .blame
        .iter()
        .find(|(u, _)| u == WAIT_ONLINE)
        .map(|(_, ms)| *ms)
    else {
        return fix(
            "boot.wait_online",
            title,
            format!("{WAIT_ONLINE} did not run at this boot."),
            false,
            "",
            None,
            "admin",
        );
    };
    let secs = ms as f64 / 1000.0;
    if ms <= 2000 {
        return fix(
            "boot.wait_online",
            title,
            format!("It took {secs:.1} s at this boot, which is short."),
            false,
            "",
            None,
            "admin",
        );
    }
    let mounts = network_mounts(
        &root.read("/proc/mounts").unwrap_or_default(),
        &root.read("/etc/fstab").unwrap_or_default(),
    );
    // Only units that started at this boot count: a timer's unit (such as fwupd-refresh) may want
    // the network but does not run at boot.
    let started: std::collections::HashSet<&str> =
        boot.blame.iter().map(|(u, _)| u.as_str()).collect();
    let wanting: Vec<String> = command::output(
        "systemctl",
        &[
            "list-dependencies",
            "--reverse",
            "--plain",
            "--no-pager",
            "network-online.target",
        ],
        Duration::from_secs(5),
    )
    .await
    .map(|l| wanting_network(&l))
    .unwrap_or_default()
    .into_iter()
    .filter(|u| started.contains(u.as_str()))
    .collect();
    if !mounts.is_empty() {
        return fix(
            "boot.wait_online",
            title,
            format!(
                "It delayed boot by {secs:.1} s, but {} needs the network to mount, so it stays.",
                mounts.join(", ")
            ),
            false,
            "",
            None,
            "admin",
        );
    }
    if !wanting.is_empty() {
        return fix(
            "boot.wait_online",
            title,
            format!(
                "It delayed boot by {secs:.1} s, but {} {} for the network at this boot, so it stays.",
                wanting.join(", "),
                if wanting.len() == 1 { "waits" } else { "wait" }
            ),
            false,
            "",
            None,
            "admin",
        );
    }
    fix(
        "boot.wait_online",
        title,
        format!(
            "It delayed boot by {secs:.1} s, and nothing here needs the network before sign-in (no network mounts, no service waits for it). The network still connects as usual."
        ),
        true,
        "services.disable",
        Some(WAIT_ONLINE.to_owned()),
        "admin",
    )
}

async fn trim(root: &SystemRoot) -> Check {
    let title = "Turn on the weekly TRIM";
    let ssds: Vec<String> = storage_detail::read(root)
        .disks
        .into_iter()
        .filter(|d| !d.rotational && d.trim)
        .map(|d| d.name)
        .collect();
    if ssds.is_empty() {
        return fix(
            "storage.trim",
            title,
            "No SSD that supports TRIM was found.".into(),
            false,
            "",
            None,
            "admin",
        );
    }
    let state = command::output_any(
        "systemctl",
        &["is-enabled", "fstrim.timer"],
        Duration::from_secs(5),
    )
    .await
    .unwrap_or_default();
    let state = state.trim();
    match state {
        "enabled" | "static" | "alias" | "indirect" | "generated" => fix(
            "storage.trim",
            title,
            format!(
                "fstrim.timer is {state}: {} get TRIM every week.",
                ssds.join(", ")
            ),
            false,
            "",
            None,
            "admin",
        ),
        "" | "not-found" => fix(
            "storage.trim",
            title,
            "fstrim.timer is not installed (package util-linux).".into(),
            false,
            "",
            None,
            "admin",
        ),
        _ => fix(
            "storage.trim",
            title,
            format!(
                "{} support TRIM, but fstrim.timer is {state}, so free blocks are never reported and the SSD can slow down over time.",
                ssds.join(", ")
            ),
            true,
            "services.enable",
            Some("fstrim.timer".into()),
            "admin",
        ),
    }
}

/// True when the folder under `/sys` or `/dev` has an entry whose name starts with `prefix`.
fn any_entry(root: &SystemRoot, dir: &str, prefix: &str) -> bool {
    std::fs::read_dir(root.path(dir))
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| e.file_name().to_string_lossy().starts_with(prefix))
}

/// An rfkill switch of this type (`wwan`, `bluetooth`) exists.
fn rfkill_has(root: &SystemRoot, kind: &str) -> bool {
    std::fs::read_dir(root.path("/sys/class/rfkill"))
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| {
            root.read(&format!(
                "/sys/class/rfkill/{}/type",
                e.file_name().to_string_lossy()
            ))
            .is_ok_and(|t| t.trim() == kind)
        })
}

/// Any sign of a mobile modem: a WWAN device or switch, a modem control port, a `ww*` network
/// interface, or a USB serial port (some USB modems show only that, so it counts too).
pub fn has_modem(root: &SystemRoot) -> bool {
    any_entry(root, "/sys/class/wwan", "")
        || rfkill_has(root, "wwan")
        || any_entry(root, "/dev", "cdc-wdm")
        || any_entry(root, "/dev", "ttyUSB")
        || any_entry(root, "/dev", "ttyACM")
        || any_entry(root, "/sys/class/net", "ww")
}

/// Any Bluetooth chip, even one switched off (its rfkill switch stays).
pub fn has_bluetooth(root: &SystemRoot) -> bool {
    any_entry(root, "/sys/class/bluetooth", "") || rfkill_has(root, "bluetooth")
}

/// Services that start at every boot to wait for hardware this computer does not have.
async fn no_hardware_services(root: &SystemRoot) -> Vec<Check> {
    let mut out = Vec::new();
    for (unit, name, hardware, has) in [
        (
            "ModemManager.service",
            "ModemManager",
            "mobile modem (no WWAN device and no USB modem)",
            has_modem(root),
        ),
        (
            "bluetooth.service",
            "The Bluetooth service",
            "Bluetooth chip",
            has_bluetooth(root),
        ),
    ] {
        let state = command::output_any("systemctl", &["is-enabled", unit], Duration::from_secs(5))
            .await
            .unwrap_or_default();
        let state = state.trim();
        if state.is_empty() || state == "not-found" {
            continue;
        }
        let enabled = state == "enabled";
        let (offered, measured) = if has {
            (
                false,
                format!(
                    "{name} is needed: this computer has a {}.",
                    hardware.split(" (").next().unwrap_or(hardware)
                ),
            )
        } else if !enabled {
            (
                false,
                format!(
                    "{name} is {state}, and this computer has no {}.",
                    hardware.split(" (").next().unwrap_or(hardware)
                ),
            )
        } else {
            (
                true,
                format!(
                    "{name} starts at every boot and waits, but this computer has no {hardware}. Turning it off saves a little memory and background work. If you add one later, undo this in DeeCure, or run: sudo systemctl enable --now {unit}"
                ),
            )
        };
        out.push(fix(
            &format!("services.no_hardware:{unit}"),
            &format!(
                "Turn off {} (no hardware for it)",
                unit.trim_end_matches(".service")
            ),
            measured,
            offered,
            "services.disable",
            Some(unit.into()),
            "admin",
        ));
    }
    out
}

fn old_kernels(root: &SystemRoot) -> Check {
    let k = kernel::read(root);
    let old: Vec<&kernel::InstalledKernel> =
        k.installed.iter().filter(|i| i.status == "old").collect();
    let size: u64 = old.iter().map(|i| i.size).sum();
    let names: Vec<&str> = old.iter().map(|i| i.version.as_str()).collect();
    fix(
        "packages.old_kernels",
        "Remove old kernels",
        if old.is_empty() {
            format!(
                "{} kernels are installed: the running one and at most one fallback.",
                k.installed.len()
            )
        } else {
            format!(
                "{} old kernels besides the running one and its fallback: {} ({} MB).",
                old.len(),
                names.join(", "),
                size / 1_000_000
            )
        },
        !old.is_empty(),
        "system.remove_old_kernels",
        None,
        "admin",
    )
}

/// The same rule as the app's Power › Sleep list (`pciPowerSavingSafe`): it has a driver that
/// lets it sleep (no second hold in `runtime_usage`), and is not storage, a bridge, graphics,
/// sound, or core system parts.
pub fn pci_saving_safe(d: &pci_detail::PciDevice) -> bool {
    d.driver.is_some()
        && d.sys_path.is_some()
        && d.power_control.as_deref() == Some("on")
        && d.runtime_usage.is_none_or(|holds| holds <= 1)
        && ![
            "Storage",
            "Bridge",
            "Display",
            "Audio and video",
            "System",
            "Memory",
        ]
        .contains(&d.class.as_deref().unwrap_or(""))
}

fn device_saving(root: &SystemRoot, power: &power_detail::PowerDetail) -> Vec<Check> {
    let title = |name: &str| format!("Let {name} save power when idle");
    let mut out: Vec<Check> = pci_detail::read(root)
        .into_iter()
        .filter(pci_saving_safe)
        .map(|d| {
            let name = d.model.clone().unwrap_or_else(|| d.address.clone());
            let path = d.sys_path.clone().unwrap_or_default();
            fix(
                &format!("power.device_saving:{path}"),
                &title(&name),
                format!(
                    "{name} ({}) has power saving off (power/control is \"on\"), so it never sleeps when idle. If it misbehaves after the change, undo it.",
                    d.class.as_deref().unwrap_or("PCI device")
                ),
                true,
                "power.set_device_power_saving",
                Some(format!("{path}|on")),
                "admin",
            )
        })
        .collect();
    out.extend(power.usb.iter().filter(|u| u.control.as_deref() == Some("on") && !u.keep_awake).filter_map(|u| {
        let path = u.sys_path.clone()?;
        let name = u.product.clone().unwrap_or_else(|| format!("USB device {}", u.port));
        Some(fix(
            &format!("power.device_saving:{path}"),
            &title(&name),
            format!("{name} has power saving off (power/control is \"on\"), so it never sleeps when idle. It is not a keyboard, mouse, sound device, or Bluetooth radio. If it misbehaves after the change, undo it."),
            true,
            "power.set_device_power_saving",
            Some(format!("{path}|on")),
            "admin",
        ))
    }));
    out
}

/// Heat slowdowns added up from the stored counter, which starts again at 0 after a restart.
pub fn slowdowns(values: &[f64]) -> u64 {
    added_up(values).round() as u64
}

/// What a counter added over readings; after a restart it starts again at 0.
fn added_up(values: &[f64]) -> f64 {
    values
        .windows(2)
        .map(|w| if w[1] >= w[0] { w[1] - w[0] } else { w[1] })
        .sum()
}

/// The share of time heat must slow the CPU before cleaning is suggested: 1% is about
/// 14 minutes of a day. Laptop chips that boost to their limit slow down for milliseconds
/// thousands of times a day; that is normal and needs no cleaning.
pub const HEAT_SHARE: f64 = 0.01;

/// Seconds the computer was on between readings (gaps over 5 minutes are sleep or off), and
/// seconds heat slowed the CPU in that time, from `cpu.throttle_seconds_total` readings.
pub fn slowed_share(points: &[(i64, f64)]) -> (f64, f64) {
    let on: f64 = points
        .windows(2)
        .map(|w| (w[1].0 - w[0].0) as f64 / 1000.0)
        .filter(|gap| *gap > 0.0 && *gap <= 300.0)
        .sum();
    let values: Vec<f64> = points.iter().map(|p| p.1).collect();
    (on, added_up(&values))
}

/// "45 s", "3 min 20 s", or "1 h 5 min".
fn duration_text(seconds: f64) -> String {
    let s = seconds.round() as u64;
    match s {
        0..60 => format!("{s} s"),
        60..3600 => format!("{} min {} s", s / 60, s % 60),
        _ => format!("{} h {} min", s / 3600, s % 3600 / 60),
    }
}

fn heat(agent: &Agent, cpu: &cpu_detail::CpuDetail) -> Check {
    let now = Timestamp::now();
    let day = Timestamp(now.0 - 86_400_000);
    let store = crate::store::lock(&agent.store);
    let points = store
        .metrics("cpu.throttle_total", None, day, now)
        .unwrap_or_default();
    let time: Vec<(i64, f64)> = store
        .metrics("cpu.throttle_seconds_total", None, day, now)
        .unwrap_or_default()
        .iter()
        .map(|m| (m.at.as_millis(), m.value))
        .collect();
    drop(store);
    let values: Vec<f64> = points.iter().map(|m| m.value).collect();
    let n = slowdowns(&values);
    let since_start = cpu.package_throttle_count.unwrap_or(0);
    const WHAT: &str = "Dust in the fan or vents is the usual cause. Clean them, keep the vents free (not on a bed or lap), and check that the fan spins. Hardware › Sensors shows the temperatures where it slows down.";
    if time.len() >= 2 {
        // The kernel reports how long it slowed down: judge by that, not by how often.
        let (on, slowed) = slowed_share(&time);
        let share = if on > 0.0 { slowed / on } else { 0.0 };
        // Right after an update the time counter is new: wait for an hour of readings.
        if on < 3600.0 {
            return advice(
                "hardware.heat",
                "Clean the fans, or check the cooling",
                format!(
                    "Not enough readings yet: DeeCure has measured how long heat slows the CPU for {}, and needs an hour.",
                    duration_text(on)
                ),
                false,
                WHAT,
            );
        }
        let measured = if slowed < 0.5 {
            "The CPU was not slowed by heat in the last 24 hours.".to_owned()
        } else {
            format!(
                "The CPU was slowed by heat for {} in the last 24 hours: {:.1}% of the time it was on, in {}.{}",
                duration_text(slowed),
                share * 100.0,
                count(n as usize, "slowdown"),
                if share < HEAT_SHARE {
                    " Short slowdowns like these are normal for laptop chips that boost to their limit; cleaning is suggested from 1%."
                } else {
                    ""
                }
            )
        };
        return advice(
            "hardware.heat",
            "Clean the fans, or check the cooling",
            measured,
            share >= HEAT_SHARE,
            WHAT,
        );
    }
    // A CPU that does not report the time (or the first minutes after an update): by count.
    advice(
        "hardware.heat",
        "Clean the fans, or check the cooling",
        if values.len() < 2 {
            "Not enough readings yet: DeeCure keeps one a minute.".into()
        } else if n == 0 {
            "The CPU did not slow down from heat in the last 24 hours.".into()
        } else {
            format!(
                "The CPU slowed down from heat {} in the last 24 hours ({} since the last start).",
                count(n as usize, "time"),
                count(since_start as usize, "time")
            )
        },
        n > 0,
        WHAT,
    )
}

/// PCIe links narrower than both ends support, and drives slower than both ends support.
/// Graphics and other devices lower their speed when idle to save power, so only drive speed counts.
fn pcie(root: &SystemRoot) -> Vec<Check> {
    let gt = |s: &Option<String>| {
        s.as_deref()
            .and_then(|t| t.split_whitespace().next()?.parse::<f64>().ok())
    };
    pci_detail::read(root)
        .into_iter()
        // A link that is down (x0, such as an empty port), an unknown width (255, such as built-in
        // graphics, which has no real PCIe link), and bridges are skipped: the device behind a
        // bridge is checked itself.
        .filter(|d| {
            d.class.as_deref() != Some("Bridge")
                && d.link_width.is_some_and(|w| (1..=32).contains(&w))
                && d.max_link_width.is_none_or(|w| w <= 32)
        })
        .filter_map(|d| {
            let name = d.model.clone().unwrap_or_else(|| d.address.clone());
            let width_max = match (d.max_link_width, d.slot_max_link_width) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
            let narrow = matches!((d.link_width, width_max), (Some(w), Some(m)) if w < m);
            let speed_max = match (gt(&d.max_link_speed), gt(&d.slot_max_link_speed)) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
            let slow = d.class.as_deref() == Some("Storage") && matches!((gt(&d.link_speed), speed_max), (Some(s), Some(m)) if s < m);
            (narrow || slow).then(|| {
                advice(
                    &format!("hardware.pcie:{}", d.address),
                    "Move the device, or check a firmware setting",
                    format!(
                        "{name} runs at {} x{} while it and its slot support {} x{}.",
                        d.link_speed.as_deref().unwrap_or("?"),
                        d.link_width.map_or("?".into(), |w| w.to_string()),
                        speed_max.map_or("?".into(), |s| format!("{s} GT/s")),
                        width_max.map_or("?".into(), |w| w.to_string())
                    ),
                    true,
                    "Check that it is fully seated, try another slot, and look for a BIOS setting that shares lanes (such as a second M.2 slot or SATA ports).",
                )
            })
        })
        .collect()
}

/// USB 3 devices on a USB 2 link while the computer has USB 3 ports.
fn usb_speed(root: &SystemRoot) -> Vec<Check> {
    let dir = root.path("/sys/bus/usb/devices");
    let read = |name: &str, f: &str| {
        root.read(&format!("/sys/bus/usb/devices/{name}/{f}"))
            .ok()
            .map(|s| s.trim().to_owned())
    };
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !n.contains(':'))
        .collect();
    names.sort();
    let has_usb3 = names.iter().any(|n| {
        n.starts_with("usb")
            && read(n, "speed")
                .and_then(|s| s.parse::<f64>().ok())
                .is_some_and(|s| s >= 5000.0)
    });
    if !has_usb3 {
        return Vec::new();
    }
    names
        .iter()
        .filter(|n| !n.starts_with("usb"))
        .filter_map(|n| {
            let version: f64 = read(n, "version")?.parse().ok()?;
            let speed: f64 = read(n, "speed")?.parse().ok()?;
            // Hubs pass on what is plugged into them; the device behind them is counted.
            let hub = read(n, "bDeviceClass").as_deref() == Some("09");
            (version >= 3.0 && speed <= 480.0 && !hub).then(|| {
                let name = read(n, "product").unwrap_or_else(|| format!("USB device {n}"));
                advice(
                    &format!("hardware.usb_speed:{n}"),
                    "Plug the device into a faster USB port",
                    format!("{name} is a USB 3 device but runs at {speed} Mbit/s (USB 2 speed) on port {n}."),
                    true,
                    "Use a blue or \"SS\" marked port or a USB-C port, and a USB 3 cable; some hubs and extension cables only carry USB 2.",
                )
            })
        })
        .collect()
}

fn battery_health(root: &SystemRoot) -> Vec<Check> {
    let dir = "/sys/class/power_supply";
    let num = |b: &str, f: &str| {
        root.read(&format!("{dir}/{b}/{f}"))
            .ok()?
            .trim()
            .parse::<f64>()
            .ok()
    };
    let mut names: Vec<String> = std::fs::read_dir(root.path(dir))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
        .into_iter()
        .filter(|b| {
            root.read(&format!("{dir}/{b}/type")).is_ok_and(|t| t.trim() == "Battery")
                && root.read(&format!("{dir}/{b}/scope")).map_or(true, |s| s.trim() != "Device")
        })
        .filter_map(|b| {
            let (full, design) = match (num(&b, "energy_full"), num(&b, "energy_full_design")) {
                (Some(f), Some(d)) => (f, d),
                _ => (num(&b, "charge_full")?, num(&b, "charge_full_design")?),
            };
            (design > 0.0).then(|| {
                let pct = (full / design * 100.0).round();
                advice(
                    &format!("hardware.battery:{b}"),
                    "Replace the battery",
                    format!("Battery {b} holds {pct}% of the charge it held when new."),
                    pct < 60.0,
                    "Below 60% the battery runs out much sooner and can shut down early under load. A replacement from the laptop maker brings back the full time.",
                )
            })
        })
        .collect()
}

fn restart_needed(root: &SystemRoot, cpu: &cpu_detail::CpuDetail) -> Check {
    let hex = |s: &Option<String>| {
        s.as_deref()
            .and_then(|v| u64::from_str_radix(v.trim_start_matches("0x"), 16).ok())
    };
    let microcode = matches!((hex(&cpu.microcode_loaded), hex(&cpu.microcode_on_disk)), (Some(a), Some(b)) if b > a);
    let k = kernel::read(root);
    let newest = k.installed.first().map(|i| i.version.clone());
    let kernel = newest.as_ref().is_some_and(|n| *n != k.running);
    let mut parts = Vec::new();
    if microcode {
        parts.push(format!(
            "microcode {} is installed, {} is running",
            cpu.microcode_on_disk.as_deref().unwrap_or("?"),
            cpu.microcode_loaded.as_deref().unwrap_or("?")
        ));
    }
    if kernel {
        parts.push(format!(
            "kernel {} is installed, {} is running",
            newest.as_deref().unwrap_or("?"),
            k.running
        ));
    }
    advice(
        "system.restart",
        "Restart to load the new kernel or microcode",
        if parts.is_empty() {
            "The newest installed kernel and microcode are running.".into()
        } else {
            format!("A restart is waiting: {}.", parts.join("; "))
        },
        !parts.is_empty(),
        "Restart when it suits you. Security fixes in a kernel or microcode only work after a restart.",
    )
}

async fn memory_errors(root: &SystemRoot) -> Check {
    let m = memory_detail::read(root).await;
    let ue: u64 = m.controllers.iter().map(|c| c.uncorrected).sum();
    advice(
        "hardware.memory_errors",
        "Replace a memory module",
        if m.controllers.is_empty() {
            "This computer does not report memory errors (no EDAC driver), so they cannot be counted.".into()
        } else {
            format!("{ue} uncorrected memory errors reported since the last start.")
        },
        ue > 0,
        "Uncorrected errors can crash apps or damage files. Hardware › Memory Modules shows which module; test it with memtest86+ from the boot menu and replace it.",
    )
}

/// The parts of the checks, in the order the page runs them one by one, so it can show which
/// part runs now and what each found. The inventory kind of a part is `optimize_<part>`.
pub const PARTS: [&str; 8] = [
    "boot", "disks", "kernels", "devices", "hardware", "cleanup", "history", "drivers",
];

/// The checks of one part, offered ones first; nothing for a part that does not exist.
pub async fn read_part(agent: &Agent, part: &str) -> Vec<Check> {
    let root = &agent.root;
    let mut checks = match part {
        "boot" => vec![boot_wait_online(root).await],
        "disks" => vec![trim(root).await],
        "kernels" => vec![old_kernels(root)],
        "devices" => {
            let power = power_detail::read(root);
            let mut checks = device_saving(root, &power);
            checks.extend(pcie(root));
            checks.extend(usb_speed(root));
            checks.extend(no_hardware_services(root).await);
            checks
        }
        "hardware" => {
            let cpu = cpu_detail::read(root);
            let mut checks = vec![heat(agent, &cpu)];
            checks.extend(battery_health(root));
            checks.push(restart_needed(root, &cpu));
            checks.push(memory_errors(root).await);
            checks
        }
        "cleanup" => cleanup::checks(agent).await,
        "history" => history::checks(agent).await,
        "drivers" => drivers::checks(agent).await,
        _ => Vec::new(),
    };
    checks.sort_by_key(|c| !c.offered);
    checks
}

/// Every check, offered ones first.
pub async fn read(agent: &Agent) -> Vec<Check> {
    let mut checks = Vec::new();
    for part in PARTS {
        checks.extend(read_part(agent, part).await);
    }
    checks.sort_by_key(|c| !c.offered);
    checks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_modem_or_bluetooth_chip_is_found_in_any_of_its_forms() {
        use crate::collect::test_support::fixture_root;
        let bare = fixture_root(
            "opt-bare",
            &[
                ("/sys/class/net/wlp45s0/type", "1\n"),
                ("/sys/class/rfkill/rfkill0/type", "wlan\n"),
            ],
        );
        assert!(!has_modem(&bare));
        assert!(!has_bluetooth(&bare));
        let usb_modem = fixture_root("opt-usb-modem", &[("/dev/ttyUSB0", "")]);
        assert!(has_modem(&usb_modem));
        let wwan = fixture_root("opt-wwan", &[("/sys/class/net/wwan0/type", "65534\n")]);
        assert!(has_modem(&wwan));
        // A Bluetooth chip that is switched off still counts.
        let blocked = fixture_root(
            "opt-bt-off",
            &[("/sys/class/rfkill/rfkill2/type", "bluetooth\n")],
        );
        assert!(has_bluetooth(&blocked));
    }

    #[test]
    fn heat_advice_counts_how_long_not_how_often() {
        // A minute apart for 5 hours, slowed for 115 s in all (this laptop on 2026-11-16).
        let points: Vec<(i64, f64)> = (0..=300)
            .map(|i| (i * 60_000, i as f64 * 115.0 / 300.0))
            .collect();
        let (on, slowed) = slowed_share(&points);
        assert_eq!(on, 18_000.0);
        assert!((slowed - 115.0).abs() < 1e-6);
        assert!(slowed / on < HEAT_SHARE, "0.6% is normal");
        // A restart sets the counter back to 0; a long gap (sleep) is not time on.
        let (on, slowed) = slowed_share(&[
            (0, 50.0),
            (60_000, 80.0),
            (3_660_000, 10.0),
            (3_720_000, 20.0),
        ]);
        assert_eq!(on, 120.0);
        assert_eq!(slowed, 30.0 + 10.0 + 10.0);
        assert_eq!(duration_text(115.0), "1 min 55 s");
        assert_eq!(duration_text(45.2), "45 s");
        assert_eq!(duration_text(3900.0), "1 h 5 min");
    }

    #[test]
    fn offers_power_saving_only_where_the_driver_lets_the_device_sleep() {
        let device = |holds: Option<u32>| pci_detail::PciDevice {
            driver: Some("rtw89_8852be".into()),
            sys_path: Some("pci0000:00/0000:00:1d.1/0000:2d:00.0".into()),
            power_control: Some("on".into()),
            class: Some("Network".into()),
            runtime_usage: holds,
            ..Default::default()
        };
        // The kernel's own hold only: turning power saving on lets it sleep.
        assert!(pci_saving_safe(&device(Some(1))));
        // The driver holds it too: it would never sleep, so nothing is offered.
        assert!(!pci_saving_safe(&device(Some(2))));
        // A kernel that does not show the count: as before.
        assert!(pci_saving_safe(&device(None)));
    }

    #[test]
    fn every_part_is_an_inventory_kind() {
        for part in PARTS {
            let kind = format!("optimize_{part}");
            assert!(super::super::KINDS.contains(&kind.as_str()), "{kind}");
        }
    }

    #[test]
    fn network_mounts_come_from_type_or_netdev() {
        let mounts = "/dev/nvme0n1p2 / ext4 rw 0 0\nnas:/data /mnt/data nfs4 rw 0 0\n";
        let fstab = "# //nas/x /mnt/x cifs _netdev 0 0\nUUID=1 /backup ext4 defaults,_netdev 0 2\n";
        assert_eq!(network_mounts(mounts, fstab), ["/backup", "/mnt/data"]);
        assert!(network_mounts("/dev/sda1 / ext4 rw 0 0\n", "").is_empty());
    }

    #[test]
    fn services_that_wait_for_the_network_are_listed() {
        let list = "network-online.target\n● kerneloops.service\n○ cups-browsed.service\n  multi-user.target\n";
        assert_eq!(
            wanting_network(list),
            ["kerneloops.service", "cups-browsed.service"]
        );
        assert!(wanting_network("network-online.target\n").is_empty());
    }

    #[test]
    fn slowdowns_add_up_across_a_restart() {
        assert_eq!(slowdowns(&[100.0, 120.0, 150.0]), 50);
        // Restart: the counter starts again from 0.
        assert_eq!(slowdowns(&[100.0, 150.0, 4.0, 10.0]), 60);
        assert_eq!(slowdowns(&[5.0]), 0);
    }
}
