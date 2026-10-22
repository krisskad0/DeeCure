// SPDX-License-Identifier: Apache-2.0

//! What can go wrong with the database.

use std::fmt;

/// An error from the database layer.
#[derive(Debug)]
pub enum StoreError {
    /// SQLite reported an error.
    Sqlite(rusqlite::Error),
    /// A JSON column could not be written or read.
    Json(serde_json::Error),
    /// The database folder could not be created or protected.
    Io(std::io::Error),
    /// The database was made by a newer DeeCure, so this one cannot use it safely.
    TooNew {
        /// The schema version found in the file.
        found: u32,
        /// The newest schema version this build knows.
        supported: u32,
    },
    /// A stored value could not be read back.
    Corrupt(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Sqlite(e) => write!(f, "database error: {e}"),
            StoreError::Json(e) => write!(f, "stored JSON error: {e}"),
            StoreError::Io(e) => write!(f, "database file error: {e}"),
            StoreError::TooNew { found, supported } => write!(
                f,
                "the database uses schema version {found}, but this DeeCure only knows up to {supported}; update DeeCure"
            ),
            StoreError::Corrupt(what) => write!(f, "unreadable stored value: {what}"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StoreError::Sqlite(e) => Some(e),
            StoreError::Json(e) => Some(e),
            StoreError::Io(e) => Some(e),
            StoreError::TooNew { .. } | StoreError::Corrupt(_) => None,
        }
    }
}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError::Sqlite(e)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        StoreError::Json(e)
    }
}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e)
    }
}
