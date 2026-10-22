// SPDX-License-Identifier: Apache-2.0

//! The health score (`docs/02-architecture/health-score.md`).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Category, Finding, FindingState};

/// The most points one area can take off the score.
pub const AREA_SHARE: u8 = 15;

/// Points one area takes off the score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deduction {
    /// The area.
    pub category: Category,
    /// Points taken off, at most [`AREA_SHARE`].
    pub points: u8,
}

/// The score out of 100 and what lowered it, biggest first.
///
/// Each open finding takes points off its area (critical 15, warning 5, info 0),
/// and an area can lose at most [`AREA_SHARE`] points.
pub fn score(findings: &[Finding]) -> (u8, Vec<Deduction>) {
    let mut per_area: BTreeMap<Category, u32> = BTreeMap::new();
    for finding in findings.iter().filter(|f| f.state == FindingState::Open) {
        *per_area.entry(finding.category()).or_default() +=
            u32::from(finding.severity.score_deduction());
    }
    let mut deductions: Vec<Deduction> = per_area
        .into_iter()
        .filter(|(_, points)| *points > 0)
        .map(|(category, points)| Deduction {
            category,
            points: u8::try_from(points.min(u32::from(AREA_SHARE))).unwrap_or(AREA_SHARE),
        })
        .collect();
    deductions.sort_by(|a, b| b.points.cmp(&a.points).then(a.category.cmp(&b.category)));
    let lost: u32 = deductions.iter().map(|d| u32::from(d.points)).sum();
    (
        u8::try_from(100u32.saturating_sub(lost)).unwrap_or(0),
        deductions,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RuleId, Severity, Timestamp};

    fn finding(rule: &str, severity: Severity, subject: &str) -> Finding {
        Finding {
            rule: RuleId::new(rule).unwrap(),
            subject: Some(subject.into()),
            severity,
            state: FindingState::Open,
            title: String::new(),
            explain: String::new(),
            first_seen: Timestamp(0),
            last_seen: Timestamp(0),
            fixes: vec![],
        }
    }

    #[test]
    fn deducts_per_area_with_a_cap() {
        let findings = [
            finding("storage.root_almost_full", Severity::Critical, "/"),
            finding("services.unit_failed", Severity::Warning, "a.service"),
            finding("services.unit_failed", Severity::Warning, "b.service"),
            finding("services.unit_failed", Severity::Warning, "c.service"),
            finding("services.unit_failed", Severity::Warning, "d.service"),
            finding("power.battery_worn", Severity::Info, "BAT0"),
        ];
        let (score, deductions) = score(&findings);
        assert_eq!(score, 70, "storage 15 + services capped at 15");
        assert_eq!(
            deductions,
            [
                Deduction {
                    category: Category::Services,
                    points: 15
                },
                Deduction {
                    category: Category::Storage,
                    points: 15
                },
            ]
        );
    }

    #[test]
    fn closed_findings_and_info_cost_nothing() {
        let mut resolved = finding("storage.root_almost_full", Severity::Critical, "/");
        resolved.state = FindingState::Resolved;
        let info = finding("power.battery_worn", Severity::Info, "BAT0");
        assert_eq!(score(&[resolved, info]), (100, vec![]));
    }
}
