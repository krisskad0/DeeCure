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

/// Version 2 (Phase 3): history that stays small for 2 years.
///
/// - Metric names are stored once, in `series`. Each point keeps only the series number,
///   which makes a point about a third of its old size. The `(resolution, at)` index is
///   gone: cleanup now walks one series at a time along the primary key.
/// - Events get a `key`, so reading a log twice never stores an event twice, and a JSON
///   `data` column for details such as package versions or a file's diff.
/// - New tables: `watched_file` (what config files looked like, to show what changed),
///   `notification` (what was sent, so nothing repeats within 24 hours), and `report`
///   (weekly summaries).
const V2: &str = "
CREATE TABLE series (
    id      INTEGER PRIMARY KEY,
    name    TEXT    NOT NULL,
    subject TEXT    NOT NULL DEFAULT '',
    unit    TEXT    NOT NULL,
    UNIQUE (name, subject)
);
CREATE TABLE metric_v2 (
    series     INTEGER NOT NULL,     -- series.id
    resolution INTEGER NOT NULL,     -- seconds per point: 60, 900 or 86400
    at         INTEGER NOT NULL,     -- start of the period, ms since epoch
    value      REAL    NOT NULL,     -- average over the period
    samples    INTEGER NOT NULL,     -- how many readings the average covers
    PRIMARY KEY (series, resolution, at)
) WITHOUT ROWID;
INSERT INTO series (name, subject, unit)
    SELECT name, subject, max(unit) FROM metric GROUP BY name, subject;
INSERT INTO metric_v2 (series, resolution, at, value, samples)
    SELECT s.id, m.resolution, m.at, m.value, m.samples
    FROM metric AS m JOIN series AS s ON s.name = m.name AND s.subject = m.subject;
DROP TABLE metric;
ALTER TABLE metric_v2 RENAME TO metric;

CREATE TABLE event_v2 (
    id      INTEGER PRIMARY KEY,
    key     TEXT    NOT NULL UNIQUE, -- the same thing seen twice has the same key
    kind    TEXT    NOT NULL,
    at      INTEGER NOT NULL,
    title   TEXT    NOT NULL,
    detail  TEXT,
    subject TEXT,
    source  TEXT    NOT NULL,
    data    TEXT                     -- JSON object with details, or NULL
);
INSERT INTO event_v2 (id, key, kind, at, title, detail, subject, source)
    SELECT id, 'v1:' || id, kind, at, title, detail, subject, source FROM event;
DROP TABLE event;
ALTER TABLE event_v2 RENAME TO event;
CREATE INDEX event_by_time ON event (at);

CREATE TABLE watched_file (
    path    TEXT    PRIMARY KEY,
    mtime   INTEGER NOT NULL,        -- ms since epoch
    size    INTEGER NOT NULL,
    hash    TEXT,                    -- FNV-1a of the content; NULL when it cannot be read
    content TEXT                     -- small text files only, to show a diff later
);

CREATE TABLE notification (
    id      INTEGER PRIMARY KEY,
    key     TEXT    NOT NULL,        -- rule|subject|severity, or weekly
    sent_at INTEGER NOT NULL,
    title   TEXT    NOT NULL,
    body    TEXT    NOT NULL
);
CREATE INDEX notification_by_key ON notification (key, sent_at);

CREATE TABLE report (
    id         INTEGER PRIMARY KEY,
    created_at INTEGER NOT NULL,
    since      INTEGER NOT NULL,
    until      INTEGER NOT NULL,
    data       TEXT    NOT NULL      -- JSON: deecure_core::report::Report
);
";

/// Every migration, oldest first. The schema version is the number applied.
const MIGRATIONS: &[&str] = &[V1, V2];

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
                "event",
                "finding",
                "fix_run",
                "metric",
                "notification",
                "report",
                "series",
                "setting",
                "snapshot",
                "watched_file"
            ]
        );
    }

    #[test]
    fn version_1_data_moves_to_version_2() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(V1).unwrap();
        conn.pragma_update(None, "user_version", 1).unwrap();
        conn.execute_batch(
            "INSERT INTO metric VALUES ('cpu.usage', '', 60, 60000, 12.5, 2, 'percent');
             INSERT INTO metric VALUES ('cpu.usage', '', 900, 0, 10.0, 15, 'percent');
             INSERT INTO metric VALUES ('disk.used', '/', 60, 60000, 5.0, 1, 'bytes');
             INSERT INTO event (kind, at, title, source) VALUES ('boot', 5, 'Started', 'journal');",
        )
        .unwrap();
        migrate(&conn).unwrap();
        assert_eq!(version(&conn).unwrap(), LATEST);
        let points: Vec<(String, String, i64, f64, i64, String)> = conn
            .prepare(
                "SELECT s.name, s.subject, m.resolution, m.value, m.samples, s.unit
                 FROM metric m JOIN series s ON s.id = m.series ORDER BY s.name, m.resolution",
            )
            .unwrap()
            .query_map([], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            })
            .unwrap()
            .collect::<std::result::Result<_, _>>()
            .unwrap();
        assert_eq!(points.len(), 3);
        assert_eq!(
            points[0],
            (
                "cpu.usage".into(),
                String::new(),
                60,
                12.5,
                2,
                "percent".into()
            )
        );
        assert_eq!(points[2].1, "/");
        let key: String = conn
            .query_row("SELECT key FROM event", [], |r| r.get(0))
            .unwrap();
        assert_eq!(key, "v1:1");
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
