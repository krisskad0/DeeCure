// SPDX-License-Identifier: Apache-2.0

//! Events, snapshots, and settings.

use deecure_core::{Event, EventKind, Snapshot, SnapshotKind, Timestamp};
use rusqlite::{OptionalExtension, params};
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::{Result, Store, from_text, to_text};

impl Store {
    /// Records something that happened.
    pub fn add_event(&self, event: &Event) -> Result<()> {
        self.conn.execute(
            "INSERT INTO event (kind, at, title, detail, subject, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                to_text(&event.kind)?,
                event.at.as_millis(),
                event.title,
                event.detail,
                event.subject,
                event.source,
            ],
        )?;
        Ok(())
    }

    /// Events from `since` onward, newest first, at most `limit` of them.
    pub fn events_since(&self, since: Timestamp, limit: usize) -> Result<Vec<Event>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT kind, at, title, detail, subject, source FROM event
             WHERE at >= ?1 ORDER BY at DESC, id DESC LIMIT ?2",
        )?;
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let rows = stmt.query_map(params![since.as_millis(), limit], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;
        rows.map(|row| {
            let (kind, at, title, detail, subject, source) = row?;
            Ok(Event {
                kind: from_text::<EventKind>(kind)?,
                at: Timestamp(at),
                title,
                detail,
                subject,
                source,
            })
        })
        .collect()
    }

    /// Stores an inventory.
    pub fn add_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        self.conn.execute(
            "INSERT INTO snapshot (kind, taken_at, data) VALUES (?1, ?2, ?3)",
            params![
                to_text(&snapshot.kind)?,
                snapshot.taken_at.as_millis(),
                serde_json::to_string(&snapshot.data)?,
            ],
        )?;
        Ok(())
    }

    /// The newest inventory of one kind, if any.
    pub fn latest_snapshot(&self, kind: SnapshotKind) -> Result<Option<Snapshot>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT taken_at, data FROM snapshot WHERE kind = ?1
             ORDER BY taken_at DESC, id DESC LIMIT 1",
        )?;
        let row = stmt
            .query_row(params![to_text(&kind)?], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .optional()?;
        row.map(|(taken_at, data)| {
            Ok(Snapshot {
                kind,
                taken_at: Timestamp(taken_at),
                data: serde_json::from_str(&data)?,
            })
        })
        .transpose()
    }

    /// Reads a setting, or `None` if it was never set.
    pub fn setting<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT value FROM setting WHERE key = ?1")?;
        let value = stmt
            .query_row(params![key], |row| row.get::<_, String>(0))
            .optional()?;
        value
            .map(|json| Ok(serde_json::from_str(&json)?))
            .transpose()
    }

    /// Saves a setting.
    pub fn set_setting<T: Serialize + ?Sized>(&self, key: &str, value: &T) -> Result<()> {
        self.conn.execute(
            "INSERT INTO setting (key, value) VALUES (?1, ?2)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            params![key, serde_json::to_string(value)?],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn event(at: i64, title: &str) -> Event {
        Event {
            kind: EventKind::PackageUpgraded,
            at: Timestamp(at),
            title: title.into(),
            detail: None,
            subject: Some("linux-image-generic".into()),
            source: "dpkg".into(),
        }
    }

    #[test]
    fn events_come_back_newest_first_from_since() {
        let store = Store::open_in_memory().unwrap();
        for (at, title) in [(10, "a"), (30, "c"), (20, "b")] {
            store.add_event(&event(at, title)).unwrap();
        }
        let titles: Vec<_> = store
            .events_since(Timestamp(15), 10)
            .unwrap()
            .into_iter()
            .map(|e| e.title)
            .collect();
        assert_eq!(titles, ["c", "b"]);
        assert_eq!(
            store.events_since(Timestamp(0), 1).unwrap()[0],
            event(30, "c")
        );
    }

    #[test]
    fn latest_snapshot_wins() {
        let store = Store::open_in_memory().unwrap();
        assert!(
            store
                .latest_snapshot(SnapshotKind::Services)
                .unwrap()
                .is_none()
        );
        for (at, failed) in [(1, 0), (2, 1)] {
            store
                .add_snapshot(&Snapshot {
                    kind: SnapshotKind::Services,
                    taken_at: Timestamp(at),
                    data: json!({"failed": failed}),
                })
                .unwrap();
        }
        let latest = store
            .latest_snapshot(SnapshotKind::Services)
            .unwrap()
            .unwrap();
        assert_eq!(latest.data, json!({"failed": 1}));
    }

    #[test]
    fn settings_round_trip_and_overwrite() {
        let store = Store::open_in_memory().unwrap();
        assert_eq!(store.setting::<bool>("expert").unwrap(), None);
        store.set_setting("expert", &false).unwrap();
        store.set_setting("expert", &true).unwrap();
        assert_eq!(store.setting::<bool>("expert").unwrap(), Some(true));
    }
}
