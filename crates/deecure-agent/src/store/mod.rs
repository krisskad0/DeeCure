// SPDX-License-Identifier: Apache-2.0

//! The agent's SQLite database: history, findings, and settings.
//!
//! Tables, units, and how long data is kept: `docs/02-architecture/storage.md`.

mod error;
mod findings;
mod fixes;
mod metrics;
mod records;
mod retention;
mod schema;

use std::fs::{self, DirBuilder, Permissions};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use rusqlite::Connection;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub use error::StoreError;
pub use retention::RetentionReport;

/// A short name for results in this module.
pub type Result<T> = std::result::Result<T, StoreError>;

/// The store, shared between the socket server and background jobs.
pub type SharedStore = Arc<Mutex<Store>>;

/// Locks the shared store. A panic in another task does not make the data unusable,
/// because every write runs in its own SQLite transaction.
pub fn lock(store: &Mutex<Store>) -> MutexGuard<'_, Store> {
    store.lock().unwrap_or_else(PoisonError::into_inner)
}

/// An open DeeCure database.
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Where the database lives by default: `$XDG_DATA_HOME/deecure/deecure.db`,
    /// which is usually `~/.local/share/deecure/deecure.db`.
    pub fn default_path() -> Option<PathBuf> {
        let data_home = std::env::var_os("XDG_DATA_HOME")
            .filter(|dir| !dir.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .filter(|home| !home.is_empty())
                    .map(|home| PathBuf::from(home).join(".local").join("share"))
            })?;
        Some(data_home.join("deecure").join("deecure.db"))
    }

    /// Opens (or creates) the database at `path`, readable only by this user,
    /// and brings its tables up to date.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            DirBuilder::new().recursive(true).mode(0o700).create(dir)?;
        }
        let conn = Connection::open(path)?;
        // SQLite gives its -wal and -shm files the same mode as the database.
        fs::set_permissions(path, Permissions::from_mode(0o600))?;
        Self::init(conn, true)
    }

    /// Opens a throwaway database in memory, for tests.
    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?, false)
    }

    fn init(conn: Connection, on_disk: bool) -> Result<Self> {
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.pragma_update(None, "foreign_keys", true)?;
        if on_disk {
            // WAL lets the app read while the agent writes.
            conn.pragma_update_and_check(None, "journal_mode", "WAL", |row| {
                row.get::<_, String>(0)
            })?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
        }
        schema::migrate(&conn)?;
        Ok(Store { conn })
    }

    /// The schema version stored in the database.
    pub fn schema_version(&self) -> Result<u32> {
        schema::version(&self.conn)
    }
}

/// The text form of a serde enum, such as `"critical"` for `Severity::Critical`.
fn to_text<T: Serialize>(value: &T) -> Result<String> {
    match serde_json::to_value(value)? {
        Value::String(text) => Ok(text),
        other => Err(StoreError::Corrupt(format!("expected text, got {other}"))),
    }
}

/// Reads back a value written with [`to_text`].
fn from_text<T: DeserializeOwned>(text: String) -> Result<T> {
    Ok(serde_json::from_value(Value::String(text))?)
}

/// Subjects are stored as `''` when there is none, so they can be part of a primary key.
fn subject_to_db(subject: Option<&str>) -> &str {
    subject.unwrap_or("")
}

fn subject_from_db(subject: String) -> Option<String> {
    (!subject.is_empty()).then_some(subject)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("deecure-store-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("data").join("deecure.db")
    }

    #[test]
    fn on_disk_database_is_private_and_uses_wal() {
        let path = temp_db("private");
        let store = Store::open(&path).unwrap();
        let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&path), 0o600);
        assert_eq!(mode(path.parent().unwrap()), 0o700);
        let journal: String = store
            .conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        assert_eq!(journal, "wal");
        assert_eq!(store.schema_version().unwrap(), schema::LATEST);
        let _ = fs::remove_dir_all(path.parent().unwrap().parent().unwrap());
    }

    #[test]
    fn reopening_keeps_data() {
        let path = temp_db("reopen");
        Store::open(&path)
            .unwrap()
            .set_setting("time_range", &"24h")
            .unwrap();
        let again = Store::open(&path).unwrap();
        assert_eq!(
            again.setting::<String>("time_range").unwrap().as_deref(),
            Some("24h")
        );
        let _ = fs::remove_dir_all(path.parent().unwrap().parent().unwrap());
    }

    #[test]
    fn default_path_follows_xdg() {
        let path = Store::default_path().unwrap();
        assert!(path.ends_with("deecure/deecure.db"));
    }
}
