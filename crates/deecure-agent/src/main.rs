// SPDX-License-Identifier: Apache-2.0

//! The DeeCure agent.
//!
//! Runs as a systemd user service and answers the desktop app and CLI over a
//! private socket. It reads the system every minute and keeps the history.
//! Running rules comes in a later Phase 0 task.

use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use deecure_agent::collect::{self, SystemRoot};
use deecure_agent::store::{self, SharedStore, Store};
use deecure_agent::{server, socket};
use deecure_core::rpc::default_socket_path;
use deecure_core::{Timestamp, VERSION};
use tokio::signal::unix::{SignalKind, signal};

/// How often old data is rolled up and deleted.
const CLEANUP_EVERY: Duration = Duration::from_secs(60 * 60);

const USAGE: &str = "\
Usage: deecure-agent [OPTIONS]

Runs the DeeCure agent in the foreground. systemd normally starts it for you.

Options:
  --socket PATH  Listen on PATH instead of $XDG_RUNTIME_DIR/deecure/agent.sock
  --db PATH      Use the database at PATH instead of ~/.local/share/deecure/deecure.db
  -V, --version  Print the version
  -h, --help     Print this help";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut socket_path = None;
    let mut db_path = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--version" | "-V" => {
                println!("deecure-agent {VERSION}");
                return ExitCode::SUCCESS;
            }
            "--help" | "-h" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            "--socket" => match args.next() {
                Some(path) => socket_path = Some(PathBuf::from(path)),
                None => return usage_error("--socket needs a path"),
            },
            "--db" => match args.next() {
                Some(path) => db_path = Some(PathBuf::from(path)),
                None => return usage_error("--db needs a path"),
            },
            other => return usage_error(&format!("unknown argument '{other}'")),
        }
    }

    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(e) => {
            eprintln!("deecure-agent: could not start: {e}");
            return ExitCode::FAILURE;
        }
    };
    match runtime.block_on(run(socket_path, db_path)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("deecure-agent: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage_error(message: &str) -> ExitCode {
    eprintln!("deecure-agent: {message}\n\n{USAGE}");
    ExitCode::from(2)
}

async fn run(socket_path: Option<PathBuf>, db_path: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    let db_path = db_path.or_else(Store::default_path).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "HOME is not set; pass --db PATH")
    })?;
    let store: SharedStore = Arc::new(Mutex::new(Store::open(&db_path)?));
    eprintln!("deecure-agent: database {}", db_path.display());
    tokio::spawn(clean_up_regularly(store.clone()));
    tokio::spawn(collect::run(store.clone(), SystemRoot::real()));

    // Prefer the socket systemd opened for us. Otherwise open our own, and remove it on exit.
    let (listener, owned_path) = match socket::from_systemd()? {
        Some(listener) => {
            eprintln!("deecure-agent {VERSION}: started by systemd socket activation");
            (listener, None)
        }
        None => {
            let path = socket_path.or_else(default_socket_path).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "XDG_RUNTIME_DIR is not set; pass --socket PATH",
                )
            })?;
            let listener = socket::bind(&path)?;
            eprintln!("deecure-agent {VERSION}: listening on {}", path.display());
            (listener, Some(path))
        }
    };

    server::serve(listener, store, shutdown_signal()).await;

    if let Some(path) = owned_path {
        let _ = std::fs::remove_file(path);
    }
    eprintln!("deecure-agent: stopped");
    Ok(())
}

/// Rolls up and deletes old data now, then every [`CLEANUP_EVERY`].
async fn clean_up_regularly(store: SharedStore) {
    let mut timer = tokio::time::interval(CLEANUP_EVERY);
    loop {
        timer.tick().await;
        if let Err(e) = store::lock(&store).apply_retention(Timestamp::now()) {
            eprintln!("deecure-agent: cleanup failed: {e}");
        }
    }
}

/// Completes on SIGTERM (systemd stop) or SIGINT (Ctrl+C).
async fn shutdown_signal() {
    let (Ok(mut term), Ok(mut int)) = (
        signal(SignalKind::terminate()),
        signal(SignalKind::interrupt()),
    ) else {
        eprintln!("deecure-agent: could not listen for stop signals; stop it with SIGKILL");
        return std::future::pending().await;
    };
    tokio::select! {
        _ = term.recv() => {}
        _ = int.recv() => {}
    }
}
