// SPDX-License-Identifier: Apache-2.0

//! Memory and swap from `/proc/meminfo`.

use std::collections::HashMap;
use std::io;

use deecure_core::{Timestamp, Unit};

use super::{Collector, Sample, SystemRoot, invalid, metric};

/// The `/proc/meminfo` fields we use, in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemInfo {
    /// All usable RAM.
    pub total: u64,
    /// RAM that programs can still get without swapping.
    pub available: u64,
    /// All swap space.
    pub swap_total: u64,
    /// Unused swap space.
    pub swap_free: u64,
    /// Page cache and reclaimable kernel memory (given back when programs need it).
    pub cache: u64,
    /// Buffers for block devices.
    pub buffers: u64,
    /// Completely unused memory.
    pub free: u64,
}

/// Reads `/proc/meminfo`. Values there are in kB (1024 bytes).
pub fn parse_meminfo(text: &str) -> io::Result<MemInfo> {
    let fields: HashMap<&str, u64> = text
        .lines()
        .filter_map(|line| {
            let (key, rest) = line.split_once(':')?;
            let number = rest
                .trim()
                .trim_end_matches("kB")
                .trim()
                .parse::<u64>()
                .ok()?;
            Some((key, number.saturating_mul(1024)))
        })
        .collect();
    let field = |key: &str| fields.get(key).copied();
    let total = field("MemTotal").ok_or_else(|| invalid("/proc/meminfo", "no MemTotal"))?;
    // MemAvailable exists since Linux 3.14; estimate it on anything older.
    let available = field("MemAvailable").unwrap_or_else(|| {
        field("MemFree").unwrap_or(0) + field("Buffers").unwrap_or(0) + field("Cached").unwrap_or(0)
    });
    Ok(MemInfo {
        total,
        available: available.min(total),
        swap_total: field("SwapTotal").unwrap_or(0),
        swap_free: field("SwapFree").unwrap_or(0),
        cache: field("Cached").unwrap_or(0) + field("SReclaimable").unwrap_or(0),
        buffers: field("Buffers").unwrap_or(0),
        free: field("MemFree").unwrap_or(0),
    })
}

/// Memory use. Swap metrics appear only when the system has swap.
#[derive(Debug, Default)]
pub struct Memory;

impl Collector for Memory {
    fn name(&self) -> &'static str {
        "memory"
    }

    fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let info = parse_meminfo(&root.read("/proc/meminfo")?)?;
        let used = info.total - info.available;
        let mut sample = Sample::default();
        let mut push =
            |name, value, unit| sample.metrics.push(metric(name, None, value, unit, now));
        push("memory.used", used as f64, Unit::Bytes);
        push("memory.total", info.total as f64, Unit::Bytes);
        push("memory.cache", info.cache as f64, Unit::Bytes);
        push("memory.buffers", info.buffers as f64, Unit::Bytes);
        push("memory.free", info.free as f64, Unit::Bytes);
        if info.total > 0 {
            push(
                "memory.used_percent",
                used as f64 * 100.0 / info.total as f64,
                Unit::Percent,
            );
        }
        if info.swap_total > 0 {
            let swap_used = info.swap_total.saturating_sub(info.swap_free);
            push("swap.used", swap_used as f64, Unit::Bytes);
            push(
                "swap.used_percent",
                swap_used as f64 * 100.0 / info.swap_total as f64,
                Unit::Percent,
            );
        }
        Ok(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::{fixture_root, value};

    const MEMINFO: &str = "\
MemTotal:        8000000 kB
MemFree:          500000 kB
MemAvailable:    2000000 kB
Buffers:          100000 kB
Cached:          1000000 kB
SwapTotal:       2000000 kB
SwapFree:        1500000 kB
HugePages_Total:       0
";

    #[test]
    fn parses_fields_into_bytes() {
        let info = parse_meminfo(MEMINFO).unwrap();
        assert_eq!(info.total, 8_000_000 * 1024);
        assert_eq!(info.available, 2_000_000 * 1024);
        assert!(parse_meminfo("Nothing: 1 kB").is_err());
    }

    #[test]
    fn old_kernels_without_memavailable_get_an_estimate() {
        let info =
            parse_meminfo("MemTotal: 1000 kB\nMemFree: 100 kB\nBuffers: 50 kB\nCached: 250 kB\n")
                .unwrap();
        assert_eq!(info.available, 400 * 1024);
    }

    #[test]
    fn reports_memory_and_swap() {
        let root = fixture_root("memory", &[("/proc/meminfo", MEMINFO)]);
        let sample = Memory.collect(&root, Timestamp(0)).unwrap();
        assert_eq!(value(&sample, "memory.used_percent", None), Some(75.0));
        assert_eq!(
            value(&sample, "memory.used", None),
            Some(6_000_000.0 * 1024.0)
        );
        assert_eq!(value(&sample, "swap.used_percent", None), Some(25.0));
    }

    #[test]
    fn no_swap_means_no_swap_metrics() {
        let root = fixture_root(
            "noswap",
            &[(
                "/proc/meminfo",
                "MemTotal: 1000 kB\nMemAvailable: 500 kB\nSwapTotal: 0 kB\n",
            )],
        );
        let sample = Memory.collect(&root, Timestamp(0)).unwrap();
        assert_eq!(value(&sample, "swap.used", None), None);
        assert_eq!(value(&sample, "memory.total", None), Some(1000.0 * 1024.0));
    }
}
