// SPDX-License-Identifier: Apache-2.0

//! CPU usage from `/proc/stat` and load from `/proc/loadavg`.

use std::io;

use deecure_core::{Timestamp, Unit};

use super::{Collector, Sample, SystemRoot, invalid, metric};

/// The CPU time counters from the `cpu` line of `/proc/stat`, in clock ticks since boot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CpuTimes {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
}

impl CpuTimes {
    fn total(&self) -> u64 {
        // `guest` time is already counted inside `user`, so it is left out.
        self.user
            + self.nice
            + self.system
            + self.idle
            + self.iowait
            + self.irq
            + self.softirq
            + self.steal
    }
}

/// Reads the all-CPU line of `/proc/stat`.
pub fn parse_stat(text: &str) -> io::Result<CpuTimes> {
    let line = text
        .lines()
        .find(|line| line.starts_with("cpu "))
        .ok_or_else(|| invalid("/proc/stat", "no 'cpu' line"))?;
    parse_cpu_line(line)
}

/// Reads the per-core lines (`cpu0`, `cpu1`, …) of `/proc/stat`, in order.
pub fn parse_cores(text: &str) -> Vec<CpuTimes> {
    text.lines()
        .filter(|line| {
            line.starts_with("cpu") && line.as_bytes().get(3).is_some_and(u8::is_ascii_digit)
        })
        .filter_map(|line| parse_cpu_line(line).ok())
        .collect()
}

fn parse_cpu_line(line: &str) -> io::Result<CpuTimes> {
    let numbers = line
        .split_whitespace()
        .skip(1)
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| invalid("/proc/stat", e))?;
    if numbers.len() < 4 {
        return Err(invalid("/proc/stat", "the 'cpu' line is too short"));
    }
    let field = |i: usize| numbers.get(i).copied().unwrap_or(0);
    Ok(CpuTimes {
        user: field(0),
        nice: field(1),
        system: field(2),
        idle: field(3),
        iowait: field(4),
        irq: field(5),
        softirq: field(6),
        steal: field(7),
    })
}

/// Reads the 1-minute load average from `/proc/loadavg`.
pub fn parse_loadavg(text: &str) -> io::Result<f64> {
    text.split_whitespace()
        .next()
        .ok_or_else(|| invalid("/proc/loadavg", "empty"))?
        .parse()
        .map_err(|e| invalid("/proc/loadavg", e))
}

/// Percentages of CPU time spent between two readings.
fn usage(before: CpuTimes, after: CpuTimes) -> Option<[(&'static str, f64); 4]> {
    let total = after.total().saturating_sub(before.total());
    if total == 0 {
        return None;
    }
    let percent = |ticks: u64| ticks as f64 * 100.0 / total as f64;
    let idle = after.idle.saturating_sub(before.idle);
    let iowait = after.iowait.saturating_sub(before.iowait);
    let user = (after.user + after.nice).saturating_sub(before.user + before.nice);
    let system = (after.system + after.irq + after.softirq)
        .saturating_sub(before.system + before.irq + before.softirq);
    let busy = total.saturating_sub(idle + iowait);
    Some([
        ("cpu.usage", percent(busy)),
        ("cpu.user", percent(user)),
        ("cpu.system", percent(system)),
        ("cpu.iowait", percent(iowait)),
    ])
}

/// CPU usage needs two readings, so the first run reports only the load average.
#[derive(Debug, Default)]
pub struct Cpu {
    previous: Option<CpuTimes>,
    previous_cores: Vec<CpuTimes>,
}

impl Collector for Cpu {
    fn name(&self) -> &'static str {
        "cpu"
    }

    fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let stat = root.read("/proc/stat")?;
        let times = parse_stat(&stat)?;
        let cores = parse_cores(&stat);
        let load = parse_loadavg(&root.read("/proc/loadavg")?)?;
        let mut sample = Sample::default();
        if let Some(percentages) = self
            .previous
            .replace(times)
            .and_then(|before| usage(before, times))
        {
            for (name, value) in percentages {
                sample
                    .metrics
                    .push(metric(name, None, value, Unit::Percent, now));
            }
        }
        let before = std::mem::replace(&mut self.previous_cores, cores.clone());
        for (i, (old, new)) in before.iter().zip(&cores).enumerate() {
            if let Some([(_, busy), ..]) = usage(*old, *new) {
                sample.metrics.push(metric(
                    "cpu.core",
                    Some(&i.to_string()),
                    busy,
                    Unit::Percent,
                    now,
                ));
            }
        }
        sample
            .metrics
            .push(metric("cpu.load_1m", None, load, Unit::Count, now));
        Ok(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::{fixture_root, value};

    const STAT_BEFORE: &str =
        "cpu  1000 0 500 8000 500 0 0 0 0 0\ncpu0 500 0 250 4000 250 0 0 0 0 0\n";
    const STAT_AFTER: &str =
        "cpu  1300 100 600 8800 700 0 0 0 0 0\ncpu0 650 50 300 4400 350 0 0 0 0 0\n";

    #[test]
    fn parses_the_all_cpu_line() {
        let times = parse_stat(STAT_BEFORE).unwrap();
        assert_eq!(times.total(), 10_000);
        assert!(parse_stat("intr 1 2 3").is_err());
        assert!(parse_stat("cpu  1 2").is_err());
    }

    #[test]
    fn usage_splits_into_user_system_and_iowait() {
        // Between the readings: total 1500, idle 800, iowait 200, user+nice 400, system 100.
        let [usage, user, system, iowait] = usage(
            parse_stat(STAT_BEFORE).unwrap(),
            parse_stat(STAT_AFTER).unwrap(),
        )
        .unwrap();
        assert_eq!(usage, ("cpu.usage", 500.0 * 100.0 / 1500.0));
        assert_eq!(user.1, 400.0 * 100.0 / 1500.0);
        assert_eq!(system.1, 100.0 * 100.0 / 1500.0);
        assert_eq!(iowait.1, 200.0 * 100.0 / 1500.0);
    }

    #[test]
    fn first_run_reports_load_only_then_usage() {
        let root = fixture_root(
            "cpu",
            &[
                ("/proc/stat", STAT_BEFORE),
                ("/proc/loadavg", "0.84 0.70 0.65 2/1031 12345\n"),
            ],
        );
        let mut cpu = Cpu::default();
        let first = cpu.collect(&root, Timestamp(0)).unwrap();
        assert_eq!(value(&first, "cpu.load_1m", None), Some(0.84));
        assert_eq!(value(&first, "cpu.usage", None), None);

        std::fs::write(root.path("/proc/stat"), STAT_AFTER).unwrap();
        let second = cpu.collect(&root, Timestamp(60_000)).unwrap();
        let usage = value(&second, "cpu.usage", None).unwrap();
        assert!((usage - 33.33).abs() < 0.01, "usage was {usage}");
        // cpu0: total 750, idle 400, iowait 100 → busy 250 → 33.3%
        let core = value(&second, "cpu.core", Some("0")).unwrap();
        assert!((core - 33.33).abs() < 0.01, "core 0 was {core}");
    }
}
