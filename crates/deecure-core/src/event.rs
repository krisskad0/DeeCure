// SPDX-License-Identifier: Apache-2.0

//! Things that happened, for the "what changed" timeline.

use serde::{Deserialize, Serialize};

use crate::Timestamp;

/// What kind of thing happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// A package was installed.
    PackageInstalled,
    /// A package was upgraded.
    PackageUpgraded,
    /// A package was removed.
    PackageRemoved,
    /// A config file changed.
    ConfigChanged,
    /// A device was plugged in.
    DeviceAdded,
    /// A device was unplugged.
    DeviceRemoved,
    /// The computer started.
    Boot,
    /// The computer stopped without an orderly shutdown: power loss, a freeze, or a
    /// forced power-off.
    UncleanShutdown,
    /// A service failed.
    ServiceFailed,
    /// A program crashed.
    Crash,
    /// The kernel stopped a program because memory ran out.
    OutOfMemoryKill,
    /// The kernel logged a hardware or kernel problem: disk errors, a graphics hang, a kernel
    /// crash, a machine check, a firmware crash, an overloaded USB port, or a thermal shutdown.
    HardwareError,
    /// Someone failed to log in.
    LoginFailed,
    /// DeeCure applied a fix.
    FixApplied,
    /// DeeCure undid a fix.
    FixUndone,
}

/// Something that happened at one moment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// The stored event's number; `None` before it is saved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// What kind of thing happened.
    pub kind: EventKind,
    /// When it happened.
    pub at: Timestamp,
    /// One short line, such as "Kernel updated 6.8 → 6.11".
    pub title: String,
    /// More detail, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// What it is about: a package, a file path, a device, a unit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Where DeeCure learned about it, such as `dpkg`, `journal`, or `udev`.
    pub source: String,
    /// Details that depend on the kind, such as package versions or a file's diff
    /// (`docs/02-architecture/storage.md#events`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_shape_is_stable() {
        let event = Event {
            id: None,
            data: None,
            kind: EventKind::OutOfMemoryKill,
            at: Timestamp(7),
            title: "Chrome was stopped: memory ran out".into(),
            detail: None,
            subject: Some("chrome".into()),
            source: "journal".into(),
        };
        assert_eq!(
            serde_json::to_value(&event).unwrap(),
            json!({
                "kind": "out_of_memory_kill",
                "at": 7,
                "title": "Chrome was stopped: memory ran out",
                "subject": "chrome",
                "source": "journal"
            })
        );
    }
}
