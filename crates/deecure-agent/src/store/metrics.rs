// SPDX-License-Identifier: Apache-2.0

//! Time series: CPU, memory, temperature, disk, battery.

use deecure_core::{Metric, Timestamp, Unit};
use rusqlite::{Connection, OptionalExtension, params};

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

/// The series number for `name` and `subject`, created on first use. A changed unit is
/// saved, so the newest reading decides the unit.
fn series_id(conn: &Connection, name: &str, subject: &str, unit: &str) -> Result<i64> {
    let found = conn
        .prepare_cached("SELECT id, unit FROM series WHERE name = ?1 AND subject = ?2")?
        .query_row(params![name, subject], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .optional()?;
    match found {
        Some((id, stored)) => {
            if stored != unit {
                conn.prepare_cached("UPDATE series SET unit = ?1 WHERE id = ?2")?
                    .execute(params![unit, id])?;
            }
            Ok(id)
        }
        None => {
            conn.prepare_cached("INSERT INTO series (name, subject, unit) VALUES (?1, ?2, ?3)")?
                .execute(params![name, subject, unit])?;
            Ok(conn.last_insert_rowid())
        }
    }
}

fn insert_metric(conn: &Connection, metric: &Metric) -> Result<()> {
    let series = series_id(
        conn,
        &metric.name,
        subject_to_db(metric.subject.as_deref()),
        &to_text(&metric.unit)?,
    )?;
    conn.prepare_cached(
        "INSERT INTO metric (series, resolution, at, value, samples)
         VALUES (?1, 60, ?2, ?3, 1)
         ON CONFLICT (series, resolution, at) DO UPDATE SET
             value = (value * samples + excluded.value) / (samples + 1),
             samples = samples + 1",
    )?
    .execute(params![
        series,
        floor_to(metric.at.as_millis(), MINUTE_MS),
        metric.value,
    ])?;
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

    /// The newest 1-minute point of every metric and subject that has one from `since` on.
    pub fn latest_metrics(&self, since: Timestamp) -> Result<Vec<Metric>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT s.name, s.subject, m.at, m.value, s.unit
             FROM series AS s JOIN metric AS m
               ON m.series = s.id AND m.resolution = 60
              AND m.at = (SELECT max(at) FROM metric WHERE series = s.id AND resolution = 60)
             WHERE m.at >= ?1
             ORDER BY s.name, s.subject",
        )?;
        let rows = stmt.query_map(params![since.as_millis()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;
        rows.map(|row| {
            let (name, subject, at, value, unit) = row?;
            Ok(Metric {
                name,
                subject: subject_from_db(subject),
                value,
                unit: from_text::<Unit>(unit)?,
                at: Timestamp(at),
            })
        })
        .collect()
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
        let subject_db = subject_to_db(subject);
        let Some((series, unit)) = self
            .conn
            .prepare_cached("SELECT id, unit FROM series WHERE name = ?1 AND subject = ?2")?
            .query_row(params![name, subject_db], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .optional()?
        else {
            return Ok(Vec::new());
        };
        let unit = from_text::<Unit>(unit)?;
        // Naming each resolution lets SQLite use the primary key for all three ranges.
        let mut stmt = self.conn.prepare_cached(
            "SELECT at, value FROM metric
             WHERE series = ?1 AND resolution IN (60, 900, 86400) AND at BETWEEN ?2 AND ?3
             ORDER BY at",
        )?;
        let rows = stmt.query_map(
            params![series, since.as_millis(), until.as_millis()],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?)),
        )?;
        rows.map(|row| {
            let (at, value) = row?;
            Ok(Metric {
                name: name.to_owned(),
                subject: subject.map(str::to_owned),
                value,
                unit,
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
    fn latest_gives_the_newest_point_of_each_series() {
        let store = Store::open_in_memory().unwrap();
        let mut disk = cpu(120_000, 94.0);
        disk.name = "disk.used_percent".into();
        disk.subject = Some("/".into());
        store
            .add_metrics(&[cpu(0, 10.0), cpu(60_000, 20.0), disk, cpu(-600_000, 5.0)])
            .unwrap();
        let latest: Vec<_> = store
            .latest_metrics(Timestamp(30_000))
            .unwrap()
            .into_iter()
            .map(|m| (m.name, m.subject, m.value))
            .collect();
        assert_eq!(
            latest,
            [
                ("cpu.usage".to_owned(), None, 20.0),
                ("disk.used_percent".to_owned(), Some("/".to_owned()), 94.0)
            ]
        );
    }

    #[test]
    fn floor_handles_negative_times() {
        assert_eq!(floor_to(-1, MINUTE_MS), -MINUTE_MS);
        assert_eq!(floor_to(119_999, MINUTE_MS), 60_000);
    }
}
