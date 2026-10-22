// SPDX-License-Identifier: Apache-2.0

//! Package install history from `/var/log/apt/history.log`.

use serde::Serialize;

use crate::collect::SystemRoot;

/// How many runs to return.
const KEEP: usize = 100;

/// One apt run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AptRun {
    /// Start time as written in the log, `YYYY-MM-DD  HH:MM:SS`, local time.
    pub start: String,
    /// Who or what started it: "unattended-upgrades", "Software Updater", or the command.
    pub source: String,
    /// Packages installed, upgraded, or removed.
    pub packages: usize,
    /// Names of the packages, with what happened: "upgrade openssl".
    pub changes: Vec<String>,
    /// True when apt logged an error.
    pub failed: bool,
}

fn source(commandline: &str) -> String {
    if commandline.contains("unattended-upgrade") {
        "unattended-upgrades".into()
    } else if commandline.is_empty()
        || commandline.contains("packagekit")
        || commandline.contains("aptd")
    {
        "Software Updater".into()
    } else {
        commandline.trim_start_matches("/usr/bin/").to_owned()
    }
}

/// Reads the log, newest run first.
pub fn parse(text: &str) -> Vec<AptRun> {
    let mut runs = Vec::new();
    for block in text.split("\n\n") {
        let mut run = AptRun {
            start: String::new(),
            source: String::new(),
            packages: 0,
            changes: Vec::new(),
            failed: false,
        };
        let mut commandline = String::new();
        for line in block.lines() {
            let Some((key, value)) = line.split_once(": ") else {
                continue;
            };
            match key {
                "Start-Date" => run.start = value.trim().to_owned(),
                "Commandline" => commandline = value.trim().to_owned(),
                "Error" => run.failed = true,
                "Install" | "Upgrade" | "Remove" | "Purge" | "Downgrade" | "Reinstall" => {
                    // "pkg:amd64 (1.0, 1.1), other:amd64 (2.0)"; versions may hold commas, so split on "), ".
                    for item in value.split("), ") {
                        let name = item.split([':', ' ']).next().unwrap_or("");
                        if !name.is_empty() {
                            run.packages += 1;
                            run.changes.push(format!("{} {name}", key.to_lowercase()));
                        }
                    }
                }
                _ => {}
            }
        }
        if !run.start.is_empty() {
            run.source = source(&commandline);
            runs.push(run);
        }
    }
    runs.reverse();
    runs.truncate(KEEP);
    runs
}

/// One package change, with versions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PackageChange {
    pub start: String,
    /// install, upgrade, remove, purge, downgrade, or reinstall.
    pub action: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    pub source: String,
}

/// Every package change in the log, newest first.
pub fn changes(text: &str) -> Vec<PackageChange> {
    let mut out = Vec::new();
    for block in text.split("\n\n") {
        let (mut start, mut commandline) = (String::new(), String::new());
        let mut items = Vec::new();
        for line in block.lines() {
            let Some((key, value)) = line.split_once(": ") else {
                continue;
            };
            match key {
                "Start-Date" => start = value.trim().to_owned(),
                "Commandline" => commandline = value.trim().to_owned(),
                "Install" | "Upgrade" | "Remove" | "Purge" | "Downgrade" | "Reinstall" => {
                    for item in value.split("), ") {
                        // "pkg:amd64 (old, new" or "pkg:amd64 (ver, automatic" or "pkg:amd64 (ver"
                        let (name, versions) = item.split_once(" (").unwrap_or((item, ""));
                        let name = name.split(':').next().unwrap_or(name).trim().to_owned();
                        let v: Vec<&str> = versions
                            .trim_end_matches(')')
                            .split(", ")
                            .filter(|x| !x.is_empty() && *x != "automatic")
                            .collect();
                        let (from, to) = match (key, v.as_slice()) {
                            ("Upgrade" | "Downgrade", [a, b, ..]) => {
                                (Some((*a).to_owned()), Some((*b).to_owned()))
                            }
                            ("Install" | "Reinstall", [a, ..]) => (None, Some((*a).to_owned())),
                            (_, [a, ..]) => (Some((*a).to_owned()), None),
                            _ => (None, None),
                        };
                        if !name.is_empty() {
                            items.push((key.to_lowercase(), name, from, to));
                        }
                    }
                }
                _ => {}
            }
        }
        if !start.is_empty() {
            let source = source(&commandline);
            out.extend(
                items
                    .into_iter()
                    .map(|(action, name, from, to)| PackageChange {
                        start: start.clone(),
                        action,
                        name,
                        from,
                        to,
                        source: source.clone(),
                    }),
            );
        }
    }
    out.reverse();
    out.truncate(2000);
    out
}

/// Reads the current log. Rotated, compressed logs are not read.
pub fn read(root: &SystemRoot) -> Vec<AptRun> {
    parse(&root.read("/var/log/apt/history.log").unwrap_or_default())
}

/// Every package change in the current log, newest first.
pub fn read_changes(root: &SystemRoot) -> Vec<PackageChange> {
    changes(&root.read("/var/log/apt/history.log").unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOG: &str = "
Start-Date: 2026-10-20  18:40:01
Commandline: aptdaemon role='role-commit-packages' sender=':1.123'
Upgrade: linux-firmware:amd64 (20240318.git3b128b60-0ubuntu2.5, 20240318.git3b128b60-0ubuntu2.6), git:amd64 (1:2.43.0-1ubuntu7.1, 1:2.43.0-1ubuntu7.2)
End-Date: 2026-10-20  18:41:00

Start-Date: 2026-10-23  00:34:58
Commandline: /usr/bin/unattended-upgrade
Upgrade: openssl:amd64 (3.0.13-0ubuntu3.4, 3.0.13-0ubuntu3.5)
Error: Sub-process /usr/bin/dpkg returned an error code (1)
End-Date: 2026-10-23  00:35:10

Start-Date: 2026-10-23  09:00:00
Commandline: apt install htop
Requested-By: riya (1000)
Install: htop:amd64 (3.3.0-4build1)
End-Date: 2026-10-23  09:00:02
";

    #[test]
    fn lists_each_package_change_with_versions() {
        let c = changes(LOG);
        assert_eq!(c.len(), 4);
        assert_eq!(
            (c[0].action.as_str(), c[0].name.as_str(), c[0].to.as_deref()),
            ("install", "htop", Some("3.3.0-4build1"))
        );
        assert_eq!(
            (c[1].name.as_str(), c[1].from.as_deref(), c[1].to.as_deref()),
            (
                "openssl",
                Some("3.0.13-0ubuntu3.4"),
                Some("3.0.13-0ubuntu3.5")
            )
        );
        assert_eq!(c[3].source, "Software Updater");
    }

    #[test]
    fn reads_runs_newest_first() {
        let runs = parse(LOG);
        assert_eq!(runs.len(), 3);
        assert_eq!(
            (runs[0].source.as_str(), runs[0].packages),
            ("apt install htop", 1)
        );
        assert_eq!(runs[0].changes, ["install htop"]);
        assert!(runs[1].failed);
        assert_eq!(runs[1].source, "unattended-upgrades");
        assert_eq!(
            (runs[2].source.as_str(), runs[2].packages),
            ("Software Updater", 2)
        );
    }
}
