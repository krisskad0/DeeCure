// SPDX-License-Identifier: Apache-2.0

//! CPU and memory per app over time (docs/02-architecture/collectors.md, "Use per app").
//!
//! Every 5 minutes, from the control groups of each app and service (no admin rights):
//!
//! - `app.cpu_seconds`: CPU time each one used since the reading before (an amount, so a
//!   day's use is the sum).
//! - `app.memory`: its own memory now (`anon`, without file cache the kernel can drop at once),
//!   in bytes (a level, so a day gives an average and a peak).
//!
//! The subject is `<kind>:<name>` as in `app.battery_mwh`; apps with the same name count once.
//! Only the biggest ones are kept, the rest added up as `other`, so the history stays small.

use std::collections::HashMap;
use std::io;

use deecure_core::{Timestamp, Unit};

use super::{Collector, Sample, SystemRoot, metric};
use crate::inventory::cgroups;

pub const CPU: &str = "app.cpu_seconds";
pub const MEMORY: &str = "app.memory";
/// How often it reads, by the clock. A little early still counts, so timer jitter never skips one.
const EVERY_MS: i64 = 5 * 60_000 - 5_000;
/// At most this many apps per reading and metric; the rest go to `other`.
const KEEP: usize = 15;
/// Smaller than this, an app goes to `other`: 1 CPU second in 5 minutes, 50 MB of memory.
const MIN_CPU_SECONDS: f64 = 1.0;
const MIN_MEMORY: f64 = 50.0 * 1024.0 * 1024.0;

/// CPU and memory per app, every 5 minutes.
#[derive(Debug, Default)]
pub struct AppUsage {
    last: Option<Timestamp>,
    /// CPU time per control group at the reading before, in microseconds, by unit.
    cpu: HashMap<String, u64>,
}

impl Collector for AppUsage {
    fn name(&self) -> &'static str {
        "app usage"
    }

    fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        if self
            .last
            .is_some_and(|l| now.as_millis() - l.as_millis() < EVERY_MS)
        {
            return Ok(Sample::default());
        }
        let first = self.last.replace(now).is_none();
        let groups = cgroups::read(root).groups;
        let mut cpu: HashMap<String, f64> = HashMap::new();
        let mut memory: HashMap<String, f64> = HashMap::new();
        let mut counters = HashMap::new();
        for g in &groups {
            let subject = subject(g.kind, &g.name);
            if let Some(usec) = g.cpu_usec {
                counters.insert(g.unit.clone(), usec);
                // A group seen for the first time counts from now; a restarted one too.
                if let Some(&before) = self.cpu.get(&g.unit)
                    && usec > before
                {
                    *cpu.entry(subject.clone()).or_default() += (usec - before) as f64 / 1e6;
                }
            }
            if let Some(bytes) = g.memory_own.or(g.memory) {
                *memory.entry(subject).or_default() += bytes as f64;
            }
        }
        self.cpu = counters;
        let mut sample = Sample::default();
        let mut push = |name: &str, unit: Unit, parts: Vec<(String, f64)>| {
            for (subject, value) in parts {
                sample
                    .metrics
                    .push(metric(name, Some(&subject), value, unit, now));
            }
        };
        if !first {
            push(CPU, Unit::Seconds, biggest(cpu, MIN_CPU_SECONDS));
        }
        push(MEMORY, Unit::Bytes, biggest(memory, MIN_MEMORY));
        Ok(sample)
    }
}

/// `app:Files`, `service:cups`; the desktop's own parts get clearer names.
pub fn subject(kind: &str, name: &str) -> String {
    let name = match name.split('@').next().unwrap_or(name) {
        "org.gnome.Shell" => "Desktop (GNOME Shell)",
        _ => name,
    };
    format!("{kind}:{name}")
}

/// The `KEEP` biggest values at or over `min`, biggest first, and the rest added up as `other`
/// (left out when it is 0).
pub fn biggest(values: HashMap<String, f64>, min: f64) -> Vec<(String, f64)> {
    let mut all: Vec<(String, f64)> = values.into_iter().collect();
    all.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut kept = Vec::new();
    let mut other = 0.0;
    for (subject, value) in all {
        if kept.len() < KEEP && value >= min {
            kept.push((subject, value));
        } else {
            other += value;
        }
    }
    if other > 0.0 {
        kept.push(("other".to_owned(), other));
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::{fixture_root, value};

    #[test]
    fn keeps_the_biggest_and_adds_up_the_rest() {
        let mut values: HashMap<String, f64> = (0..20)
            .map(|i| (format!("app:{i:02}"), f64::from(i) + 1.0))
            .collect();
        values.insert("app:tiny".into(), 0.5);
        let kept = biggest(values, 1.0);
        assert_eq!(kept.len(), KEEP + 1);
        assert_eq!(kept[0], ("app:19".to_owned(), 20.0));
        // 1 to 5 did not fit, and tiny is under the minimum: 15.5.
        assert_eq!(kept[KEEP], ("other".to_owned(), 15.5));
        assert!(biggest(HashMap::new(), 1.0).is_empty());
        assert_eq!(
            subject("user-service", "org.gnome.Shell@wayland"),
            "user-service:Desktop (GNOME Shell)"
        );
    }

    fn laptop(name: &str, cpu_usec: u64) -> SystemRoot {
        let app = "/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice";
        let files = [
            (
                "/proc/self/status".to_owned(),
                "Uid:\t1000\t1000\t1000\t1000\n".to_owned(),
            ),
            (
                format!("{app}/app-gnome-org.gnome.Nautilus-42.scope/cpu.stat"),
                format!("usage_usec {cpu_usec}\n"),
            ),
            (
                format!("{app}/app-gnome-org.gnome.Nautilus-42.scope/memory.current"),
                "300000000\n".into(),
            ),
            (
                format!("{app}/app-gnome-org.gnome.Nautilus-42.scope/memory.stat"),
                "anon 200000000\nfile 100000000\n".into(),
            ),
            // The same app started a second time: counts once, with the first.
            (
                format!("{app}/app-gnome-org.gnome.Nautilus-43.scope/cpu.stat"),
                "usage_usec 0\n".into(),
            ),
            (
                format!("{app}/app-gnome-org.gnome.Nautilus-43.scope/memory.current"),
                "1000\n".into(),
            ),
        ];
        let files: Vec<(&str, &str)> = files
            .iter()
            .map(|(p, c)| (p.as_str(), c.as_str()))
            .collect();
        fixture_root(name, &files)
    }

    #[test]
    fn reads_every_five_minutes_and_cpu_from_the_second_reading() {
        let mut c = AppUsage::default();
        let s = c
            .collect(&laptop("appuse-1", 1_000_000), Timestamp(0))
            .unwrap();
        let files = Some("app:org.gnome.Nautilus");
        assert_eq!(
            value(&s, MEMORY, files),
            Some(200_001_000.0),
            "own memory, both together"
        );
        assert_eq!(value(&s, CPU, files), None, "CPU needs a reading before");
        // A minute later: not due.
        let s = c
            .collect(&laptop("appuse-2", 5_000_000), Timestamp(60_000))
            .unwrap();
        assert!(s.metrics.is_empty());
        let s = c
            .collect(&laptop("appuse-3", 31_000_000), Timestamp(300_000))
            .unwrap();
        assert_eq!(value(&s, CPU, files), Some(30.0));
    }
}
