// SPDX-License-Identifier: Apache-2.0

//! Events, snapshots, and settings.

use deecure_core::{Event, EventKind, Snapshot, SnapshotKind, Timestamp};
use rusqlite::{OptionalExtension, params};
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::{Result, Store, from_text, to_text};

impl Store {
    /// Records something that happened, once: an event whose `key` is already stored is
    /// left as it is. Returns `true` if it was new.
    pub fn add_event(&self, key: &str, event: &Event) -> Result<bool> {
        if event.at.as_millis() < self.history_floor.get() {
            return Ok(false);
        }
        Ok(self
            .conn
            .prepare_cached(
                "INSERT INTO event (key, kind, at, title, detail, subject, source, data)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT (key) DO NOTHING",
            )?
            .execute(params![
                key,
                to_text(&event.kind)?,
                event.at.as_millis(),
                event.title,
                event.detail,
                event.subject,
                event.source,
                event.data.as_ref().map(serde_json::to_string).transpose()?,
            ])?
            == 1)
    }

    /// Records an event, or replaces the stored one with the same `key` (keeping its
    /// number). For events that grow, such as "crashed 3 times this hour".
    pub fn put_event(&self, key: &str, event: &Event) -> Result<()> {
        if event.at.as_millis() < self.history_floor.get() {
            return Ok(());
        }
        self.conn
            .prepare_cached(
                "INSERT INTO event (key, kind, at, title, detail, subject, source, data)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT (key) DO UPDATE SET
                     kind = excluded.kind, at = excluded.at, title = excluded.title,
                     detail = excluded.detail, subject = excluded.subject,
                     source = excluded.source, data = excluded.data",
            )?
            .execute(params![
                key,
                to_text(&event.kind)?,
                event.at.as_millis(),
                event.title,
                event.detail,
                event.subject,
                event.source,
                event.data.as_ref().map(serde_json::to_string).transpose()?,
            ])?;
        Ok(())
    }

    /// Events from `since` to `until` (both included), newest first, at most `limit`.
    /// With `kinds`, only those kinds.
    pub fn events(
        &self,
        since: Timestamp,
        until: Timestamp,
        kinds: &[EventKind],
        limit: usize,
    ) -> Result<Vec<Event>> {
        let kinds = kinds.iter().map(to_text).collect::<Result<Vec<_>>>()?;
        // A JSON list keeps the statement fixed, so it can be cached.
        let kinds = (!kinds.is_empty()).then(|| serde_json::Value::from(kinds).to_string());
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, kind, at, title, detail, subject, source, data FROM event
             WHERE at BETWEEN ?1 AND ?2
               AND (?3 IS NULL OR kind IN (SELECT value FROM json_each(?3)))
             ORDER BY at DESC, id DESC LIMIT ?4",
        )?;
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let rows = stmt.query_map(
            params![since.as_millis(), until.as_millis(), kinds, limit],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                ))
            },
        )?;
        rows.map(|row| {
            let (id, kind, at, title, detail, subject, source, data) = row?;
            Ok(Event {
                id: Some(id),
                kind: from_text::<EventKind>(kind)?,
                at: Timestamp(at),
                title,
                detail,
                subject,
                source,
                data: data.map(|d| serde_json::from_str(&d)).transpose()?,
            })
        })
        .collect()
    }

    /// Events from `since` onward, newest first, at most `limit` of them.
    pub fn events_since(&self, since: Timestamp, limit: usize) -> Result<Vec<Event>> {
        self.events(since, Timestamp(i64::MAX), &[], limit)
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

    /// Stores an inventory only if it differs from the newest one of the same kind.
    /// Returns `true` if it was stored. Keeps the database small when little changes.
    pub fn add_snapshot_if_changed(&self, snapshot: &Snapshot) -> Result<bool> {
        let unchanged = self
            .latest_snapshot(snapshot.kind)?
            .is_some_and(|latest| latest.data == snapshot.data);
        if unchanged {
            return Ok(false);
        }
        self.add_snapshot(snapshot)?;
        Ok(true)
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
            id: None,
            data: None,
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
            assert!(store.add_event(title, &event(at, title)).unwrap());
        }
        let titles: Vec<_> = store
            .events_since(Timestamp(15), 10)
            .unwrap()
            .into_iter()
            .map(|e| e.title)
            .collect();
        assert_eq!(titles, ["c", "b"]);
        let newest = &store.events_since(Timestamp(0), 1).unwrap()[0];
        assert_eq!(newest.id, Some(2));
        assert_eq!(newest.title, "c");
    }

    #[test]
    fn an_event_seen_twice_is_stored_once() {
        let store = Store::open_in_memory().unwrap();
        assert!(store.add_event("k", &event(1, "first")).unwrap());
        assert!(!store.add_event("k", &event(1, "again")).unwrap());
        let all = store.events_since(Timestamp(0), 10).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].title, "first");
    }

    #[test]
    fn put_event_replaces_and_keeps_the_number() {
        let store = Store::open_in_memory().unwrap();
        store.put_event("crash", &event(1, "crashed once")).unwrap();
        let mut grown = event(5, "crashed 3 times");
        grown.data = Some(json!({"count": 3}));
        store.put_event("crash", &grown).unwrap();
        let all = store.events_since(Timestamp(0), 10).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(
            (all[0].id, all[0].at, all[0].title.as_str()),
            (Some(1), Timestamp(5), "crashed 3 times")
        );
        assert_eq!(all[0].data, Some(json!({"count": 3})));
    }

    #[test]
    fn events_filter_by_time_and_kind() {
        let store = Store::open_in_memory().unwrap();
        store.add_event("a", &event(10, "upgrade")).unwrap();
        let mut boot = event(20, "boot");
        boot.kind = EventKind::Boot;
        store.add_event("b", &boot).unwrap();
        store.add_event("c", &event(30, "later")).unwrap();
        let only_boots = store
            .events(Timestamp(0), Timestamp(100), &[EventKind::Boot], 10)
            .unwrap();
        assert_eq!(only_boots.len(), 1);
        assert_eq!(only_boots[0].title, "boot");
        let window = store.events(Timestamp(10), Timestamp(20), &[], 10).unwrap();
        assert_eq!(window.len(), 2);
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
    fn unchanged_snapshot_is_not_stored_twice() {
        let store = Store::open_in_memory().unwrap();
        let snapshot = |at, data| Snapshot {
            kind: SnapshotKind::Filesystems,
            taken_at: Timestamp(at),
            data,
        };
        assert!(
            store
                .add_snapshot_if_changed(&snapshot(1, json!(["/"])))
                .unwrap()
        );
        assert!(
            !store
                .add_snapshot_if_changed(&snapshot(2, json!(["/"])))
                .unwrap()
        );
        assert!(
            store
                .add_snapshot_if_changed(&snapshot(3, json!(["/", "/home"])))
                .unwrap()
        );
        let latest = store
            .latest_snapshot(SnapshotKind::Filesystems)
            .unwrap()
            .unwrap();
        assert_eq!(latest.taken_at, Timestamp(3));
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
