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

/// Reads the current log. Rotated, compressed logs are not read.
pub fn read(root: &SystemRoot) -> Vec<AptRun> {
    parse(&root.read("/var/log/apt/history.log").unwrap_or_default())
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
