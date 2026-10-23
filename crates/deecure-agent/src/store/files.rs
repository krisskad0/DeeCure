// SPDX-License-Identifier: Apache-2.0

//! Watched config files (what each looked like last time) and small helpers for
//! the history collectors.

use std::collections::HashMap;

use deecure_core::{Event, EventKind, Timestamp};
use rusqlite::{OptionalExtension, params};

use super::{Result, Store, from_text};

/// What a watched file looked like at the last scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchedFile {
    /// Modification time, ms since epoch.
    pub mtime: i64,
    /// Size in bytes.
    pub size: i64,
    /// FNV-1a of the content, or `None` when it could not be read.
    pub hash: Option<String>,
}

impl Store {
    /// Runs `f` in one transaction: everything it writes is saved together, or not at all.
    pub fn batch<T>(&self, f: impl FnOnce(&Store) -> Result<T>) -> Result<T> {
        let tx = self.conn.unchecked_transaction()?;
        let value = f(self)?;
        tx.commit()?;
        Ok(value)
    }

    /// Every watched file, without contents.
    pub fn watched_files(&self) -> Result<HashMap<String, WatchedFile>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT path, mtime, size, hash FROM watched_file")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                WatchedFile {
                    mtime: row.get(1)?,
                    size: row.get(2)?,
                    hash: row.get(3)?,
                },
            ))
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// The saved text of a watched file, if it was small text.
    pub fn watched_content(&self, path: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .prepare_cached("SELECT content FROM watched_file WHERE path = ?1")?
            .query_row(params![path], |row| row.get::<_, Option<String>>(0))
            .optional()?
            .flatten())
    }

    /// Saves what a watched file looks like now.
    pub fn put_watched(&self, path: &str, file: &WatchedFile, content: Option<&str>) -> Result<()> {
        self.conn
            .prepare_cached(
                "INSERT INTO watched_file (path, mtime, size, hash, content)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT (path) DO UPDATE SET mtime = excluded.mtime, size = excluded.size,
                     hash = excluded.hash, content = excluded.content",
            )?
            .execute(params![path, file.mtime, file.size, file.hash, content])?;
        Ok(())
    }

    /// Stops remembering a file that is gone.
    pub fn forget_watched(&self, path: &str) -> Result<()> {
        self.conn
            .prepare_cached("DELETE FROM watched_file WHERE path = ?1")?
            .execute(params![path])?;
        Ok(())
    }

    /// The stored event with `key`, if any.
    pub fn event_by_key(&self, key: &str) -> Result<Option<Event>> {
        let row = self
            .conn
            .prepare_cached(
                "SELECT id, kind, at, title, detail, subject, source, data FROM event WHERE key = ?1",
            )?
            .query_row(params![key], |row| {
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
            })
            .optional()?;
        row.map(|(id, kind, at, title, detail, subject, source, data)| {
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
        .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watched_files_round_trip() {
        let store = Store::open_in_memory().unwrap();
        let file = WatchedFile {
            mtime: 5,
            size: 3,
            hash: Some("ab".into()),
        };
        store.put_watched("/etc/hosts", &file, Some("x\n")).unwrap();
        assert_eq!(store.watched_files().unwrap()["/etc/hosts"], file);
        assert_eq!(
            store.watched_content("/etc/hosts").unwrap().as_deref(),
            Some("x\n")
        );
        store.put_watched("/etc/hosts", &file, None).unwrap();
        assert_eq!(store.watched_content("/etc/hosts").unwrap(), None);
        store.forget_watched("/etc/hosts").unwrap();
        assert!(store.watched_files().unwrap().is_empty());
    }

    #[test]
    fn a_failed_batch_saves_nothing() {
        let store = Store::open_in_memory().unwrap();
        let file = WatchedFile {
            mtime: 1,
            size: 1,
            hash: None,
        };
        let result: Result<()> = store.batch(|s| {
            s.put_watched("/etc/a", &file, None)?;
            Err(super::super::StoreError::Corrupt("stop".into()))
        });
        assert!(result.is_err());
        assert!(store.watched_files().unwrap().is_empty());
    }
}
