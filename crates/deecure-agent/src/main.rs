// SPDX-License-Identifier: Apache-2.0

//! The DeeCure agent.
//!
//! Runs as a systemd user service. It will collect system data, run rules,
//! store history, and answer the desktop app and CLI over a local socket.
//! For now it only reports its version: the socket server comes in Phase 0, task 5.

use std::process::ExitCode;

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("--version" | "-V") => {
            println!("deecure-agent {}", deecure_core::VERSION);
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!(
                "deecure-agent {}: the socket server is not built yet (Phase 0, task 5)",
                deecure_core::VERSION
            );
            ExitCode::FAILURE
        }
    }
}
