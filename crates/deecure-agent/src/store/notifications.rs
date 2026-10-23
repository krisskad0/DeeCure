// SPDX-License-Identifier: Apache-2.0

//! Sent notifications, so the same alert never repeats within 24 hours.

use deecure_core::Timestamp;
use deecure_core::notify::SentNotification;
use rusqlite::{OptionalExtension, params};

use super::{Result, Store};

impl Store {
    /// When a notification with `key` was last shown, if ever (within the 90 days kept).
    pub fn last_notified(&self, key: &str) -> Result<Option<Timestamp>> {
        Ok(self
            .conn
            .prepare_cached("SELECT max(sent_at) FROM notification WHERE key = ?1")?
            .query_row(params![key], |row| row.get::<_, Option<i64>>(0))
            .optional()?
            .flatten()
            .map(Timestamp))
    }

    /// Records a shown notification.
    pub fn add_notification(
        &self,
        key: &str,
        at: Timestamp,
        title: &str,
        body: &str,
    ) -> Result<()> {
        self.conn
            .prepare_cached(
                "INSERT INTO notification (key, sent_at, title, body) VALUES (?1, ?2, ?3, ?4)",
            )?
            .execute(params![key, at.as_millis(), title, body])?;
        Ok(())
    }

    /// The newest shown notifications, one per moment (a summary covers several keys).
    pub fn recent_notifications(&self, limit: u32) -> Result<Vec<SentNotification>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT sent_at, title, body FROM notification
             GROUP BY sent_at, title ORDER BY sent_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(SentNotification {
                at: Timestamp(row.get(0)?),
                title: row.get(1)?,
                body: row.get(2)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remembers_when_each_key_was_sent() {
        let store = Store::open_in_memory().unwrap();
        assert_eq!(store.last_notified("a").unwrap(), None);
        store
            .add_notification("a", Timestamp(5), "Disk full", "x")
            .unwrap();
        store
            .add_notification("a", Timestamp(9), "Disk full", "x")
            .unwrap();
        store
            .add_notification("b", Timestamp(9), "Disk full", "x")
            .unwrap();
        assert_eq!(store.last_notified("a").unwrap(), Some(Timestamp(9)));
        let recent = store.recent_notifications(10).unwrap();
        assert_eq!(recent.len(), 2, "keys sent in one summary show once");
        assert_eq!(recent[0].at, Timestamp(9));
    }
}
