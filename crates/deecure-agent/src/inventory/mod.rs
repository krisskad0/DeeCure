// SPDX-License-Identifier: Apache-2.0

//! Live inventories: read from the system when the app or CLI asks, not stored.
//! Used for things that change too fast to keep, or are cheap to read again.

mod accounts;
mod audio;
pub(crate) mod backups;
mod bluetooth;
mod command;
mod containers;
mod diagnostics;
mod display;
mod firmware;
mod gpu;
mod hardware;
mod history;
mod jobs;
mod kernel;
mod logs;
pub(crate) mod net;
mod network;
mod os;
mod packages;
mod power;
mod privacy;
mod processes;
mod security;
mod sleep;
mod sockets;
mod storage;
mod system;
mod units;
mod usage;

use serde_json::Value;

use crate::context::Agent;

pub use processes::{Process, ProcessSampler};
pub use system::SystemInfo;
pub use usage::UsageScan;

/// The inventories the agent can read.
pub const KINDS: &[&str] = &[
    "system",
    "processes",
    "network",
    "gpu",
    "os",
    "kernel",
    "update_history",
    "hardware",
    "power",
    "display",
    "audio",
    "bluetooth",
    "firmware",
    "sleep",
    "storage",
    "disk_usage",
    "user_services",
    "service_memory",
    "startup",
    "jobs",
    "journal",
    "errors",
    "kernel_log",
    "crashes",
    "boots",
    "device_events",
    "config_changes",
    "package_history",
    "net_details",
    "sockets",
    "accounts",
    "security",
    "privacy",
    "packages",
    "containers",
    "backups",
    "diagnose_performance",
    "diagnose_network",
    "agent",
    "kb",
];

/// Reads one inventory. `Err` holds a plain-English reason.
pub async fn read(kind: &str, agent: &Agent) -> Result<Value, String> {
    let value = match kind {
        "system" => serde_json::to_value(system::read(&agent.root)),
        "processes" => serde_json::to_value(
            agent
                .processes()
                .read(&agent.root)
                .map_err(|e| e.to_string())?,
        ),
        "network" => serde_json::to_value(network::read(&agent.root)),
        "gpu" => serde_json::to_value(gpu::read(&agent.root)),
        "os" => serde_json::to_value(os::read(&agent.root)),
        "kernel" => serde_json::to_value(kernel::read(&agent.root)),
        "update_history" => serde_json::to_value(history::read(&agent.root)),
        "hardware" => serde_json::to_value(hardware::read(&agent.root)),
        "power" => serde_json::to_value(power::read(agent).await),
        "display" => serde_json::to_value(display::read(agent).await),
        "audio" => serde_json::to_value(audio::read().await),
        "bluetooth" => serde_json::to_value(bluetooth::read(agent).await),
        "firmware" => serde_json::to_value(firmware::read(agent).await),
        "sleep" => serde_json::to_value(sleep::read().await),
        "storage" => serde_json::to_value(storage::read(&agent.root).await),
        "disk_usage" => serde_json::to_value(disk_usage(agent)),
        "user_services" => serde_json::to_value(units::user_services(agent).await?),
        "service_memory" => serde_json::to_value(units::service_memory(agent).await?),
        "startup" => serde_json::to_value(units::startup(
            &agent.root,
            std::env::var_os("HOME")
                .map(std::path::PathBuf::from)
                .as_deref(),
        )),
        "jobs" => serde_json::to_value(jobs::read(agent).await),
        "journal" => serde_json::to_value(logs::read_journal().await),
        "errors" => serde_json::to_value(logs::read_errors().await),
        "kernel_log" => serde_json::to_value(logs::read_kernel_log().await),
        "crashes" => serde_json::to_value(logs::read_crashes(&agent.root).await),
        "boots" => serde_json::to_value(logs::read_boots().await),
        "device_events" => serde_json::to_value(logs::read_device_events().await),
        "config_changes" => serde_json::to_value(logs::config_changes(
            &agent.root,
            &processes::user_names(&agent.root),
        )),
        "package_history" => serde_json::to_value(history::read_changes(&agent.root)),
        "net_details" => serde_json::to_value(net::read(agent).await),
        "sockets" => serde_json::to_value(sockets::read(&agent.root)),
        "accounts" => serde_json::to_value(accounts::read(agent).await),
        "security" => serde_json::to_value(
            security::read(&agent.root, &processes::user_names(&agent.root)).await,
        ),
        "privacy" => serde_json::to_value(privacy::read(&agent.root).await),
        "packages" => serde_json::to_value(packages::read(&agent.root).await),
        "containers" => serde_json::to_value(containers::read(&agent.root).await),
        "backups" => serde_json::to_value(backups::read(&agent.root).await),
        "diagnose_performance" => serde_json::to_value(diagnostics::performance(agent).await),
        "diagnose_network" => serde_json::to_value(diagnostics::network(agent).await),
        "agent" => serde_json::to_value(diagnostics::agent_info(agent)),
        "kb" => serde_json::to_value(agent.facts.get().map(|f| &f.kb)),
        other => {
            return Err(format!(
                "unknown inventory '{other}' (known: {})",
                KINDS.join(", ")
            ));
        }
    };
    value.map_err(|e| e.to_string())
}

/// The last scan, and whether a new one is running.
#[derive(Debug, Default)]
pub struct UsageState {
    last: Option<(std::time::Instant, UsageScan)>,
    scanning: bool,
}

/// What `disk_usage` answers.
#[derive(Debug, serde::Serialize)]
struct UsageAnswer {
    /// True while a scan runs; ask again in a few seconds.
    scanning: bool,
    /// The newest finished scan, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    scan: Option<UsageScan>,
}

/// Returns the last scan, and starts a new one in the background when it is old or missing.
fn disk_usage(agent: &Agent) -> UsageAnswer {
    let state = agent.usage.clone();
    let mut guard = state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let stale = guard
        .last
        .as_ref()
        .is_none_or(|(at, _)| at.elapsed() >= usage::FRESH);
    if stale && !guard.scanning {
        guard.scanning = true;
        let shared = state.clone();
        let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
        tokio::task::spawn_blocking(move || {
            let result = usage::scan(home.as_deref());
            let mut guard = shared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            guard.last = Some((std::time::Instant::now(), result));
            guard.scanning = false;
        });
    }
    UsageAnswer {
        scanning: guard.scanning,
        scan: guard.last.as_ref().map(|(_, s)| s.clone()),
    }
}
