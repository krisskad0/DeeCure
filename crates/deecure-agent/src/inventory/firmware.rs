// SPDX-License-Identifier: Apache-2.0

//! Firmware updates from fwupd over D-Bus (read-only). fwupd is asked only when it is
//! already running, and only for what its local metadata already knows: no download.

use std::collections::HashMap;

use serde::Serialize;
use zbus::zvariant::OwnedValue;

use crate::context::{Agent, Bus};

const FWUPD: &str = "org.freedesktop.fwupd";
/// fwupd's "can be updated" device flag.
const UPDATABLE: u64 = 1 << 1;

/// One device fwupd knows.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FirmwareDevice {
    pub name: String,
    /// fwupd's device ID, for an update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The newest version offered, when it is newer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available: Option<String>,
    /// False when the vendor does not publish updates on LVFS for it.
    pub updatable: bool,
}

/// The `firmware` inventory.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct FirmwareInfo {
    /// False when fwupd is not running, so updates are unknown.
    pub available: bool,
    pub devices: Vec<FirmwareDevice>,
}

type Dict = HashMap<String, OwnedValue>;

fn get<T: TryFrom<OwnedValue>>(d: &Dict, key: &str) -> Option<T> {
    T::try_from(d.get(key)?.try_clone().ok()?).ok()
}

/// Reads devices and offered updates.
pub async fn read(agent: &Agent) -> FirmwareInfo {
    if !agent.is_running(Bus::System, FWUPD).await {
        return FirmwareInfo::default();
    }
    let Ok(bus) = agent.bus(Bus::System).await else {
        return FirmwareInfo::default();
    };
    let call = |method: &'static str, id: Option<String>| {
        let bus = bus.clone();
        async move {
            let reply = match id {
                Some(id) => {
                    bus.call_method(Some(FWUPD), "/", Some(FWUPD), method, &(id,))
                        .await
                }
                None => {
                    bus.call_method(Some(FWUPD), "/", Some(FWUPD), method, &())
                        .await
                }
            };
            reply.and_then(|r| r.body().deserialize::<Vec<Dict>>())
        }
    };
    let Ok(devices) = call("GetDevices", None).await else {
        return FirmwareInfo {
            available: true,
            devices: Vec::new(),
        };
    };
    let mut out = Vec::new();
    for d in devices {
        let updatable = get::<u64>(&d, "Flags").is_some_and(|f| f & UPDATABLE != 0);
        // Only devices with firmware a person would care about: those with a version.
        let Some(version) = get::<String>(&d, "Version") else {
            continue;
        };
        // "No upgrades" comes back as an error; that just means up to date.
        let id = get::<String>(&d, "DeviceId");
        let available = match (updatable, id.clone()) {
            (true, Some(id)) => call("GetUpgrades", Some(id))
                .await
                .ok()
                .and_then(|r| r.first().and_then(|rel| get::<String>(rel, "Version"))),
            _ => None,
        };
        out.push(FirmwareDevice {
            name: get(&d, "Name").unwrap_or_else(|| "Unnamed device".into()),
            id,
            vendor: get(&d, "Vendor"),
            version: Some(version),
            available,
            updatable,
        });
    }
    out.sort_by(|a, b| {
        b.available
            .is_some()
            .cmp(&a.available.is_some())
            .then_with(|| a.name.cmp(&b.name))
    });
    FirmwareInfo {
        available: true,
        devices: out,
    }
}
