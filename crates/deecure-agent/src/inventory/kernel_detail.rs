// SPDX-License-Identifier: Apache-2.0

//! Kernel settings and system limits for the Expert level (docs/01-product/expert-coverage.md,
//! section 11): every kernel setting from the `sysctl.d` files with its file, line, owner
//! package, whether that file was changed from the package's version, and the value now; file
//! watches and open files against their limits; modules with their taint flags, users, and
//! devices; extra drivers (DKMS); kernel lockdown; and time sync. No admin rights.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::Serialize;

use super::command;
use crate::collect::SystemRoot;

const LIMIT: Duration = Duration::from_secs(10);

/// One kernel setting from a file.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub file: String,
    pub line: usize,
    /// The package that ships the file; missing when no package does (set on this computer).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// The file differs from the package's version.
    pub file_changed: bool,
    /// The value now, from `/proc/sys`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub now: Option<String>,
    /// Another file sets the same key later, so this one is not in effect.
    pub overridden: bool,
}

/// One process using file watches.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WatchUser {
    pub pid: u32,
    pub name: String,
    pub watches: u64,
}

/// One loaded kernel module.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Module {
    pub name: String,
    pub size: u64,
    /// Taint letters: `O` from outside the kernel, `E` unsigned, `P` proprietary, …
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taint: Option<String>,
    /// Other modules that use it.
    pub used_by: Vec<String>,
    /// Devices bound to its drivers.
    pub devices: usize,
}

/// One extra driver built with DKMS.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DkmsModule {
    pub module: String,
    pub version: String,
    /// Kernels it is built and installed for.
    pub kernels: Vec<String>,
}

/// What `kernel_detail` answers.
#[derive(Debug, Default, Serialize)]
pub struct KernelDetail {
    pub settings: Vec<Setting>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watch_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watch_instances_limit: Option<u64>,
    /// File watches of the user's own processes, and the processes that use the most.
    pub watches_used: u64,
    pub watch_users: Vec<WatchUser>,
    /// Open files system-wide, and the most allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_files: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_files_limit: Option<u64>,
    pub modules: Vec<Module>,
    /// Missing when DKMS is not installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dkms: Option<Vec<DkmsModule>>,
    /// Kernels with a modules folder, to compare with DKMS builds.
    pub kernels: Vec<String>,
    /// `none`, `integrity`, or `confidentiality`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lockdown: Option<String>,
    /// `timedatectl timesync-status` fields, such as `Server`, `Offset`, `Poll interval`.
    pub time_sync: BTreeMap<String, String>,
}

fn text(root: &SystemRoot, path: &str) -> Option<String> {
    root.read(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn names(root: &SystemRoot, dir: &str) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(root.path(dir))
        .map(|d| {
            d.flatten()
                .filter_map(|e| e.file_name().to_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// The `sysctl.d` files in the order systemd applies them: by name over all folders, a file in
/// `/etc` replacing one with the same name in `/run` or `/usr/lib`, and `/etc/sysctl.conf` last.
pub fn sysctl_files(root: &SystemRoot) -> Vec<String> {
    let mut by_name: BTreeMap<String, String> = BTreeMap::new();
    for dir in [
        "/usr/lib/sysctl.d",
        "/lib/sysctl.d",
        "/usr/local/lib/sysctl.d",
        "/run/sysctl.d",
        "/etc/sysctl.d",
    ] {
        for n in names(root, dir)
            .into_iter()
            .filter(|n| n.ends_with(".conf"))
        {
            by_name.insert(n.clone(), format!("{dir}/{n}"));
        }
    }
    let mut files: Vec<String> = by_name.into_values().collect();
    if root.path("/etc/sysctl.conf").exists() {
        files.push("/etc/sysctl.conf".into());
    }
    files
}

/// `key = value` lines of one sysctl file; keys with `*` (patterns) and `-` (ignore errors) kept.
pub fn parse_sysctl(text: &str) -> Vec<(usize, String, String)> {
    text.lines()
        .enumerate()
        .filter_map(|(i, raw)| {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                return None;
            }
            let (k, v) = line.split_once('=')?;
            let key = k.trim().trim_start_matches('-').replace('/', ".");
            Some((i + 1, key, v.trim().to_owned()))
        })
        .collect()
}

/// `dpkg -S` lines (`procps: /etc/sysctl.d/10-ptrace.conf`) as file → package.
pub fn parse_dpkg_s(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter_map(|l| {
            let (pkg, path) = l.split_once(": ")?;
            Some((
                path.trim().to_owned(),
                pkg.split(':').next()?.trim().to_owned(),
            ))
        })
        .collect()
}

/// Files `dpkg -V` reports with a changed checksum (`??5?????? c /etc/x.conf`).
pub fn parse_dpkg_v(text: &str) -> Vec<String> {
    text.lines()
        .filter(|l| l.get(2..3) == Some("5"))
        .filter_map(|l| l.split_whitespace().last().map(str::to_owned))
        .collect()
}

/// `dkms status` lines (`nvidia/535.183, 6.8.0-45-generic, x86_64: installed`).
pub fn parse_dkms(text: &str) -> Vec<DkmsModule> {
    let mut out: Vec<DkmsModule> = Vec::new();
    for line in text.lines() {
        let Some((head, state)) = line.rsplit_once(':') else {
            continue;
        };
        let parts: Vec<&str> = head.split(',').map(str::trim).collect();
        let Some((module, version)) = parts.first().and_then(|p| p.split_once('/')) else {
            continue;
        };
        let kernel = parts
            .get(1)
            .filter(|_| state.trim().starts_with("installed"))
            .map(|k| (*k).to_owned());
        match out
            .iter_mut()
            .find(|m| m.module == module && m.version == version)
        {
            Some(m) => m.kernels.extend(kernel),
            None => out.push(DkmsModule {
                module: module.to_owned(),
                version: version.to_owned(),
                kernels: kernel.into_iter().collect(),
            }),
        }
    }
    out
}

fn watches(root: &SystemRoot) -> (u64, Vec<WatchUser>) {
    let mut users = Vec::new();
    for e in std::fs::read_dir(root.path("/proc"))
        .into_iter()
        .flatten()
        .flatten()
    {
        let Some(pid) = e.file_name().to_str().and_then(|n| n.parse::<u32>().ok()) else {
            continue;
        };
        let Ok(fds) = std::fs::read_dir(root.path(&format!("/proc/{pid}/fd"))) else {
            continue;
        };
        let mut count = 0;
        for fd in fds.flatten() {
            if std::fs::read_link(fd.path()).is_ok_and(|t| t.as_os_str() == "anon_inode:inotify") {
                count += root
                    .read(&format!(
                        "/proc/{pid}/fdinfo/{}",
                        fd.file_name().to_string_lossy()
                    ))
                    .map(|t| t.lines().filter(|l| l.starts_with("inotify ")).count() as u64)
                    .unwrap_or(0);
            }
        }
        if count > 0 {
            let name = text(root, &format!("/proc/{pid}/comm")).unwrap_or_else(|| pid.to_string());
            users.push(WatchUser {
                pid,
                name,
                watches: count,
            });
        }
    }
    users.sort_by_key(|u| std::cmp::Reverse(u.watches));
    let total = users.iter().map(|u| u.watches).sum();
    users.truncate(10);
    (total, users)
}

fn modules(root: &SystemRoot) -> Vec<Module> {
    root.read("/proc/modules")
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split_whitespace().collect();
            let name = (*f.first()?).to_owned();
            let used_by = f
                .get(3)
                .map(|u| {
                    u.split(',')
                        .filter(|x| !x.is_empty() && *x != "-")
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default();
            let devices = names(root, &format!("/sys/module/{name}/drivers"))
                .iter()
                .map(|d| {
                    std::fs::read_dir(root.path(&format!("/sys/module/{name}/drivers/{d}")))
                        .map(|r| {
                            r.flatten()
                                .filter(|e| {
                                    e.file_name()
                                        .to_str()
                                        .is_some_and(|n| n.contains(':') || n.contains('-'))
                                })
                                .count()
                        })
                        .unwrap_or(0)
                })
                .sum();
            Some(Module {
                size: f.get(1)?.parse().ok()?,
                taint: text(root, &format!("/sys/module/{name}/taint")),
                used_by,
                devices,
                name,
            })
        })
        .collect()
}

/// Reads `kernel_detail`.
pub async fn read(root: &SystemRoot) -> KernelDetail {
    let files = sysctl_files(root);
    let mut settings: Vec<Setting> = Vec::new();
    for file in &files {
        for (line, key, value) in root
            .read(file)
            .map(|t| parse_sysctl(&t))
            .unwrap_or_default()
        {
            for earlier in settings.iter_mut().filter(|s| s.key == key) {
                earlier.overridden = true;
            }
            // The kernel separates values with tabs; files use spaces.
            let now = (!key.contains('*'))
                .then(|| text(root, &format!("/proc/sys/{}", key.replace('.', "/"))))
                .flatten()
                .map(|v| v.split_whitespace().collect::<Vec<_>>().join(" "));
            let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
            settings.push(Setting {
                key,
                value,
                file: file.clone(),
                line,
                now,
                ..Default::default()
            });
        }
    }
    // With Ubuntu's merged /usr, dpkg may know a file as /lib/… while it is found at /usr/lib/….
    let alias = |f: &str| -> Option<String> {
        f.strip_prefix("/usr")
            .filter(|r| r.starts_with("/lib/"))
            .map(str::to_owned)
            .or_else(|| f.starts_with("/lib/").then(|| format!("/usr{f}")))
    };
    let aliases: Vec<String> = files.iter().filter_map(|f| alias(f)).collect();
    let file_args: Vec<&str> = files.iter().chain(&aliases).map(String::as_str).collect();
    let owners = if file_args.is_empty() {
        BTreeMap::new()
    } else {
        let mut args = vec!["-S"];
        args.extend(&file_args);
        command::output_any("dpkg", &args, LIMIT)
            .await
            .map(|t| parse_dpkg_s(&t))
            .unwrap_or_default()
    };
    let mut packages: Vec<&str> = owners.values().map(String::as_str).collect();
    packages.sort_unstable();
    packages.dedup();
    let changed = if packages.is_empty() {
        Vec::new()
    } else {
        let mut args = vec!["-V"];
        args.extend(&packages);
        command::output_any("dpkg", &args, LIMIT)
            .await
            .map(|t| parse_dpkg_v(&t))
            .unwrap_or_default()
    };
    for s in &mut settings {
        s.package = owners
            .get(&s.file)
            .or_else(|| alias(&s.file).and_then(|a| owners.get(&a)))
            .cloned();
        s.file_changed =
            changed.contains(&s.file) || alias(&s.file).is_some_and(|a| changed.contains(&a));
    }
    let (watches_used, watch_users) = watches(root);
    let file_nr: Vec<u64> = text(root, "/proc/sys/fs/file-nr")
        .map(|t| {
            t.split_whitespace()
                .filter_map(|v| v.parse().ok())
                .collect()
        })
        .unwrap_or_default();
    let time_sync = command::output("timedatectl", &["timesync-status", "--no-pager"], LIMIT)
        .await
        .map(|t| {
            t.lines()
                .filter_map(|l| l.split_once(':'))
                .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
                .collect()
        })
        .unwrap_or_default();
    KernelDetail {
        settings,
        watch_limit: text(root, "/proc/sys/fs/inotify/max_user_watches")
            .and_then(|v| v.parse().ok()),
        watch_instances_limit: text(root, "/proc/sys/fs/inotify/max_user_instances")
            .and_then(|v| v.parse().ok()),
        watches_used,
        watch_users,
        open_files: file_nr.first().copied(),
        open_files_limit: file_nr.get(2).copied(),
        modules: modules(root),
        dkms: command::output("dkms", &["status"], LIMIT)
            .await
            .map(|t| parse_dkms(&t)),
        kernels: names(root, "/lib/modules")
            .into_iter()
            .filter(|k| root.path(&format!("/lib/modules/{k}/modules.dep")).exists())
            .collect(),
        lockdown: text(root, "/sys/kernel/security/lockdown").and_then(|t| {
            let start = t.find('[')? + 1;
            Some(t[start..start + t[start..].find(']')?].to_owned())
        }),
        time_sync,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn sysctl_files_and_lines() {
        let root = fixture_root(
            "kernel-detail",
            &[
                ("/usr/lib/sysctl.d/50-default.conf", "kernel.sysrq = 16\n"),
                (
                    "/usr/lib/sysctl.d/10-ptrace.conf",
                    "kernel.yama.ptrace_scope = 0\n",
                ),
                (
                    "/etc/sysctl.d/10-ptrace.conf",
                    "# Ubuntu\nkernel.yama.ptrace_scope = 1\n",
                ),
                (
                    "/etc/sysctl.d/99-me.conf",
                    "vm.swappiness=10\n-net/ipv4/ip_forward = 1\n; comment\n",
                ),
                ("/etc/sysctl.conf", "#nothing\n"),
                ("/etc/sysctl.d/README.sysctl", "not a conf\n"),
            ],
        );
        assert_eq!(
            sysctl_files(&root),
            [
                "/etc/sysctl.d/10-ptrace.conf",
                "/usr/lib/sysctl.d/50-default.conf",
                "/etc/sysctl.d/99-me.conf",
                "/etc/sysctl.conf"
            ]
        );
        let lines = parse_sysctl("vm.swappiness=10\n-net/ipv4/ip_forward = 1\n; c\n# c\n");
        assert_eq!(
            lines,
            [
                (1, "vm.swappiness".into(), "10".into()),
                (2, "net.ipv4.ip_forward".into(), "1".into())
            ]
        );
    }

    #[test]
    fn dpkg_dkms_and_parsers() {
        let owners = parse_dpkg_s(
            "procps: /etc/sysctl.d/10-ptrace.conf\nsystemd:amd64: /usr/lib/sysctl.d/50-default.conf\ndpkg-query: no path found matching pattern /etc/sysctl.d/99-me.conf\n",
        );
        assert_eq!(
            owners
                .get("/etc/sysctl.d/10-ptrace.conf")
                .map(String::as_str),
            Some("procps")
        );
        assert_eq!(
            owners
                .get("/usr/lib/sysctl.d/50-default.conf")
                .map(String::as_str),
            Some("systemd")
        );
        assert!(!owners.contains_key("/etc/sysctl.d/99-me.conf"));
        assert_eq!(
            parse_dpkg_v("??5?????? c /etc/sysctl.d/10-ptrace.conf\n????????? c /etc/other\n"),
            ["/etc/sysctl.d/10-ptrace.conf"]
        );
        let d = parse_dkms(
            "nvidia/535.183, 6.8.0-45-generic, x86_64: installed\nnvidia/535.183, 6.8.0-49-generic, x86_64: built\nv4l2loopback/0.12.7: added\n",
        );
        assert_eq!(d.len(), 2);
        assert_eq!(
            d[0].kernels,
            ["6.8.0-45-generic"],
            "built but not installed does not count"
        );
        assert!(d[1].kernels.is_empty());
    }

    #[tokio::test]
    async fn settings_know_what_overrides_what() {
        let root = fixture_root(
            "kernel-settings",
            &[
                ("/usr/lib/sysctl.d/50-default.conf", "vm.swappiness = 60\n"),
                ("/etc/sysctl.d/99-me.conf", "vm.swappiness = 10\n"),
                ("/proc/sys/vm/swappiness", "60\n"),
                ("/proc/sys/fs/inotify/max_user_watches", "65536\n"),
                ("/proc/sys/fs/file-nr", "16834\t0\t9223372036854775807\n"),
                (
                    "/proc/modules",
                    "i915 4194304 30 kvmgt, - Live 0x0\nnvidia 1000 0 - Live 0x0 (POE)\n",
                ),
                ("/sys/module/nvidia/taint", "POE\n"),
                (
                    "/sys/kernel/security/lockdown",
                    "none [integrity] confidentiality\n",
                ),
            ],
        );
        let k = read(&root).await;
        assert_eq!(k.settings.len(), 2);
        assert!(k.settings[0].overridden && !k.settings[1].overridden);
        assert_eq!(
            k.settings[1].now.as_deref(),
            Some("60"),
            "the value now differs from the file"
        );
        assert_eq!(
            (k.watch_limit, k.open_files, k.open_files_limit),
            (Some(65536), Some(16834), Some(9_223_372_036_854_775_807))
        );
        assert_eq!(k.modules[0].used_by, ["kvmgt"]);
        assert_eq!(k.modules[1].taint.as_deref(), Some("POE"));
        assert_eq!(k.lockdown.as_deref(), Some("integrity"));
    }
}
