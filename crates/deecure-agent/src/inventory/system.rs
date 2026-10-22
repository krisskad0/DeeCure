// SPDX-License-Identifier: Apache-2.0

//! System facts: OS, kernel, machine, CPU, memory, uptime.

use serde::Serialize;

use crate::collect::SystemRoot;

/// Facts about this computer. Values that cannot be read are left out.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct SystemInfo {
    /// Host name.
    pub hostname: Option<String>,
    /// OS name, such as "Ubuntu 24.04.5 LTS".
    pub os: Option<String>,
    /// OS version, such as "24.04".
    pub os_version: Option<String>,
    /// Kernel release, such as "6.8.0-47-generic".
    pub kernel: Option<String>,
    /// CPU architecture, such as "x86_64".
    pub arch: String,
    /// Maker and model, such as "Dell Inc. Latitude 3450".
    pub model: Option<String>,
    /// CPU model name.
    pub cpu: Option<String>,
    /// Logical CPUs (threads).
    pub cpu_threads: usize,
    /// All memory, in bytes.
    pub memory_total: Option<u64>,
    /// Seconds since boot.
    pub uptime_seconds: Option<f64>,
    /// True when an update asks for a restart.
    pub reboot_required: bool,
    /// The packages that asked for the restart.
    pub reboot_packages: Vec<String>,
}

/// Reads the facts.
pub fn read(root: &SystemRoot) -> SystemInfo {
    let text = |path: &str| {
        root.read(path)
            .ok()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
    };
    let os_release = root.read("/etc/os-release").unwrap_or_default();
    let os_field = |key: &str| {
        os_release
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{key}=")))
            .map(|v| v.trim_matches('"').to_owned())
    };
    let cpuinfo = root.read("/proc/cpuinfo").unwrap_or_default();
    let model = [
        text("/sys/class/dmi/id/sys_vendor"),
        text("/sys/class/dmi/id/product_name"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ");
    SystemInfo {
        hostname: text("/proc/sys/kernel/hostname"),
        os: os_field("PRETTY_NAME"),
        os_version: os_field("VERSION_ID"),
        kernel: text("/proc/sys/kernel/osrelease"),
        arch: std::env::consts::ARCH.to_owned(),
        model: (!model.is_empty()).then_some(model),
        cpu: cpuinfo
            .lines()
            .find_map(|line| {
                line.strip_prefix("model name")
                    .and_then(|rest| rest.split_once(':'))
            })
            .map(|(_, name)| name.trim().to_owned()),
        cpu_threads: cpuinfo
            .lines()
            .filter(|line| line.starts_with("processor"))
            .count(),
        memory_total: root.read("/proc/meminfo").ok().and_then(|m| {
            m.lines()
                .find_map(|line| line.strip_prefix("MemTotal:"))
                .and_then(|v| v.trim().trim_end_matches("kB").trim().parse::<u64>().ok())
                .map(|kb| kb * 1024)
        }),
        uptime_seconds: text("/proc/uptime")
            .and_then(|u| u.split_whitespace().next()?.parse().ok()),
        reboot_required: root.path("/run/reboot-required").exists(),
        reboot_packages: text("/run/reboot-required.pkgs")
            .map(|p| p.lines().map(str::to_owned).collect())
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn reads_facts_and_leaves_out_what_is_missing() {
        let root = fixture_root(
            "system-info",
            &[
                (
                    "/etc/os-release",
                    "PRETTY_NAME=\"Ubuntu 24.04.1 LTS\"\nVERSION_ID=\"24.04\"\n",
                ),
                ("/proc/sys/kernel/hostname", "riya-laptop\n"),
                ("/proc/sys/kernel/osrelease", "6.8.0-47-generic\n"),
                (
                    "/proc/cpuinfo",
                    "processor\t: 0\nmodel name\t: AMD Ryzen 5 4500U\nprocessor\t: 1\nmodel name\t: AMD Ryzen 5 4500U\n",
                ),
                ("/proc/meminfo", "MemTotal:        7969792 kB\n"),
                ("/proc/uptime", "45360.12 300000.00\n"),
                ("/run/reboot-required", "*** System restart required ***\n"),
                (
                    "/run/reboot-required.pkgs",
                    "libc6\nlinux-image-6.8.0-47-generic\n",
                ),
            ],
        );
        let info = read(&root);
        assert_eq!(info.os.as_deref(), Some("Ubuntu 24.04.1 LTS"));
        assert_eq!(info.os_version.as_deref(), Some("24.04"));
        assert_eq!(info.cpu.as_deref(), Some("AMD Ryzen 5 4500U"));
        assert_eq!(info.cpu_threads, 2);
        assert_eq!(info.memory_total, Some(7_969_792 * 1024));
        assert_eq!(info.uptime_seconds, Some(45360.12));
        assert!(info.reboot_required);
        assert_eq!(info.reboot_packages.len(), 2);
        assert_eq!(info.model, None, "no DMI files in this fixture");
    }
}
