// SPDX-License-Identifier: Apache-2.0

//! Disk reads and writes from `/proc/diskstats`.

use std::collections::BTreeMap;
use std::io;
use std::time::Instant;

use deecure_core::{Timestamp, Unit};

use super::{Collector, Sample, SystemRoot, metric};

/// Byte counters per device or interface: (in, out).
type Counters = BTreeMap<String, (u64, u64)>;

/// Bytes read and written per whole disk since boot. Partitions, loop devices, and RAM
/// disks are left out: only names that have a `/sys/block/<name>/device` count.
pub fn parse_diskstats(text: &str, is_disk: impl Fn(&str) -> bool) -> Counters {
    text.lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let name = *fields.get(2)?;
            if !is_disk(name) {
                return None;
            }
            let sectors_read: u64 = fields.get(5)?.parse().ok()?;
            let sectors_written: u64 = fields.get(9)?.parse().ok()?;
            Some((name.to_owned(), (sectors_read * 512, sectors_written * 512)))
        })
        .collect()
}

/// Read and write rates, in total and per disk.
#[derive(Debug, Default)]
pub struct DiskIo {
    previous: Option<(Instant, Counters)>,
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
            let (mut read_total, mut write_total) = (0.0, 0.0);
            for (name, (read, written)) in &counters {
                let Some((old_read, old_written)) = old.get(name) else {
                    continue;
                };
                let read_rate = read.saturating_sub(*old_read) as f64 / seconds;
                let write_rate = written.saturating_sub(*old_written) as f64 / seconds;
                read_total += read_rate;
                write_total += write_rate;
                sample.metrics.push(metric(
                    "disk.read",
                    Some(name),
                    read_rate,
                    Unit::BytesPerSecond,
                    now,
                ));
                sample.metrics.push(metric(
                    "disk.write",
                    Some(name),
                    write_rate,
                    Unit::BytesPerSecond,
                    now,
                ));
            }
            sample.metrics.push(metric(
                "disk.read",
                None,
                read_total,
                Unit::BytesPerSecond,
                now,
            ));
            sample.metrics.push(metric(
                "disk.write",
                None,
                write_total,
                Unit::BytesPerSecond,
                now,
            ));
        }
        Ok(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_whole_disks_and_counts_bytes() {
        let text =
            " 259       0 nvme0n1 131613 70118 10439479 98171 930327 793292 88651866 21175527 0 0 0
 259       1 nvme0n1p1 100 0 2000 10 0 0 0 0 0 0 0
   7       0 loop0 5 0 10 0 0 0 0 0 0 0 0";
        let disks = parse_diskstats(text, |name| name == "nvme0n1");
        assert_eq!(disks.len(), 1);
        assert_eq!(disks["nvme0n1"], (10_439_479 * 512, 88_651_866 * 512));
    }
}
