// SPDX-License-Identifier: Apache-2.0

//! Security in depth for the Expert level (docs/01-product/expert-coverage.md, section 14): the
//! AppArmor state of every running process, and whether automatic security updates are on and
//! really install (their settings and the last runs from their log). No admin rights.

use serde::Serialize;

use crate::collect::SystemRoot;

/// One AppArmor profile and the processes running under it.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Confinement {
    /// The profile, or `unconfined`.
    pub profile: String,
    /// `enforce`, `complain`, `unconfined` (a profile that allows everything), or `none`.
    pub mode: String,
    pub processes: usize,
    /// Up to 5 process names under it.
    pub names: Vec<String>,
}

/// One run of unattended-upgrades, from its log.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct UpgradeRun {
    /// `2026-10-22 10:43:11`.
    pub at: String,
    /// `INFO`, `WARNING`, or `ERROR`.
    pub level: String,
    pub message: String,
}

/// What `security_detail` answers.
#[derive(Debug, Default, Serialize)]
pub struct SecurityDetail {
    pub confinement: Vec<Confinement>,
    /// `APT::Periodic` settings, such as `Unattended-Upgrade` = `1`.
    pub periodic: Vec<(String, String)>,
    pub unattended_installed: bool,
    /// The last lines of the unattended-upgrades log that say what a run did, newest first.
    /// Missing when the log cannot be read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgrade_runs: Option<Vec<UpgradeRun>>,
}

/// `snap.firefox.firefox (enforce)` gives the profile and its mode.
pub fn label(text: &str) -> (String, String) {
    let text = text.trim_end_matches('\0').trim();
    if text == "unconfined" || text.is_empty() {
        return ("unconfined".into(), "none".into());
    }
    match text.rsplit_once(" (") {
        Some((profile, mode)) => (profile.to_owned(), mode.trim_end_matches(')').to_owned()),
        None => (text.to_owned(), "enforce".into()),
    }
}

/// `APT::Periodic::Unattended-Upgrade "1";` lines of the apt configuration files.
pub fn periodic(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            let rest = l.strip_prefix("APT::Periodic::")?;
            let (key, value) = rest.split_once(char::is_whitespace)?;
            Some((
                key.to_owned(),
                value
                    .trim()
                    .trim_end_matches(';')
                    .trim_matches('"')
                    .to_owned(),
            ))
        })
        .collect()
}

/// The lines of an unattended-upgrades log that tell what a run did, newest first.
pub fn upgrade_runs(log: &str) -> Vec<UpgradeRun> {
    const MEANINGFUL: &[&str] = &[
        "All upgrades installed",
        "No packages found",
        "Packages that will be upgraded",
        "metered",
        "stopping",
        "error",
        "Error",
        "Initial blacklist",
        "Allowed origins",
    ];
    let mut out: Vec<UpgradeRun> = log
        .lines()
        .filter_map(|l| {
            // "2026-10-22 10:43:11,096 WARNING System is on metered connection, stopping"
            let (stamp, rest) = l.get(..19).zip(l.get(19..))?;
            let rest = rest
                .trim_start_matches(|c: char| c == ',' || c.is_ascii_digit())
                .trim();
            let (level, message) = rest.split_once(' ')?;
            if !["INFO", "WARNING", "ERROR"].contains(&level) {
                return None;
            }
            if level == "INFO" && !MEANINGFUL.iter().any(|m| message.contains(m)) {
                return None;
            }
            if message.starts_with("Allowed origins") || message.starts_with("Initial blacklist") {
                return None;
            }
            Some(UpgradeRun {
                at: stamp.to_owned(),
                level: level.to_owned(),
                message: message.to_owned(),
            })
        })
        .collect();
    out.reverse();
    out.truncate(15);
    out
}

/// Reads `security_detail`.
pub fn read(root: &SystemRoot) -> SecurityDetail {
    let mut groups: Vec<Confinement> = Vec::new();
    for e in std::fs::read_dir(root.path("/proc"))
        .into_iter()
        .flatten()
        .flatten()
    {
        let Some(pid) = e.file_name().to_str().and_then(|n| n.parse::<u32>().ok()) else {
            continue;
        };
        let Some(text) = root
            .read(&format!("/proc/{pid}/attr/apparmor/current"))
            .or_else(|_| root.read(&format!("/proc/{pid}/attr/current")))
            .ok()
        else {
            continue;
        };
        let (profile, mode) = label(&text);
        let name = root
            .read(&format!("/proc/{pid}/comm"))
            .map(|c| c.trim().to_owned())
            .unwrap_or_default();
        match groups
            .iter_mut()
            .find(|g| g.profile == profile && g.mode == mode)
        {
            Some(g) => {
                g.processes += 1;
                if g.names.len() < 5 && !g.names.contains(&name) {
                    g.names.push(name);
                }
            }
            None => groups.push(Confinement {
                profile,
                mode,
                processes: 1,
                names: vec![name],
            }),
        }
    }
    groups.sort_by(|a, b| {
        b.processes
            .cmp(&a.processes)
            .then_with(|| a.profile.cmp(&b.profile))
    });
    let mut files: Vec<String> = std::fs::read_dir(root.path("/etc/apt/apt.conf.d"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().to_str().map(str::to_owned))
        .collect();
    files.sort();
    // Later files win, as apt reads them in order.
    let mut settings: Vec<(String, String)> = Vec::new();
    for f in files {
        for (k, v) in periodic(
            &root
                .read(&format!("/etc/apt/apt.conf.d/{f}"))
                .unwrap_or_default(),
        ) {
            settings.retain(|(key, _)| *key != k);
            settings.push((k, v));
        }
    }
    SecurityDetail {
        confinement: groups,
        periodic: settings,
        unattended_installed: root.path("/usr/bin/unattended-upgrade").exists(),
        upgrade_runs: root
            .read("/var/log/unattended-upgrades/unattended-upgrades.log")
            .ok()
            .map(|t| upgrade_runs(&t)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn labels_settings_and_log() {
        assert_eq!(
            label("snap.firefox.firefox (enforce)\n"),
            ("snap.firefox.firefox".into(), "enforce".into())
        );
        assert_eq!(
            label("vscode (unconfined)"),
            ("vscode".into(), "unconfined".into())
        );
        assert_eq!(label("unconfined\n"), ("unconfined".into(), "none".into()));
        assert_eq!(
            periodic(
                "APT::Periodic::Update-Package-Lists \"1\";\nAPT::Periodic::Unattended-Upgrade \"0\";\n// comment\n"
            ),
            [
                ("Update-Package-Lists".into(), "1".into()),
                ("Unattended-Upgrade".into(), "0".into())
            ]
        );
        let runs = upgrade_runs(
            "2026-10-21 10:36:50,445 INFO Starting unattended upgrades script\n2026-10-21 10:36:50,445 INFO Allowed origins are: o=Ubuntu\n2026-10-21 10:36:50,446 WARNING System is on metered connection, stopping\n2026-10-22 06:10:00,100 INFO All upgrades installed\n",
        );
        assert_eq!(runs.len(), 2);
        assert_eq!(
            (runs[0].at.as_str(), runs[0].level.as_str()),
            ("2026-10-22 06:10:00", "INFO")
        );
        assert_eq!(runs[1].message, "System is on metered connection, stopping");
    }

    #[test]
    fn processes_are_grouped_by_profile() {
        let root = fixture_root(
            "security-detail",
            &[
                ("/proc/1/attr/apparmor/current", "unconfined\n"),
                ("/proc/1/comm", "systemd\n"),
                (
                    "/proc/2/attr/apparmor/current",
                    "snap.firefox.firefox (enforce)\n",
                ),
                ("/proc/2/comm", "firefox\n"),
                (
                    "/proc/3/attr/apparmor/current",
                    "snap.firefox.firefox (enforce)\n",
                ),
                ("/proc/3/comm", "firefox\n"),
                (
                    "/etc/apt/apt.conf.d/20auto-upgrades",
                    "APT::Periodic::Unattended-Upgrade \"1\";\n",
                ),
                (
                    "/etc/apt/apt.conf.d/99local",
                    "APT::Periodic::Unattended-Upgrade \"0\";\n",
                ),
            ],
        );
        let d = read(&root);
        assert_eq!(
            d.confinement[0],
            Confinement {
                profile: "snap.firefox.firefox".into(),
                mode: "enforce".into(),
                processes: 2,
                names: vec!["firefox".into()]
            }
        );
        assert_eq!(
            d.periodic,
            [("Unattended-Upgrade".into(), "0".into())],
            "a later file wins"
        );
        assert!(d.upgrade_runs.is_none());
    }
}
