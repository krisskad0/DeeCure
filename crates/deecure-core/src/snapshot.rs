// SPDX-License-Identifier: Apache-2.0

//! Inventories of the computer at one moment.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Timestamp;

/// Which inventory a snapshot holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotKind {
    /// CPU, memory, disks, graphics, sensors, battery.
    Hardware,
    /// Mounted filesystems and their usage.
    Filesystems,
    /// Installed packages (deb, snap, flatpak).
    Packages,
    /// systemd units and their state.
    Services,
    /// User accounts and groups.
    Users,
    /// Scheduled jobs: cron, timers, anacron, at.
    Jobs,
    /// Listening ports.
    Ports,
}

/// An inventory of one kind, taken at one moment.
///
/// `data` holds the collector's own typed record as JSON. Each collector
/// defines that record next to its code, so this crate stays small.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    /// Which inventory this is.
    pub kind: SnapshotKind,
    /// When it was taken.
    pub taken_at: Timestamp,
    /// The inventory itself.
    pub data: Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_shape_is_stable() {
        let snapshot = Snapshot {
            kind: SnapshotKind::Filesystems,
            taken_at: Timestamp(9),
            data: json!([{"mount": "/", "used_percent": 94.2}]),
        };
        assert_eq!(
            serde_json::to_value(&snapshot).unwrap(),
            json!({
                "kind": "filesystems",
                "taken_at": 9,
                "data": [{"mount": "/", "used_percent": 94.2}]
            })
        );
    }
}
