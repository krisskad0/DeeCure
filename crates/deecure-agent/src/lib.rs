// SPDX-License-Identifier: Apache-2.0

//! The DeeCure agent, as a library so tests can drive it.
//!
//! The binary in `main.rs` only parses arguments and starts [`server::serve`].

pub mod collect;
pub mod context;
pub mod fixes;
pub mod handler;
pub mod history;
pub mod inventory;
pub mod kb;
pub mod notify;
pub mod rules;
pub mod server;
pub mod socket;
pub mod store;
