// SPDX-License-Identifier: Apache-2.0

//! The DeeCure agent.
//!
//! Runs as a systemd user service and answers the desktop app and CLI over a
//! private socket. Every minute it reads the system, keeps the history, and
//! checks the rules, so `status` always shows the current problems.

use std::collections::HashSet;
use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use deecure_agent::collect::{self, SystemRoot};
use deecure_agent::context::{Agent, Facts};
use deecure_agent::store::{self, SharedStore, Store};
use deecure_agent::{history, kb, notify, rules, server, socket};
use deecure_core::rpc::default_socket_path;
use deecure_core::{Metric, Timestamp, Unit, VERSION, health};
use tokio::signal::unix::{SignalKind, signal};

/// How often old data is rolled up and deleted.
const CLEANUP_EVERY: Duration = Duration::from_secs(60 * 60);

const USAGE: &str = "\
Usage: deecure-agent [OPTIONS]

Runs the DeeCure agent in the foreground. systemd normally starts it for you.

Options:
  --socket PATH  Listen on PATH instead of $XDG_RUNTIME_DIR/deecure/agent.sock
  --db PATH      Use the database at PATH instead of ~/.local/share/deecure/deecure.db
  --rules DIR    Read rules from DIR instead of /usr/share/deecure/rules
  -V, --version  Print the version
  -h, --help     Print this help";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut socket_path = None;
    let mut db_path = None;
    let mut rules_dir = None;
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
            "--rules" => match args.next() {
                Some(path) => rules_dir = Some(PathBuf::from(path)),
                None => return usage_error("--rules needs a folder"),
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
    let result = runtime.block_on(run(socket_path, db_path, rules_dir));
    // A read-only scan (folder sizes) can run for minutes on its own thread. Wait a little for
    // background work, then exit: the database is in WAL mode, so an unfinished write is rolled
    // back, never left half done. Without this, stopping the agent waited for the scan.
    runtime.shutdown_timeout(std::time::Duration::from_secs(5));
    match result {
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

async fn run(
    socket_path: Option<PathBuf>,
    db_path: Option<PathBuf>,
    rules_dir: Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let db_path = db_path.or_else(Store::default_path).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "HOME is not set; pass --db PATH")
    })?;
    let store: SharedStore = Arc::new(Mutex::new(Store::open(&db_path)?));
    eprintln!("deecure-agent: database {}", db_path.display());
    tokio::spawn(clean_up_regularly(store.clone()));
    let rules_dir = rules_dir.unwrap_or_else(|| PathBuf::from(rules::INSTALLED_DIR));
    let started = std::time::Instant::now();
    let (rules, errors) = rules::load_dir(&rules_dir);
    let rule_count = rules.len();
    let links: Vec<(String, String)> = rules
        .iter()
        .filter_map(|r| Some((r.id.to_string(), r.learn_more.clone()?)))
        .collect();
    let kb_dir = kb::beside(&rules_dir);
    let (knowledge, kb_errors) = kb::load_dir(&kb_dir, &links);
    for error in &kb_errors {
        eprintln!("deecure-agent: knowledge base: {error}");
    }
    eprintln!(
        "deecure-agent: knowledge base from {}: {} services, {} ports, {} errors, {} topics",
        kb_dir.display(),
        knowledge.services.len(),
        knowledge.ports.len(),
        knowledge.errors.len(),
        knowledge.topics.len()
    );
    for error in &errors {
        eprintln!("deecure-agent: skipped a rule: {error}");
    }
    eprintln!(
        "deecure-agent: {} rules from {}",
        rules.len(),
        rules_dir.display()
    );

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

    let agent = Agent::new(store.clone(), SystemRoot::real());
    tokio::spawn(history::run(store.clone(), SystemRoot::real()));
    tokio::spawn(collect::run(
        store.clone(),
        SystemRoot::real(),
        agent.recheck.clone(),
        rule_checker(rules),
    ));
    tokio::spawn(notify::run(agent.clone()));
    tokio::spawn(deecure_agent::updates::run(agent.clone()));
    let _ = agent.facts.set(Facts {
        database: db_path.clone(),
        rules_dir: rules_dir.clone(),
        rules: rule_count,
        started,
        kb: knowledge,
    });
    server::serve(listener, agent, shutdown_signal()).await;

    if let Some(path) = owned_path {
        let _ = std::fs::remove_file(path);
    }
    eprintln!("deecure-agent: stopped");
    Ok(())
}

/// Checks every rule against the newest data. Each rule mistake is logged once.
fn rule_checker(rules: Vec<rules::Rule>) -> impl FnMut(&Store, Timestamp) + Send + 'static {
    let mut logged = HashSet::new();
    move |store, now| {
        let result =
            rules::gather(store, now).and_then(|facts| rules::apply(store, &rules, &facts, now));
        match result {
            Ok(mistakes) => {
                for mistake in mistakes {
                    if logged.insert(mistake.clone()) {
                        eprintln!("deecure-agent: {mistake}");
                    }
                }
                if let Err(e) = record_score(store, now) {
                    eprintln!("deecure-agent: could not save the health score: {e}");
                }
            }
            Err(e) => eprintln!("deecure-agent: could not check rules: {e}"),
        }
    }
}

/// Keeps the health score as a metric, so the app can show how it changed over time.
fn record_score(store: &Store, now: Timestamp) -> Result<(), store::StoreError> {
    let (score, _) = health::score(&store.open_findings()?);
    store.add_metric(&Metric {
        name: "health.score".into(),
        subject: None,
        value: f64::from(score),
        unit: Unit::Count,
        at: now,
    })
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
