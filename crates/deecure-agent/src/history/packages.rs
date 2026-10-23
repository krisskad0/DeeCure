// SPDX-License-Identifier: Apache-2.0

//! Package changes: apt runs from `/var/log/apt/history.log`, packages installed with
//! `dpkg -i` from `/var/log/dpkg.log`, and snap and Flatpak apps (by comparing the
//! list of installed apps with the last one).
//!
//! The newest rotated log (`.1` or `.1.gz`) is read too, so changes written just before
//! a log was rotated are not lost.

use std::collections::BTreeMap;

use deecure_core::{Event, EventKind, Timestamp, local};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// One package change in an apt run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// install, upgrade, downgrade, remove, purge, or reinstall.
    pub action: String,
    pub name: String,
    pub arch: String,
    pub from: Option<String>,
    pub to: Option<String>,
}

/// One apt run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AptRun {
    pub start: i64,
    pub end: i64,
    /// "unattended-upgrades", "Software Updater", or the command.
    pub source: String,
    pub failed: bool,
    pub items: Vec<Item>,
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

/// Reads apt's history log. Times are local, as apt writes them.
pub fn parse_apt(text: &str, tz: &local::TimeZone) -> Vec<AptRun> {
    let mut runs = Vec::new();
    for block in text.split("\n\n") {
        let (mut start, mut end, mut commandline, mut failed) = (None, None, String::new(), false);
        let mut items = Vec::new();
        for line in block.lines() {
            let Some((key, value)) = line.split_once(": ") else {
                continue;
            };
            match key {
                "Start-Date" => start = local::parse_in(value, tz).map(Timestamp::as_millis),
                "End-Date" => end = local::parse_in(value, tz).map(Timestamp::as_millis),
                "Commandline" => commandline = value.trim().to_owned(),
                "Error" => failed = true,
                "Install" | "Upgrade" | "Downgrade" | "Remove" | "Purge" | "Reinstall" => {
                    // "pkg:amd64 (old, new), other:amd64 (2.0, automatic)"; versions may hold
                    // commas, so items are split on "), ".
                    for item in value.split("), ") {
                        let (name, versions) = item.split_once(" (").unwrap_or((item, ""));
                        let (name, arch) = name.trim().split_once(':').unwrap_or((name.trim(), ""));
                        let v: Vec<&str> = versions
                            .trim_end_matches(')')
                            .split(", ")
                            .filter(|x| !x.is_empty() && *x != "automatic")
                            .collect();
                        let (from, to) = match (key, v.as_slice()) {
                            ("Upgrade" | "Downgrade", [a, b, ..]) => (Some(*a), Some(*b)),
                            ("Install" | "Reinstall", [a, ..]) => (None, Some(*a)),
                            (_, [a, ..]) => (Some(*a), None),
                            _ => (None, None),
                        };
                        if !name.is_empty() {
                            items.push(Item {
                                action: key.to_lowercase(),
                                name: name.to_owned(),
                                arch: arch.to_owned(),
                                from: from.map(str::to_owned),
                                to: to.map(str::to_owned),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        if let Some(start) = start {
            runs.push(AptRun {
                start,
                // A run cut short (power loss) has no end; give it an hour.
                end: end.unwrap_or(start + 3_600_000).max(start),
                source: source(&commandline),
                failed,
                items,
            });
        }
    }
    runs
}

/// One line of `/var/log/dpkg.log` that changed a package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DpkgChange {
    pub at: i64,
    pub item: Item,
}

/// Reads dpkg's log: `2026-10-23 09:00:01 upgrade openssl:amd64 3.0.13-0ubuntu3.4 3.0.13-0ubuntu3.5`.
pub fn parse_dpkg(text: &str, tz: &local::TimeZone) -> Vec<DpkgChange> {
    let mut out = Vec::new();
    for line in text.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        let [date, time, action, package, old, new] = f.as_slice() else {
            continue;
        };
        if !matches!(*action, "install" | "upgrade" | "remove" | "purge") {
            continue;
        }
        let Some(at) = local::parse_in(&format!("{date} {time}"), tz) else {
            continue;
        };
        let version = |v: &str| (v != "<none>").then(|| v.to_owned());
        let (name, arch) = package.split_once(':').unwrap_or((package, ""));
        out.push(DpkgChange {
            at: at.as_millis(),
            item: Item {
                action: (*action).to_owned(),
                name: name.to_owned(),
                arch: arch.to_owned(),
                from: version(old),
                to: version(new),
            },
        });
    }
    out
}

/// Changes made with `dpkg` directly, outside every apt run (apt logs its own). A purge
/// right after a remove of the same package is one change.
pub fn outside_apt(changes: Vec<DpkgChange>, runs: &[AptRun]) -> Vec<DpkgChange> {
    let inside = |at: i64| {
        runs.iter()
            .any(|r| at >= r.start - 2_000 && at <= r.end + 2_000)
    };
    let mut out: Vec<DpkgChange> = Vec::new();
    for c in changes.into_iter().filter(|c| !inside(c.at)) {
        if c.item.action == "purge"
            && let Some(prev) = out.iter_mut().rev().find(|p| {
                p.item.name == c.item.name && p.item.action == "remove" && c.at - p.at <= 10_000
            })
        {
            prev.item.action = "purge".into();
            continue;
        }
        out.push(c);
    }
    out
}

fn kind_of(action: &str) -> EventKind {
    match action {
        "install" | "reinstall" => EventKind::PackageInstalled,
        "remove" | "purge" => EventKind::PackageRemoved,
        _ => EventKind::PackageUpgraded,
    }
}

/// A short title: "Upgraded openssl 3.0.13-0ubuntu3.4 → 3.0.13-0ubuntu3.5".
pub fn title(item: &Item, format: &str) -> String {
    let what = match format {
        "snap" => format!("snap {}", item.name),
        "flatpak" => format!("Flatpak app {}", item.name),
        _ => item.name.clone(),
    };
    fn v(v: &Option<String>) -> Option<&str> {
        v.as_deref().filter(|v| !v.is_empty())
    }
    match (item.action.as_str(), v(&item.from), v(&item.to)) {
        ("install", _, Some(to)) => format!("Installed {what} {to}"),
        ("reinstall", _, Some(to)) => format!("Reinstalled {what} {to}"),
        ("upgrade", Some(a), Some(b)) => format!("Upgraded {what} {a} → {b}"),
        ("downgrade", Some(a), Some(b)) => format!("Downgraded {what} {a} → {b}"),
        ("purge", _, _) => format!("Removed {what} and its settings"),
        ("remove", Some(a), _) => format!("Removed {what} {a}"),
        ("upgrade", None, Some(to)) => format!("Updated {what} {to}"),
        ("install", _, _) => format!("Installed {what}"),
        ("upgrade", _, _) => format!("Updated {what}"),
        _ => format!("Removed {what}"),
    }
}

/// The event for one package change.
pub fn event(
    item: &Item,
    at: i64,
    source: &str,
    format: &str,
    run: Option<i64>,
    failed: bool,
) -> Event {
    let mut data = json!({
        "action": item.action, "format": format,
        "from": item.from, "to": item.to,
    });
    if let Some(run) = run {
        data["run"] = json!(run);
    }
    if failed {
        data["failed"] = json!(true);
    }
    // A new kernel matters when something breaks right after it.
    if item.name.starts_with("linux-image-") {
        data["kernel"] = json!(true);
    }
    Event {
        id: None,
        kind: kind_of(&item.action),
        at: Timestamp(at),
        title: title(item, format),
        detail: None,
        subject: Some(item.name.clone()),
        source: source.to_owned(),
        data: Some(data),
    }
}

/// Every apt change as `(key, event)`.
pub fn apt_events(runs: &[AptRun]) -> Vec<(String, Event)> {
    runs.iter()
        .flat_map(|r| {
            r.items.iter().map(move |i| {
                (
                    format!("apt:{}:{}:{}:{}", r.start, i.action, i.name, i.arch),
                    event(i, r.start, &r.source, "deb", Some(r.start), r.failed),
                )
            })
        })
        .collect()
}

/// Every direct dpkg change as `(key, event)`.
pub fn dpkg_events(changes: &[DpkgChange]) -> Vec<(String, Event)> {
    changes
        .iter()
        .map(|c| {
            (
                format!(
                    "dpkg:{}:{}:{}:{}",
                    c.at, c.item.action, c.item.name, c.item.arch
                ),
                event(&c.item, c.at, "dpkg", "deb", None, false),
            )
        })
        .collect()
}

/// An installed snap or Flatpak app: its version and what tells two builds apart
/// (a snap's revision, a Flatpak's commit).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct App {
    pub version: String,
    pub build: String,
}

/// Reads `snap list`: Name, Version, Rev, Tracking, Publisher, Notes.
pub fn parse_snaps(text: &str) -> BTreeMap<String, App> {
    text.lines()
        .skip(1)
        .filter_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            Some((
                (*f.first()?).to_owned(),
                App {
                    version: (*f.get(1)?).to_owned(),
                    build: (*f.get(2)?).to_owned(),
                },
            ))
        })
        .collect()
}

/// Reads `flatpak list --app --columns=application,version,branch,installation,active`.
pub fn parse_flatpaks(text: &str) -> BTreeMap<String, App> {
    text.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            let [app, version, branch, installation, active] = f.as_slice() else {
                return None;
            };
            Some((
                format!("{app}/{branch}/{installation}"),
                App {
                    version: (*version).to_owned(),
                    build: (*active).to_owned(),
                },
            ))
        })
        .collect()
}

/// What changed between two app lists, as package items.
pub fn compare(old: &BTreeMap<String, App>, new: &BTreeMap<String, App>) -> Vec<Item> {
    let mut out = Vec::new();
    let name = |key: &str| key.split('/').next().unwrap_or(key).to_owned();
    for (key, app) in new {
        match old.get(key) {
            None => out.push(Item {
                action: "install".into(),
                name: name(key),
                arch: String::new(),
                from: None,
                to: Some(app.version.clone()),
            }),
            // A new build with the same version (a snap revision, a Flatpak commit) has no "from".
            Some(was) if was != app => out.push(Item {
                action: "upgrade".into(),
                name: name(key),
                arch: String::new(),
                from: (was.version != app.version).then(|| was.version.clone()),
                to: Some(app.version.clone()),
            }),
            Some(_) => {}
        }
    }
    for (key, was) in old {
        if !new.contains_key(key) {
            out.push(Item {
                action: "remove".into(),
                name: name(key),
                arch: String::new(),
                from: Some(was.version.clone()),
                to: None,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tz() -> local::TimeZone {
        local::TimeZone::UTC
    }

    const APT: &str = "
Start-Date: 2026-10-20  18:40:01
Commandline: aptdaemon role='role-commit-packages' sender=':1.123'
Upgrade: linux-image-generic:amd64 (6.8.0-45.45, 6.8.0-47.47), git:amd64 (1:2.43.0-1ubuntu7.1, 1:2.43.0-1ubuntu7.2)
Install: linux-image-6.8.0-47-generic:amd64 (6.8.0-47.47, automatic)
End-Date: 2026-10-20  18:41:00

Start-Date: 2026-10-23  00:34:58
Commandline: /usr/bin/unattended-upgrade
Upgrade: openssl:amd64 (3.0.13-0ubuntu3.4, 3.0.13-0ubuntu3.5)
Error: Sub-process /usr/bin/dpkg returned an error code (1)
End-Date: 2026-10-23  00:35:10

Start-Date: 2026-10-23  09:00:00
Commandline: apt purge thunderbird
Requested-By: riya (1000)
Purge: thunderbird:amd64 (2:1.0)
";

    #[test]
    fn reads_apt_runs_with_versions_and_sources() {
        let runs = parse_apt(APT, &tz());
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0].source, "Software Updater");
        assert_eq!(runs[0].end - runs[0].start, 59_000);
        assert_eq!(runs[0].items.len(), 3);
        assert_eq!(
            runs[0].items[1],
            Item {
                action: "upgrade".into(),
                name: "git".into(),
                arch: "amd64".into(),
                from: Some("1:2.43.0-1ubuntu7.1".into()),
                to: Some("1:2.43.0-1ubuntu7.2".into())
            }
        );
        assert!(runs[1].failed);
        assert_eq!(
            runs[2].end - runs[2].start,
            3_600_000,
            "no End-Date: an hour"
        );

        let events = apt_events(&runs);
        assert_eq!(events.len(), 5);
        let (key, kernel) = &events[2];
        assert!(
            key.starts_with("apt:") && key.ends_with(":install:linux-image-6.8.0-47-generic:amd64")
        );
        assert_eq!(
            kernel.title,
            "Installed linux-image-6.8.0-47-generic 6.8.0-47.47"
        );
        assert_eq!(kernel.data.as_ref().unwrap()["kernel"], true);
        assert_eq!(kernel.data.as_ref().unwrap()["run"], runs[0].start);
        assert_eq!(events[4].1.title, "Removed thunderbird and its settings");
        assert_eq!(events[4].1.kind, EventKind::PackageRemoved);
        assert_eq!(events[3].1.data.as_ref().unwrap()["failed"], true);
    }

    #[test]
    fn dpkg_changes_outside_apt_runs_are_kept() {
        let log = "2026-10-20 18:40:30 upgrade git:amd64 1:2.43.0-1ubuntu7.1 1:2.43.0-1ubuntu7.2
2026-10-20 18:40:31 status installed git:amd64 1:2.43.0-1ubuntu7.2
2026-10-22 11:00:00 install google-chrome-stable:amd64 <none> 130.0.6723.58-1
2026-10-22 12:00:00 remove zoom:amd64 6.2.0 <none>
2026-10-22 12:00:03 purge zoom:amd64 6.2.0 <none>
garbage line
";
        let runs = parse_apt(APT, &tz());
        let all = parse_dpkg(log, &tz());
        assert_eq!(all.len(), 4);
        let own = outside_apt(all, &runs);
        assert_eq!(own.len(), 2, "{own:?}");
        assert_eq!(own[0].item.name, "google-chrome-stable");
        assert_eq!(own[1].item.action, "purge");
        let events = dpkg_events(&own);
        assert_eq!(
            events[0].1.title,
            "Installed google-chrome-stable 130.0.6723.58-1"
        );
        assert_eq!(events[0].1.source, "dpkg");
        assert_eq!(events[1].1.title, "Removed zoom and its settings");
    }

    #[test]
    fn snap_and_flatpak_changes_come_from_the_lists() {
        let before = parse_snaps(
            "Name  Version  Rev  Tracking  Publisher  Notes\ncode  1.94.2  170  latest/stable  vscode✓  classic\nfirefox  131.0  4955  latest/stable  mozilla✓  -\nthunderbird 128.3.1 500 latest/stable canonical✓ -\n",
        );
        let after = parse_snaps(
            "Name  Version  Rev  Tracking  Publisher  Notes\ncode  1.94.2  171  latest/stable  vscode✓  classic\nfirefox  131.0  4955  latest/stable  mozilla✓  -\nspotify 1.2 80 latest/stable spotify -\n",
        );
        let changes = compare(&before, &after);
        let titles: Vec<String> = changes.iter().map(|i| title(i, "snap")).collect();
        assert_eq!(
            titles,
            [
                "Updated snap code 1.94.2",
                "Installed snap spotify 1.2",
                "Removed snap thunderbird 128.3.1"
            ]
        );
        let flat = parse_flatpaks(
            "com.spotify.Client\t1.2.45\tstable\tsystem\tabc123\norg.gimp.GIMP\t\tstable\tuser\tdef456\n",
        );
        assert_eq!(flat["com.spotify.Client/stable/system"].version, "1.2.45");
        let mut newer = flat.clone();
        newer.insert(
            "org.gimp.GIMP/stable/user".into(),
            App {
                version: String::new(),
                build: "fff999".into(),
            },
        );
        let item = &compare(&flat, &newer)[0];
        assert_eq!(item.name, "org.gimp.GIMP");
        assert_eq!(title(item, "flatpak"), "Updated Flatpak app org.gimp.GIMP");
        assert!(parse_snaps("").is_empty());
    }
}
