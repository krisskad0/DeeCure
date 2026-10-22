// SPDX-License-Identifier: Apache-2.0

//! Time series: CPU, memory, temperature, disk, battery.

use deecure_core::{Metric, Timestamp, Unit};
use rusqlite::{Connection, params};

use super::{Result, Store, from_text, subject_from_db, subject_to_db, to_text};

/// One minute, in milliseconds.
pub(super) const MINUTE_MS: i64 = 60_000;
/// Fifteen minutes, in milliseconds.
pub(super) const QUARTER_HOUR_MS: i64 = 15 * MINUTE_MS;
/// One day, in milliseconds.
pub(super) const DAY_MS: i64 = 24 * 60 * MINUTE_MS;

/// Rounds `ms` down to a multiple of `step`.
pub(super) fn floor_to(ms: i64, step: i64) -> i64 {
    ms.div_euclid(step) * step
}

fn insert_metric(conn: &Connection, metric: &Metric) -> Result<()> {
    conn.execute(
        "INSERT INTO metric (name, subject, resolution, at, value, samples, unit)
         VALUES (?1, ?2, 60, ?3, ?4, 1, ?5)
         ON CONFLICT (name, subject, resolution, at) DO UPDATE SET
             value = (value * samples + excluded.value) / (samples + 1),
             samples = samples + 1,
             unit = excluded.unit",
        params![
            metric.name,
            subject_to_db(metric.subject.as_deref()),
            floor_to(metric.at.as_millis(), MINUTE_MS),
            metric.value,
            to_text(&metric.unit)?,
        ],
    )?;
    Ok(())
}

impl Store {
    /// Records one reading. Readings in the same minute are averaged into one point.
    pub fn add_metric(&self, metric: &Metric) -> Result<()> {
        insert_metric(&self.conn, metric)
    }

    /// Records many readings in one transaction (one disk write instead of many).
    pub fn add_metrics(&self, metrics: &[Metric]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        for metric in metrics {
            insert_metric(&tx, metric)?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Points for one metric from `since` to `until` (both included), oldest first.
    ///
    /// Recent points are one minute apart. Older points are 15-minute or daily
    /// averages, because [`Store::apply_retention`] rolls them up.
    pub fn metrics(
        &self,
        name: &str,
        subject: Option<&str>,
        since: Timestamp,
        until: Timestamp,
    ) -> Result<Vec<Metric>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT subject, at, value, unit FROM metric
             WHERE name = ?1 AND subject = ?2 AND at BETWEEN ?3 AND ?4
             ORDER BY at",
        )?;
        let rows = stmt.query_map(
            params![
                name,
                subject_to_db(subject),
                since.as_millis(),
                until.as_millis()
            ],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )?;
        rows.map(|row| {
            let (subject, at, value, unit) = row?;
            Ok(Metric {
                name: name.to_owned(),
                subject: subject_from_db(subject),
                value,
                unit: from_text::<Unit>(unit)?,
                at: Timestamp(at),
            })
        })
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cpu(at_ms: i64, value: f64) -> Metric {
        Metric {
            name: "cpu.usage".into(),
            subject: None,
            value,
            unit: Unit::Percent,
            at: Timestamp(at_ms),
        }
    }

    #[test]
    fn readings_in_one_minute_are_averaged() {
        let store = Store::open_in_memory().unwrap();
        store.add_metric(&cpu(1_000, 10.0)).unwrap();
        store.add_metric(&cpu(30_000, 20.0)).unwrap();
        store.add_metric(&cpu(59_999, 30.0)).unwrap();
        store.add_metric(&cpu(60_000, 50.0)).unwrap();

        let points = store
            .metrics("cpu.usage", None, Timestamp(0), Timestamp(120_000))
            .unwrap();
        let values: Vec<_> = points.iter().map(|m| (m.at.as_millis(), m.value)).collect();
        assert_eq!(values, [(0, 20.0), (60_000, 50.0)]);
        assert_eq!(points[0].unit, Unit::Percent);
    }

    #[test]
    fn subjects_are_kept_apart() {
        let store = Store::open_in_memory().unwrap();
        let mut root = cpu(0, 94.0);
        root.name = "disk.used".into();
        root.subject = Some("/".into());
        let mut home = root.clone();
        home.subject = Some("/home".into());
        home.value = 40.0;
        store.add_metric(&root).unwrap();
        store.add_metric(&home).unwrap();

        let points = store
            .metrics("disk.used", Some("/home"), Timestamp(0), Timestamp(1))
            .unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].value, 40.0);
        assert_eq!(points[0].subject.as_deref(), Some("/home"));
    }

    #[test]
    fn batched_readings_are_all_stored() {
        let store = Store::open_in_memory().unwrap();
        store
            .add_metrics(&[cpu(0, 10.0), cpu(60_000, 20.0), cpu(120_000, 30.0)])
            .unwrap();
        let points = store
            .metrics("cpu.usage", None, Timestamp(0), Timestamp(120_000))
            .unwrap();
        assert_eq!(points.len(), 3);
    }

    #[test]
    fn floor_handles_negative_times() {
        assert_eq!(floor_to(-1, MINUTE_MS), -MINUTE_MS);
        assert_eq!(floor_to(119_999, MINUTE_MS), 60_000);
    }
}
