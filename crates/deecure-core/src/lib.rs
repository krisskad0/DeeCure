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
pub mod notify;
pub mod report;
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

/// The app ID: the desktop file, AppStream, and notifications use it. Open decision #6
/// (`docs/01-product/open-decisions.md`): it must never change after the first release.
pub const APP_ID: &str = "io.github.krisskad0.DeeCure";

/// The DeeCure version, taken from the workspace `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_semver() {
        // MAJOR.MINOR.PATCH, with an optional pre-release such as -alpha.1 or -rc.2.
        let (core, pre) = VERSION.split_once('-').unwrap_or((VERSION, ""));
        let parts: Vec<&str> = core.split('.').collect();
        assert_eq!(parts.len(), 3, "version must be MAJOR.MINOR.PATCH");
        assert!(parts.iter().all(|p| p.parse::<u32>().is_ok()));
        if !pre.is_empty() {
            let (kind, n) = pre.split_once('.').expect("pre-release looks like alpha.1");
            assert!(["alpha", "beta", "rc"].contains(&kind), "{pre}");
            assert!(n.parse::<u32>().is_ok(), "{pre}");
        }
    }
}
