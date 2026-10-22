// SPDX-License-Identifier: Apache-2.0

//! Points in time.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// A point in time, as milliseconds since 1970-01-01 UTC.
///
/// One plain number keeps JSON, SQLite, and JavaScript (`new Date(ms)`) in step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(pub i64);

impl Timestamp {
    /// The current time.
    pub fn now() -> Self {
        let since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);
        Timestamp(i64::try_from(since_epoch.as_millis()).unwrap_or(i64::MAX))
    }

    /// Milliseconds since 1970-01-01 UTC.
    pub const fn as_millis(self) -> i64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_is_after_2026() {
        // 2026-01-01T00:00:00Z
        assert!(Timestamp::now() > Timestamp(1_767_225_600_000));
    }

    #[test]
    fn serializes_as_a_plain_number() {
        assert_eq!(serde_json::to_string(&Timestamp(42)).unwrap(), "42");
    }
}
