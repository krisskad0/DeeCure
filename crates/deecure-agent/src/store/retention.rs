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
/// Daily points and events are kept 2 years.
const KEEP_HISTORY_MS: i64 = 730 * DAY_MS;
/// Snapshots, and findings that are no longer open, are kept 90 days.
/// The newest snapshot of each kind is always kept.
const KEEP_RECENT_MS: i64 = 90 * DAY_MS;

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
        report.rolled_up += roll_up(&tx, 60, 900, minute_cutoff)?;
        let quarter_cutoff = floor_to(now - KEEP_QUARTER_POINTS_MS, DAY_MS);
        report.rolled_up += roll_up(&tx, 900, 86_400, quarter_cutoff)?;

        let history_cutoff = now - KEEP_HISTORY_MS;
        let recent_cutoff = now - KEEP_RECENT_MS;
        report.deleted += tx.execute(
            "DELETE FROM metric WHERE resolution = 86400 AND at < ?1",
            params![history_cutoff],
        )?;
        report.deleted += tx.execute("DELETE FROM event WHERE at < ?1", params![history_cutoff])?;
        report.deleted += tx.execute(
            "DELETE FROM snapshot WHERE taken_at < ?1
             AND id NOT IN (SELECT max(id) FROM snapshot GROUP BY kind)",
            params![recent_cutoff],
        )?;
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

/// Replaces points of `from` seconds older than `cutoff` with weighted averages of `to` seconds.
fn roll_up(tx: &Transaction<'_>, from: i64, to: i64, cutoff: i64) -> Result<usize> {
    let step_ms = to * 1000;
    tx.execute(
        "INSERT INTO metric (name, subject, resolution, at, value, samples, unit)
         SELECT name, subject, ?2, (at / ?3) * ?3,
                sum(value * samples) / sum(samples), sum(samples), max(unit)
         FROM metric WHERE resolution = ?1 AND at < ?4
         GROUP BY name, subject, at / ?3
         ON CONFLICT (name, subject, resolution, at) DO UPDATE SET
             value = (value * samples + excluded.value * excluded.samples)
                     / (samples + excluded.samples),
             samples = samples + excluded.samples",
        params![from, to, step_ms, cutoff],
    )?;
    Ok(tx.execute(
        "DELETE FROM metric WHERE resolution = ?1 AND at < ?2",
        params![from, cutoff],
    )?)
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
            kind: EventKind::Boot,
            at: Timestamp(at),
            title: "Started".into(),
            detail: None,
            subject: None,
            source: "journal".into(),
        };
        store.add_event(&event(NOW - 800 * DAY_MS)).unwrap();
        store.add_event(&event(NOW - DAY_MS)).unwrap();
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
}
