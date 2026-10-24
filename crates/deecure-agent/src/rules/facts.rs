// SPDX-License-Identifier: Apache-2.0

//! Facts: the values rules can use, built from the newest data in the database.
//!
//! The list of facts is in `docs/02-architecture/rules-engine.md`.

use std::collections::BTreeMap;

use deecure_core::{EventKind, SnapshotKind, Timestamp};
use serde_json::Value as Json;

use super::expr::{Lookup, Value};
use crate::store::{self, Store};

/// How old a reading may be and still count as "now". Older readings are
/// ignored, so a rule never acts on stale data.
pub const FRESH_FOR_MS: i64 = 10 * 60_000;

/// The lists a rule can go through with `for_each`.
pub const LISTS: &[&str] = &[
    "filesystems",
    "batteries",
    "services",
    "sensors",
    "interfaces",
];

/// CPU sensor chips: the CPU warning covers them, so the "part too hot" warning leaves them out.
const CPU_CHIPS: &[&str] = &["coretemp", "k10temp", "zenpower", "cpu_thermal"];
/// Counters whose rise over the last 24 hours rules can use as `<name>_24h`.
const RISE_24H: &[&str] = &["memory.errors_corrected", "pcie.errors_corrected"];
/// Hardware kinds in the log (`history/journal.rs`), counted as `log.<kind>_24h` and `_7d`.
const LOG_KINDS: &[&str] = &[
    "disk_io",
    "gpu_hang",
    "kernel_crash",
    "machine_check",
    "firmware_crash",
    "usb_overcurrent",
    "thermal_shutdown",
];
const DAY_MS: i64 = 86_400_000;

/// One entry of a list, such as one filesystem.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Item {
    /// What a finding about this entry is about: a mount point, a battery, a unit name.
    pub subject: String,
    /// Reachable in rules as `item.<name>`.
    pub fields: BTreeMap<String, Value>,
}

/// Everything rules can know right now.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Facts {
    /// Single values, such as `memory.used_percent` or `root.free_gb`.
    pub values: BTreeMap<String, Value>,
    /// Lists for `for_each`. A list that is missing means "not known", not "empty".
    pub lists: BTreeMap<String, Vec<Item>>,
}

impl Lookup for Facts {
    fn get(&self, name: &str) -> Option<Value> {
        self.values.get(name).cloned()
    }
}

/// The facts, plus one list entry reachable as `item.<field>`.
pub struct ItemScope<'a> {
    /// All facts.
    pub facts: &'a Facts,
    /// The current list entry.
    pub item: &'a Item,
}

impl Lookup for ItemScope<'_> {
    fn get(&self, name: &str) -> Option<Value> {
        match name.strip_prefix("item.") {
            Some(field) => self.item.fields.get(field).cloned(),
            None => self.facts.get(name),
        }
    }
}

/// Builds the facts from the newest readings and inventories.
pub fn gather(store: &Store, now: Timestamp) -> store::Result<Facts> {
    let mut facts = Facts::default();
    let mut filesystems: BTreeMap<String, Item> = BTreeMap::new();
    let mut batteries: BTreeMap<String, Item> = BTreeMap::new();
    let mut sensors: BTreeMap<String, Item> = BTreeMap::new();
    let mut interfaces: BTreeMap<String, Item> = BTreeMap::new();

    for metric in store.latest_metrics(Timestamp(now.as_millis() - FRESH_FOR_MS))? {
        let value = Value::Number(metric.value);
        match (metric.name.as_str(), metric.subject) {
            (name, None) => {
                facts.values.insert(name.to_owned(), value);
            }
            ("disk.used_percent", Some(mount)) => {
                entry(&mut filesystems, mount)
                    .fields
                    .insert("used_percent".into(), value);
            }
            ("disk.free", Some(mount)) => {
                let gb = Value::Number(metric.value / 1e9);
                entry(&mut filesystems, mount)
                    .fields
                    .insert("free_gb".into(), gb);
            }
            ("disk.inodes_percent" | "disk.fs_errors" | "disk.read_only", Some(mount)) => {
                let field = metric.name.trim_start_matches("disk.").to_owned();
                entry(&mut filesystems, mount).fields.insert(field, value);
            }
            ("temp" | "temp.limit" | "temp.critical" | "temp.hot_minutes", Some(sensor)) => {
                let field = match metric.name.as_str() {
                    "temp" => "celsius",
                    other => other.trim_start_matches("temp."),
                };
                let chip = sensor.split('/').next().unwrap_or("").to_owned();
                let item = entry(&mut sensors, sensor);
                item.fields.insert(field.into(), value);
                item.fields.insert(
                    "cpu".into(),
                    Value::Bool(CPU_CHIPS.contains(&chip.as_str())),
                );
            }
            ("net.error_percent", Some(iface)) => {
                entry(&mut interfaces, iface)
                    .fields
                    .insert("error_percent".into(), value);
            }
            (name, Some(battery)) => {
                if let Some(field) = name.strip_prefix("battery.") {
                    entry(&mut batteries, battery)
                        .fields
                        .insert(field.into(), value);
                }
            }
        }
    }

    // Add the things that rarely change (device, type, size) to filesystems with fresh readings.
    if let Some(snapshot) = store.latest_snapshot(SnapshotKind::Filesystems)? {
        for row in snapshot.data.as_array().into_iter().flatten() {
            let Some(item) = text(row, "mount_point").and_then(|m| filesystems.get_mut(&m)) else {
                continue;
            };
            for field in ["device", "fs_type"] {
                if let Some(value) = text(row, field) {
                    item.fields.insert(field.into(), Value::Text(value));
                }
            }
            if let Some(size) = row.get("size").and_then(Json::as_f64) {
                item.fields
                    .insert("size_gb".into(), Value::Number(size / 1e9));
            }
        }
    }

    // Trends: how fast each disk fills, from its last 7 days.
    for (mount, item) in &mut filesystems {
        let since = Timestamp(now.as_millis() - super::forecast::WINDOW_MS);
        let points = store.metrics("disk.used_percent", Some(mount), since, now)?;
        // Too little history: it cannot be said to be filling, so an earlier "filling" finding
        // closes. A gap in a longer history stays unknown, so a real one does not flicker.
        if super::forecast::history_days(&points) < super::forecast::MIN_HISTORY_DAYS {
            item.fields.insert("filling".into(), Value::Bool(false));
        }
        if let Some(f) = super::forecast::disk(points) {
            item.fields
                .insert("percent_per_day".into(), Value::Number(f.percent_per_day));
            // GB a day reads better than "% of the disk"; the size comes from the filesystem
            // list, or from free space and use.
            let num = |k: &str| match item.fields.get(k) {
                Some(Value::Number(n)) => Some(*n),
                _ => None,
            };
            let size = num("size_gb").or_else(|| {
                let (free, used) = (num("free_gb")?, num("used_percent")?);
                (used < 100.0).then(|| free * 100.0 / (100.0 - used))
            });
            if let Some(size) = size {
                item.fields.insert(
                    "gb_per_day".into(),
                    Value::Number(f.percent_per_day / 100.0 * size),
                );
            }
            item.fields
                .insert("filling".into(), Value::Bool(f.days_until_full.is_some()));
            if let Some(days) = f.days_until_full {
                // "about 0 days" reads oddly; under a day is "about 1 day".
                let days = days.round().max(1.0);
                item.fields
                    .insert("days_until_full".into(), Value::Number(days));
            }
        }
    }

    // Free space in percent, for "low on space" (the rule language has no arithmetic).
    for item in filesystems.values_mut() {
        if let Some(Value::Number(used)) = item.fields.get("used_percent").cloned() {
            item.fields
                .insert("free_percent".into(), Value::Number(100.0 - used));
        }
    }

    // The main disk is used so often that it gets its own names: root.used_percent, root.free_gb, …
    if let Some(root) = filesystems.get("/") {
        for (field, value) in &root.fields {
            facts.values.insert(format!("root.{field}"), value.clone());
        }
    }

    if let Some(snapshot) = store.latest_snapshot(SnapshotKind::Services)? {
        let units = snapshot.data.as_array().into_iter().flatten();
        let items = units
            .filter_map(|unit| {
                let mut item = Item {
                    subject: text(unit, "name")?,
                    ..Item::default()
                };
                for field in ["name", "description", "load", "active", "sub", "enabled"] {
                    if let Some(value) = text(unit, field) {
                        item.fields.insert(field.into(), Value::Text(value));
                    }
                }
                // Always known: a service without the field is not started by a timer.
                let timer = unit.get("timer").and_then(Json::as_bool).unwrap_or(false);
                item.fields.insert("timer".into(), Value::Bool(timer));
                if let Some(restarts) = unit.get("restarts").and_then(Json::as_f64) {
                    item.fields
                        .insert("restarts".into(), Value::Number(restarts));
                }
                Some(item)
            })
            .collect();
        facts.lists.insert("services".into(), items);
    }

    // How much error counters rose in 24 hours (counters start again at 0 after a restart).
    for name in RISE_24H {
        let points = store.metrics(name, None, Timestamp(now.as_millis() - DAY_MS), now)?;
        if points.len() >= 2 {
            let rise: f64 = points
                .windows(2)
                .map(|w| {
                    if w[1].value >= w[0].value {
                        w[1].value - w[0].value
                    } else {
                        w[1].value
                    }
                })
                .sum();
            facts
                .values
                .insert(format!("{name}_24h"), Value::Number(rise));
        }
    }

    // Problems the kernel logged, and apps stopped for memory, over 24 hours and 7 days.
    let week = store.events(
        Timestamp(now.as_millis() - 7 * DAY_MS),
        now,
        &[EventKind::HardwareError, EventKind::OutOfMemoryKill],
        10_000,
    )?;
    let count = |e: &deecure_core::Event| {
        e.data
            .as_ref()
            .and_then(|d| d.get("count")?.as_f64())
            .unwrap_or(1.0)
    };
    for kind in LOG_KINDS {
        let of = |days: i64| -> f64 {
            week.iter()
                .filter(|e| e.kind == EventKind::HardwareError)
                .filter(|e| e.subject.as_deref() == Some(*kind))
                .filter(|e| e.at.as_millis() >= now.as_millis() - days * DAY_MS)
                .map(count)
                .sum()
        };
        facts
            .values
            .insert(format!("log.{kind}_24h"), Value::Number(of(1)));
        facts
            .values
            .insert(format!("log.{kind}_7d"), Value::Number(of(7)));
    }
    let oom: f64 = week
        .iter()
        .filter(|e| e.kind == EventKind::OutOfMemoryKill)
        .filter(|e| e.at.as_millis() >= now.as_millis() - DAY_MS)
        .map(count)
        .sum();
    facts
        .values
        .insert("log.oom_kills_24h".into(), Value::Number(oom));

    // Plain names for parts, and a critical limit that is never reached when the sensor has
    // none (a missing fact would make the whole rule "unknown").
    for item in sensors.values_mut() {
        let name = item.subject.clone();
        let (chip, label) = name.split_once('/').unwrap_or((name.as_str(), ""));
        let part = match chip {
            "nvme" => format!("The NVMe drive ({label})"),
            "drivetemp" => format!("A disk drive ({label})"),
            c if c.starts_with("iwlwifi")
                || c.starts_with("mt79")
                || c.starts_with("ath")
                || c.starts_with("rtw") =>
            {
                "The Wi-Fi card".to_owned()
            }
            "amdgpu" | "nouveau" | "radeon" | "i915" | "xe" => {
                format!("The graphics card ({label})")
            }
            c if c.starts_with("pch_") => "The chipset".to_owned(),
            "acpitz" => "The board (ACPI zone)".to_owned(),
            _ => format!("{chip} {label}"),
        };
        item.fields
            .insert("part".into(), Value::Text(part.trim().to_owned()));
        item.fields
            .entry("critical".into())
            .or_insert(Value::Number(1000.0));
    }
    if !sensors.is_empty() {
        facts
            .lists
            .insert("sensors".into(), sensors.into_values().collect());
    }
    if !interfaces.is_empty() {
        facts
            .lists
            .insert("interfaces".into(), interfaces.into_values().collect());
    }
    if !filesystems.is_empty() {
        facts
            .lists
            .insert("filesystems".into(), filesystems.into_values().collect());
    }
    if !batteries.is_empty() {
        facts
            .lists
            .insert("batteries".into(), batteries.into_values().collect());
    }
    Ok(facts)
}

/// The entry for `subject`, created with its `name` (or `mount_point`) field if new.
fn entry(items: &mut BTreeMap<String, Item>, subject: String) -> &mut Item {
    items.entry(subject.clone()).or_insert_with(|| {
        let key = if subject.starts_with('/') {
            "mount_point"
        } else {
            "name"
        };
        Item {
            fields: BTreeMap::from([(key.to_owned(), Value::Text(subject.clone()))]),
            subject,
        }
    })
}

fn text(row: &Json, field: &str) -> Option<String> {
    row.get(field).and_then(Json::as_str).map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use deecure_core::{Metric, Snapshot, Unit};
    use serde_json::json;

    const NOW: i64 = 10_000_000;

    fn reading(name: &str, subject: Option<&str>, value: f64, age_ms: i64) -> Metric {
        Metric {
            name: name.into(),
            subject: subject.map(str::to_owned),
            value,
            unit: Unit::Percent,
            at: Timestamp(NOW - age_ms),
        }
    }

    #[test]
    fn builds_values_lists_and_root_names() {
        let store = Store::open_in_memory().unwrap();
        store
            .add_metrics(&[
                reading("memory.used_percent", None, 78.0, 0),
                reading("disk.used_percent", Some("/"), 94.2, 0),
                reading("disk.free", Some("/"), 13.8e9, 0),
                reading("battery.health", Some("BAT0"), 71.0, 0),
                reading("cpu.usage", None, 50.0, FRESH_FOR_MS + 60_000), // too old
            ])
            .unwrap();
        store
            .add_snapshot(&Snapshot {
                kind: SnapshotKind::Filesystems,
                taken_at: Timestamp(1),
                data: json!([{"mount_point": "/", "device": "/dev/nvme0n1p2", "fs_type": "ext4", "size": 238e9}]),
            })
            .unwrap();
        store
            .add_snapshot(&Snapshot {
                kind: SnapshotKind::Services,
                taken_at: Timestamp(1),
                data: json!([{"name": "bluetooth.service", "description": "Bluetooth", "load": "loaded", "active": "failed", "sub": "failed"}]),
            })
            .unwrap();

        let facts = gather(&store, Timestamp(NOW)).unwrap();
        assert_eq!(facts.get("memory.used_percent"), Some(Value::Number(78.0)));
        assert_eq!(facts.get("cpu.usage"), None, "stale readings are ignored");
        assert_eq!(facts.get("root.used_percent"), Some(Value::Number(94.2)));
        assert_eq!(facts.get("root.free_gb"), Some(Value::Number(13.8)));
        assert_eq!(facts.get("root.fs_type"), Some(Value::Text("ext4".into())));
        assert_eq!(facts.get("root.size_gb"), Some(Value::Number(238.0)));

        let battery = &facts.lists["batteries"][0];
        assert_eq!(battery.subject, "BAT0");
        assert_eq!(battery.fields["health"], Value::Number(71.0));
        assert_eq!(battery.fields["name"], Value::Text("BAT0".into()));

        let service = &facts.lists["services"][0];
        let scope = ItemScope {
            facts: &facts,
            item: service,
        };
        assert_eq!(scope.get("item.active"), Some(Value::Text("failed".into())));
        assert_eq!(scope.get("root.used_percent"), Some(Value::Number(94.2)));
    }

    #[test]
    fn a_filling_disk_gets_a_forecast() {
        let store = Store::open_in_memory().unwrap();
        let day = 86_400_000;
        let points: Vec<Metric> = (0..=72)
            .map(|h| Metric {
                name: "disk.used_percent".into(),
                subject: Some("/".into()),
                value: 88.0 + 2.0 * f64::from(h) / 24.0, // 2% a day for 3 days, 94% now
                unit: Unit::Percent,
                at: Timestamp(NOW - 3 * day + i64::from(h) * 3_600_000),
            })
            .collect();
        store.add_metrics(&points).unwrap();
        let facts = gather(&store, Timestamp(NOW)).unwrap();
        assert_eq!(facts.get("root.filling"), Some(Value::Bool(true)));
        assert_eq!(facts.get("root.days_until_full"), Some(Value::Number(3.0)));
    }

    #[test]
    fn too_little_history_is_not_filling() {
        let store = Store::open_in_memory().unwrap();
        let points: Vec<Metric> = (0..=30)
            .map(|h| Metric {
                name: "disk.used_percent".into(),
                subject: Some("/".into()),
                value: 42.0 + 8.0 * f64::from(h) / 30.0, // a build added 8% in a day
                unit: Unit::Percent,
                at: Timestamp(NOW - 30 * 3_600_000 + i64::from(h) * 3_600_000),
            })
            .collect();
        store.add_metrics(&points).unwrap();
        let facts = gather(&store, Timestamp(NOW)).unwrap();
        assert_eq!(facts.get("root.filling"), Some(Value::Bool(false)));
        assert_eq!(facts.get("root.days_until_full"), None);
    }

    #[test]
    fn missing_data_means_missing_lists() {
        let store = Store::open_in_memory().unwrap();
        let facts = gather(&store, Timestamp(NOW)).unwrap();
        assert!(facts.lists.is_empty());
        // Only the log counts, which are 0 when nothing was logged.
        assert!(
            facts.values.keys().all(|k| k.starts_with("log.")),
            "{:?}",
            facts.values
        );
        assert_eq!(facts.get("log.disk_io_24h"), Some(Value::Number(0.0)));
    }
}
