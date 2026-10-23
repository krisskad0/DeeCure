// SPDX-License-Identifier: Apache-2.0

//! systemd system services, read over D-Bus.

use std::io;

use deecure_core::{Snapshot, SnapshotKind, Timestamp, Unit};
use serde::{Deserialize, Serialize};
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

use super::{Sample, metric};

/// One row of systemd's `ListUnits` reply.
pub(crate) type UnitRow = (
    String,          // name
    String,          // description
    String,          // load state
    String,          // active state
    String,          // sub state
    String,          // followed unit
    OwnedObjectPath, // unit object path
    u32,             // queued job ID
    String,          // job type
    OwnedObjectPath, // job object path
);

/// The state of one systemd unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitState {
    /// The unit name, such as `bluetooth.service`.
    pub name: String,
    /// systemd's one-line description.
    pub description: String,
    /// `loaded`, `not-found`, `masked`, …
    pub load: String,
    /// `active`, `inactive`, `failed`, `activating`, …
    pub active: String,
    /// The detailed state, such as `running`, `exited`, or `dead`.
    pub sub: String,
    /// Whether it starts at boot: `enabled`, `disabled`, `static`, `masked`, …
    /// Missing when systemd does not say (for example, transient units).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<String>,
    /// How many times systemd restarted it since boot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restarts: Option<u32>,
    /// True when a timer with the same name starts it: it is a scheduled job.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub timer: bool,
}

const SYSTEMD: &str = "org.freedesktop.systemd1";

/// Counts running and failed services, and keeps an inventory of their states.
///
/// Keeps one system-bus connection open between runs and reconnects after an error.
#[derive(Debug, Default)]
pub struct Services {
    bus: Option<zbus::Connection>,
}

impl Services {
    /// The name used in logs.
    pub const NAME: &'static str = "services";

    /// Asks systemd for every loaded unit.
    pub async fn collect(&mut self, now: Timestamp) -> io::Result<Sample> {
        match self.list_units().await {
            Ok(units) => Ok(summarize(units, now)),
            Err(e) => {
                self.bus = None;
                Err(io::Error::other(e))
            }
        }
    }

    async fn list_units(&mut self) -> zbus::Result<Vec<UnitState>> {
        let bus = match &self.bus {
            Some(bus) => bus.clone(),
            None => {
                let bus = zbus::Connection::system().await?;
                self.bus = Some(bus.clone());
                bus
            }
        };
        let reply = bus
            .call_method(
                Some(SYSTEMD),
                "/org/freedesktop/systemd1",
                Some("org.freedesktop.systemd1.Manager"),
                "ListUnits",
                &(),
            )
            .await?;
        let body = reply.body();
        let rows: Vec<UnitRow> = body.deserialize()?;
        let mut units = Vec::new();
        for (name, description, load, active, sub, _, path, ..) in rows {
            // Only services get the extra properties; other units are dropped later anyway.
            let (enabled, restarts) = if name.ends_with(".service") {
                let enabled: Option<String> = property(
                    &bus,
                    &path,
                    "org.freedesktop.systemd1.Unit",
                    "UnitFileState",
                )
                .await;
                let restarts =
                    property(&bus, &path, "org.freedesktop.systemd1.Service", "NRestarts").await;
                (enabled.filter(|state| !state.is_empty()), restarts)
            } else {
                (None, None)
            };
            units.push(UnitState {
                name,
                description,
                load,
                active,
                sub,
                enabled,
                restarts,
                timer: false,
            });
        }
        Ok(units)
    }
}

/// Reads one property of a systemd unit, or `None` if it is missing or of another type.
async fn property<T: TryFrom<OwnedValue>>(
    bus: &zbus::Connection,
    path: &OwnedObjectPath,
    interface: &str,
    name: &str,
) -> Option<T> {
    let reply = bus
        .call_method(
            Some(SYSTEMD),
            path.as_str(),
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &(interface, name),
        )
        .await
        .ok()?;
    let value: OwnedValue = reply.body().deserialize().ok()?;
    T::try_from(value).ok()
}

/// Turns the unit list into counts and a sorted inventory of `.service` units.
pub fn summarize(units: Vec<UnitState>, now: Timestamp) -> Sample {
    let timers: std::collections::HashSet<String> = units
        .iter()
        .filter_map(|u| u.name.strip_suffix(".timer").map(str::to_owned))
        .collect();
    let mut services: Vec<UnitState> = units
        .into_iter()
        .filter(|unit| unit.name.ends_with(".service"))
        .map(|mut unit| {
            unit.timer = unit
                .name
                .strip_suffix(".service")
                .is_some_and(|stem| timers.contains(stem));
            unit
        })
        .collect();
    services.sort_by(|a, b| a.name.cmp(&b.name));
    let count = |test: fn(&UnitState) -> bool| services.iter().filter(|u| test(u)).count() as f64;
    let running = count(|u| u.active == "active" && u.sub == "running");
    let failed = count(|u| u.active == "failed");
    Sample {
        metrics: vec![
            metric("services.running", None, running, Unit::Count, now),
            metric("services.failed", None, failed, Unit::Count, now),
        ],
        snapshots: vec![Snapshot {
            kind: SnapshotKind::Services,
            taken_at: now,
            data: serde_json::to_value(&services).unwrap_or_default(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::value;

    fn unit(name: &str, active: &str, sub: &str) -> UnitState {
        UnitState {
            name: name.into(),
            description: format!("{name} description"),
            load: "loaded".into(),
            active: active.into(),
            sub: sub.into(),
            enabled: None,
            restarts: None,
            timer: false,
        }
    }

    #[test]
    fn counts_running_and_failed_services_only() {
        let sample = summarize(
            vec![
                unit("snapd.service", "active", "running"),
                unit("bluetooth.service", "failed", "failed"),
                unit("apt-daily.service", "inactive", "dead"),
                unit("docker.socket", "active", "running"),
                unit("home.mount", "failed", "failed"),
                unit("apt-daily.timer", "active", "waiting"),
            ],
            Timestamp(0),
        );
        assert_eq!(value(&sample, "services.running", None), Some(1.0));
        assert_eq!(value(&sample, "services.failed", None), Some(1.0));
        let names: Vec<_> = sample.snapshots[0]
            .data
            .as_array()
            .unwrap()
            .iter()
            .map(|u| u["name"].as_str().unwrap().to_owned())
            .collect();
        assert_eq!(
            names,
            ["apt-daily.service", "bluetooth.service", "snapd.service"]
        );
        let timers: Vec<bool> = sample.snapshots[0]
            .data
            .as_array()
            .unwrap()
            .iter()
            .map(|u| u["timer"] == true)
            .collect();
        assert_eq!(
            timers,
            [true, false, false],
            "apt-daily.service is started by apt-daily.timer"
        );
    }
}
