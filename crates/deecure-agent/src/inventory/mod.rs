// SPDX-License-Identifier: Apache-2.0

//! Live inventories: read from the system when the app or CLI asks, not stored.
//! Used for things that change too fast to keep, or are cheap to read again.

mod audio;
mod bluetooth;
mod command;
mod display;
mod firmware;
mod gpu;
mod hardware;
mod history;
mod kernel;
mod network;
mod os;
mod power;
mod processes;
mod sleep;
mod system;

use serde_json::Value;

use crate::context::Agent;

pub use processes::{Process, ProcessSampler};
pub use system::SystemInfo;

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
        other => {
            return Err(format!(
                "unknown inventory '{other}' (known: {})",
                KINDS.join(", ")
            ));
        }
    };
    value.map_err(|e| e.to_string())
}
