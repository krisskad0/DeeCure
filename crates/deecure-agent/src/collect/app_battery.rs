// SPDX-License-Identifier: Apache-2.0

//! Battery use per app, an estimate (docs/02-architecture/collectors.md, "Battery use per
//! app"). Linux has no power meter per app, so each minute on battery the measured drain is
//! split, like phones do:
//!
//! - The usual low drain of the last 6 hours on battery (its 10th percentile) is the screen,
//!   Wi-Fi, and the system at rest. It goes to `rest`.
//! - The drain above it is split between apps and services by the CPU time (from their
//!   control groups) and graphics time (DRM `fdinfo`) each used in that minute.
//!
//! Each reading stores the energy used since the reading before, in milliwatt hours, as
//! `app.battery_mwh` with the subject `<kind>:<name>` (kind: `app`, `service`,
//! `user-service`), `other` for the small ones together, or `rest`.

use std::collections::{HashMap, VecDeque};
use std::io;

use deecure_core::{Timestamp, Unit};

use super::{Collector, Sample, SystemRoot, app_usage, battery, metric};
use crate::inventory::{cgroups, gpu_detail};

/// The metric name.
pub const METRIC: &str = "app.battery_mwh";
/// How far back the drain at rest is looked for.
const FLOOR_WINDOW_MS: i64 = 6 * 60 * 60_000;
/// A gap longer than this between readings (a suspend, an agent stop) starts counting anew.
const MAX_GAP_MS: i64 = 150_000;
/// Apps under this share of the drain above rest are added up as `other`.
const SMALL_SHARE: f64 = 0.01;

/// Counters from the reading before, to work out what each app used since.
#[derive(Debug, Default)]
pub struct AppBattery {
    last: Option<Timestamp>,
    /// CPU time per control group, in microseconds, by unit.
    cpu: HashMap<String, u64>,
    /// Graphics engine time per process, in nanoseconds.
    gpu: HashMap<u32, u64>,
    /// Drain readings on battery: (ms, watts), oldest first.
    drains: VecDeque<(i64, f64)>,
}

impl Collector for AppBattery {
    fn name(&self) -> &'static str {
        "app battery"
    }

    fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let Some(watts) = battery::on_battery_watts(root) else {
            // On the charger, or no battery: nothing to split; the next minute on battery
            // starts counting anew.
            self.last = None;
            return Ok(Sample::default());
        };
        let at = now.as_millis();
        self.drains.push_back((at, watts));
        while self
            .drains
            .front()
            .is_some_and(|(t, _)| at - t > FLOOR_WINDOW_MS)
        {
            self.drains.pop_front();
        }

        let groups = cgroups::read(root).groups;
        let cpu: HashMap<String, u64> = groups
            .iter()
            .filter_map(|g| Some((g.unit.clone(), g.cpu_usec?)))
            .collect();
        let gpu = gpu_detail::engine_ns_by_pid(root);
        let previous = self.last.replace(now);
        let old_cpu = std::mem::replace(&mut self.cpu, cpu);
        let old_gpu = std::mem::replace(&mut self.gpu, gpu);
        let Some(previous) = previous else {
            return Ok(Sample::default());
        };
        let elapsed = at - previous.as_millis();
        if elapsed <= 0 || elapsed > MAX_GAP_MS {
            return Ok(Sample::default());
        }

        // Seconds of CPU and graphics time each app used since the reading before.
        let mut weights: HashMap<String, f64> = HashMap::new();
        let subject_of: HashMap<&str, String> = groups
            .iter()
            .map(|g| (g.unit.as_str(), app_usage::subject(g.kind, &g.name)))
            .collect();
        for (unit, &usec) in &self.cpu {
            if let (Some(&before), Some(subject)) =
                (old_cpu.get(unit), subject_of.get(unit.as_str()))
                && usec > before
            {
                *weights.entry(subject.clone()).or_default() += (usec - before) as f64 / 1e6;
            }
        }
        for (pid, &ns) in &self.gpu {
            if let Some(&before) = old_gpu.get(pid)
                && ns > before
                && let Some(subject) = unit_of(root, *pid).and_then(|u| subject_of.get(u.as_str()))
            {
                *weights.entry(subject.clone()).or_default() += (ns - before) as f64 / 1e9;
            }
        }

        let floor = floor(self.drains.iter().map(|(_, w)| *w));
        let hours = elapsed as f64 / 3_600_000.0;
        let mut weights: Vec<(String, f64)> = weights.into_iter().collect();
        weights.sort_by(|a, b| a.0.cmp(&b.0));
        let mut sample = Sample::default();
        for (subject, mwh) in split(watts, floor, hours, &weights) {
            sample
                .metrics
                .push(metric(METRIC, Some(&subject), mwh, Unit::Count, now));
        }
        Ok(sample)
    }
}

/// The drain at rest: the 10th percentile of the recent drain readings.
fn floor(drains: impl Iterator<Item = f64>) -> f64 {
    let mut all: Vec<f64> = drains.collect();
    if all.is_empty() {
        return 0.0;
    }
    all.sort_by(f64::total_cmp);
    all[(all.len() - 1) / 10]
}

/// Splits `watts` over `hours` into milliwatt hours: `floor` watts to `rest`, the drain above
/// it to each subject by its weight, small shares to `other`. Zero amounts are left out.
pub fn split(watts: f64, floor: f64, hours: f64, weights: &[(String, f64)]) -> Vec<(String, f64)> {
    let total = watts * hours * 1000.0;
    let mut active = (watts - floor).max(0.0) * hours * 1000.0;
    let sum: f64 = weights.iter().map(|(_, w)| w).sum();
    let mut out = Vec::new();
    if sum <= 0.0 {
        active = 0.0;
    } else {
        let mut other = 0.0;
        for (subject, w) in weights {
            let share = w / sum;
            if share < SMALL_SHARE {
                other += share * active;
            } else {
                out.push((subject.clone(), share * active));
            }
        }
        out.push(("other".to_owned(), other));
    }
    out.push(("rest".to_owned(), total - active));
    out.retain(|(_, mwh)| *mwh > 0.0);
    out
}

/// The systemd unit a process runs in, from `/proc/<pid>/cgroup`.
fn unit_of(root: &SystemRoot, pid: u32) -> Option<String> {
    let text = root.read(&format!("/proc/{pid}/cgroup")).ok()?;
    let path = text.lines().find_map(|l| l.strip_prefix("0::"))?;
    path.rsplit('/')
        .find(|part| part.ends_with(".scope") || part.ends_with(".service"))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    fn weights(list: &[(&str, f64)]) -> Vec<(String, f64)> {
        list.iter().map(|(s, w)| ((*s).to_owned(), *w)).collect()
    }

    #[test]
    fn the_drain_above_rest_is_split_by_use() {
        // 10 W for 6 minutes = 1000 mWh; 4 W of it at rest.
        let parts = split(
            10.0,
            4.0,
            0.1,
            &weights(&[("app:Firefox", 3.0), ("service:cups", 1.0)]),
        );
        let get = |s: &str| parts.iter().find(|(x, _)| x == s).map(|(_, v)| *v);
        assert!((get("app:Firefox").unwrap() - 450.0).abs() < 1e-9);
        assert!((get("service:cups").unwrap() - 150.0).abs() < 1e-9);
        assert!((get("rest").unwrap() - 400.0).abs() < 1e-9);
        assert_eq!(get("other"), None, "nothing small");
        let sum: f64 = parts.iter().map(|(_, v)| v).sum();
        assert!(
            (sum - 1000.0).abs() < 1e-9,
            "every bit of the drain counts once"
        );
    }

    #[test]
    fn small_users_add_up_as_other_and_idle_goes_to_rest() {
        let parts = split(
            10.0,
            0.0,
            1.0,
            &weights(&[("app:Big", 995.0), ("app:Tiny", 5.0)]),
        );
        assert_eq!(parts[0].0, "app:Big");
        assert_eq!(parts[1].0, "other");
        assert!((parts[1].1 - 50.0).abs() < 1e-9);
        // Nobody used the CPU: all of it is rest.
        assert_eq!(split(8.0, 5.0, 1.0, &[]), [("rest".to_owned(), 8000.0)]);
        // Below the usual rest: all of it is rest too.
        assert_eq!(
            split(3.0, 5.0, 1.0, &weights(&[("app:A", 1.0)])),
            [("rest".to_owned(), 3000.0)]
        );
    }

    #[test]
    fn the_rest_is_the_usual_low_drain() {
        assert_eq!(floor([].into_iter()), 0.0);
        assert_eq!(floor([7.0].into_iter()), 7.0);
        let readings = (1..=20).map(f64::from);
        assert_eq!(floor(readings), 2.0);
    }

    fn laptop(name: &str, battery: &[(&str, &str)], cpu_usec: u64) -> SystemRoot {
        let app = "/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/app-gnome-org.gnome.Nautilus-42.scope";
        let usage = format!("usage_usec {cpu_usec}\n");
        let mut files: Vec<(&str, &str)> = vec![
            ("/proc/self/status", "Uid:\t1000\t1000\t1000\t1000\n"),
            ("/sys/class/power_supply/BAT0/type", "Battery\n"),
            ("/sys/class/power_supply/AC/type", "Mains\n"),
        ];
        files.extend_from_slice(battery);
        let cpu = format!("{app}/cpu.stat");
        files.push((&cpu, &usage));
        fixture_root(name, &files)
    }

    #[test]
    fn counts_only_on_battery_and_from_the_second_reading() {
        let on_battery = [
            ("/sys/class/power_supply/BAT0/status", "Discharging\n"),
            ("/sys/class/power_supply/BAT0/power_now", "12000000\n"),
            ("/sys/class/power_supply/AC/online", "0\n"),
        ];
        let mut c = AppBattery::default();
        let root = laptop("appbat-1", &on_battery, 1_000_000);
        assert!(c.collect(&root, Timestamp(0)).unwrap().metrics.is_empty());
        let root = laptop("appbat-2", &on_battery, 31_000_000);
        let sample = c.collect(&root, Timestamp(60_000)).unwrap();
        // 12 W for a minute = 200 mWh. The rest is the 10th percentile of [12, 12] = 12 W,
        // so this first minute is all rest; the app's share needs a lower drain seen before.
        let total: f64 = sample.metrics.iter().map(|m| m.value).sum();
        assert!((total - 200.0).abs() < 1e-9);

        let mut c = AppBattery::default();
        c.drains.push_back((0, 6.0));
        c.collect(&laptop("appbat-3", &on_battery, 1_000_000), Timestamp(0))
            .unwrap();
        let sample = c
            .collect(
                &laptop("appbat-4", &on_battery, 31_000_000),
                Timestamp(60_000),
            )
            .unwrap();
        let get = |s: &str| {
            sample
                .metrics
                .iter()
                .find(|m| m.subject.as_deref() == Some(s))
                .map(|m| m.value)
        };
        // Rest 6 W (the lowest of 6, 12, 12) = 100 mWh; Files used all the CPU: 100 mWh.
        assert!((get("app:org.gnome.Nautilus").unwrap() - 100.0).abs() < 1e-9);
        assert!((get("rest").unwrap() - 100.0).abs() < 1e-9);

        // On the charger: nothing, and counting starts anew.
        let plugged = [
            ("/sys/class/power_supply/BAT0/status", "Charging\n"),
            ("/sys/class/power_supply/AC/online", "1\n"),
        ];
        let root = laptop("appbat-5", &plugged, 40_000_000);
        assert!(
            c.collect(&root, Timestamp(120_000))
                .unwrap()
                .metrics
                .is_empty()
        );
        assert!(c.last.is_none());
    }

    #[test]
    fn processes_map_to_their_unit() {
        let root = fixture_root(
            "appbat-cgroup",
            &[(
                "/proc/7/cgroup",
                "0::/user.slice/user-1000.slice/user@1000.service/app.slice/app-gnome-firefox-9.scope\n",
            )],
        );
        assert_eq!(
            unit_of(&root, 7).as_deref(),
            Some("app-gnome-firefox-9.scope")
        );
        assert_eq!(unit_of(&root, 8), None);
    }
}
