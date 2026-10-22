// SPDX-License-Identifier: Apache-2.0

//! How long the last boot took, read from systemd over D-Bus, like `systemd-analyze`.
//! Read once when the agent starts: the numbers do not change until the next boot.

use std::collections::HashMap;
use std::io;

use deecure_core::{Snapshot, SnapshotKind, Timestamp, Unit};
use serde::{Deserialize, Serialize};
use zbus::zvariant::OwnedValue;

use super::services::UnitRow;
use super::{Sample, SystemRoot, metric};

const SYSTEMD: &str = "org.freedesktop.systemd1";
const MANAGER: &str = "/org/freedesktop/systemd1";
/// How many of the slowest units to keep.
const SLOWEST: usize = 15;

/// Boot phases in seconds. A phase the firmware or loader does not report is left out.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BootTimes {
    /// Firmware (BIOS or UEFI) setup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware: Option<f64>,
    /// Boot loader (such as GRUB).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loader: Option<f64>,
    /// Linux kernel.
    pub kernel: f64,
    /// Initial RAM disk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initrd: Option<f64>,
    /// System services, until the boot finished.
    pub userspace: f64,
    /// All phases together.
    pub total: f64,
}

/// One unit and how long it took to start.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitTime {
    /// The unit name, such as `snapd.service`.
    pub name: String,
    /// Seconds from "activating" to "active".
    pub seconds: f64,
}

/// The `boot` inventory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BootInfo {
    /// The kernel's random ID for this boot.
    pub boot_id: String,
    /// When the computer started.
    pub booted_at: Timestamp,
    /// The systemd version, such as "255.4-1ubuntu8".
    pub systemd: Option<String>,
    /// Phase times. Missing while the boot has not finished.
    pub times: Option<BootTimes>,
    /// The slowest units, slowest first.
    pub slowest: Vec<UnitTime>,
}

/// Works out the phases from the manager's monotonic timestamps (microseconds), the
/// same way `systemd-analyze time` does. `None` until the boot has finished.
pub fn phases(t: &HashMap<String, u64>) -> Option<BootTimes> {
    let get = |k: &str| t.get(k).copied().unwrap_or(0);
    let (firmware, loader) = (
        get("FirmwareTimestampMonotonic"),
        get("LoaderTimestampMonotonic"),
    );
    let (initrd, userspace, finish) = (
        get("InitRDTimestampMonotonic"),
        get("UserspaceTimestampMonotonic"),
        get("FinishTimestampMonotonic"),
    );
    if finish == 0 || userspace == 0 || finish < userspace {
        return None;
    }
    let s = |us: u64| us as f64 / 1e6;
    let kernel_done = if initrd > 0 { initrd } else { userspace };
    Some(BootTimes {
        firmware: (firmware > loader).then(|| s(firmware - loader)),
        loader: (loader > 0).then(|| s(loader)),
        kernel: s(kernel_done),
        initrd: (initrd > 0 && userspace > initrd).then(|| s(userspace - initrd)),
        userspace: s(finish - userspace),
        total: s(firmware.max(loader) + finish),
    })
}

/// Units that started during boot, slowest first.
pub fn slowest(units: Vec<(String, u64, u64)>, finish: u64) -> Vec<UnitTime> {
    let mut times: Vec<UnitTime> = units
        .into_iter()
        .filter(|(_, activating, active)| {
            *activating > 0 && *active > *activating && (finish == 0 || *active <= finish)
        })
        .map(|(name, activating, active)| UnitTime {
            name,
            seconds: (active - activating) as f64 / 1e6,
        })
        .collect();
    times.sort_by(|a, b| b.seconds.total_cmp(&a.seconds));
    times.truncate(SLOWEST);
    times
}

/// Reads the boot once. Later calls in the same boot return nothing.
#[derive(Debug, Default)]
pub struct Boot {
    done: Option<String>,
}

impl Boot {
    /// The name used in logs.
    pub const NAME: &'static str = "boot";

    /// Reads the boot if this boot has not been read yet, or has not finished before.
    pub async fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let boot_id = root
            .read("/proc/sys/kernel/random/boot_id")?
            .trim()
            .to_owned();
        if self.done.as_deref() == Some(boot_id.as_str()) {
            return Ok(Sample::default());
        }
        let uptime: f64 = root
            .read("/proc/uptime")?
            .split_whitespace()
            .next()
            .and_then(|u| u.parse().ok())
            .unwrap_or(0.0);
        let info = read_bus(
            boot_id.clone(),
            Timestamp(now.as_millis() - (uptime * 1000.0) as i64),
        )
        .await
        .map_err(io::Error::other)?;
        let mut sample = Sample::default();
        if let Some(times) = &info.times {
            self.done = Some(boot_id);
            sample.metrics.push(metric(
                "boot.time",
                None,
                times.total,
                Unit::Seconds,
                info.booted_at,
            ));
        }
        sample.snapshots.push(Snapshot {
            kind: SnapshotKind::Boot,
            taken_at: now,
            data: serde_json::to_value(&info).map_err(io::Error::other)?,
        });
        Ok(sample)
    }
}

async fn read_bus(boot_id: String, booted_at: Timestamp) -> zbus::Result<BootInfo> {
    let bus = zbus::Connection::system().await?;
    let manager = get_all(&bus, MANAGER, "org.freedesktop.systemd1.Manager").await?;
    let numbers: HashMap<String, u64> = manager
        .iter()
        .filter_map(|(k, v)| u64::try_from(v).ok().map(|n| (k.clone(), n)))
        .collect();
    let systemd = manager
        .get("Version")
        .and_then(|v| String::try_from(v.try_clone().ok()?).ok());
    let reply = bus
        .call_method(
            Some(SYSTEMD),
            MANAGER,
            Some("org.freedesktop.systemd1.Manager"),
            "ListUnits",
            &(),
        )
        .await?;
    let rows: Vec<UnitRow> = reply.body().deserialize()?;
    let mut units = Vec::new();
    for (name, _, _, _, _, _, path, ..) in rows {
        // A unit that vanished in between is simply skipped.
        let Ok(props) = get_all(&bus, path.as_str(), "org.freedesktop.systemd1.Unit").await else {
            continue;
        };
        let at = |k: &str| {
            props
                .get(k)
                .and_then(|v| u64::try_from(v).ok())
                .unwrap_or(0)
        };
        units.push((
            name,
            at("InactiveExitTimestampMonotonic"),
            at("ActiveEnterTimestampMonotonic"),
        ));
    }
    let finish = numbers
        .get("FinishTimestampMonotonic")
        .copied()
        .unwrap_or(0);
    Ok(BootInfo {
        boot_id,
        booted_at,
        systemd,
        times: phases(&numbers),
        slowest: slowest(units, finish),
    })
}

async fn get_all(
    bus: &zbus::Connection,
    path: &str,
    interface: &str,
) -> zbus::Result<HashMap<String, OwnedValue>> {
    let reply = bus
        .call_method(
            Some(SYSTEMD),
            path,
            Some("org.freedesktop.DBus.Properties"),
            "GetAll",
            &(interface,),
        )
        .await?;
    reply.body().deserialize()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stamps(pairs: &[(&str, u64)]) -> HashMap<String, u64> {
        pairs.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect()
    }

    #[test]
    fn phases_match_systemd_analyze() {
        // "8.1s (firmware) + 2.4s (loader) + 3.9s (kernel) + 26.8s (userspace) = 41.2s"
        let t = stamps(&[
            ("FirmwareTimestampMonotonic", 10_500_000),
            ("LoaderTimestampMonotonic", 2_400_000),
            ("UserspaceTimestampMonotonic", 3_900_000),
            ("FinishTimestampMonotonic", 30_700_000),
        ]);
        let p = phases(&t).unwrap();
        assert_eq!(p.firmware, Some(8.1));
        assert_eq!(p.loader, Some(2.4));
        assert_eq!(p.kernel, 3.9);
        assert_eq!(p.initrd, None);
        assert!((p.userspace - 26.8).abs() < 1e-9);
        assert!((p.total - 41.2).abs() < 1e-9);
    }

    #[test]
    fn unfinished_boot_and_virtual_machines() {
        assert_eq!(phases(&stamps(&[("UserspaceTimestampMonotonic", 1)])), None);
        // No firmware or loader numbers (common in VMs), but an initrd.
        let p = phases(&stamps(&[
            ("InitRDTimestampMonotonic", 1_000_000),
            ("UserspaceTimestampMonotonic", 4_000_000),
            ("FinishTimestampMonotonic", 9_000_000),
        ]))
        .unwrap();
        assert_eq!(
            (p.firmware, p.loader, p.kernel, p.initrd),
            (None, None, 1.0, Some(3.0))
        );
        assert_eq!(p.total, 9.0);
    }

    #[test]
    fn slowest_keeps_boot_units_only() {
        let units = vec![
            ("snapd.service".into(), 5_000_000, 11_200_000),
            ("late.service".into(), 40_000_000, 45_000_000),
            ("never.service".into(), 0, 0),
            ("fast.service".into(), 6_000_000, 6_100_000),
        ];
        let out = slowest(units, 30_000_000);
        assert_eq!(
            out.iter().map(|u| u.name.as_str()).collect::<Vec<_>>(),
            ["snapd.service", "fast.service"]
        );
        assert!((out[0].seconds - 6.2).abs() < 1e-9);
    }
}
