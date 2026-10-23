// SPDX-License-Identifier: Apache-2.0

//! Keeping the database small: roll old points up into averages, delete what is too old.

use deecure_core::Timestamp;
use rusqlite::{Transaction, params};

use super::Result;
use super::Store;
use super::metrics::{DAY_MS, QUARTER_HOUR_MS, floor_to};

/// 1-minute points are kept 7 days, then rolled up into 15-minute averages.
const KEEP_MINUTE_POINTS_MS: i64 = 7 * DAY_MS;
/// 15-minute points are kept 90 days, then rolled up into daily averages.
const KEEP_QUARTER_POINTS_MS: i64 = 90 * DAY_MS;
/// Daily points, events, and weekly reports are kept 2 years.
const KEEP_HISTORY_MS: i64 = 730 * DAY_MS;
/// Snapshots, closed findings, and sent notifications are kept 90 days.
/// The newest snapshot of each kind is always kept.
const KEEP_RECENT_MS: i64 = 90 * DAY_MS;
/// Snapshots older than a day are thinned to the newest one of each hour, and older
/// than a week to the newest one of each day. Only the newest is ever read; the rest
/// is kept for support, and a service that flaps every few minutes must not fill the disk.
const SNAPSHOTS_HOURLY_AFTER_MS: i64 = DAY_MS;
const SNAPSHOTS_DAILY_AFTER_MS: i64 = 7 * DAY_MS;
/// A last guard: never more events than this. The oldest go first.
const MAX_EVENTS: i64 = 200_000;
const HOUR_MS: i64 = 60 * 60_000;

/// The setting that holds [`deecure_core::notify::DataSettings`].
pub const DATA_SETTING: &str = "data";

/// What one cleanup run did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RetentionReport {
    /// Points replaced by coarser averages.
    pub rolled_up: usize,
    /// Rows deleted because they were too old.
    pub deleted: usize,
}

impl Store {
    /// Rolls up and deletes old data. Safe to run as often as you like.
    pub fn apply_retention(&self, now: Timestamp) -> Result<RetentionReport> {
        let now = now.as_millis();
        let tx = self.conn.unchecked_transaction()?;
        let mut report = RetentionReport::default();

        // Cut at period boundaries, so only complete periods are rolled up.
        let minute_cutoff = floor_to(now - KEEP_MINUTE_POINTS_MS, QUARTER_HOUR_MS);
        let quarter_cutoff = floor_to(now - KEEP_QUARTER_POINTS_MS, DAY_MS);
        // Settings › Data › Keep history for: 90 days, 1 year, or 2 years.
        let keep = self
            .setting::<deecure_core::notify::DataSettings>(DATA_SETTING)?
            .unwrap_or_default()
            .keep_history
            .days()
            * DAY_MS;
        let history_cutoff = now - keep.min(KEEP_HISTORY_MS);
        let recent_cutoff = now - KEEP_RECENT_MS;

        // One series at a time, so every statement walks the primary key.
        let series: Vec<i64> = tx
            .prepare("SELECT id FROM series")?
            .query_map([], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        for id in series {
            report.rolled_up += roll_up(&tx, id, 60, 900, minute_cutoff)?;
            report.rolled_up += roll_up(&tx, id, 900, 86_400, quarter_cutoff)?;
            report.deleted += tx.execute(
                "DELETE FROM metric WHERE series = ?1 AND resolution = 86400 AND at < ?2",
                params![id, history_cutoff],
            )?;
        }
        // A series with no points left (a USB disk not seen for 2 years) is forgotten.
        tx.execute(
            "DELETE FROM series WHERE NOT EXISTS (SELECT 1 FROM metric WHERE series = series.id)",
            [],
        )?;

        report.deleted += tx.execute("DELETE FROM event WHERE at < ?1", params![history_cutoff])?;
        report.deleted += cap_events(&tx, MAX_EVENTS)?;
        report.deleted += tx.execute(
            "DELETE FROM report WHERE created_at < ?1",
            params![history_cutoff],
        )?;
        report.deleted += tx.execute(
            "DELETE FROM notification WHERE sent_at < ?1",
            params![recent_cutoff],
        )?;
        report.deleted += tx.execute(
            "DELETE FROM snapshot WHERE taken_at < ?1
             AND id NOT IN (SELECT max(id) FROM snapshot GROUP BY kind)",
            params![recent_cutoff],
        )?;
        for (older_than, bucket) in [
            (SNAPSHOTS_HOURLY_AFTER_MS, HOUR_MS),
            (SNAPSHOTS_DAILY_AFTER_MS, DAY_MS),
        ] {
            report.deleted += tx.execute(
                "DELETE FROM snapshot WHERE taken_at < ?1 AND id NOT IN
                     (SELECT max(id) FROM snapshot WHERE taken_at < ?1 GROUP BY kind, taken_at / ?2)",
                params![now - older_than, bucket],
            )?;
        }
        report.deleted += tx.execute(
            "DELETE FROM finding WHERE state != 'open' AND last_seen < ?1",
            params![recent_cutoff],
        )?;
        tx.commit()?;

        // Give freed pages back to the file system.
        self.conn.execute_batch("PRAGMA incremental_vacuum")?;
        Ok(report)
    }
}

impl Store {
    /// Deletes all history: metrics, events, reports, sent notifications, every snapshot
    /// but the newest of each kind, and closed findings. Fix runs (restore points),
    /// settings, open findings, and what config files look like now are kept. Events
    /// from before `now` are never stored again, even if a log still holds them.
    pub fn delete_history(&self, now: Timestamp) -> Result<deecure_core::notify::Deleted> {
        let size = |s: &Store| -> Result<u64> {
            let pages: i64 = s
                .conn
                .pragma_query_value(None, "page_count", |r| r.get(0))?;
            let free: i64 = s
                .conn
                .pragma_query_value(None, "freelist_count", |r| r.get(0))?;
            let page: i64 = s.conn.pragma_query_value(None, "page_size", |r| r.get(0))?;
            Ok(u64::try_from((pages - free) * page).unwrap_or(0))
        };
        let before = size(self)?;
        let tx = self.conn.unchecked_transaction()?;
        let mut rows = 0;
        for sql in [
            "DELETE FROM metric",
            "DELETE FROM series",
            "DELETE FROM event",
            "DELETE FROM notification",
            "DELETE FROM report",
            "DELETE FROM snapshot WHERE id NOT IN (SELECT max(id) FROM snapshot GROUP BY kind)",
            "DELETE FROM finding WHERE state != 'open'",
        ] {
            rows += tx.execute(sql, [])?;
        }
        tx.execute(
            "INSERT INTO setting (key, value) VALUES (?1, ?2)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            params![super::HISTORY_DELETED_AT, now.as_millis().to_string()],
        )?;
        tx.commit()?;
        self.history_floor.set(now.as_millis());
        self.conn.execute_batch("PRAGMA incremental_vacuum")?;
        let after = size(self)?;
        Ok(deecure_core::notify::Deleted {
            rows: rows as u64,
            bytes_freed: before.saturating_sub(after),
        })
    }
}

/// Deletes the oldest events beyond `max`.
fn cap_events(tx: &Transaction<'_>, max: i64) -> Result<usize> {
    Ok(tx.execute(
        "DELETE FROM event WHERE id IN
             (SELECT id FROM event ORDER BY at DESC, id DESC LIMIT -1 OFFSET ?1)",
        params![max],
    )?)
}

/// Replaces one series' points of `from` seconds older than `cutoff` with weighted
/// averages of `to` seconds.
fn roll_up(tx: &Transaction<'_>, series: i64, from: i64, to: i64, cutoff: i64) -> Result<usize> {
    let step_ms = to * 1000;
    tx.prepare_cached(
        "INSERT INTO metric (series, resolution, at, value, samples)
         SELECT series, ?3, (at / ?4) * ?4, sum(value * samples) / sum(samples), sum(samples)
         FROM metric WHERE series = ?1 AND resolution = ?2 AND at < ?5
         GROUP BY at / ?4
         ON CONFLICT (series, resolution, at) DO UPDATE SET
             value = (value * samples + excluded.value * excluded.samples)
                     / (samples + excluded.samples),
             samples = samples + excluded.samples",
    )?
    .execute(params![series, from, to, step_ms, cutoff])?;
    Ok(tx
        .prepare_cached("DELETE FROM metric WHERE series = ?1 AND resolution = ?2 AND at < ?3")?
        .execute(params![series, from, cutoff])?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::metrics::MINUTE_MS;
    use deecure_core::{Event, EventKind, Metric, Snapshot, SnapshotKind, Unit};
    use serde_json::json;

    /// A fixed "now": 2026-10-22T00:00:00Z, a whole day.
    const NOW: i64 = 1_792_627_200_000;

    fn cpu(at: i64, value: f64) -> Metric {
        Metric {
            name: "cpu.usage".into(),
            subject: None,
            value,
            unit: Unit::Percent,
            at: Timestamp(at),
        }
    }

    fn points(store: &Store) -> Vec<(i64, f64)> {
        store
            .metrics("cpu.usage", None, Timestamp(0), Timestamp(i64::MAX))
            .unwrap()
            .into_iter()
            .map(|m| (m.at.as_millis(), m.value))
            .collect()
    }

    #[test]
    fn old_minutes_become_quarter_hour_averages() {
        let store = Store::open_in_memory().unwrap();
        let old = NOW - 8 * DAY_MS; // past the 7-day limit, on a quarter-hour boundary
        store.add_metric(&cpu(old, 10.0)).unwrap();
        store.add_metric(&cpu(old + MINUTE_MS, 20.0)).unwrap();
        store.add_metric(&cpu(old + MINUTE_MS + 1, 60.0)).unwrap(); // same minute: avg 40, 2 samples
        store.add_metric(&cpu(NOW - MINUTE_MS, 99.0)).unwrap(); // recent: untouched

        let report = store.apply_retention(Timestamp(NOW)).unwrap();
        assert_eq!(report.rolled_up, 2);
        // (10×1 + 40×2) / 3 = 30
        assert_eq!(points(&store), [(old, 30.0), (NOW - MINUTE_MS, 99.0)]);

        // Running again changes nothing.
        assert_eq!(
            store.apply_retention(Timestamp(NOW)).unwrap(),
            RetentionReport::default()
        );
    }

    #[test]
    fn very_old_points_end_up_daily_then_deleted() {
        let store = Store::open_in_memory().unwrap();
        let day = NOW - 100 * DAY_MS;
        store.add_metric(&cpu(day, 10.0)).unwrap();
        store
            .add_metric(&cpu(day + 12 * 60 * MINUTE_MS, 30.0))
            .unwrap();
        store.add_metric(&cpu(NOW - 800 * DAY_MS, 5.0)).unwrap();

        store.apply_retention(Timestamp(NOW)).unwrap();
        assert_eq!(points(&store), [(day, 20.0)]);
    }

    #[test]
    fn old_events_snapshots_and_closed_findings_are_deleted() {
        let store = Store::open_in_memory().unwrap();
        let event = |at| Event {
            id: None,
            data: None,
            kind: EventKind::Boot,
            at: Timestamp(at),
            title: "Started".into(),
            detail: None,
            subject: None,
            source: "journal".into(),
        };
        store.add_event("old", &event(NOW - 800 * DAY_MS)).unwrap();
        store.add_event("new", &event(NOW - DAY_MS)).unwrap();
        for at in [NOW - 200 * DAY_MS, NOW - 100 * DAY_MS] {
            store
                .add_snapshot(&Snapshot {
                    kind: SnapshotKind::Hardware,
                    taken_at: Timestamp(at),
                    data: json!({}),
                })
                .unwrap();
        }

        let report = store.apply_retention(Timestamp(NOW)).unwrap();
        assert_eq!(report.deleted, 2, "one old event and one old snapshot");
        assert_eq!(store.events_since(Timestamp(0), 10).unwrap().len(), 1);
        // The newest hardware snapshot stays, even though it is over 90 days old.
        assert!(
            store
                .latest_snapshot(SnapshotKind::Hardware)
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn a_series_with_no_points_left_is_forgotten() {
        let store = Store::open_in_memory().unwrap();
        store.add_metric(&cpu(NOW - 800 * DAY_MS, 5.0)).unwrap();
        store.apply_retention(Timestamp(NOW)).unwrap();
        let series: i64 = store
            .conn
            .query_row("SELECT count(*) FROM series", [], |r| r.get(0))
            .unwrap();
        assert_eq!(series, 0);
        // It comes back on the next reading.
        store.add_metric(&cpu(NOW, 7.0)).unwrap();
        assert_eq!(points(&store), [(NOW, 7.0)]);
    }

    #[test]
    fn old_snapshots_are_thinned_to_one_per_hour_then_one_per_day() {
        let store = Store::open_in_memory().unwrap();
        let add = |at: i64, n: i64| {
            store
                .add_snapshot(&Snapshot {
                    kind: SnapshotKind::Services,
                    taken_at: Timestamp(at),
                    data: json!({ "n": n }),
                })
                .unwrap();
        };
        let hour = 60 * MINUTE_MS;
        // Every 5 minutes for 2 hours, 3 days ago; then every 5 minutes for 1 hour, 10 days ago.
        for i in 0..24 {
            add(NOW - 3 * DAY_MS + i * 5 * MINUTE_MS, i);
        }
        for i in 0..12 {
            add(NOW - 10 * DAY_MS + i * 5 * MINUTE_MS, 100 + i);
        }
        // Every 5 minutes in the last hour: all kept.
        for i in 0..12 {
            add(NOW - hour + i * 5 * MINUTE_MS, 200 + i);
        }
        store.apply_retention(Timestamp(NOW)).unwrap();
        let left: Vec<i64> = store
            .conn
            .prepare("SELECT json_extract(data, '$.n') FROM snapshot ORDER BY taken_at")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        let mut expected = vec![111, 11, 23];
        expected.extend(200..212);
        assert_eq!(left, expected);
    }

    #[test]
    fn the_event_cap_drops_the_oldest() {
        let store = Store::open_in_memory().unwrap();
        for at in 1..=5 {
            let e = Event {
                id: None,
                data: None,
                kind: EventKind::Boot,
                at: Timestamp(at),
                title: format!("boot {at}"),
                detail: None,
                subject: None,
                source: "journal".into(),
            };
            store.add_event(&at.to_string(), &e).unwrap();
        }
        let tx = store.conn.unchecked_transaction().unwrap();
        assert_eq!(cap_events(&tx, 3).unwrap(), 2);
        tx.commit().unwrap();
        let left: Vec<_> = store
            .events_since(Timestamp(0), 10)
            .unwrap()
            .into_iter()
            .map(|e| e.at.as_millis())
            .collect();
        assert_eq!(left, [5, 4, 3]);
    }

    #[test]
    fn keep_history_follows_the_setting() {
        let store = Store::open_in_memory().unwrap();
        let event = |at| Event {
            id: None,
            data: None,
            kind: EventKind::Boot,
            at: Timestamp(at),
            title: "Started".into(),
            detail: None,
            subject: None,
            source: "journal".into(),
        };
        store.add_event("a", &event(NOW - 200 * DAY_MS)).unwrap();
        store.add_metric(&cpu(NOW - 200 * DAY_MS, 5.0)).unwrap();
        store.apply_retention(Timestamp(NOW)).unwrap();
        assert_eq!(
            store.events_since(Timestamp(0), 10).unwrap().len(),
            1,
            "2 years by default"
        );
        store
            .set_setting(DATA_SETTING, &serde_json::json!({"keep_history": "90d"}))
            .unwrap();
        store.apply_retention(Timestamp(NOW)).unwrap();
        assert!(store.events_since(Timestamp(0), 10).unwrap().is_empty());
        assert!(points(&store).is_empty());
    }

    #[test]
    fn deleted_history_stays_deleted() {
        let store = Store::open_in_memory().unwrap();
        let event = |at| Event {
            id: None,
            data: None,
            kind: EventKind::PackageUpgraded,
            at: Timestamp(at),
            title: "Upgraded x".into(),
            detail: None,
            subject: None,
            source: "apt".into(),
        };
        store.add_event("apt:1", &event(NOW - DAY_MS)).unwrap();
        store.add_metric(&cpu(NOW - MINUTE_MS, 5.0)).unwrap();
        for at in [1, 2] {
            store
                .add_snapshot(&Snapshot {
                    kind: SnapshotKind::Services,
                    taken_at: Timestamp(at),
                    data: json!({"n": at}),
                })
                .unwrap();
        }
        let gone = store.delete_history(Timestamp(NOW)).unwrap();
        assert_eq!(
            gone.rows, 4,
            "an event, a point, its series, and the older snapshot"
        );
        assert!(store.events_since(Timestamp(0), 10).unwrap().is_empty());
        assert!(points(&store).is_empty());
        assert!(
            store
                .latest_snapshot(SnapshotKind::Services)
                .unwrap()
                .is_some(),
            "the newest stays"
        );
        // A log read again does not bring the old event back; new ones are stored.
        assert!(!store.add_event("apt:1", &event(NOW - DAY_MS)).unwrap());
        assert!(store.add_event("apt:2", &event(NOW + 1)).unwrap());
    }

    /// Phase 3 "done when": the database stays under 200 MB after 2 years.
    ///
    /// Builds what 2 years leave behind on a big machine (150 metric series: 16 cores,
    /// 30 sensors, 6 network interfaces, 3 disks and 3 filesystems), in the order the
    /// agent writes it: time first, every series at each step, so pages fill the way they
    /// do on a real computer. The last 7 days go through the real `add_metrics`. Then the
    /// real cleanup runs, and the file on disk is measured.
    ///
    /// Slow in a debug build: `make db-size` runs it in release mode.
    #[test]
    #[ignore = "slow; run with make db-size"]
    fn two_years_of_data_stay_under_200_mb() {
        const SERIES: usize = 150;
        const LIMIT_BYTES: u64 = 200 * 1000 * 1000;
        let dir = std::env::temp_dir().join(format!("deecure-size-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("deecure.db");
        let store = Store::open(&path).unwrap();

        let mut names: Vec<(String, Option<String>)> = Vec::new();
        names.extend((0..16).map(|c| ("cpu.core".to_owned(), Some(c.to_string()))));
        names.extend((0..30).map(|t| ("temp".to_owned(), Some(format!("coretemp/Core {t}")))));
        for n in ["net.rx", "net.tx"] {
            for i in ["wlp2s0", "enp3s0", "docker0", "tun0", "br-1f2e", "wwan0"] {
                names.push((n.to_owned(), Some(i.to_owned())));
            }
        }
        for n in [
            "disk.read",
            "disk.write",
            "disk.iops",
            "disk.busy",
            "disk.await",
        ] {
            for d in ["nvme0n1", "sda", "sdb"] {
                names.push((n.to_owned(), Some(d.to_owned())));
            }
        }
        for n in [
            "disk.used_percent",
            "disk.free",
            "disk.used",
            "disk.inodes_percent",
        ] {
            for m in ["/", "/home", "/boot/efi"] {
                names.push((n.to_owned(), Some(m.to_owned())));
            }
        }
        let mut i = 0;
        while names.len() < SERIES {
            names.push((format!("misc.metric_{i}"), None));
            i += 1;
        }
        let value =
            |k: usize, t: i64| ((k as i64 * 7919 + t / 60_000) % 1000) as f64 / 10.0 + 0.123;

        // Daily and 15-minute averages, written the way cleanup writes them.
        for (name, subject) in &names {
            store
                .add_metric(&Metric {
                    name: name.clone(),
                    subject: subject.clone(),
                    value: 1.0,
                    unit: Unit::Percent,
                    at: Timestamp(NOW),
                })
                .unwrap();
        }
        let ids: Vec<i64> = store
            .conn
            .prepare("SELECT id FROM series ORDER BY id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        store.conn.execute("DELETE FROM metric", []).unwrap();
        let tx = store.conn.unchecked_transaction().unwrap();
        for (resolution, step, from, to) in [
            (
                86_400,
                DAY_MS,
                NOW - KEEP_HISTORY_MS,
                NOW - KEEP_QUARTER_POINTS_MS,
            ),
            (
                900,
                QUARTER_HOUR_MS,
                NOW - KEEP_QUARTER_POINTS_MS,
                NOW - KEEP_MINUTE_POINTS_MS,
            ),
        ] {
            let mut stmt = tx
                .prepare("INSERT INTO metric VALUES (?1, ?2, ?3, ?4, ?5)")
                .unwrap();
            let mut at = floor_to(from, step) + step;
            while at < to {
                for (k, id) in ids.iter().enumerate() {
                    stmt.execute(params![id, resolution, at, value(k, at), step / 60_000])
                        .unwrap();
                }
                at += step;
            }
        }
        tx.commit().unwrap();

        // The last 7 days, one reading a minute, through the real path.
        let mut at = NOW - KEEP_MINUTE_POINTS_MS + MINUTE_MS;
        while at <= NOW {
            let batch: Vec<Metric> = names
                .iter()
                .enumerate()
                .map(|(k, (name, subject))| Metric {
                    name: name.clone(),
                    subject: subject.clone(),
                    value: value(k, at),
                    unit: Unit::Percent,
                    at: Timestamp(at),
                })
                .collect();
            store.add_metrics(&batch).unwrap();
            at += MINUTE_MS;
        }

        // 2 years of events: 40 a day, with details, like package upgrades and config diffs.
        let tx = store.conn.unchecked_transaction().unwrap();
        for day in 0..730 {
            for n in 0..40 {
                let at = NOW - day * DAY_MS - n * 600_000;
                let e = Event {
                    id: None,
                    kind: EventKind::PackageUpgraded,
                    at: Timestamp(at),
                    title: format!(
                        "Upgraded libexample{n} 1.2.{day}-0ubuntu1 → 1.2.{day}-0ubuntu2"
                    ),
                    detail: Some("Packages · unattended-upgrades".into()),
                    subject: Some(format!("libexample{n}")),
                    source: "unattended-upgrades".into(),
                    data: Some(json!({
                        "action": "upgrade", "from": format!("1.2.{day}-0ubuntu1"),
                        "to": format!("1.2.{day}-0ubuntu2"), "run": at,
                        "diff": "-  old line of a config file\n+  new line of a config file\n".repeat(3),
                    })),
                };
                store.add_event(&format!("pkg:{at}:{n}"), &e).unwrap();
            }
        }
        tx.commit().unwrap();

        // Snapshots at the density cleanup leaves: every 5 minutes for a day, hourly for
        // a week, then daily. A services snapshot is about 32 KB.
        let services = json!((0..220).map(|u| json!({
            "name": format!("unit-number-{u}.service"), "description": "A service with a description",
            "load": "loaded", "active": "active", "sub": "running", "enabled": "enabled", "restarts": 0
        })).collect::<Vec<_>>());
        let mut times = Vec::new();
        times.extend((0..288).map(|i| NOW - i * 5 * MINUTE_MS));
        times.extend((24..168).map(|h| NOW - h * 60 * MINUTE_MS));
        times.extend((7..90).map(|d| NOW - d * DAY_MS));
        for at in times {
            store
                .add_snapshot(&Snapshot {
                    kind: SnapshotKind::Services,
                    taken_at: Timestamp(at),
                    data: services.clone(),
                })
                .unwrap();
        }

        // Config file copies (2,000 files of 1.5 KB), notifications, and weekly reports.
        let tx = store.conn.unchecked_transaction().unwrap();
        for f in 0..2000 {
            tx.execute(
                "INSERT INTO watched_file VALUES (?1, 1, 1500, 'abc', ?2)",
                params![
                    format!("/etc/some/folder/file-{f}.conf"),
                    "x = 1\n".repeat(250)
                ],
            )
            .unwrap();
        }
        for n in 0..270 {
            tx.execute(
                "INSERT INTO notification (key, sent_at, title, body) VALUES (?1, ?2, ?3, ?4)",
                params![
                    format!("rule|{n}|critical"),
                    NOW - n * 8 * 3_600_000,
                    "Main disk is almost full",
                    "Only 3 GB left."
                ],
            )
            .unwrap();
        }
        for w in 0..104 {
            tx.execute(
                "INSERT INTO report (created_at, since, until, data) VALUES (?1, ?2, ?3, ?4)",
                params![
                    NOW - w * 7 * DAY_MS,
                    0,
                    0,
                    "{\"x\":\"".to_owned() + &"y".repeat(10_000) + "\"}"
                ],
            )
            .unwrap();
        }
        tx.commit().unwrap();

        let report = store.apply_retention(Timestamp(NOW)).unwrap();
        store
            .conn
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
            .unwrap();
        let size = |p: &std::path::Path| std::fs::metadata(p).map_or(0, |m| m.len());
        let wal = dir.join("deecure.db-wal");
        let total = size(&path) + size(&wal);
        let points: i64 = store
            .conn
            .query_row("SELECT count(*) FROM metric", [], |r| r.get(0))
            .unwrap();
        eprintln!(
            "two years: {:.1} MB on disk, {points} points in {SERIES} series ({:.0} bytes a point), cleanup {report:?}",
            total as f64 / 1e6,
            total as f64 / points as f64,
        );
        assert_eq!(
            report.deleted, 0,
            "the simulation already matches what cleanup keeps"
        );
        assert!(
            total < LIMIT_BYTES,
            "{total} bytes is over the 200 MB budget"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
