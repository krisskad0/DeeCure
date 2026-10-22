// SPDX-License-Identifier: Apache-2.0

//! The `deecure` command.
//!
//! A thin client of the agent. For now it only knows `--version` and `--help`:
//! `deecure status` comes in Phase 0, task 9.

use std::process::ExitCode;

const USAGE: &str = "\
Usage: deecure [OPTIONS]

Options:
  -V, --version  Print the version
  -h, --help     Print this help

Commands such as `deecure status` are coming soon.";

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("--version" | "-V") => {
            println!("deecure {}", deecure_core::VERSION);
            ExitCode::SUCCESS
        }
        Some("--help" | "-h") | None => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("deecure: unknown argument '{other}'\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}
