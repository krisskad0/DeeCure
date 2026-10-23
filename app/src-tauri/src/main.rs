// SPDX-License-Identifier: Apache-2.0

//! The DeeCure desktop app: a Tauri window around the React UI.
//!
//! The UI never talks to the agent directly. It calls the commands below, and
//! they talk to the agent over its private socket (`docs/02-architecture/ipc-protocol.md`).

use std::path::PathBuf;

use deecure_core::client::{self, ClientError};
use deecure_core::fix::{FixRun, Plan};
use deecure_core::rpc::{StatusReport, default_socket_path, method};
use deecure_core::{Metric, Snapshot};
use serde::Serialize;
use serde_json::{Value, json};
use tauri::Emitter;

/// An error the UI can show: `kind` tells it which screen to show, `message` is for people.
#[derive(Debug, Serialize)]
struct AppError {
    /// `agent_not_running` or `other`.
    kind: &'static str,
    /// A plain-English sentence.
    message: String,
}

impl From<ClientError> for AppError {
    fn from(e: ClientError) -> Self {
        let kind = match e {
            ClientError::NotRunning(_) => "agent_not_running",
            _ => "other",
        };
        AppError {
            kind,
            message: e.to_string(),
        }
    }
}

fn socket() -> Result<PathBuf, AppError> {
    default_socket_path().ok_or_else(|| AppError {
        kind: "other",
        message: "XDG_RUNTIME_DIR is not set, so the agent's socket cannot be found.".into(),
    })
}

/// Asks the agent one question. Socket calls block, so they run off the UI thread.
async fn ask<T: serde::de::DeserializeOwned + Send + 'static>(
    method: &'static str,
    params: Option<Value>,
) -> Result<T, AppError> {
    let socket = socket()?;
    tauri::async_runtime::spawn_blocking(move || client::call(&socket, method, params))
        .await
        .map_err(|e| AppError {
            kind: "other",
            message: e.to_string(),
        })?
        .map_err(AppError::from)
}

/// The open findings, most serious first, with the health score.
#[tauri::command]
async fn status() -> Result<StatusReport, AppError> {
    ask(method::STATUS, None).await
}

/// The newest inventory of one kind (such as `services`), or `None` if there is none yet.
#[tauri::command]
async fn snapshot(kind: String) -> Result<Option<Snapshot>, AppError> {
    ask(method::SNAPSHOT, Some(json!({ "kind": kind }))).await
}

/// Live data read now, such as `system` facts or `processes`.
#[tauri::command]
async fn inventory(kind: String) -> Result<Value, AppError> {
    ask(method::INVENTORY, Some(json!({ "kind": kind }))).await
}

/// Points for one metric over a time range (milliseconds since 1970), averaged down
/// to about `max_points`.
#[tauri::command]
async fn metrics(
    name: String,
    subject: Option<String>,
    since: i64,
    until: i64,
    max_points: Option<usize>,
) -> Result<Vec<Metric>, AppError> {
    let params = json!({
        "name": name, "subject": subject, "since": since, "until": until, "max_points": max_points,
    });
    ask(method::METRICS, Some(params)).await
}

/// The newest value of every metric.
#[tauri::command]
async fn latest() -> Result<Vec<Metric>, AppError> {
    ask(method::LATEST, None).await
}

/// What changed from `since` (ms) on, newest first, without config diffs.
#[tauri::command]
async fn events(
    since: i64,
    until: Option<i64>,
    kinds: Option<Vec<String>>,
    limit: Option<usize>,
) -> Result<Value, AppError> {
    let params = json!({
        "since": since, "until": until, "kinds": kinds.unwrap_or_default(), "limit": limit,
    });
    ask(method::EVENTS, Some(params)).await
}

/// One event with all its details, such as a config file's diff.
#[tauri::command]
async fn event(id: i64) -> Result<Value, AppError> {
    ask(method::EVENT, Some(json!({ "id": id }))).await
}

/// A fix's plan: what it will do, before anything runs.
#[tauri::command]
async fn fix_plan(action: String, subject: Option<String>) -> Result<Plan, AppError> {
    ask(
        method::FIX_PLAN,
        Some(json!({ "action": action, "subject": subject })),
    )
    .await
}

/// What `fix_apply` returns: the run as saved.
#[derive(Debug, Serialize)]
struct Applied {
    run: FixRun,
}

/// Runs the chosen steps here, in the person's session (so polkit can ask for the password),
/// sends a `fix-progress` event after each step, then saves the run with the agent.
#[tauri::command]
async fn fix_apply(
    app: tauri::AppHandle,
    plan: Plan,
    chosen: Vec<String>,
) -> Result<Applied, AppError> {
    let mut run = deecure_fix::apply(&plan, &chosen, |step| {
        let _ = app.emit("fix-progress", step);
    })
    .await;
    run.id = ask(method::FIX_RECORD, Some(json!({ "run": run }))).await?;
    Ok(Applied { run })
}

/// Past fixes, newest first.
#[tauri::command]
async fn fix_runs() -> Result<Vec<FixRun>, AppError> {
    ask(method::FIX_RUNS, None).await
}

/// Takes a fix back, then marks it undone.
#[tauri::command]
async fn fix_undo(id: i64) -> Result<Vec<String>, AppError> {
    let runs: Vec<FixRun> = ask(method::FIX_RUNS, None).await?;
    let run = runs
        .into_iter()
        .find(|r| r.id == id)
        .ok_or_else(|| other(format!("no fix run {id}")))?;
    let done = deecure_fix::undo(&run).await.map_err(other)?;
    let _: bool = ask(method::FIX_UNDONE, Some(json!({ "id": id }))).await?;
    Ok(done)
}

/// True when the admin helper is installed.
#[tauri::command]
async fn helper_status() -> bool {
    deecure_fix::helper_installed().await
}

/// An admin read-out through the helper (polkit may ask for the password).
#[tauri::command]
async fn helper_read(what: String, arg: String) -> Result<String, AppError> {
    deecure_fix::helper_read(&what, &arg).await.map_err(other)
}

fn other(message: impl Into<String>) -> AppError {
    AppError {
        kind: "other",
        message: message.into(),
    }
}

/// This computer, for the top bar's machine picker.
#[derive(Debug, Serialize)]
struct Machine {
    /// The host name, such as `riya-laptop`.
    hostname: String,
    /// The OS name, such as `Ubuntu 24.04.5 LTS`.
    os: String,
    /// The user's login name, for the avatar.
    user: String,
    /// The desktop, such as `ubuntu:GNOME`, from the session the app runs in.
    desktop: Option<String>,
    /// `wayland` or `x11`.
    session_type: Option<String>,
}

/// Reads the host name, OS name, and user name. Missing values become "unknown".
#[tauri::command]
fn machine() -> Machine {
    let read = |path: &str| std::fs::read_to_string(path).unwrap_or_default();
    let os = read("/etc/os-release")
        .lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        .map(|name| name.trim_matches('"').to_owned());
    let hostname = read("/proc/sys/kernel/hostname").trim().to_owned();
    let user = std::env::var("USER").or_else(|_| std::env::var("LOGNAME"));
    let known = |value: String| {
        if value.is_empty() {
            "unknown".to_owned()
        } else {
            value
        }
    };
    Machine {
        hostname: known(hostname),
        os: known(os.unwrap_or_default()),
        user: known(user.unwrap_or_default()),
        desktop: std::env::var("XDG_CURRENT_DESKTOP")
            .ok()
            .filter(|v| !v.is_empty()),
        session_type: std::env::var("XDG_SESSION_TYPE")
            .ok()
            .filter(|v| !v.is_empty()),
    }
}

fn main() {
    let result = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            status,
            snapshot,
            inventory,
            metrics,
            latest,
            events,
            event,
            machine,
            fix_plan,
            fix_apply,
            fix_runs,
            fix_undo,
            helper_status,
            helper_read
        ])
        .run(tauri::generate_context!());
    if let Err(e) = result {
        eprintln!("deecure-desktop: could not start: {e}");
        std::process::exit(1);
    }
}
