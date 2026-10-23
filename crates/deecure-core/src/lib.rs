// SPDX-License-Identifier: Apache-2.0

//! Shared types for DeeCure.
//!
//! The agent, the CLI, the root helper, and the desktop app all depend on this
//! crate, so they always agree on the shape of findings, metrics, events, and
//! the messages they exchange. Every type serializes to the JSON shown in its
//! tests; change those tests only on purpose.

mod category;
pub mod client;
mod event;
mod finding;
pub mod fix;
pub mod health;
mod id;
pub mod local;
mod metric;
pub mod rpc;
mod severity;
mod snapshot;
mod time;

pub use category::{Category, UnknownCategory};
pub use event::{Event, EventKind};
pub use finding::{Finding, FindingState};
pub use id::{FixActionId, IdError, RuleId};
pub use metric::{Metric, Unit, thin};
pub use severity::{Severity, Status};
pub use snapshot::{Snapshot, SnapshotKind};
pub use time::Timestamp;

/// The DeeCure version, taken from the workspace `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_semver() {
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert_eq!(parts.len(), 3, "version must be MAJOR.MINOR.PATCH");
        assert!(parts.iter().all(|p| p.parse::<u32>().is_ok()));
    }
}
