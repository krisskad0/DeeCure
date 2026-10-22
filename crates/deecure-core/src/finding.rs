// SPDX-License-Identifier: Apache-2.0

//! Findings: problems and warnings that rules report.

use serde::{Deserialize, Serialize};

use crate::{Category, FixActionId, RuleId, Severity, Timestamp};

/// Whether a finding still needs attention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingState {
    /// The rule still matches.
    Open,
    /// The rule no longer matches, by a fix or by itself.
    Resolved,
    /// The user chose to hide it.
    Dismissed,
}

/// A problem or warning that a rule found.
///
/// One rule can match many things (for example one finding per failed service),
/// so a finding is identified by its rule plus its subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// The rule that reported it.
    pub rule: RuleId,
    /// What it is about, when the rule can match many things: a mount point,
    /// a unit name, a device.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// How serious it is right now.
    pub severity: Severity,
    /// Whether it still needs attention.
    pub state: FindingState,
    /// One short line, such as "Main disk is almost full".
    pub title: String,
    /// Plain-English meaning, with the user's own numbers filled in.
    pub explain: String,
    /// When it was first reported.
    pub first_seen: Timestamp,
    /// When the rule last matched.
    pub last_seen: Timestamp,
    /// Fix actions that can solve it, in the order to offer them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixes: Vec<FixActionId>,
}

impl Finding {
    /// The category of the rule that reported it.
    pub fn category(&self) -> Category {
        self.rule.category()
    }

    /// The pair that identifies this finding: rule and subject.
    pub fn key(&self) -> (&RuleId, Option<&str>) {
        (&self.rule, self.subject.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn disk_full() -> Finding {
        Finding {
            rule: RuleId::new("storage.root_almost_full").unwrap(),
            subject: Some("/".into()),
            severity: Severity::Critical,
            state: FindingState::Open,
            title: "Main disk is almost full".into(),
            explain: "Your main disk has only 13.8 GB left.".into(),
            first_seen: Timestamp(1_000),
            last_seen: Timestamp(2_000),
            fixes: vec![FixActionId::new("storage.clean_apt_cache").unwrap()],
        }
    }

    #[test]
    fn json_shape_is_stable() {
        assert_eq!(
            serde_json::to_value(disk_full()).unwrap(),
            json!({
                "rule": "storage.root_almost_full",
                "subject": "/",
                "severity": "critical",
                "state": "open",
                "title": "Main disk is almost full",
                "explain": "Your main disk has only 13.8 GB left.",
                "first_seen": 1000,
                "last_seen": 2000,
                "fixes": ["storage.clean_apt_cache"]
            })
        );
    }

    #[test]
    fn optional_fields_may_be_missing() {
        let finding: Finding = serde_json::from_value(json!({
            "rule": "services.unit_failed",
            "severity": "warning",
            "state": "open",
            "title": "A service keeps failing",
            "explain": "…",
            "first_seen": 1,
            "last_seen": 1
        }))
        .unwrap();
        assert_eq!(finding.subject, None);
        assert!(finding.fixes.is_empty());
        assert_eq!(finding.category(), Category::Services);
    }

    #[test]
    fn key_is_rule_plus_subject() {
        let finding = disk_full();
        assert_eq!(finding.key().0.as_str(), "storage.root_almost_full");
        assert_eq!(finding.key().1, Some("/"));
    }
}
