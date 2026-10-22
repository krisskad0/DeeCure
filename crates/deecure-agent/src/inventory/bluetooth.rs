// SPDX-License-Identifier: Apache-2.0

//! Bluetooth adapters and devices, from BlueZ over D-Bus (read-only).

use std::collections::HashMap;

use serde::Serialize;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

use crate::context::{Agent, Bus};

const BLUEZ: &str = "org.bluez";

/// One paired or seen device.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BluetoothDevice {
    pub name: String,
    /// BlueZ's icon name, such as `audio-headphones` or `input-mouse`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub paired: bool,
    pub connected: bool,
    /// Battery level in percent, when the device reports one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery: Option<u8>,
}

/// The `bluetooth` inventory.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct BluetoothInfo {
    /// False when BlueZ is not running, so nothing below is known.
    pub available: bool,
    pub adapters: usize,
    /// True when any adapter is switched on.
    pub powered: bool,
    pub devices: Vec<BluetoothDevice>,
}

type Objects = HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>>;

fn get<T: TryFrom<OwnedValue>>(props: &HashMap<String, OwnedValue>, key: &str) -> Option<T> {
    T::try_from(props.get(key)?.try_clone().ok()?).ok()
}

/// Reads adapters and devices. BlueZ is asked only when it is already running.
pub async fn read(agent: &Agent) -> BluetoothInfo {
    if !agent.is_running(Bus::System, BLUEZ).await {
        return BluetoothInfo::default();
    }
    let Ok(bus) = agent.bus(Bus::System).await else {
        return BluetoothInfo::default();
    };
    let reply = bus
        .call_method(
            Some(BLUEZ),
            "/",
            Some("org.freedesktop.DBus.ObjectManager"),
            "GetManagedObjects",
            &(),
        )
        .await;
    let Ok(objects) = reply.and_then(|r| r.body().deserialize::<Objects>()) else {
        return BluetoothInfo {
            available: true,
            ..BluetoothInfo::default()
        };
    };
    let mut info = BluetoothInfo {
        available: true,
        ..BluetoothInfo::default()
    };
    for interfaces in objects.values() {
        if let Some(adapter) = interfaces.get("org.bluez.Adapter1") {
            info.adapters += 1;
            info.powered |= get::<bool>(adapter, "Powered").unwrap_or(false);
        }
        if let Some(device) = interfaces.get("org.bluez.Device1") {
            let paired = get::<bool>(device, "Paired").unwrap_or(false);
            let connected = get::<bool>(device, "Connected").unwrap_or(false);
            // Only devices the user has paired or is using; nearby strangers are left out.
            if !paired && !connected {
                continue;
            }
            info.devices.push(BluetoothDevice {
                name: get::<String>(device, "Alias")
                    .or_else(|| get(device, "Name"))
                    .unwrap_or_else(|| "Unnamed device".into()),
                kind: get(device, "Icon"),
                paired,
                connected,
                battery: interfaces
                    .get("org.bluez.Battery1")
                    .and_then(|b| get(b, "Percentage")),
            });
        }
    }
    info.devices.sort_by(|a, b| {
        b.connected
            .cmp(&a.connected)
            .then_with(|| a.name.cmp(&b.name))
    });
    info
}
