// SPDX-License-Identifier: Apache-2.0

//! Disk reads and writes from `/proc/diskstats`.

use std::collections::BTreeMap;
use std::io;
use std::time::Instant;

use deecure_core::{Metric, Timestamp, Unit};

use super::{Collector, Sample, SystemRoot, metric};

/// Counters for one disk since boot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DiskCounters {
    /// Bytes read.
    pub read: u64,
    /// Bytes written.
    pub written: u64,
    /// Reads and writes completed.
    pub ios: u64,
    /// Milliseconds reads and writes took, added up.
    pub io_ms: u64,
    /// Milliseconds the disk had work to do.
    pub busy_ms: u64,
}

/// Counters per whole disk. Partitions, loop devices, and RAM disks are left out:
/// only names that have a `/sys/block/<name>/device` count.
pub fn parse_diskstats(
    text: &str,
    is_disk: impl Fn(&str) -> bool,
) -> BTreeMap<String, DiskCounters> {
    text.lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let name = *fields.get(2)?;
            if !is_disk(name) {
                return None;
            }
            let num = |i: usize| fields.get(i).and_then(|f| f.parse::<u64>().ok());
            Some((
                name.to_owned(),
                DiskCounters {
                    read: num(5)? * 512,
                    written: num(9)? * 512,
                    ios: num(3)? + num(7)?,
                    io_ms: num(6)? + num(10)?,
                    busy_ms: num(12).unwrap_or(0),
                },
            ))
        })
        .collect()
}

/// Rates between two readings `seconds` apart.
fn rates(
    old: DiskCounters,
    new: DiskCounters,
    seconds: f64,
    subject: Option<&str>,
    now: Timestamp,
) -> Vec<Metric> {
    let delta = |a: u64, b: u64| b.saturating_sub(a) as f64;
    let ios = delta(old.ios, new.ios);
    let mut out = vec![
        metric(
            "disk.read",
            subject,
            delta(old.read, new.read) / seconds,
            Unit::BytesPerSecond,
            now,
        ),
        metric(
            "disk.write",
            subject,
            delta(old.written, new.written) / seconds,
            Unit::BytesPerSecond,
            now,
        ),
        metric("disk.iops", subject, ios / seconds, Unit::Count, now),
        metric(
            "disk.busy",
            subject,
            (delta(old.busy_ms, new.busy_ms) / (seconds * 10.0)).min(100.0),
            Unit::Percent,
            now,
        ),
    ];
    if ios > 0.0 {
        let wait = delta(old.io_ms, new.io_ms) / ios / 1000.0;
        out.push(metric("disk.await", subject, wait, Unit::Seconds, now));
    }
    out
}

/// Read and write rates, operations, busy time, and wait time, in total and per disk.
#[derive(Debug, Default)]
pub struct DiskIo {
    previous: Option<(Instant, BTreeMap<String, DiskCounters>)>,
}

impl Collector for DiskIo {
    fn name(&self) -> &'static str {
        "disk I/O"
    }

    fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let is_disk = |name: &str| root.path(&format!("/sys/block/{name}/device")).exists();
        let counters = parse_diskstats(&root.read("/proc/diskstats")?, is_disk);
        let at = Instant::now();
        let mut sample = Sample::default();
        if let Some((before, old)) = self.previous.replace((at, counters.clone())) {
            let seconds = at.duration_since(before).as_secs_f64().max(0.001);
            let (mut old_sum, mut new_sum) = (DiskCounters::default(), DiskCounters::default());
            let mut busiest: f64 = 0.0;
            for (name, new) in &counters {
                let Some(old) = old.get(name) else { continue };
                for m in rates(*old, *new, seconds, Some(name), now) {
                    if m.name == "disk.busy" {
                        busiest = busiest.max(m.value);
                    }
                    sample.metrics.push(m);
                }
                for (sum, c) in [(&mut old_sum, old), (&mut new_sum, new)] {
                    sum.read += c.read;
                    sum.written += c.written;
                    sum.ios += c.ios;
                    sum.io_ms += c.io_ms;
                }
            }
            for mut m in rates(old_sum, new_sum, seconds, None, now) {
                // For all disks together, "busy" is the busiest disk.
                if m.name == "disk.busy" {
                    m.value = busiest;
                }
                sample.metrics.push(m);
            }
        }
        Ok(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::value;

    #[test]
    fn keeps_whole_disks_and_reads_counters() {
        let text =
            " 259       0 nvme0n1 131613 70118 10439479 98171 930327 793292 88651866 21175527 0 3000 0
 259       1 nvme0n1p1 100 0 2000 10 0 0 0 0 0 0 0
   7       0 loop0 5 0 10 0 0 0 0 0 0 0 0";
        let disks = parse_diskstats(text, |name| name == "nvme0n1");
        assert_eq!(disks.len(), 1);
        let d = disks["nvme0n1"];
        assert_eq!((d.read, d.written), (10_439_479 * 512, 88_651_866 * 512));
        assert_eq!(
            (d.ios, d.io_ms, d.busy_ms),
            (131_613 + 930_327, 98_171 + 21_175_527, 3000)
        );
    }

    #[test]
    fn rates_between_two_readings() {
        let old = DiskCounters::default();
        let new = DiskCounters {
            read: 6_000_000,
            written: 0,
            ios: 120,
            io_ms: 60,
            busy_ms: 6000,
        };
        let sample = Sample {
            metrics: rates(old, new, 60.0, None, Timestamp(0)),
            ..Sample::default()
        };
        assert_eq!(value(&sample, "disk.read", None), Some(100_000.0));
        assert_eq!(value(&sample, "disk.iops", None), Some(2.0));
        assert_eq!(value(&sample, "disk.busy", None), Some(10.0));
        assert_eq!(value(&sample, "disk.await", None), Some(0.0005));
    }
}
