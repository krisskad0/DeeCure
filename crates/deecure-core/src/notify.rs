// SPDX-License-Identifier: Apache-2.0

//! Notification settings, shared by the agent (which sends them) and the app (which edits them).
//! Behaviour: `docs/02-architecture/notifications.md`.

use serde::{Deserialize, Serialize};

use crate::Timestamp;

/// When notifications wait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QuietHours {
    /// Never quiet.
    #[serde(rename = "off")]
    Off,
    /// 22:00 to 07:00, local time.
    #[default]
    #[serde(rename = "22-07")]
    TenToSeven,
    /// 23:00 to 08:00, local time.
    #[serde(rename = "23-08")]
    ElevenToEight,
}

impl QuietHours {
    /// Start and end, in minutes since local midnight; `None` when off.
    pub fn range(self) -> Option<(u16, u16)> {
        match self {
            QuietHours::Off => None,
            QuietHours::TenToSeven => Some((22 * 60, 7 * 60)),
            QuietHours::ElevenToEight => Some((23 * 60, 8 * 60)),
        }
    }

    /// True when `minute_of_day` (0–1439, local) is inside the quiet hours.
    pub fn contains(self, minute_of_day: u16) -> bool {
        match self.range() {
            None => false,
            Some((start, end)) if start > end => minute_of_day >= start || minute_of_day < end,
            Some((start, end)) => minute_of_day >= start && minute_of_day < end,
        }
    }
}

/// What the person wants to be told about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NotifySettings {
    /// Critical problems right away (on by default).
    pub critical: bool,
    /// Warnings right away too (off by default: they wait for the weekly summary).
    pub warnings: bool,
    /// The weekly summary, Sunday at 18:00 (on by default).
    pub weekly: bool,
    /// When nothing is shown; notifications wait until the quiet hours end.
    pub quiet_hours: QuietHours,
}

impl Default for NotifySettings {
    fn default() -> Self {
        NotifySettings {
            critical: true,
            warnings: false,
            weekly: true,
            quiet_hours: QuietHours::default(),
        }
    }
}

/// A notification that was shown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SentNotification {
    /// When it was shown.
    pub at: Timestamp,
    /// Its title.
    pub title: String,
    /// Its text.
    pub body: String,
}

/// How long history is kept (Settings › Data).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum KeepHistory {
    /// 90 days.
    #[serde(rename = "90d")]
    Days90,
    /// 1 year.
    #[serde(rename = "1y")]
    Year1,
    /// 2 years (the default).
    #[default]
    #[serde(rename = "2y")]
    Years2,
}

impl KeepHistory {
    /// In days.
    pub fn days(self) -> i64 {
        match self {
            KeepHistory::Days90 => 90,
            KeepHistory::Year1 => 365,
            KeepHistory::Years2 => 730,
        }
    }
}

/// Data settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DataSettings {
    /// How long history is kept.
    pub keep_history: KeepHistory,
}

/// What `delete_history` removed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deleted {
    /// Rows deleted.
    pub rows: u64,
    /// Bytes the database file gave back.
    pub bytes_freed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keep_history_choices() {
        assert_eq!(DataSettings::default().keep_history.days(), 730);
        let d: DataSettings = serde_json::from_value(json!({"keep_history": "90d"})).unwrap();
        assert_eq!(d.keep_history.days(), 90);
        assert!(serde_json::from_value::<DataSettings>(json!({"keep_history": "5y"})).is_err());
    }

    #[test]
    fn quiet_hours_cross_midnight() {
        let q = QuietHours::TenToSeven;
        assert!(q.contains(22 * 60));
        assert!(q.contains(23 * 60 + 59));
        assert!(q.contains(0));
        assert!(q.contains(6 * 60 + 59));
        assert!(!q.contains(7 * 60));
        assert!(!q.contains(12 * 60));
        assert!(!QuietHours::Off.contains(23 * 60));
    }

    #[test]
    fn json_shape_and_defaults() {
        assert_eq!(
            serde_json::to_value(NotifySettings::default()).unwrap(),
            json!({"critical": true, "warnings": false, "weekly": true, "quiet_hours": "22-07"})
        );
        let partial: NotifySettings = serde_json::from_value(json!({"warnings": true})).unwrap();
        assert!(partial.warnings && partial.critical);
        assert!(serde_json::from_value::<NotifySettings>(json!({"quiet_hours": "21-06"})).is_err());
        assert!(serde_json::from_value::<NotifySettings>(json!({"loud": true})).is_err());
    }
}
