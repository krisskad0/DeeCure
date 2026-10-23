// SPDX-License-Identifier: Apache-2.0

//! Facts: the values rules can use, built from the newest data in the database.
//!
//! The list of facts is in `docs/02-architecture/rules-engine.md`.

use std::collections::BTreeMap;

use deecure_core::{SnapshotKind, Timestamp};
use serde_json::Value as Json;

use super::expr::{Lookup, Value};
use crate::store::{self, Store};

/// How old a reading may be and still count as "now". Older readings are
/// ignored, so a rule never acts on stale data.
pub const FRESH_FOR_MS: i64 = 10 * 60_000;

/// The lists a rule can go through with `for_each`.
pub const LISTS: &[&str] = &["filesystems", "batteries", "services"];

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
        if let Some(f) = super::forecast::disk(points) {
            item.fields
                .insert("percent_per_day".into(), Value::Number(f.percent_per_day));
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
        let points: Vec<Metric> = (0..=48)
            .map(|h| Metric {
                name: "disk.used_percent".into(),
                subject: Some("/".into()),
                value: 90.0 + 2.0 * f64::from(h) / 24.0, // 2% a day, 94% now
                unit: Unit::Percent,
                at: Timestamp(NOW - 2 * day + i64::from(h) * 3_600_000),
            })
            .collect();
        store.add_metrics(&points).unwrap();
        let facts = gather(&store, Timestamp(NOW)).unwrap();
        assert_eq!(facts.get("root.filling"), Some(Value::Bool(true)));
        assert_eq!(facts.get("root.days_until_full"), Some(Value::Number(3.0)));
    }

    #[test]
    fn missing_data_means_missing_lists() {
        let store = Store::open_in_memory().unwrap();
        let facts = gather(&store, Timestamp(NOW)).unwrap();
        assert!(facts.lists.is_empty());
        assert!(facts.values.is_empty());
    }
}
