// SPDX-License-Identifier: Apache-2.0

//! Table definitions and schema upgrades.
//!
//! The schema version lives in SQLite's `user_version`. Each entry in
//! [`MIGRATIONS`] moves the database up one version. Never edit a released
//! migration: add a new one instead.

use rusqlite::Connection;

use super::{Result, StoreError};

/// Version 1: the six tables from `docs/02-architecture/storage.md`.
const V1: &str = "
CREATE TABLE snapshot (
    id       INTEGER PRIMARY KEY,
    kind     TEXT    NOT NULL,
    taken_at INTEGER NOT NULL,       -- ms since epoch
    data     TEXT    NOT NULL        -- JSON
);
CREATE INDEX snapshot_by_kind ON snapshot (kind, taken_at);

CREATE TABLE metric (
    name       TEXT    NOT NULL,
    subject    TEXT    NOT NULL DEFAULT '',
    resolution INTEGER NOT NULL,     -- seconds per point: 60, 900 or 86400
    at         INTEGER NOT NULL,     -- start of the period, ms since epoch
    value      REAL    NOT NULL,     -- average over the period
    samples    INTEGER NOT NULL,     -- how many readings the average covers
    unit       TEXT    NOT NULL,
    PRIMARY KEY (name, subject, resolution, at)
) WITHOUT ROWID;
CREATE INDEX metric_by_age ON metric (resolution, at);

CREATE TABLE event (
    id      INTEGER PRIMARY KEY,
    kind    TEXT    NOT NULL,
    at      INTEGER NOT NULL,
    title   TEXT    NOT NULL,
    detail  TEXT,
    subject TEXT,
    source  TEXT    NOT NULL
);
CREATE INDEX event_by_time ON event (at);

CREATE TABLE finding (
    rule       TEXT    NOT NULL,
    subject    TEXT    NOT NULL DEFAULT '',
    severity   TEXT    NOT NULL,
    state      TEXT    NOT NULL,
    title      TEXT    NOT NULL,
    explain    TEXT    NOT NULL,
    fixes      TEXT    NOT NULL DEFAULT '[]',   -- JSON array of fix action IDs
    first_seen INTEGER NOT NULL,
    last_seen  INTEGER NOT NULL,
    PRIMARY KEY (rule, subject)
);

CREATE TABLE fix_run (
    id              INTEGER PRIMARY KEY,
    finding_rule    TEXT,
    finding_subject TEXT,
    started_at      INTEGER NOT NULL,
    finished_at     INTEGER,
    status          TEXT    NOT NULL,       -- running, succeeded, failed, undone
    steps           TEXT    NOT NULL,       -- JSON: steps, commands, results
    undo            TEXT                    -- JSON: what is needed to undo, if anything
);

CREATE TABLE setting (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL                     -- JSON
) WITHOUT ROWID;
";

/// Every migration, oldest first. The schema version is the number applied.
const MIGRATIONS: &[&str] = &[V1];

/// The newest schema version this build knows.
pub(super) const LATEST: u32 = MIGRATIONS.len() as u32;

/// The schema version stored in the database (0 for a new file).
pub(super) fn version(conn: &Connection) -> Result<u32> {
    Ok(conn.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

/// Applies every migration the database has not seen yet, each in its own transaction.
pub(super) fn migrate(conn: &Connection) -> Result<()> {
    let current = version(conn)?;
    if current > LATEST {
        return Err(StoreError::TooNew {
            found: current,
            supported: LATEST,
        });
    }
    if current == 0 {
        // Must be set before the first table exists. Lets deleted space be given back.
        conn.pragma_update(None, "auto_vacuum", "INCREMENTAL")?;
    }
    for (done, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", done as u32 + 1)?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_database_gets_every_table() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        assert_eq!(version(&conn).unwrap(), LATEST);
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_schema WHERE type = 'table' ORDER BY name")
            .unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<std::result::Result<_, _>>()
            .unwrap();
        assert_eq!(
            tables,
            [
                "event", "finding", "fix_run", "metric", "setting", "snapshot"
            ]
        );
    }

    #[test]
    fn migrating_twice_changes_nothing() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
        assert_eq!(version(&conn).unwrap(), LATEST);
    }

    #[test]
    fn newer_database_is_refused() {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "user_version", LATEST + 1)
            .unwrap();
        assert!(matches!(
            migrate(&conn),
            Err(StoreError::TooNew { found, supported }) if found == LATEST + 1 && supported == LATEST
        ));
    }
}
