// SPDX-License-Identifier: Apache-2.0

//! The DeeCure desktop app: a Tauri window around the React UI.
//!
//! The UI never talks to the agent directly. It calls the commands below, and
//! they talk to the agent over its private socket (`docs/02-architecture/ipc-protocol.md`).

use std::path::PathBuf;

use deecure_core::client::{self, ClientError};
use deecure_core::rpc::{StatusReport, default_socket_path, method};
use serde::Serialize;

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

/// The open findings, most serious first.
#[tauri::command]
async fn status() -> Result<StatusReport, AppError> {
    let socket = socket()?;
    // Socket calls block, so they run off the UI thread.
    tauri::async_runtime::spawn_blocking(move || client::call(&socket, method::STATUS, None))
        .await
        .map_err(|e| AppError {
            kind: "other",
            message: e.to_string(),
        })?
        .map_err(AppError::from)
}

fn main() {
    let result = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![status])
        .run(tauri::generate_context!());
    if let Err(e) = result {
        eprintln!("deecure-desktop: could not start: {e}");
        std::process::exit(1);
    }
}
