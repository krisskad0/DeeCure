// SPDX-License-Identifier: Apache-2.0

//! The DeeCure agent, as a library so tests can drive it.
//!
//! The binary in `main.rs` only parses arguments and starts [`server::serve`].

pub mod handler;
pub mod server;
pub mod socket;
pub mod store;
