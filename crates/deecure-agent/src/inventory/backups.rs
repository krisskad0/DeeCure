// SPDX-License-Identifier: Apache-2.0

//! Backups: Déjà Dup (personal files) and Timeshift (system snapshots). Read from their
//! settings and snapshot folders; nothing is started or changed.

use std::time::{Duration, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;

use super::command;
use crate::collect::SystemRoot;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct DejaDup {
    pub installed: bool,
    /// True when backups run on a schedule.
    pub scheduled: bool,
    /// Days between backups.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub every_days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_backup: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Snapshot {
    pub created: i64,
    /// On demand, Boot, Hourly, Daily, Weekly, Monthly.
    pub kind: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub comment: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Timeshift {
    pub installed: bool,
    /// True when it was set up (it has a backup device).
    pub configured: bool,
    /// `rsync` or `btrfs`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub mode: String,
    /// Monthly, Weekly, Daily, Hourly, Boot.
    pub schedule: Vec<String>,
    /// Newest first. Empty when the backup drive is not connected.
    pub snapshots: Vec<Snapshot>,
}

/// The `backups` inventory.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct BackupsInfo {
    pub deja_dup: DejaDup,
    pub timeshift: Timeshift,
}

impl BackupsInfo {
    /// True when any tool backs up on a schedule.
    pub fn configured(&self) -> bool {
        self.deja_dup.scheduled
            || (self.timeshift.configured && !self.timeshift.schedule.is_empty())
    }

    /// The newest backup or snapshot, in milliseconds since 1970.
    pub fn newest(&self) -> Option<i64> {
        self.deja_dup
            .last_backup
            .into_iter()
            .chain(self.timeshift.snapshots.first().map(|s| s.created))
            .max()
    }
}

/// "2026-05-26_18-00-01" (a Timeshift snapshot folder) → milliseconds since 1970, local time
/// taken as UTC (the hour can be off by the time zone; the day is right for the age check).
pub fn snapshot_time(name: &str) -> Option<i64> {
    let iso = format!(
        "{}T{}:00+00:00",
        name.get(..10)?,
        name.get(11..16)?.replace('-', ":")
    );
    super::accounts::parse_iso(&iso)
}

fn tags(tag: &str) -> String {
    match tag.chars().next() {
        Some('O') => "On demand",
        Some('B') => "Boot",
        Some('H') => "Hourly",
        Some('D') => "Daily",
        Some('W') => "Weekly",
        Some('M') => "Monthly",
        _ => "Snapshot",
    }
    .into()
}

fn timeshift(root: &SystemRoot) -> Timeshift {
    let installed = root.path("/usr/bin/timeshift").exists();
    let Some(conf) = root
        .read("/etc/timeshift/timeshift.json")
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
    else {
        return Timeshift {
            installed,
            ..Timeshift::default()
        };
    };
    let flag = |k: &str| conf.get(k).and_then(Value::as_str) == Some("true");
    let schedule = [
        ("schedule_monthly", "Monthly"),
        ("schedule_weekly", "Weekly"),
        ("schedule_daily", "Daily"),
        ("schedule_hourly", "Hourly"),
        ("schedule_boot", "Boot"),
    ]
    .iter()
    .filter(|(k, _)| flag(k))
    .map(|(_, n)| (*n).to_owned())
    .collect();
    let mut snapshots = Vec::new();
    for dir in [
        "/timeshift/snapshots",
        "/run/timeshift/backup/timeshift/snapshots",
    ] {
        for e in std::fs::read_dir(root.path(dir))
            .map(|d| d.flatten().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            let name = e.file_name().to_string_lossy().into_owned();
            let Some(created) = snapshot_time(&name) else {
                continue;
            };
            let info: Value = std::fs::read_to_string(e.path().join("info.json"))
                .ok()
                .and_then(|t| serde_json::from_str(&t).ok())
                .unwrap_or(Value::Null);
            let text = |k: &str| info.get(k).and_then(Value::as_str).unwrap_or("").to_owned();
            if !snapshots.iter().any(|s: &Snapshot| s.created == created) {
                snapshots.push(Snapshot {
                    created,
                    kind: tags(&text("tags")),
                    comment: text("comments"),
                });
            }
        }
    }
    snapshots.sort_by_key(|s| std::cmp::Reverse(s.created));
    Timeshift {
        installed,
        configured: conf
            .get("backup_device_uuid")
            .and_then(Value::as_str)
            .is_some_and(|u| !u.is_empty()),
        mode: if flag("btrfs_mode") { "btrfs" } else { "rsync" }.into(),
        schedule,
        snapshots,
    }
}

async fn deja_dup() -> DejaDup {
    let Some(text) = command::output(
        "gsettings",
        &["list-recursively", "org.gnome.DejaDup"],
        Duration::from_secs(3),
    )
    .await
    else {
        return DejaDup::default();
    };
    let get = |k: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(&format!("org.gnome.DejaDup {k} ")))
            .map(|v| v.trim().trim_matches('\'').to_owned())
    };
    DejaDup {
        installed: true,
        scheduled: get("periodic").as_deref() == Some("true"),
        every_days: get("periodic-period").and_then(|v| v.parse().ok()),
        last_backup: get("last-backup")
            .and_then(|v| super::accounts::parse_iso(&normalize_iso(&v))),
        backend: get("backend"),
    }
}

/// Déjà Dup writes "2026-09-30T21:12:00.123456Z" or "…+05:30"; make it "…+00:00" style.
fn normalize_iso(v: &str) -> String {
    let (date, rest) = v.split_at(v.len().min(19));
    let zone = if rest.ends_with('Z') || rest.is_empty() {
        "+00:00".to_owned()
    } else {
        rest[rest.len().saturating_sub(6)..].to_owned()
    };
    format!("{date}{zone}")
}

pub async fn read(root: &SystemRoot) -> BackupsInfo {
    BackupsInfo {
        deja_dup: deja_dup().await,
        timeshift: timeshift(root),
    }
}

/// Days since the newest backup, for rules.
pub fn age_days(info: &BackupsInfo) -> Option<f64> {
    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis() as i64;
    info.newest().map(|t| (now - t) as f64 / 86_400_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn reads_timeshift_snapshots() {
        let root = fixture_root(
            "timeshift",
            &[
                ("/usr/bin/timeshift", ""),
                (
                    "/etc/timeshift/timeshift.json",
                    r#"{"backup_device_uuid":"6c3d","btrfs_mode":"false","schedule_monthly":"true","schedule_daily":"false"}"#,
                ),
                (
                    "/timeshift/snapshots/2026-05-26_18-00-01/info.json",
                    r#"{"tags":"M","comments":"before upgrade"}"#,
                ),
                (
                    "/timeshift/snapshots/2026-06-26_18-00-01/info.json",
                    r#"{"tags":"O"}"#,
                ),
                ("/timeshift/snapshots/not-a-snapshot/x", ""),
            ],
        );
        let t = timeshift(&root);
        assert!(t.configured);
        assert_eq!(t.schedule, ["Monthly"]);
        assert_eq!(
            t.snapshots
                .iter()
                .map(|s| s.kind.as_str())
                .collect::<Vec<_>>(),
            ["On demand", "Monthly"]
        );
        assert_eq!(t.snapshots[1].comment, "before upgrade");
        let info = BackupsInfo {
            timeshift: t,
            ..BackupsInfo::default()
        };
        assert!(info.configured());
        assert_eq!(
            normalize_iso("2026-09-30T21:12:00.123456Z"),
            "2026-09-30T21:12:00+00:00"
        );
    }
}
