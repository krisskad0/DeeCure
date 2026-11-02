// SPDX-License-Identifier: Apache-2.0

//! Admin read-outs that measure the kernel for a moment: power meters (powercap energy
//! counters over one second), the chip's low-power time (pmc_core or amd_pmc in debugfs),
//! and wake-ups per process (a private tracefs instance for five seconds).
//!
//! Every path is built from a root, so tests use a temporary folder and never the real
//! `/sys`. The tracing instance is the only write; it is private to this helper, removed
//! again in every case, and changes nothing else the kernel does.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

/// One powercap zone's power over the measured second.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PowerMeter {
    /// The zone, such as `intel-rapl:0:1`.
    pub zone: String,
    /// What it measures, such as `package-0`, `core`, `uncore` (built-in graphics), `dram`,
    /// or `psys` (the whole platform).
    pub name: String,
    pub watts: f64,
}

fn number(path: &Path) -> Option<u64> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

/// RAPL zones (`intel-rapl:N` and below; AMD uses the same names). The MMIO copy of the
/// package counter is left out, so the package is not counted twice.
fn zones(root: &Path) -> Vec<(String, PathBuf)> {
    let base = root.join("sys/class/powercap");
    let mut v: Vec<(String, PathBuf)> = std::fs::read_dir(&base)
        .map(|d| {
            d.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| {
                    n.strip_prefix("intel-rapl:").is_some_and(|r| {
                        !r.is_empty() && r.chars().all(|c| c.is_ascii_digit() || c == ':')
                    })
                })
                .map(|n| (n.clone(), base.join(&n)))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// Reads every zone's energy counter, calls `wait` (one second in the helper), and reads
/// again. A counter that wrapped is corrected with its `max_energy_range_uj`.
pub fn power_meters(root: &Path, seconds: f64, wait: impl FnOnce()) -> Vec<PowerMeter> {
    let zones = zones(root);
    let before: Vec<Option<u64>> = zones
        .iter()
        .map(|(_, p)| number(&p.join("energy_uj")))
        .collect();
    wait();
    zones
        .iter()
        .zip(before)
        .filter_map(|((zone, path), before)| {
            let (a, b) = (before?, number(&path.join("energy_uj"))?);
            let range = number(&path.join("max_energy_range_uj")).unwrap_or(0);
            let used = if b >= a {
                b - a
            } else {
                range.checked_sub(a)? + b
            };
            Some(PowerMeter {
                zone: zone.clone(),
                name: std::fs::read_to_string(path.join("name"))
                    .map(|n| n.trim().to_owned())
                    .unwrap_or_else(|_| zone.clone()),
                watts: (used as f64 / 1e6 / seconds * 100.0).round() / 100.0,
            })
        })
        .collect()
}

/// One low-power counter, such as ("Package C10", 123456) or ("S0ix Residency Time", 789).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Counter {
    /// The debugfs file it came from, such as `pmc_core/package_cstate_show`.
    pub source: String,
    pub name: String,
    pub value: u64,
}

/// Debugfs files with the chip's low-power counters. Each is read only when it exists.
const LOW_POWER_FILES: &[&str] = &[
    "pmc_core/slp_s0_residency_usec",
    "pmc_core/package_cstate_show",
    "pmc_core/substate_residencies",
    "amd_pmc/s0ix_stats",
];

/// `name : 123`, `name: 123`, or `name   123` lines; a file that is one number is named
/// after the file. Lines without a number are headings and are skipped.
pub fn counters(source: &str, text: &str) -> Vec<Counter> {
    let file = source.rsplit('/').next().unwrap_or(source);
    if let Ok(n) = text.trim().parse::<u64>() {
        return vec![Counter {
            source: source.to_owned(),
            name: file.to_owned(),
            value: n,
        }];
    }
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            let (name, value) = match l.rsplit_once(':') {
                Some((n, v)) if v.trim().parse::<u64>().is_ok() => (n, v),
                _ => l.rsplit_once(char::is_whitespace)?,
            };
            let name = name.trim().trim_end_matches(':').trim();
            let value = value.trim().parse().ok()?;
            (!name.is_empty()).then(|| Counter {
                source: source.to_owned(),
                name: name.to_owned(),
                value,
            })
        })
        .collect()
}

/// The chip's low-power counters from debugfs (`<root>/sys/kernel/debug`).
pub fn low_power(root: &Path) -> Vec<Counter> {
    let debug = root.join("sys/kernel/debug");
    LOW_POWER_FILES
        .iter()
        .filter_map(|f| Some(counters(f, &std::fs::read_to_string(debug.join(f)).ok()?)))
        .flatten()
        .collect()
}

/// Wake-ups of one process over the measured time.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Wakeups {
    pub pid: u32,
    pub name: String,
    pub per_second: f64,
    /// Kernel threads (such as `kworker`) have no user-space program.
    pub kernel: bool,
}

/// `{ pid: 1234, comm: Xorg } hitcount: 56` lines from a tracing histogram.
pub fn hist_lines(text: &str) -> Vec<(u32, String, u64)> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            let body = l.strip_prefix('{')?;
            let (keys, rest) = body.rsplit_once('}')?;
            let count = rest.trim().strip_prefix("hitcount:")?.trim().parse().ok()?;
            let (pid, comm) = keys.split_once(',')?;
            let pid = pid.trim().strip_prefix("pid:")?.trim().parse().ok()?;
            let comm = comm.trim().strip_prefix("comm:")?.trim().to_owned();
            Some((pid, comm, count))
        })
        .collect()
}

/// Adds thread wake-ups up per process. `owner` gives a thread's process ID and name,
/// and whether it is a kernel thread; a thread that ended keeps its own name.
pub fn per_process(
    lines: &[(u32, String, u64)],
    seconds: f64,
    owner: impl Fn(u32) -> Option<(u32, String, bool)>,
) -> Vec<Wakeups> {
    let mut sum: BTreeMap<u32, (String, bool, u64)> = BTreeMap::new();
    for (tid, comm, count) in lines {
        let (pid, name, kernel) = owner(*tid).unwrap_or((*tid, comm.clone(), false));
        sum.entry(pid).or_insert((name, kernel, 0)).2 += count;
    }
    let mut out: Vec<Wakeups> = sum
        .into_iter()
        .map(|(pid, (name, kernel, n))| Wakeups {
            pid,
            name,
            per_second: (n as f64 / seconds * 10.0).round() / 10.0,
            kernel,
        })
        .collect();
    out.sort_by(|a, b| b.per_second.total_cmp(&a.per_second));
    out.truncate(40);
    out
}

/// The process, name, and kernel-thread flag of a thread, from `<root>/proc`.
pub fn thread_owner(root: &Path, tid: u32) -> Option<(u32, String, bool)> {
    let status = std::fs::read_to_string(root.join(format!("proc/{tid}/status"))).ok()?;
    let field = |k: &str| {
        status
            .lines()
            .find_map(|l| Some(l.strip_prefix(k)?.trim().to_owned()))
    };
    let pid: u32 = field("Tgid:")?.parse().ok()?;
    let name = std::fs::read_to_string(root.join(format!("proc/{pid}/comm")))
        .map(|c| c.trim().to_owned())
        .ok()
        .or_else(|| field("Name:"))?;
    // Kernel threads are children of kthreadd (PID 2), or kthreadd itself.
    let kernel = pid == 2 || field("PPid:").as_deref() == Some("2");
    Some((pid, name, kernel))
}

/// The tracing folder: `/sys/kernel/tracing`, or the older place in debugfs.
pub fn tracing_dir(root: &Path) -> Option<PathBuf> {
    ["sys/kernel/tracing", "sys/kernel/debug/tracing"]
        .iter()
        .map(|p| root.join(p))
        .find(|p| p.join("instances").is_dir())
}

const TRIGGER: &str = "hist:keys=pid,comm:vals=hitcount:sort=hitcount.descending:size=8192";

/// Removes a tracing instance: its trigger first, then the folder.
struct Remove(PathBuf);

impl Drop for Remove {
    fn drop(&mut self) {
        let trigger = self.0.join("events/sched/sched_wakeup/trigger");
        let _ = std::fs::write(trigger, format!("!{TRIGGER}"));
        let _ = std::fs::remove_dir(&self.0);
    }
}

/// Instances left by a helper that was stopped while counting (`deecure-<pid>` whose process
/// is gone), so a leftover trigger never keeps costing time on every wake-up.
fn remove_stale(tracing: &Path, id: u32, proc: &Path) {
    let Ok(dir) = std::fs::read_dir(tracing.join("instances")) else {
        return;
    };
    for e in dir.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let Some(pid) = name
            .strip_prefix("deecure-")
            .and_then(|p| p.parse::<u32>().ok())
        else {
            continue;
        };
        if pid != id && !proc.join(pid.to_string()).exists() {
            drop(Remove(e.path()));
        }
    }
}

/// Counts `sched_wakeup` per thread for as long as `wait` takes, in a tracing instance of
/// its own (`instances/deecure-<id>`), and returns the histogram text. The instance is
/// removed again whatever happens; one that already exists is never touched.
pub fn trace_wakeups(tracing: &Path, id: u32, wait: impl FnOnce()) -> Result<String, String> {
    remove_stale(tracing, id, Path::new("/proc"));
    let instance = tracing.join("instances").join(format!("deecure-{id}"));
    std::fs::create_dir(&instance).map_err(|e| format!("tracing is not available: {e}"))?;
    let _remove = Remove(instance.clone());
    let event = instance.join("events/sched/sched_wakeup");
    std::fs::write(event.join("trigger"), TRIGGER)
        .map_err(|e| format!("the wake-up event cannot be counted: {e}"))?;
    wait();
    std::fs::read_to_string(event.join("hist")).map_err(|e| format!("no wake-up counts: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("deecure-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn measures_power_with_counter_wrap() {
        let root = temp("rapl");
        let z = root.join("sys/class/powercap");
        write(&z.join("intel-rapl:0/name"), "package-0\n");
        write(&z.join("intel-rapl:0/energy_uj"), "1000000\n");
        write(
            &z.join("intel-rapl:0/max_energy_range_uj"),
            "262143328850\n",
        );
        write(&z.join("intel-rapl:0:1/name"), "uncore\n");
        write(&z.join("intel-rapl:0:1/energy_uj"), "262143000000\n");
        write(
            &z.join("intel-rapl:0:1/max_energy_range_uj"),
            "262143328850\n",
        );
        write(&z.join("intel-rapl-mmio:0/energy_uj"), "1\n");
        let meters = power_meters(&root, 1.0, || {
            write(&z.join("intel-rapl:0/energy_uj"), "8500000\n");
            write(&z.join("intel-rapl:0:1/energy_uj"), "671150\n");
        });
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(meters.len(), 2, "the MMIO copy is left out: {meters:?}");
        assert_eq!(
            (meters[0].name.as_str(), meters[0].watts),
            ("package-0", 7.5)
        );
        assert_eq!((meters[1].name.as_str(), meters[1].watts), ("uncore", 1.0));
    }

    #[test]
    fn reads_low_power_counters() {
        let root = temp("pmc");
        let d = root.join("sys/kernel/debug");
        write(&d.join("pmc_core/slp_s0_residency_usec"), "123456\n");
        write(
            &d.join("pmc_core/package_cstate_show"),
            "Package C2 : 1000\nPackage C10 : 50000\n",
        );
        write(
            &d.join("pmc_core/substate_residencies"),
            "Substate   Residency\nS0i2.0     300\nS0i3.0     0\n",
        );
        let c = low_power(&root);
        std::fs::remove_dir_all(&root).unwrap();
        let names: Vec<(&str, u64)> = c.iter().map(|c| (c.name.as_str(), c.value)).collect();
        assert_eq!(
            names,
            [
                ("slp_s0_residency_usec", 123456),
                ("Package C2", 1000),
                ("Package C10", 50000),
                ("S0i2.0", 300),
                ("S0i3.0", 0)
            ]
        );
        let amd = counters(
            "amd_pmc/s0ix_stats",
            "=== S0ix statistics ===\nS0ix Entry Time: 1200\nResidency Time: 345\n",
        );
        assert_eq!(
            amd[1],
            Counter {
                source: "amd_pmc/s0ix_stats".into(),
                name: "Residency Time".into(),
                value: 345
            }
        );
    }

    #[test]
    fn adds_wakeups_up_per_process() {
        let hist = "# event histogram\n#\n{ pid:        812, comm: gnome-shell     } hitcount:        500\n{ pid:        813, comm: gdbus           } hitcount:        100\n{ pid:         15, comm: kworker/0:1, x  } hitcount:         50\n\nTotals:\n    Hits: 650\n";
        let lines = hist_lines(hist);
        assert_eq!(lines[2], (15, "kworker/0:1, x".into(), 50));
        let w = per_process(&lines, 5.0, |tid| match tid {
            812 | 813 => Some((812, "gnome-shell".into(), false)),
            15 => Some((15, "kworker/0:1".into(), true)),
            _ => None,
        });
        assert_eq!(
            w[0],
            Wakeups {
                pid: 812,
                name: "gnome-shell".into(),
                per_second: 120.0,
                kernel: false
            }
        );
        assert!(w[1].kernel);
    }

    #[test]
    fn finds_the_thread_owner() {
        let root = temp("proc");
        write(
            &root.join("proc/813/status"),
            "Name:\tgdbus\nTgid:\t812\nPPid:\t700\n",
        );
        write(&root.join("proc/812/comm"), "gnome-shell\n");
        write(
            &root.join("proc/15/status"),
            "Name:\tkworker/0:1\nTgid:\t15\nPPid:\t2\n",
        );
        let a = thread_owner(&root, 813);
        let b = thread_owner(&root, 15);
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(a, Some((812, "gnome-shell".into(), false)));
        assert_eq!(b, Some((15, "kworker/0:1".into(), true)));
    }

    #[test]
    fn traces_in_its_own_instance_and_always_removes_it() {
        let root = temp("trace");
        let tracing = root.join("sys/kernel/tracing");
        std::fs::create_dir_all(tracing.join("instances")).unwrap();
        assert_eq!(tracing_dir(&root), Some(tracing.clone()));
        // A plain folder is no kernel: the trigger cannot be written, and the instance goes.
        let err = trace_wakeups(&tracing, 7, || {}).unwrap_err();
        assert!(err.contains("cannot be counted"), "{err}");
        assert!(!tracing.join("instances/deecure-7").exists());
        // An instance that exists already is never used or removed.
        std::fs::create_dir(tracing.join("instances/deecure-8")).unwrap();
        assert!(trace_wakeups(&tracing, 8, || {}).is_err());
        assert!(tracing.join("instances/deecure-8").exists());
        // A leftover of a helper that is gone is removed; a running one's and others' stay.
        std::fs::create_dir(tracing.join("instances/deecure-4000000")).unwrap();
        std::fs::create_dir(tracing.join("instances/other")).unwrap();
        std::fs::create_dir_all(root.join("proc/8")).unwrap();
        remove_stale(&tracing, 9, &root.join("proc"));
        assert!(!tracing.join("instances/deecure-4000000").exists());
        assert!(tracing.join("instances/deecure-8").exists());
        assert!(tracing.join("instances/other").exists());
        std::fs::remove_dir_all(&root).unwrap();
    }
}
