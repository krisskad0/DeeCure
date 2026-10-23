// SPDX-License-Identifier: Apache-2.0

//! Deep memory details for the Expert level (docs/01-product/expert-coverage.md, section 2):
//! the whole `/proc/meminfo`, page-fault and swap counters, swap devices, compressed swap,
//! huge pages, memory error counters (EDAC), the out-of-memory guard (systemd-oomd), and the
//! real memory (PSS, USS) of the biggest processes. No admin rights; read only when asked.
//! Counters are totals since start, with the reading time, so the app works out rates.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::Serialize;

use super::command;
use crate::collect::SystemRoot;

/// One swap device or file, from `/proc/swaps`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SwapArea {
    pub path: String,
    /// `file` or `partition`.
    pub kind: String,
    pub size: u64,
    pub used: u64,
    pub priority: i64,
}

/// Compressed swap in memory (zswap).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Zswap {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compressor: Option<String>,
    /// Most of RAM it may use, in percent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pool_percent: Option<u64>,
}

/// A compressed RAM disk used as swap (zram).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Zram {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
    /// Size it offers, in bytes.
    pub disk_size: u64,
    /// Data stored before compression, and the memory it really takes.
    pub original: u64,
    pub compressed_used: u64,
}

/// One memory module as the memory controller reports it. Its memory type is left out: some
/// drivers (such as `igen6`) report a wrong one; the module list from the helper has the real type.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Dimm {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_mb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corrected: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uncorrected: Option<u64>,
}

/// One memory controller with its error counts (EDAC).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MemoryController {
    pub name: String,
    pub corrected: u64,
    pub uncorrected: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds_since_reset: Option<u64>,
    pub dimms: Vec<Dimm>,
}

/// The out-of-memory guard, from `oomctl`.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Oomd {
    /// Stops apps when swap use is above this percent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap_limit_percent: Option<f64>,
    /// Stops apps when memory pressure stays above this percent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressure_limit_percent: Option<f64>,
    /// How long the pressure must last, such as `20s`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressure_duration: Option<String>,
    /// The groups it watches for swap, and for pressure (with their own limit).
    pub swap_watched: Vec<String>,
    pub pressure_watched: Vec<(String, Option<f64>)>,
}

/// Everything above, read at one moment.
#[derive(Debug, Default, Serialize)]
pub struct MemoryDetail {
    /// When this was read, in ms since 1970.
    pub at: u64,
    /// Every `/proc/meminfo` line, in bytes (counts such as `HugePages_Total` as they are).
    pub meminfo: BTreeMap<String, u64>,
    /// Page faults (minor and major), swap-ins, swap-outs, and out-of-memory kills since start.
    pub vmstat: BTreeMap<String, u64>,
    pub swaps: Vec<SwapArea>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zswap: Option<Zswap>,
    pub zram: Vec<Zram>,
    /// Transparent huge pages: `always`, `madvise`, or `never`; and when to make them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub huge_pages: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub huge_pages_defrag: Option<String>,
    pub controllers: Vec<MemoryController>,
    /// Missing when systemd-oomd does not run or `oomctl` cannot be read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oomd: Option<Oomd>,
}

/// Real memory of one process.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProcessMemory {
    pub pid: u32,
    pub name: String,
    /// Resident memory, shared pages counted fully.
    pub rss: u64,
    /// Shared pages split between the processes that use them. Missing for other users' processes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pss: Option<u64>,
    /// Memory only this process uses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uss: Option<u64>,
    /// Its share of swap.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap_pss: Option<u64>,
    /// Hard page faults since it started (each one waited for the disk).
    pub major_faults: u64,
    /// The kernel's out-of-memory score: the highest is stopped first.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oom_score: Option<u64>,
}

/// The vmstat counters worth showing.
const VMSTAT: &[&str] = &[
    "pgfault",
    "pgmajfault",
    "pswpin",
    "pswpout",
    "oom_kill",
    "pgscan_direct",
    "pgsteal_direct",
];

/// How many of the biggest processes get the slower PSS reading.
const PSS_TOP: usize = 40;

fn text(root: &SystemRoot, path: &str) -> Option<String> {
    root.read(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn number(root: &SystemRoot, path: &str) -> Option<u64> {
    text(root, path)?.parse().ok()
}

fn names_in(root: &SystemRoot, dir: &str, prefix: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(root.path(dir))
        .map(|d| {
            d.flatten()
                .filter_map(|e| {
                    e.file_name()
                        .to_str()
                        .filter(|n| n.starts_with(prefix))
                        .map(str::to_owned)
                })
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

/// `/proc/meminfo` in bytes. Lines without `kB` (page counts) stay as they are.
pub fn parse_meminfo(text: &str) -> BTreeMap<String, u64> {
    text.lines()
        .filter_map(|line| {
            let (key, rest) = line.split_once(':')?;
            let mut parts = rest.split_whitespace();
            let value: u64 = parts.next()?.parse().ok()?;
            let factor = if parts.next() == Some("kB") { 1024 } else { 1 };
            Some((key.trim().to_owned(), value * factor))
        })
        .collect()
}

/// `/proc/swaps`: sizes there are in KiB.
pub fn parse_swaps(text: &str) -> Vec<SwapArea> {
    text.lines()
        .skip(1)
        .filter_map(|line| {
            let f: Vec<&str> = line.split_whitespace().collect();
            Some(SwapArea {
                path: f.first()?.to_string(),
                kind: f.get(1)?.to_string(),
                size: f.get(2)?.parse::<u64>().ok()? * 1024,
                used: f.get(3)?.parse::<u64>().ok()? * 1024,
                priority: f.get(4)?.parse().ok()?,
            })
        })
        .collect()
}

/// The setting in brackets, as in `always [madvise] never`.
fn bracketed(text: &str) -> Option<String> {
    let start = text.find('[')? + 1;
    let end = text[start..].find(']')? + start;
    Some(text[start..end].to_owned())
}

/// The parts of `oomctl` output that hold settings and the watched groups.
pub fn parse_oomctl(text: &str) -> Oomd {
    let percent = |line: &str| {
        line.rsplit(':')
            .next()?
            .trim()
            .trim_end_matches('%')
            .parse::<f64>()
            .ok()
    };
    let mut out = Oomd::default();
    let mut section = "";
    let mut last_pressure: Option<usize> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with("Swap Used Limit:") {
            out.swap_limit_percent = percent(line);
        } else if line.starts_with("Default Memory Pressure Limit:") {
            out.pressure_limit_percent = percent(line);
        } else if let Some(d) = line.strip_prefix("Default Memory Pressure Duration:") {
            out.pressure_duration = Some(d.trim().to_owned());
        } else if line.starts_with("Swap Monitored CGroups") {
            section = "swap";
        } else if line.starts_with("Memory Pressure Monitored CGroups") {
            section = "pressure";
        } else if line.starts_with("System Context") {
            section = "";
        } else if let Some(path) = line.strip_prefix("Path:") {
            let path = path.trim().to_owned();
            match section {
                "swap" => out.swap_watched.push(path),
                "pressure" => {
                    out.pressure_watched.push((path, None));
                    last_pressure = Some(out.pressure_watched.len() - 1);
                }
                _ => {}
            }
        } else if section == "pressure"
            && line.starts_with("Memory Pressure Limit:")
            && let Some(i) = last_pressure
        {
            out.pressure_watched[i].1 = percent(line);
        }
    }
    out
}

fn controllers(root: &SystemRoot) -> Vec<MemoryController> {
    let base = "/sys/devices/system/edac/mc";
    names_in(root, base, "mc")
        .into_iter()
        .filter_map(|mc| {
            let at = |f: &str| format!("{base}/{mc}/{f}");
            let dimms = names_in(root, &format!("{base}/{mc}"), "dimm")
                .into_iter()
                .map(|d| {
                    let at = |f: &str| format!("{base}/{mc}/{d}/{f}");
                    Dimm {
                        label: text(root, &at("dimm_label")),
                        size_mb: number(root, &at("size")),
                        corrected: number(root, &at("dimm_ce_count")),
                        uncorrected: number(root, &at("dimm_ue_count")),
                    }
                })
                .collect();
            Some(MemoryController {
                name: text(root, &at("mc_name")).unwrap_or_else(|| mc.clone()),
                corrected: number(root, &at("ce_count"))?,
                uncorrected: number(root, &at("ue_count")).unwrap_or(0),
                seconds_since_reset: number(root, &at("seconds_since_reset")),
                dimms,
            })
        })
        .collect()
}

fn zram(root: &SystemRoot) -> Vec<Zram> {
    names_in(root, "/sys/block", "zram")
        .into_iter()
        .filter_map(|name| {
            let at = |f: &str| format!("/sys/block/{name}/{f}");
            // mm_stat: original size, compressed size, memory used, …
            let stat: Vec<u64> = text(root, &at("mm_stat"))?
                .split_whitespace()
                .filter_map(|v| v.parse().ok())
                .collect();
            Some(Zram {
                algorithm: text(root, &at("comp_algorithm"))
                    .and_then(|a| bracketed(&a).or(Some(a))),
                disk_size: number(root, &at("disksize")).unwrap_or(0),
                original: *stat.first()?,
                compressed_used: *stat.get(2)?,
                name,
            })
        })
        .collect()
}

/// Reads everything for `memory_detail`.
pub async fn read(root: &SystemRoot) -> MemoryDetail {
    let vmstat_text = root.read("/proc/vmstat").unwrap_or_default();
    let vmstat = vmstat_text
        .lines()
        .filter_map(|l| {
            let (k, v) = l.split_once(' ')?;
            if !VMSTAT.contains(&k) {
                return None;
            }
            Some((k.to_owned(), v.trim().parse().ok()?))
        })
        .collect();
    let zswap = text(root, "/sys/module/zswap/parameters/enabled").map(|e| Zswap {
        enabled: e == "Y" || e == "1",
        compressor: text(root, "/sys/module/zswap/parameters/compressor"),
        max_pool_percent: number(root, "/sys/module/zswap/parameters/max_pool_percent"),
    });
    let oomd = command::output("oomctl", &[], Duration::from_secs(3))
        .await
        .map(|t| parse_oomctl(&t));
    MemoryDetail {
        at: now_ms(),
        meminfo: root
            .read("/proc/meminfo")
            .map(|t| parse_meminfo(&t))
            .unwrap_or_default(),
        vmstat,
        swaps: root
            .read("/proc/swaps")
            .map(|t| parse_swaps(&t))
            .unwrap_or_default(),
        zswap,
        zram: zram(root),
        huge_pages: text(root, "/sys/kernel/mm/transparent_hugepage/enabled")
            .and_then(|t| bracketed(&t)),
        huge_pages_defrag: text(root, "/sys/kernel/mm/transparent_hugepage/defrag")
            .and_then(|t| bracketed(&t)),
        controllers: controllers(root),
        oomd,
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// PSS, USS, and swap share from `smaps_rollup`, in bytes.
pub fn parse_rollup(text: &str) -> (Option<u64>, Option<u64>, Option<u64>) {
    let kb = |key: &str| {
        text.lines().find_map(|l| {
            l.strip_prefix(key)?
                .trim()
                .strip_suffix("kB")?
                .trim()
                .parse::<u64>()
                .ok()
                .map(|v| v * 1024)
        })
    };
    let uss = match (kb("Private_Clean:"), kb("Private_Dirty:")) {
        (Some(a), Some(b)) => Some(a + b + kb("Private_Hugetlb:").unwrap_or(0)),
        _ => None,
    };
    (kb("Pss:"), uss, kb("SwapPss:"))
}

/// Field 12 of `/proc/<pid>/stat` (major faults), after the name in brackets.
fn major_faults(stat: &str) -> Option<u64> {
    let after = &stat[stat.rfind(')')? + 2..];
    after.split_whitespace().nth(9)?.parse().ok()
}

/// Reads `process_memory`: every process's resident memory, hard page faults, and out-of-memory
/// score, and PSS and USS for the biggest ones the user may read.
pub fn read_processes(root: &SystemRoot) -> Vec<ProcessMemory> {
    // The page size on amd64, the only supported architecture (open decision #9).
    let page = 4096u64;
    let mut out: Vec<ProcessMemory> = std::fs::read_dir(root.path("/proc"))
        .map(|d| {
            d.flatten()
                .filter_map(|e| e.file_name().to_str()?.parse::<u32>().ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
        .into_iter()
        .filter_map(|pid| {
            let stat = root.read(&format!("/proc/{pid}/stat")).ok()?;
            let name = stat.get(stat.find('(')? + 1..stat.rfind(')')?)?.to_owned();
            let rss_pages: u64 = root
                .read(&format!("/proc/{pid}/statm"))
                .ok()?
                .split_whitespace()
                .nth(1)?
                .parse()
                .ok()?;
            Some(ProcessMemory {
                pid,
                name,
                rss: rss_pages * page,
                pss: None,
                uss: None,
                swap_pss: None,
                major_faults: major_faults(&stat).unwrap_or(0),
                oom_score: number(root, &format!("/proc/{pid}/oom_score")),
            })
        })
        .filter(|p| p.rss > 0)
        .collect();
    out.sort_by_key(|p| std::cmp::Reverse(p.rss));
    for p in out.iter_mut().take(PSS_TOP) {
        if let Ok(text) = root.read(&format!("/proc/{}/smaps_rollup", p.pid)) {
            (p.pss, p.uss, p.swap_pss) = parse_rollup(&text);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn meminfo_is_in_bytes_and_page_counts_stay() {
        let m = parse_meminfo(
            "MemTotal:       16000000 kB\nHugePages_Total:       0\nSlab:   2048 kB\n",
        );
        assert_eq!(m["MemTotal"], 16_000_000 * 1024);
        assert_eq!(m["HugePages_Total"], 0);
        assert_eq!(m["Slab"], 2048 * 1024);
    }

    #[test]
    fn swaps_and_oomctl_are_read() {
        let s = parse_swaps(
            "Filename\tType\tSize\tUsed\tPriority\n/swap.img                               file\t\t8388604\t\t2411520\t\t-2\n",
        );
        assert_eq!(s[0].path, "/swap.img");
        assert_eq!(s[0].size, 8_388_604 * 1024);
        assert_eq!(s[0].priority, -2);
        let o = parse_oomctl(
            "Dry Run: no\nSwap Used Limit: 90.00%\nDefault Memory Pressure Limit: 60.00%\nDefault Memory Pressure Duration: 20s\nSystem Context:\n\tMemory: Used: 0B Total: 0B\nSwap Monitored CGroups:\nMemory Pressure Monitored CGroups:\n\tPath: /user.slice/user-1001.slice/user@1001.service\n\t\tMemory Pressure Limit: 50.00%\n\t\tPressure: Avg10: 0.00\n",
        );
        assert_eq!(o.swap_limit_percent, Some(90.0));
        assert_eq!(o.pressure_limit_percent, Some(60.0));
        assert_eq!(o.pressure_duration.as_deref(), Some("20s"));
        assert!(o.swap_watched.is_empty());
        assert_eq!(
            o.pressure_watched,
            vec![(
                "/user.slice/user-1001.slice/user@1001.service".to_owned(),
                Some(50.0)
            )]
        );
    }

    #[test]
    fn rollup_gives_pss_uss_and_swap() {
        let (pss, uss, swap) = parse_rollup(
            "00400000-7fff [rollup]\nRss:  2056 kB\nPss:   169 kB\nPrivate_Clean:  10 kB\nPrivate_Dirty:  20 kB\nSwapPss:  4 kB\n",
        );
        assert_eq!(
            (pss, uss, swap),
            (Some(169 * 1024), Some(30 * 1024), Some(4 * 1024))
        );
        assert_eq!(parse_rollup("nothing"), (None, None, None));
    }

    #[test]
    fn hard_faults_come_after_the_name() {
        assert_eq!(
            major_faults("42 (Web Content) S 1 42 42 0 -1 4194560 1000 0 77 0 5 6"),
            Some(77)
        );
    }

    #[tokio::test]
    async fn reads_a_laptop_and_leaves_out_what_is_missing() {
        let root = fixture_root(
            "memory-detail",
            &[
                (
                    "/proc/meminfo",
                    "MemTotal: 16000000 kB\nMemAvailable: 8000000 kB\n",
                ),
                (
                    "/proc/vmstat",
                    "nr_free_pages 1\npgfault 5000\npgmajfault 12\npswpin 3\npswpout 9\noom_kill 0\n",
                ),
                (
                    "/proc/swaps",
                    "Filename Type Size Used Priority\n/swap.img file 1024 512 -2\n",
                ),
                ("/sys/module/zswap/parameters/enabled", "N\n"),
                ("/sys/module/zswap/parameters/compressor", "lzo\n"),
                (
                    "/sys/kernel/mm/transparent_hugepage/enabled",
                    "always [madvise] never\n",
                ),
                (
                    "/sys/devices/system/edac/mc/mc0/mc_name",
                    "Intel_client_SoC MC#0\n",
                ),
                ("/sys/devices/system/edac/mc/mc0/ce_count", "0\n"),
                ("/sys/devices/system/edac/mc/mc0/ue_count", "0\n"),
                ("/sys/devices/system/edac/mc/mc0/dimm0/size", "8192\n"),
                ("/proc/7/stat", "7 (chrome) S 1 7 7 0 -1 0 10 0 3 0\n"),
                ("/proc/7/statm", "1000 500 10 1 0 100 0\n"),
                ("/proc/7/oom_score", "667\n"),
                (
                    "/proc/7/smaps_rollup",
                    "Pss: 1000 kB\nPrivate_Clean: 100 kB\nPrivate_Dirty: 200 kB\n",
                ),
                ("/proc/8/stat", "8 (kthreadd) S 0 0 0 0 -1 0 0 0 0 0\n"),
                ("/proc/8/statm", "0 0 0 0 0 0 0\n"),
            ],
        );
        let d = read(&root).await;
        assert_eq!(d.meminfo["MemAvailable"], 8_000_000 * 1024);
        assert_eq!(d.vmstat.get("pgmajfault"), Some(&12));
        assert!(!d.vmstat.contains_key("nr_free_pages"));
        assert_eq!(d.swaps[0].used, 512 * 1024);
        assert_eq!(d.zswap.as_ref().map(|z| z.enabled), Some(false));
        assert!(d.zram.is_empty());
        assert_eq!(d.huge_pages.as_deref(), Some("madvise"));
        assert_eq!(d.huge_pages_defrag, None);
        assert_eq!(d.controllers[0].dimms[0].size_mb, Some(8192));
        let p = read_processes(&root);
        assert_eq!(p.len(), 1, "kernel threads have no memory");
        assert_eq!(
            (p[0].rss, p[0].pss, p[0].uss),
            (500 * 4096, Some(1000 * 1024), Some(300 * 1024))
        );
        assert_eq!((p[0].major_faults, p[0].oom_score), (3, Some(667)));
    }
}
