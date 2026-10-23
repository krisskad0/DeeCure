// SPDX-License-Identifier: Apache-2.0

//! Fix runs: what DeeCure changed, for Activity › DeeCure Actions and undo.

use deecure_core::fix::{FixRun, RunStatus};
use rusqlite::{OptionalExtension, params};

use super::{Result, Store, from_text, to_text};

impl Store {
    /// Saves a finished run and returns its ID.
    pub fn add_fix_run(&self, run: &FixRun) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO fix_run (finding_rule, finding_subject, started_at, finished_at, status, steps, undo)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                run.rule.as_ref().map(|r| r.as_str().to_owned()),
                run.subject,
                run.started_at.as_millis(),
                run.finished_at.as_millis(),
                to_text(&run.status)?,
                serde_json::to_string(run)?,
                serde_json::to_string(&run.undo_calls())?,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Past runs, newest first, at most `limit`.
    pub fn fix_runs(&self, limit: u32) -> Result<Vec<FixRun>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, status, steps FROM fix_run ORDER BY started_at DESC, id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        rows.map(|row| {
            let (id, status, json) = row?;
            let mut run: FixRun = serde_json::from_str(&json)?;
            run.id = id;
            run.status = from_text(status)?;
            Ok(run)
        })
        .collect()
    }

    /// One run.
    pub fn fix_run(&self, id: i64) -> Result<Option<FixRun>> {
        let row = self
            .conn
            .query_row(
                "SELECT status, steps FROM fix_run WHERE id = ?1",
                params![id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;
        row.map(|(status, json)| {
            let mut run: FixRun = serde_json::from_str(&json)?;
            run.id = id;
            run.status = from_text(status)?;
            Ok(run)
        })
        .transpose()
    }

    /// Changes a run's status, such as to undone.
    pub fn set_fix_status(&self, id: i64, status: RunStatus) -> Result<bool> {
        Ok(self.conn.execute(
            "UPDATE fix_run SET status = ?1 WHERE id = ?2",
            params![to_text(&status)?, id],
        )? == 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deecure_core::fix::{Call, StepResult};
    use deecure_core::{FixActionId, Timestamp};

    fn run(at: i64) -> FixRun {
        FixRun {
            id: 0,
            action: FixActionId::new("services.disable").unwrap(),
            title: "Turn off a service".into(),
            rule: None,
            subject: Some("cups.service".into()),
            started_at: Timestamp(at),
            finished_at: Timestamp(at + 5),
            status: RunStatus::Succeeded,
            steps: vec![StepResult {
                id: "service".into(),
                title: "Stop it".into(),
                ok: true,
                message: "done".into(),
                undo: Some(Call::SetServiceEnabled {
                    unit: "cups.service".into(),
                    enabled: true,
                }),
            }],
        }
    }

    #[test]
    fn saves_lists_and_marks_runs() {
        let store = Store::open_in_memory().unwrap();
        let first = store.add_fix_run(&run(1000)).unwrap();
        let second = store.add_fix_run(&run(2000)).unwrap();
        let list = store.fix_runs(10).unwrap();
        assert_eq!(
            list.iter().map(|r| r.id).collect::<Vec<_>>(),
            [second, first]
        );
        assert!(store.set_fix_status(first, RunStatus::Undone).unwrap());
        assert_eq!(
            store.fix_run(first).unwrap().unwrap().status,
            RunStatus::Undone
        );
        assert_eq!(store.fix_run(99).unwrap(), None);
        assert!(!store.set_fix_status(99, RunStatus::Undone).unwrap());
    }
}
