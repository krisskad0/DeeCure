// SPDX-License-Identifier: Apache-2.0

//! Shared types for DeeCure.
//!
//! The agent, the CLI, the root helper, and the desktop app all depend on this
//! crate, so they always agree on the shape of findings, metrics, events, and
//! the messages they exchange.

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
