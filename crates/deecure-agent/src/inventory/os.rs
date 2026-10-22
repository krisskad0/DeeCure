// SPDX-License-Identifier: Apache-2.0

//! Details for the System page: release and its support dates, virtualization,
//! Ubuntu Pro, time, and locale. All read from files anyone can read.

use serde::Serialize;
use serde_json::Value;

use crate::collect::SystemRoot;

/// Release dates from `/usr/share/distro-info/ubuntu.csv` (package `distro-info-data`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Lifecycle {
    /// Release day, `YYYY-MM-DD`.
    pub release: String,
    /// End of standard support.
    pub eol: String,
    /// End of extended security maintenance (Ubuntu Pro), for LTS releases.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eol_esm: Option<String>,
}

/// The `os` inventory.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct OsDetails {
    /// Such as "Ubuntu 24.04.1 LTS (Noble Numbat)".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distribution: Option<String>,
    /// Support dates of this release.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<Lifecycle>,
    /// "None (bare metal)", "Virtual machine (QEMU)", or "Container (docker)".
    pub virtualization: String,
    /// The first and last characters of `/etc/machine-id`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine_id: Option<String>,
    /// People with an open session.
    pub users_signed_in: usize,
    /// Ubuntu Pro: attached or not. Missing when the Pro client is not installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pro_attached: Option<bool>,
    /// Livepatch state from the Pro client, such as "enabled".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub livepatch: Option<String>,
    /// Such as "Asia/Kolkata".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    /// True when systemd-timesyncd has set the clock from a time server.
    pub ntp_synchronized: bool,
    /// The time servers configured, or Ubuntu's default.
    pub time_server: String,
    /// True when the hardware clock keeps local time instead of UTC.
    pub rtc_local: bool,
    /// Such as `LANG=en_IN.UTF-8`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// Such as "us" or "gb (extd)".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyboard: Option<String>,
}

/// Reads `KEY=value` or `KEY="value"` from a shell-style file.
fn field(text: &str, key: &str) -> Option<String> {
    text.lines()
        .find_map(|l| l.trim().strip_prefix(&format!("{key}=")))
        .map(|v| v.trim_matches('"').to_owned())
        .filter(|v| !v.is_empty())
}

/// Finds a release by its code name, such as `noble`.
pub fn lifecycle(csv: &str, codename: &str) -> Option<Lifecycle> {
    csv.lines().skip(1).find_map(|line| {
        let cols: Vec<&str> = line.split(',').collect();
        (cols.get(2) == Some(&codename)).then(|| Lifecycle {
            release: cols.get(4).copied().unwrap_or_default().to_owned(),
            eol: cols.get(5).copied().unwrap_or_default().to_owned(),
            eol_esm: cols
                .get(7)
                .filter(|v| !v.is_empty())
                .map(|v| (*v).to_owned()),
        })
    })
}

fn virtualization(root: &SystemRoot) -> String {
    if let Ok(kind) = root.read("/run/systemd/container") {
        return format!("Container ({})", kind.trim());
    }
    if root.path("/.dockerenv").exists() {
        return "Container (docker)".into();
    }
    let flags = root.read("/proc/cpuinfo").unwrap_or_default();
    let hypervisor = flags
        .lines()
        .filter(|l| l.starts_with("flags"))
        .any(|l| l.split_whitespace().any(|f| f == "hypervisor"));
    if !hypervisor {
        return "None (bare metal)".into();
    }
    let vendor = [
        root.read("/sys/class/dmi/id/sys_vendor"),
        root.read("/sys/class/dmi/id/product_name"),
    ]
    .into_iter()
    .flatten()
    .map(|s| s.trim().to_owned())
    .collect::<Vec<_>>()
    .join(" ");
    let name = [
        ("QEMU", "QEMU/KVM"),
        ("KVM", "QEMU/KVM"),
        ("VMware", "VMware"),
        ("VirtualBox", "VirtualBox"),
        ("innotek", "VirtualBox"),
        ("Microsoft", "Hyper-V"),
        ("Xen", "Xen"),
        ("Parallels", "Parallels"),
        ("Amazon EC2", "Amazon EC2"),
        ("Google", "Google Cloud"),
    ]
    .iter()
    .find(|(needle, _)| vendor.contains(needle))
    .map_or("unknown hypervisor", |(_, name)| name);
    format!("Virtual machine ({name})")
}

/// Reads the details.
pub fn read(root: &SystemRoot) -> OsDetails {
    let os = root.read("/etc/os-release").unwrap_or_default();
    let codename = field(&os, "VERSION_CODENAME");
    let pretty = field(&os, "PRETTY_NAME");
    let version = field(&os, "VERSION");
    // PRETTY_NAME lacks the code name on Ubuntu; VERSION has it, as in "24.04.1 LTS (Noble Numbat)".
    let distribution = match (&pretty, version.as_deref().and_then(|v| v.split_once('('))) {
        (Some(p), Some((_, name))) if !p.contains('(') => Some(format!("{p} ({}", name)),
        _ => pretty,
    };
    let pro: Option<Value> = root
        .read("/var/lib/ubuntu-advantage/status.json")
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok());
    let livepatch = pro.as_ref().and_then(|p| {
        p.get("services")?
            .as_array()?
            .iter()
            .find(|s| s.get("name").and_then(Value::as_str) == Some("livepatch"))?
            .get("status")?
            .as_str()
            .map(str::to_owned)
    });
    let timesyncd = root.read("/etc/systemd/timesyncd.conf").unwrap_or_default();
    let keyboard = root.read("/etc/default/keyboard").unwrap_or_default();
    let locale = ["/etc/default/locale", "/etc/locale.conf"]
        .iter()
        .find_map(|p| root.read(p).ok().and_then(|t| field(&t, "LANG")));
    OsDetails {
        lifecycle: codename
            .as_deref()
            .and_then(|c| lifecycle(&root.read("/usr/share/distro-info/ubuntu.csv").ok()?, c)),
        distribution,
        virtualization: virtualization(root),
        machine_id: root
            .read("/etc/machine-id")
            .ok()
            .map(|id| id.trim().to_owned())
            .filter(|id| id.len() > 10)
            .map(|id| format!("{}…{}", &id[..4], &id[id.len() - 4..])),
        users_signed_in: std::fs::read_dir(root.path("/run/systemd/users"))
            .map(|d| d.flatten().count())
            .unwrap_or(0),
        pro_attached: pro.as_ref().and_then(|p| p.get("attached")?.as_bool()),
        livepatch,
        timezone: root
            .read("/etc/timezone")
            .ok()
            .map(|t| t.trim().to_owned())
            .filter(|t| !t.is_empty())
            .or_else(|| {
                std::fs::read_link(root.path("/etc/localtime"))
                    .ok()
                    .and_then(|p| {
                        p.to_string_lossy()
                            .split_once("zoneinfo/")
                            .map(|(_, z)| z.to_owned())
                    })
            }),
        ntp_synchronized: root.path("/run/systemd/timesync/synchronized").exists(),
        time_server: field(&timesyncd, "NTP").unwrap_or_else(|| "ntp.ubuntu.com (default)".into()),
        rtc_local: root
            .read("/etc/adjtime")
            .is_ok_and(|t| t.lines().nth(2).is_some_and(|l| l.trim() == "LOCAL")),
        locale: locale.map(|l| format!("LANG={l}")),
        keyboard: field(&keyboard, "XKBLAYOUT").map(|layout| {
            match field(&keyboard, "XKBVARIANT") {
                Some(variant) => format!("{layout} ({variant})"),
                None => layout,
            }
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    const CSV: &str = "version,codename,series,created,release,eol,eol-server,eol-esm,eol-legacy
24.04 LTS,Noble Numbat,noble,2023-10-12,2024-04-25,2029-05-31,2029-05-31,2034-04-25,2039-04-28
24.10,Oracular Oriole,oracular,2024-04-25,2024-10-10,2025-07-10
";

    #[test]
    fn finds_the_release_dates() {
        let l = lifecycle(CSV, "noble").unwrap();
        assert_eq!(
            (l.release.as_str(), l.eol.as_str(), l.eol_esm.as_deref()),
            ("2024-04-25", "2029-05-31", Some("2034-04-25"))
        );
        assert_eq!(lifecycle(CSV, "oracular").unwrap().eol_esm, None);
        assert_eq!(lifecycle(CSV, "warty"), None);
    }

    #[test]
    fn reads_a_virtual_machine_with_local_rtc() {
        let root = fixture_root(
            "os",
            &[
                (
                    "/etc/os-release",
                    "PRETTY_NAME=\"Ubuntu 24.04.1 LTS\"\nVERSION=\"24.04.1 LTS (Noble Numbat)\"\nVERSION_CODENAME=noble\n",
                ),
                ("/usr/share/distro-info/ubuntu.csv", CSV),
                ("/proc/cpuinfo", "flags\t: fpu vme hypervisor\n"),
                ("/sys/class/dmi/id/sys_vendor", "QEMU\n"),
                ("/etc/machine-id", "4f1c9d00000000000000000000a72e\n"),
                ("/etc/adjtime", "0.0 0 0\n0\nLOCAL\n"),
                (
                    "/etc/default/keyboard",
                    "XKBLAYOUT=\"gb\"\nXKBVARIANT=\"extd\"\n",
                ),
                ("/etc/default/locale", "LANG=en_IN.UTF-8\n"),
                ("/etc/timezone", "Asia/Kolkata\n"),
                (
                    "/var/lib/ubuntu-advantage/status.json",
                    r#"{"attached": false, "services": [{"name": "livepatch", "status": "disabled"}]}"#,
                ),
            ],
        );
        let os = read(&root);
        assert_eq!(
            os.distribution.as_deref(),
            Some("Ubuntu 24.04.1 LTS (Noble Numbat)")
        );
        assert_eq!(os.virtualization, "Virtual machine (QEMU/KVM)");
        assert_eq!(os.machine_id.as_deref(), Some("4f1c…a72e"));
        assert!(os.rtc_local);
        assert_eq!(os.keyboard.as_deref(), Some("gb (extd)"));
        assert_eq!(os.locale.as_deref(), Some("LANG=en_IN.UTF-8"));
        assert_eq!(
            (os.pro_attached, os.livepatch.as_deref()),
            (Some(false), Some("disabled"))
        );
        assert_eq!(os.time_server, "ntp.ubuntu.com (default)");
        assert!(!os.ntp_synchronized);
    }
}
