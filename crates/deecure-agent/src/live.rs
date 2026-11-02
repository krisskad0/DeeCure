// SPDX-License-Identifier: Apache-2.0

//! Readings taken now, for the app's Refresh button: CPU, memory, temperatures, disks,
//! network, pressure, and battery. They use collectors of their own, so a refresh never
//! adds to the history or to the "minutes in a row" counters that warnings use; the
//! background collection stays once a minute. `latest` shows these readings until the next
//! background reading is newer.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use deecure_core::{Metric, Timestamp};

use crate::collect::{
    Battery, Collector, Cpu, DiskIo, Memory, Network, Pressure, Sensors, SystemRoot,
};

/// Rates (CPU use, disk and network speed) need two readings; when the last one is older
/// than this, a refresh takes two, this far apart.
const RATE_GAP: Duration = Duration::from_secs(1);
const STALE: Duration = Duration::from_secs(30);
/// Refreshes closer together than this answer with the readings already taken.
const SAME: Duration = Duration::from_millis(1500);

/// The collectors and the readings of the last refresh.
pub struct Live {
    collectors: Vec<Box<dyn Collector>>,
    last: Option<Instant>,
    latest: Vec<Metric>,
}

impl Default for Live {
    fn default() -> Self {
        Live {
            collectors: vec![
                Box::new(Cpu::default()),
                Box::new(Memory::default()),
                Box::new(Battery::default()),
                Box::new(Network::default()),
                Box::new(DiskIo::default()),
                Box::new(Pressure::default()),
                Box::new(Sensors::default()),
            ],
            last: None,
            latest: Vec::new(),
        }
    }
}

/// Counters of time in a row belong to the background collection only.
fn shown(m: &Metric) -> bool {
    !m.name.contains("_minutes")
}

impl Live {
    fn read_once(&mut self, root: &SystemRoot) -> Vec<Metric> {
        let now = Timestamp::now();
        self.last = Some(Instant::now());
        self.collectors
            .iter_mut()
            .filter_map(|c| c.collect(root, now).ok())
            .flat_map(|s| s.metrics)
            .filter(shown)
            .collect()
    }

    /// Reads now (twice, one second apart, when rates need it). Blocks for up to a second.
    pub fn refresh(&mut self, root: &SystemRoot) -> Vec<Metric> {
        if self.last.is_some_and(|t| t.elapsed() < SAME) {
            return self.latest.clone();
        }
        if self.last.is_none_or(|t| t.elapsed() > STALE) {
            self.read_once(root);
            std::thread::sleep(RATE_GAP);
        }
        self.latest = self.read_once(root);
        self.latest.clone()
    }

    /// The readings of the last refresh.
    pub fn latest(&self) -> &[Metric] {
        &self.latest
    }
}

/// `stored` with each reading replaced by a newer one from `live` (same name and subject),
/// and live readings the store does not have yet added.
pub fn overlay(stored: Vec<Metric>, live: &[Metric]) -> Vec<Metric> {
    let mut by_key: HashMap<(String, Option<String>), Metric> = stored
        .into_iter()
        .map(|m| ((m.name.clone(), m.subject.clone()), m))
        .collect();
    for m in live {
        let key = (m.name.clone(), m.subject.clone());
        if by_key.get(&key).is_none_or(|old| m.at > old.at) {
            by_key.insert(key, m.clone());
        }
    }
    let mut out: Vec<Metric> = by_key.into_values().collect();
    out.sort_by(|a, b| (&a.name, &a.subject).cmp(&(&b.name, &b.subject)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use deecure_core::Unit;

    fn m(name: &str, subject: Option<&str>, value: f64, at: i64) -> Metric {
        Metric {
            name: name.into(),
            subject: subject.map(str::to_owned),
            value,
            unit: Unit::Percent,
            at: Timestamp(at),
        }
    }

    #[test]
    fn newer_live_readings_win_until_the_store_catches_up() {
        let stored = vec![
            m("cpu.usage", None, 10.0, 1000),
            m("memory.used_percent", None, 50.0, 1000),
        ];
        let live = [
            m("cpu.usage", None, 80.0, 2000),
            m("temp", Some("coretemp/Core 0"), 60.0, 2000),
        ];
        let out = overlay(stored, &live);
        let get = |n: &str| out.iter().find(|x| x.name == n).map(|x| x.value);
        assert_eq!(get("cpu.usage"), Some(80.0));
        assert_eq!(get("memory.used_percent"), Some(50.0));
        assert_eq!(get("temp"), Some(60.0));
        // A newer background reading replaces the live one.
        let out = overlay(vec![m("cpu.usage", None, 20.0, 3000)], &live);
        assert_eq!(
            out.iter().find(|x| x.name == "cpu.usage").map(|x| x.value),
            Some(20.0)
        );
    }

    #[test]
    fn counters_of_minutes_are_never_shown_from_a_refresh() {
        assert!(!shown(&m("temp.hot_minutes", Some("x"), 3.0, 0)));
        assert!(!shown(&m("cpu.throttle_minutes_hour", None, 3.0, 0)));
        assert!(shown(&m("temp.cpu", None, 50.0, 0)));
    }
}
