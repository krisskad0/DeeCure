// SPDX-License-Identifier: Apache-2.0

//! The running kernel, installed kernels, and loaded modules.

use std::path::Path;
use std::time::UNIX_EPOCH;

use deecure_core::Timestamp;
use serde::Serialize;

use crate::collect::SystemRoot;

/// One loaded module, from `/proc/modules`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Module {
    /// Name, such as `i915`.
    pub name: String,
    /// Memory it uses, in bytes.
    pub size: u64,
    /// How many users it has.
    pub used: u32,
    /// Modules that use it.
    pub used_by: Vec<String>,
}

/// One installed kernel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InstalledKernel {
    /// Such as `6.8.0-47-generic`.
    pub version: String,
    /// `running`, `fallback` (the newest other one), or `old`.
    pub status: String,
    /// Disk space of its files in `/boot` and `/lib/modules`, in bytes.
    pub size: u64,
    /// When its modules folder was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_at: Option<Timestamp>,
}

/// The `kernel` inventory.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct KernelInfo {
    /// The running kernel.
    pub running: String,
    /// The taint flags; 0 means untainted.
    pub tainted: u64,
    /// The boot options.
    pub command_line: String,
    /// Installed kernels, newest first.
    pub installed: Vec<InstalledKernel>,
    /// Loaded modules, biggest first.
    pub modules: Vec<Module>,
}

/// Reads `/proc/modules`: `i915 4194304 34 drm_display_helper,ttm, Live 0x0`.
pub fn parse_modules(text: &str) -> Vec<Module> {
    let mut out: Vec<Module> = text
        .lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split_whitespace().collect();
            Some(Module {
                name: (*f.first()?).to_owned(),
                size: f.get(1)?.parse().ok()?,
                used: f.get(2)?.parse().ok()?,
                used_by: f
                    .get(3)
                    .map(|u| {
                        u.split(',')
                            .filter(|m| !m.is_empty() && *m != "-")
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default(),
            })
        })
        .collect();
    out.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.name.cmp(&b.name)));
    out
}

/// Compares kernel versions such as `6.8.0-47-generic` by their numbers.
fn version_key(v: &str) -> Vec<u64> {
    v.split(|c: char| !c.is_ascii_digit())
        .filter_map(|n| n.parse().ok())
        .collect()
}

fn folder_size(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() => folder_size(&e.path()),
            Ok(t) if t.is_file() => e.metadata().map_or(0, |m| m.len()),
            _ => 0,
        })
        .sum()
}

/// Reads the kernel inventory.
pub fn read(root: &SystemRoot) -> KernelInfo {
    let running = root
        .read("/proc/sys/kernel/osrelease")
        .unwrap_or_default()
        .trim()
        .to_owned();
    let mut versions: Vec<String> = std::fs::read_dir(root.path("/lib/modules"))
        .map(|d| {
            d.flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    // Only folders with a kernel image are installed kernels; others are leftovers.
    versions.retain(|v| v == &running || root.path(&format!("/boot/vmlinuz-{v}")).exists());
    versions.sort_by_key(|v| std::cmp::Reverse(version_key(v)));
    let fallback = versions.iter().find(|v| **v != running).cloned();
    let installed = versions
        .into_iter()
        .map(|v| {
            let modules = root.path(&format!("/lib/modules/{v}"));
            let boot: u64 = ["vmlinuz", "initrd.img", "System.map", "config"]
                .iter()
                .filter_map(|f| std::fs::metadata(root.path(&format!("/boot/{f}-{v}"))).ok())
                .map(|m| m.len())
                .sum();
            let installed_at = std::fs::metadata(&modules)
                .ok()
                .and_then(|m| m.created().or_else(|_| m.modified()).ok())
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .and_then(|d| i64::try_from(d.as_millis()).ok())
                .map(Timestamp);
            InstalledKernel {
                status: if v == running {
                    "running"
                } else if Some(&v) == fallback.as_ref() {
                    "fallback"
                } else {
                    "old"
                }
                .into(),
                size: boot + folder_size(&modules),
                installed_at,
                version: v,
            }
        })
        .collect();
    KernelInfo {
        tainted: root
            .read("/proc/sys/kernel/tainted")
            .ok()
            .and_then(|t| t.trim().parse().ok())
            .unwrap_or(0),
        command_line: root
            .read("/proc/cmdline")
            .unwrap_or_default()
            .trim()
            .to_owned(),
        modules: parse_modules(&root.read("/proc/modules").unwrap_or_default()),
        installed,
        running,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn parses_modules_biggest_first() {
        let m = parse_modules(
            "btusb 77824 0 - Live 0x0\ni915 4194304 34 drm_display_helper,ttm, Live 0x0\n",
        );
        assert_eq!(m[0].name, "i915");
        assert_eq!(m[0].used_by, ["drm_display_helper", "ttm"]);
        assert!(m[1].used_by.is_empty());
    }

    #[test]
    fn marks_running_fallback_and_old_kernels() {
        let root = fixture_root(
            "kernel",
            &[
                ("/proc/sys/kernel/osrelease", "6.8.0-47-generic\n"),
                ("/proc/sys/kernel/tainted", "0\n"),
                ("/lib/modules/6.8.0-47-generic/modules.dep", "x"),
                ("/lib/modules/6.8.0-45-generic/modules.dep", "xx"),
                ("/lib/modules/6.8.0-41-generic/modules.dep", "xxx"),
                ("/lib/modules/6.8.0-9-leftover/modules.dep", ""),
                ("/boot/vmlinuz-6.8.0-47-generic", "k"),
                ("/boot/vmlinuz-6.8.0-45-generic", "k"),
                ("/boot/vmlinuz-6.8.0-41-generic", "k"),
            ],
        );
        let k = read(&root);
        let list: Vec<(&str, &str)> = k
            .installed
            .iter()
            .map(|i| (i.version.as_str(), i.status.as_str()))
            .collect();
        assert_eq!(
            list,
            [
                ("6.8.0-47-generic", "running"),
                ("6.8.0-45-generic", "fallback"),
                ("6.8.0-41-generic", "old")
            ]
        );
        assert_eq!(k.installed[2].size, 4);
    }
}
