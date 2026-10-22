// SPDX-License-Identifier: Apache-2.0

//! How serious something is, and the badge the UI shows for it.

use serde::{Deserialize, Serialize};

/// How serious a finding is. Ordered from least to most serious.
///
/// Rule files use the same words: `info`, `warning`, `critical`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Good to know. No action needed.
    Info,
    /// Will cause trouble soon, or a weak setting.
    Warning,
    /// Data loss, the PC may not boot, or a security hole is open now.
    Critical,
}

impl Severity {
    /// Points one open finding takes off its category's health score
    /// (`docs/02-architecture/health-score.md`).
    pub const fn score_deduction(self) -> u8 {
        match self {
            Severity::Info => 0,
            Severity::Warning => 5,
            Severity::Critical => 15,
        }
    }
}

/// The badge shown for a page, a panel, or a row.
///
/// Serialized with the short words the UI spec uses: `ok`, `info`, `warn`, `crit`, `off`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Status {
    /// Healthy.
    #[serde(rename = "ok")]
    Ok,
    /// Something to know about.
    #[serde(rename = "info")]
    Info,
    /// Needs attention soon.
    #[serde(rename = "warn")]
    Warn,
    /// Needs attention now.
    #[serde(rename = "crit")]
    Crit,
    /// Cannot be checked on this computer.
    #[serde(rename = "off")]
    Unknown,
}

impl Status {
    /// The word shown next to the badge icon.
    pub const fn label(self) -> &'static str {
        match self {
            Status::Ok => "Healthy",
            Status::Info => "Info",
            Status::Warn => "Warning",
            Status::Crit => "Critical",
            Status::Unknown => "Unknown",
        }
    }
}

impl From<Severity> for Status {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Info => Status::Info,
            Severity::Warning => Status::Warn,
            Severity::Critical => Status::Crit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_orders_from_least_to_most_serious() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Critical);
    }

    #[test]
    fn severity_uses_rule_file_words() {
        assert_eq!(serde_json::to_value(Severity::Warning).unwrap(), "warning");
        assert_eq!(
            serde_json::from_str::<Severity>("\"critical\"").unwrap(),
            Severity::Critical
        );
    }

    #[test]
    fn status_uses_ui_words() {
        let words: Vec<_> = [
            Status::Ok,
            Status::Info,
            Status::Warn,
            Status::Crit,
            Status::Unknown,
        ]
        .map(|s| serde_json::to_value(s).unwrap())
        .to_vec();
        assert_eq!(words, ["ok", "info", "warn", "crit", "off"]);
    }

    #[test]
    fn deductions_match_the_health_score_doc() {
        assert_eq!(Severity::Critical.score_deduction(), 15);
        assert_eq!(Severity::Warning.score_deduction(), 5);
        assert_eq!(Severity::Info.score_deduction(), 0);
    }

    #[test]
    fn severity_maps_to_status() {
        assert_eq!(Status::from(Severity::Critical), Status::Crit);
        assert_eq!(Status::from(Severity::Critical).label(), "Critical");
    }
}
