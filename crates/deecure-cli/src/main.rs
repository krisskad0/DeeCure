// SPDX-License-Identifier: Apache-2.0

//! The `deecure` command: a thin client of the agent.
//!
//! Commands are listed in `docs/02-architecture/cli.md`.

mod render;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use deecure_core::rpc::{StatusReport, default_socket_path, method};
use deecure_core::{Timestamp, client};

/// See, understand, and fix everything in your Linux PC.
#[derive(Parser)]
#[command(name = "deecure", version, arg_required_else_help = true)]
struct Cli {
    /// Talk to the agent at PATH instead of $XDG_RUNTIME_DIR/deecure/agent.sock
    #[arg(long, global = true, value_name = "PATH")]
    socket: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show the problems DeeCure found, most serious first
    Status {
        /// Print the raw data as JSON, for scripts
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let Some(socket) = cli.socket.or_else(default_socket_path) else {
        eprintln!("deecure: XDG_RUNTIME_DIR is not set; pass --socket PATH");
        return ExitCode::FAILURE;
    };
    match cli.command {
        Command::Status { json } => status(&socket, json),
    }
}

fn status(socket: &Path, json: bool) -> ExitCode {
    let report: StatusReport = match client::call(socket, method::STATUS, None) {
        Ok(report) => report,
        Err(e) => {
            eprintln!("deecure: {e}");
            return ExitCode::FAILURE;
        }
    };
    if json {
        match serde_json::to_string_pretty(&report) {
            Ok(text) => println!("{text}"),
            Err(e) => {
                eprintln!("deecure: could not write JSON: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        print!(
            "{}",
            render::status(&report, Timestamp::now(), render::Style::detect())
        );
    }
    ExitCode::SUCCESS
}
