// SPDX-License-Identifier: Apache-2.0

//! The DeeCure desktop app: a Tauri window around the React UI.
//!
//! The UI never talks to the agent directly. It calls the commands below, and
//! they talk to the agent over its private socket (`docs/02-architecture/ipc-protocol.md`).

mod tray;

use std::path::PathBuf;

use deecure_core::client::{self, ClientError};
use deecure_core::fix::{FixRun, Plan};
use deecure_core::rpc::{StatusReport, default_socket_path, method};
use deecure_core::{Metric, Snapshot};
use serde::Serialize;
use serde_json::{Value, json};
use tauri::{Emitter, Manager};

/// An error the UI can show: `kind` tells it which screen to show, `message` is for people.
#[derive(Debug, Serialize)]
struct AppError {
    /// `agent_not_running`, `agent_outdated`, or `other`.
    kind: &'static str,
    /// A plain-English sentence.
    message: String,
}

impl From<ClientError> for AppError {
    fn from(e: ClientError) -> Self {
        match e {
            ClientError::NotRunning(_) => AppError {
                kind: "agent_not_running",
                message: e.to_string(),
            },
            // The app is newer than the agent that runs (it was started before an update).
            // An older agent also does not know newer inventories.
            ClientError::Agent(ref rpc)
                if rpc.code == deecure_core::rpc::RpcError::METHOD_NOT_FOUND
                    || (rpc.code == deecure_core::rpc::RpcError::INVALID_PARAMS
                        && rpc.message.starts_with("unknown inventory")) =>
            AppError {
                kind: "agent_outdated",
                message: "The background agent is older than this app, so it cannot answer this yet. Restart the agent to use the new version.".into(),
            },
            _ => AppError {
                kind: "other",
                message: e.to_string(),
            },
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

/// The notification settings.
#[tauri::command]
async fn notify_settings() -> Result<Value, AppError> {
    ask(method::NOTIFY_SETTINGS, None).await
}

/// Saves the notification settings; the agent checks them.
#[tauri::command]
async fn set_notify_settings(settings: Value) -> Result<Value, AppError> {
    ask(
        method::SET_NOTIFY_SETTINGS,
        Some(json!({ "settings": settings })),
    )
    .await
}

/// Shows a test notification.
#[tauri::command]
async fn notify_test() -> Result<bool, AppError> {
    ask(method::NOTIFY_TEST, None).await
}

/// The last notifications shown.
#[tauri::command]
async fn notifications() -> Result<Value, AppError> {
    ask(method::NOTIFICATIONS, None).await
}

/// How long history is kept.
#[tauri::command]
async fn data_settings() -> Result<Value, AppError> {
    ask(method::DATA_SETTINGS, None).await
}

/// Saves how long history is kept; a shorter time cleans up at once.
#[tauri::command]
async fn set_data_settings(settings: Value) -> Result<Value, AppError> {
    ask(
        method::SET_DATA_SETTINGS,
        Some(json!({ "settings": settings })),
    )
    .await
}

/// Deletes all history (restore points and settings stay).
#[tauri::command]
async fn delete_history() -> Result<Value, AppError> {
    ask(method::DELETE_HISTORY, Some(json!({ "confirm": true }))).await
}

/// A report of the last 7 days, made now.
#[tauri::command]
async fn report() -> Result<Value, AppError> {
    ask(method::REPORT, None).await
}

/// The last weekly summaries, newest first.
#[tauri::command]
async fn weekly_reports() -> Result<Value, AppError> {
    ask(method::WEEKLY_REPORTS, None).await
}

/// The folder for downloads: `XDG_DOWNLOAD_DIR` from `~/.config/user-dirs.dirs`, else
/// `~/Downloads`, else the home folder.
fn downloads() -> Option<PathBuf> {
    let home = PathBuf::from(std::env::var_os("HOME").filter(|h| !h.is_empty())?);
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|c| !c.is_empty())
        .map_or_else(|| home.join(".config"), PathBuf::from);
    let named = std::fs::read_to_string(config.join("user-dirs.dirs"))
        .ok()
        .and_then(|text| {
            text.lines().find_map(|l| {
                let v = l
                    .trim()
                    .strip_prefix("XDG_DOWNLOAD_DIR=")?
                    .trim_matches('"');
                let v = v.replace("$HOME", &home.to_string_lossy());
                (v.starts_with('/')).then(|| PathBuf::from(v))
            })
        });
    [named, Some(home.join("Downloads"))]
        .into_iter()
        .flatten()
        .find(|d| d.is_dir())
        .or(Some(home))
}

/// Saves a report of the last 7 days as one HTML file in the downloads folder, never over
/// an existing file. Returns where it is.
#[tauri::command]
async fn export_report() -> Result<String, AppError> {
    let report: deecure_core::report::Report = ask(method::REPORT, None).await?;
    let tz = deecure_core::local::zone();
    let page = deecure_core::report::html(&report, &tz);
    let dir =
        downloads().ok_or_else(|| other("HOME is not set, so there is no folder to save to"))?;
    let day = deecure_core::local::format_in(report.created_at, "%Y-%m-%d", &tz);
    tauri::async_runtime::spawn_blocking(move || {
        use std::io::Write;
        for n in 1..1000 {
            let name = if n == 1 {
                format!("deecure-report-{day}.html")
            } else {
                format!("deecure-report-{day}-{n}.html")
            };
            let path = dir.join(name);
            // create_new: an existing file is never replaced.
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut file) => {
                    return file
                        .write_all(page.as_bytes())
                        .map(|()| path.to_string_lossy().into_owned())
                        .map_err(|e| other(format!("could not write {}: {e}", path.display())));
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(other(format!("could not save in {}: {e}", dir.display()))),
            }
        }
        Err(other(
            "too many reports with today's name in the downloads folder",
        ))
    })
    .await
    .map_err(|e| other(e.to_string()))?
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
    // A copy already runs (maybe only in the tray): it shows its window instead.
    if tray::instance::signal_running() {
        return;
    }
    let result = tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            tray::instance::listen(handle.clone());
            let hidden = std::env::args().any(|a| a == "--hidden");
            let tray_on = tray::load().tray
                && tauri::async_runtime::block_on(tray::host_available())
                && match tray::create(&handle) {
                    Ok(()) => true,
                    Err(e) => {
                        eprintln!("deecure-desktop: no tray icon: {e}");
                        false
                    }
                };
            // Start hidden only when the tray icon is there to bring the window back.
            if hidden
                && tray_on
                && let Some(window) = app.get_webview_window("main")
            {
                let _ = window.hide();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event
                && tray::ACTIVE.load(std::sync::atomic::Ordering::SeqCst)
            {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            status,
            snapshot,
            inventory,
            metrics,
            latest,
            events,
            event,
            notify_settings,
            set_notify_settings,
            notify_test,
            notifications,
            report,
            weekly_reports,
            data_settings,
            set_data_settings,
            delete_history,
            export_report,
            machine,
            fix_plan,
            fix_apply,
            fix_runs,
            fix_undo,
            helper_status,
            helper_read,
            tray::tray_state,
            tray::set_tray
        ])
        .run(tauri::generate_context!());
    if let Err(e) = result {
        eprintln!("deecure-desktop: could not start: {e}");
        std::process::exit(1);
    }
}
