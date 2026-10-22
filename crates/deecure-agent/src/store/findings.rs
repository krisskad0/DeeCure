// SPDX-License-Identifier: Apache-2.0

//! Findings: what rules reported, and whether each one is still open.

use deecure_core::{Finding, FindingState, FixActionId, RuleId, Timestamp};
use rusqlite::{OptionalExtension, Row, params};

use super::{Result, Store, StoreError, from_text, subject_from_db, subject_to_db, to_text};

const COLUMNS: &str =
    "rule, subject, severity, state, title, explain, fixes, first_seen, last_seen";

impl Store {
    /// Saves a finding, replacing the stored one with the same rule and subject.
    ///
    /// While a finding stays open, its first `first_seen` is kept. When a closed
    /// finding comes back, it starts again from the new `first_seen`.
    pub fn upsert_finding(&self, finding: &Finding) -> Result<()> {
        self.conn.execute(
            &format!(
                "INSERT INTO finding ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT (rule, subject) DO UPDATE SET
                     first_seen = CASE WHEN finding.state = 'open' AND excluded.state = 'open'
                                       THEN finding.first_seen ELSE excluded.first_seen END,
                     severity = excluded.severity,
                     state = excluded.state,
                     title = excluded.title,
                     explain = excluded.explain,
                     fixes = excluded.fixes,
                     last_seen = excluded.last_seen"
            ),
            params![
                finding.rule.as_str(),
                subject_to_db(finding.subject.as_deref()),
                to_text(&finding.severity)?,
                to_text(&finding.state)?,
                finding.title,
                finding.explain,
                serde_json::to_string(&finding.fixes)?,
                finding.first_seen.as_millis(),
                finding.last_seen.as_millis(),
            ],
        )?;
        Ok(())
    }

    /// One stored finding, if there is one.
    pub fn finding(&self, rule: &RuleId, subject: Option<&str>) -> Result<Option<Finding>> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM finding WHERE rule = ?1 AND subject = ?2"
        ))?;
        let row = stmt
            .query_row(params![rule.as_str(), subject_to_db(subject)], read_row)
            .optional()?;
        row.map(into_finding).transpose()
    }

    /// Open findings, most serious first, then oldest first.
    pub fn open_findings(&self) -> Result<Vec<Finding>> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM finding WHERE state = 'open'
             ORDER BY CASE severity WHEN 'critical' THEN 0 WHEN 'warning' THEN 1 ELSE 2 END,
                      first_seen, rule, subject"
        ))?;
        let rows = stmt.query_map([], read_row)?;
        rows.map(|row| into_finding(row?)).collect()
    }

    /// Changes a finding's state. Returns `false` if there is no such finding.
    pub fn set_finding_state(
        &self,
        rule: &RuleId,
        subject: Option<&str>,
        state: FindingState,
    ) -> Result<bool> {
        let changed = self.conn.execute(
            "UPDATE finding SET state = ?3 WHERE rule = ?1 AND subject = ?2",
            params![rule.as_str(), subject_to_db(subject), to_text(&state)?],
        )?;
        Ok(changed > 0)
    }
}

type RawFinding = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    i64,
);

fn read_row(row: &Row<'_>) -> rusqlite::Result<RawFinding> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
        row.get(8)?,
    ))
}

fn into_finding(raw: RawFinding) -> Result<Finding> {
    let (rule, subject, severity, state, title, explain, fixes, first_seen, last_seen) = raw;
    Ok(Finding {
        rule: RuleId::new(rule).map_err(|e| StoreError::Corrupt(e.to_string()))?,
        subject: subject_from_db(subject),
        severity: from_text(severity)?,
        state: from_text(state)?,
        title,
        explain,
        fixes: serde_json::from_str::<Vec<FixActionId>>(&fixes)?,
        first_seen: Timestamp(first_seen),
        last_seen: Timestamp(last_seen),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use deecure_core::Severity;

    fn finding(rule: &str, severity: Severity, seen: i64) -> Finding {
        Finding {
            rule: RuleId::new(rule).unwrap(),
            subject: None,
            severity,
            state: FindingState::Open,
            title: format!("title of {rule}"),
            explain: "explain".into(),
            first_seen: Timestamp(seen),
            last_seen: Timestamp(seen),
            fixes: vec![FixActionId::new("storage.clean_apt_cache").unwrap()],
        }
    }

    #[test]
    fn round_trips_every_field() {
        let store = Store::open_in_memory().unwrap();
        let mut original = finding("storage.root_almost_full", Severity::Critical, 5);
        original.subject = Some("/".into());
        store.upsert_finding(&original).unwrap();
        let loaded = store.finding(&original.rule, Some("/")).unwrap();
        assert_eq!(loaded, Some(original));
    }

    #[test]
    fn open_findings_are_most_serious_first() {
        let store = Store::open_in_memory().unwrap();
        store
            .upsert_finding(&finding("power.low_battery_action", Severity::Warning, 1))
            .unwrap();
        store
            .upsert_finding(&finding("system.reboot_required", Severity::Info, 0))
            .unwrap();
        store
            .upsert_finding(&finding("storage.root_almost_full", Severity::Critical, 9))
            .unwrap();
        let mut closed = finding("services.unit_failed", Severity::Critical, 0);
        closed.state = FindingState::Resolved;
        store.upsert_finding(&closed).unwrap();

        let order: Vec<_> = store
            .open_findings()
            .unwrap()
            .into_iter()
            .map(|f| f.rule.to_string())
            .collect();
        assert_eq!(
            order,
            [
                "storage.root_almost_full",
                "power.low_battery_action",
                "system.reboot_required"
            ]
        );
    }

    #[test]
    fn first_seen_is_kept_while_open_and_reset_after_closing() {
        let store = Store::open_in_memory().unwrap();
        let rule = RuleId::new("storage.root_almost_full").unwrap();
        store
            .upsert_finding(&finding(rule.as_str(), Severity::Warning, 100))
            .unwrap();

        let mut later = finding(rule.as_str(), Severity::Critical, 200);
        later.first_seen = Timestamp(200);
        store.upsert_finding(&later).unwrap();
        let stored = store.finding(&rule, None).unwrap().unwrap();
        assert_eq!(
            (stored.first_seen, stored.last_seen),
            (Timestamp(100), Timestamp(200))
        );
        assert_eq!(stored.severity, Severity::Critical);

        assert!(
            store
                .set_finding_state(&rule, None, FindingState::Resolved)
                .unwrap()
        );
        store
            .upsert_finding(&finding(rule.as_str(), Severity::Warning, 300))
            .unwrap();
        let back = store.finding(&rule, None).unwrap().unwrap();
        assert_eq!(back.first_seen, Timestamp(300));
    }

    #[test]
    fn changing_a_missing_finding_reports_false() {
        let store = Store::open_in_memory().unwrap();
        let rule = RuleId::new("storage.root_almost_full").unwrap();
        assert!(
            !store
                .set_finding_state(&rule, None, FindingState::Dismissed)
                .unwrap()
        );
    }
}
