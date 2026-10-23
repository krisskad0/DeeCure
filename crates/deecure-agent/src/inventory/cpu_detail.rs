// SPDX-License-Identifier: Apache-2.0

//! Deep processor details for the Expert level (docs/01-product/expert-coverage.md, section 1):
//! each core's speed policy, heat slowdowns, and sleep states, plus power limits, caches, CPU
//! flaws, interrupts, and the microcode on disk. Everything comes from `/sys` and `/proc`, needs
//! no admin rights, and is read only when the app asks. A value the computer does not report is
//! left out, never guessed. Counters are totals since start; the app turns them into rates.

use serde::Serialize;

use super::hardware::cpu_list;
use crate::collect::SystemRoot;

/// One sleep state of one core, from `cpuidle`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IdleState {
    /// Short name, such as `C1E` or `C10`.
    pub name: String,
    /// The kernel's description, such as "MWAIT 0x60".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Time to wake up from this state, in microseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_us: Option<u64>,
    /// Times the core entered this state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<u64>,
    /// Time spent in this state, in microseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_us: Option<u64>,
    /// Turned off by a setting.
    pub disabled: bool,
}

/// One logical CPU.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Core {
    pub cpu: u32,
    /// Clock speed now, in kHz.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub now_khz: Option<u64>,
    /// The lowest and highest speed the speed policy allows, in kHz.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_khz: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_khz: Option<u64>,
    /// The highest speed the hardware can reach, in kHz.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_max_khz: Option<u64>,
    /// Speed policy, such as `powersave` or `performance`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub governor: Option<String>,
    /// Energy preference, such as `balance_power`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy_preference: Option<String>,
    /// Firmware ranking of the core's top speed; the highest are the preferred cores.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highest_perf: Option<u64>,
    /// Times heat slowed this core down, and the longest slowdown in ms (Intel).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttle_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttle_max_ms: Option<u64>,
    pub idle: Vec<IdleState>,
}

/// One CPU cache.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Cache {
    pub level: u32,
    /// `Data`, `Instruction`, or `Unified`.
    pub kind: String,
    pub bytes: u64,
    /// The CPUs that share it, such as `0-1`.
    pub shared_cpus: String,
    /// How many of them share each copy.
    pub copies: usize,
}

/// One power limit from the firmware, such as the long-term limit of the whole chip.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PowerLimit {
    /// The zone, such as `package-0`.
    pub zone: String,
    /// `long_term`, `short_term`, or `peak_power`.
    pub name: String,
    pub watts: f64,
    /// How long the limit is averaged over, in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_s: Option<f64>,
    /// Whether the zone's limits are on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// One known CPU security flaw.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Flaw {
    /// The kernel's name, such as `spectre_v2`.
    pub name: String,
    /// `not_affected`, `mitigated`, `vulnerable`, or `unknown`.
    pub state: &'static str,
    /// The kernel's own words.
    pub detail: String,
}

/// One interrupt line, with its count summed over all CPUs.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Interrupt {
    /// The number, or a name such as `LOC` or `NMI`.
    pub irq: String,
    pub count: u64,
    /// The device or the kernel's description.
    pub device: String,
}

/// Everything above, read at one moment.
#[derive(Debug, Default, Serialize)]
pub struct CpuDetail {
    /// When this was read, in ms since 1970, so the app can work out rates.
    pub at: u64,
    /// Frequency driver, such as `intel_pstate`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    /// The driver's mode, such as `active`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver_mode: Option<String>,
    /// Turbo on. Missing when the CPU has no turbo switch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turbo: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stepping: Option<u32>,
    /// Microcode revision running now, such as `0x4123`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub microcode_loaded: Option<String>,
    /// Newest revision for this CPU in the installed microcode files (Intel). The app compares
    /// it with the running one: a higher one is loaded at the next restart.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub microcode_on_disk: Option<String>,
    /// Times heat slowed the whole chip, and the longest slowdown in ms (Intel).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_throttle_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_throttle_max_ms: Option<u64>,
    pub cores: Vec<Core>,
    pub caches: Vec<Cache>,
    pub power_limits: Vec<PowerLimit>,
    pub flaws: Vec<Flaw>,
    pub interrupts: Vec<Interrupt>,
}

fn text(root: &SystemRoot, path: &str) -> Option<String> {
    root.read(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn number(root: &SystemRoot, path: &str) -> Option<u64> {
    text(root, path)?.parse().ok()
}

/// `48K`, `2M`, or a plain number of bytes.
fn cache_bytes(size: &str) -> Option<u64> {
    let size = size.trim();
    let (digits, factor) = match size.chars().last()? {
        'K' => (&size[..size.len() - 1], 1024),
        'M' => (&size[..size.len() - 1], 1024 * 1024),
        'G' => (&size[..size.len() - 1], 1024 * 1024 * 1024),
        _ => (size, 1),
    };
    digits.parse::<u64>().ok().map(|n| n * factor)
}

fn flaw_state(detail: &str) -> &'static str {
    let lower = detail.to_ascii_lowercase();
    if lower.starts_with("not affected") {
        "not_affected"
    } else if lower.starts_with("vulnerable") {
        "vulnerable"
    } else if lower.starts_with("mitigation") {
        "mitigated"
    } else {
        "unknown"
    }
}

/// Reads `/proc/interrupts`: counts per CPU are summed; the rest of the line names the device.
pub fn parse_interrupts(text: &str) -> Vec<Interrupt> {
    let mut lines = text.lines();
    let cpus = lines
        .next()
        .map(|h| h.split_whitespace().count())
        .unwrap_or(0);
    lines
        .filter_map(|line| {
            let (irq, rest) = line.split_once(':')?;
            let mut fields = rest.split_whitespace().peekable();
            let mut count = 0u64;
            let mut seen = 0;
            while seen < cpus {
                match fields.peek().and_then(|f| f.parse::<u64>().ok()) {
                    Some(n) => {
                        count += n;
                        fields.next();
                        seen += 1;
                    }
                    None => break,
                }
            }
            let device = fields.collect::<Vec<_>>().join(" ");
            (seen > 0).then(|| Interrupt {
                irq: irq.trim().to_owned(),
                count,
                device,
            })
        })
        .collect()
}

/// The highest revision in an Intel microcode file (one or more 48-byte headers with data).
pub fn intel_microcode_revision(bytes: &[u8]) -> Option<u32> {
    let word = |at: usize| {
        bytes
            .get(at..at + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    let mut at = 0;
    let mut best: Option<u32> = None;
    while let Some(header_version) = word(at) {
        if header_version != 1 {
            break;
        }
        let revision = word(at + 4)?;
        best = Some(best.map_or(revision, |b: u32| b.max(revision)));
        let total = match word(at + 32)? {
            0 => 2048,
            n => n as usize,
        };
        if total < 48 {
            break;
        }
        at += total;
    }
    best
}

fn idle_states(root: &SystemRoot, cpu: u32) -> Vec<IdleState> {
    let base = format!("/sys/devices/system/cpu/cpu{cpu}/cpuidle");
    let mut states: Vec<(u32, IdleState)> = std::fs::read_dir(root.path(&base))
        .map(|dir| {
            dir.flatten()
                .filter_map(|e| {
                    e.file_name()
                        .to_str()?
                        .strip_prefix("state")?
                        .parse::<u32>()
                        .ok()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
        .into_iter()
        .filter_map(|n| {
            let at = |f: &str| format!("{base}/state{n}/{f}");
            Some((
                n,
                IdleState {
                    name: text(root, &at("name"))?,
                    description: text(root, &at("desc")),
                    latency_us: number(root, &at("latency")),
                    entries: number(root, &at("usage")),
                    time_us: number(root, &at("time")),
                    disabled: text(root, &at("disable")).as_deref() == Some("1"),
                },
            ))
        })
        .collect();
    states.sort_by_key(|(n, _)| *n);
    states.into_iter().map(|(_, s)| s).collect()
}

fn core(root: &SystemRoot, cpu: u32) -> Core {
    let at = |f: &str| format!("/sys/devices/system/cpu/cpu{cpu}/{f}");
    Core {
        cpu,
        now_khz: number(root, &at("cpufreq/scaling_cur_freq")),
        min_khz: number(root, &at("cpufreq/scaling_min_freq")),
        max_khz: number(root, &at("cpufreq/scaling_max_freq")),
        hardware_max_khz: number(root, &at("cpufreq/cpuinfo_max_freq")),
        governor: text(root, &at("cpufreq/scaling_governor")),
        energy_preference: text(root, &at("cpufreq/energy_performance_preference")),
        highest_perf: number(root, &at("acpi_cppc/highest_perf")),
        throttle_count: number(root, &at("thermal_throttle/core_throttle_count")),
        throttle_max_ms: number(root, &at("thermal_throttle/core_throttle_max_time_ms")),
        idle: idle_states(root, cpu),
    }
}

fn caches(root: &SystemRoot, cpus: &[u32]) -> Vec<Cache> {
    let mut out: Vec<Cache> = Vec::new();
    for cpu in cpus {
        let base = format!("/sys/devices/system/cpu/cpu{cpu}/cache");
        let mut indexes: Vec<String> = std::fs::read_dir(root.path(&base))
            .map(|d| {
                d.flatten()
                    .filter_map(|e| {
                        e.file_name()
                            .to_str()
                            .filter(|n| n.starts_with("index"))
                            .map(str::to_owned)
                    })
                    .collect()
            })
            .unwrap_or_default();
        indexes.sort();
        for index in indexes {
            let at = |f: &str| format!("{base}/{index}/{f}");
            let (Some(level), Some(kind), Some(bytes), Some(shared)) = (
                number(root, &at("level")),
                text(root, &at("type")),
                text(root, &at("size")).as_deref().and_then(cache_bytes),
                text(root, &at("shared_cpu_list")),
            ) else {
                continue;
            };
            if out
                .iter()
                .any(|c| c.level == level as u32 && c.kind == kind && c.shared_cpus == shared)
            {
                continue;
            }
            out.push(Cache {
                level: level as u32,
                kind,
                bytes,
                copies: cpu_list(&shared).len(),
                shared_cpus: shared,
            });
        }
    }
    out.sort_by(|a, b| (a.level, &a.kind, &a.shared_cpus).cmp(&(b.level, &b.kind, &b.shared_cpus)));
    out
}

fn power_limits(root: &SystemRoot) -> Vec<PowerLimit> {
    let mut zones: Vec<String> = std::fs::read_dir(root.path("/sys/class/powercap"))
        .map(|d| {
            d.flatten()
                .filter_map(|e| e.file_name().to_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    // Top-level zones only (`intel-rapl:0`), not their parts (`intel-rapl:0:0`).
    zones.retain(|z| z.matches(':').count() == 1);
    zones.sort();
    let mut out = Vec::new();
    for zone in zones {
        let base = format!("/sys/class/powercap/{zone}");
        let name = text(root, &format!("{base}/name")).unwrap_or_else(|| zone.clone());
        let label = if zone.starts_with("intel-rapl-mmio") {
            format!("{name} (MMIO)")
        } else {
            name
        };
        let enabled = text(root, &format!("{base}/enabled")).map(|e| e == "1");
        for n in 0..4 {
            let at = |f: &str| format!("{base}/constraint_{n}_{f}");
            let (Some(limit), Some(kind)) =
                (number(root, &at("power_limit_uw")), text(root, &at("name")))
            else {
                continue;
            };
            out.push(PowerLimit {
                zone: label.clone(),
                name: kind,
                watts: limit as f64 / 1e6,
                window_s: number(root, &at("time_window_us")).map(|w| w as f64 / 1e6),
                enabled,
            });
        }
    }
    out
}

fn flaws(root: &SystemRoot) -> Vec<Flaw> {
    let base = "/sys/devices/system/cpu/vulnerabilities";
    let mut names: Vec<String> = std::fs::read_dir(root.path(base))
        .map(|d| {
            d.flatten()
                .filter_map(|e| e.file_name().to_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
        .into_iter()
        .filter_map(|name| {
            let detail = text(root, &format!("{base}/{name}"))?;
            Some(Flaw {
                state: flaw_state(&detail),
                name,
                detail,
            })
        })
        .collect()
}

/// Reads everything for `cpu_detail`.
pub fn read(root: &SystemRoot) -> CpuDetail {
    let cpuinfo = root.read("/proc/cpuinfo").unwrap_or_default();
    let first = |key: &str| {
        cpuinfo.lines().find_map(|l| {
            l.split_once(':')
                .filter(|(k, _)| k.trim() == key)
                .map(|(_, v)| v.trim().to_owned())
        })
    };
    let family = first("cpu family").and_then(|v| v.parse().ok());
    let model = first("model").and_then(|v| v.parse().ok());
    let stepping = first("stepping").and_then(|v| v.parse().ok());
    let vendor = first("vendor_id");
    let microcode_loaded = first("microcode");
    let microcode_on_disk = match (vendor.as_deref(), family, model, stepping) {
        (Some("GenuineIntel"), Some(f), Some(m), Some(s)) => std::fs::read(root.path(&format!(
            "/lib/firmware/intel-ucode/{f:02x}-{m:02x}-{s:02x}"
        )))
        .ok()
        .and_then(|b| intel_microcode_revision(&b))
        .map(|r| format!("0x{r:x}")),
        _ => None,
    };
    let cpus: Vec<u32> = text(root, "/sys/devices/system/cpu/online")
        .map(|t| cpu_list(&t).into_iter().collect())
        .unwrap_or_default();
    let turbo = text(root, "/sys/devices/system/cpu/intel_pstate/no_turbo")
        .map(|v| v != "1")
        .or_else(|| text(root, "/sys/devices/system/cpu/cpufreq/boost").map(|v| v == "1"));
    let driver_mode = text(root, "/sys/devices/system/cpu/intel_pstate/status")
        .or_else(|| text(root, "/sys/devices/system/cpu/amd_pstate/status"));
    let at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    CpuDetail {
        at,
        driver: text(root, "/sys/devices/system/cpu/cpu0/cpufreq/scaling_driver"),
        driver_mode,
        turbo,
        vendor,
        family,
        model,
        stepping,
        microcode_on_disk,
        microcode_loaded,
        package_throttle_count: number(
            root,
            "/sys/devices/system/cpu/cpu0/thermal_throttle/package_throttle_count",
        ),
        package_throttle_max_ms: number(
            root,
            "/sys/devices/system/cpu/cpu0/thermal_throttle/package_throttle_max_time_ms",
        ),
        caches: caches(root, &cpus),
        cores: cpus.iter().map(|&c| core(root, c)).collect(),
        power_limits: power_limits(root),
        flaws: flaws(root),
        interrupts: root
            .read("/proc/interrupts")
            .map(|t| parse_interrupts(&t))
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn interrupts_are_summed_over_cpus() {
        let text = "           CPU0       CPU1\n  0:         10          0   IO-APIC   2-edge      timer\n 147:     500       700  IR-PCI-MSI-0000:00:14.3    0-edge      iwlwifi\nLOC:    9000       8000   Local timer interrupts\nERR:          0\n";
        let irqs = parse_interrupts(text);
        assert_eq!(irqs.len(), 4);
        assert_eq!(irqs[1].irq, "147");
        assert_eq!(irqs[1].count, 1200);
        assert!(irqs[1].device.ends_with("iwlwifi"));
        assert_eq!(irqs[2].device, "Local timer interrupts");
        assert_eq!(irqs[3].count, 0);
    }

    #[test]
    fn microcode_revision_is_the_highest_in_the_file() {
        let mut blob = Vec::new();
        for revision in [0x4121u32, 0x4123] {
            let mut header = vec![0u8; 64];
            header[0..4].copy_from_slice(&1u32.to_le_bytes());
            header[4..8].copy_from_slice(&revision.to_le_bytes());
            header[32..36].copy_from_slice(&64u32.to_le_bytes());
            blob.extend(header);
        }
        assert_eq!(intel_microcode_revision(&blob), Some(0x4123));
        assert_eq!(intel_microcode_revision(&[0u8; 10]), None);
        assert_eq!(intel_microcode_revision(&[]), None);
    }

    #[test]
    fn reads_a_laptop_and_leaves_out_what_it_does_not_report() {
        let cpu = "/sys/devices/system/cpu";
        let root = fixture_root(
            "cpu-detail",
            &[
                (
                    "/proc/cpuinfo",
                    "processor\t: 0\nvendor_id\t: GenuineIntel\ncpu family\t: 6\nmodel\t\t: 186\nstepping\t: 3\nmicrocode\t: 0x4121\n",
                ),
                (&format!("{cpu}/online"), "0-1\n"),
                (&format!("{cpu}/intel_pstate/no_turbo"), "0\n"),
                (&format!("{cpu}/intel_pstate/status"), "active\n"),
                (
                    &format!("{cpu}/cpu0/cpufreq/scaling_driver"),
                    "intel_pstate\n",
                ),
                (&format!("{cpu}/cpu0/cpufreq/scaling_cur_freq"), "1800000\n"),
                (&format!("{cpu}/cpu0/cpufreq/scaling_max_freq"), "4600000\n"),
                (
                    &format!("{cpu}/cpu0/cpufreq/scaling_governor"),
                    "powersave\n",
                ),
                (
                    &format!("{cpu}/cpu0/cpufreq/energy_performance_preference"),
                    "balance_power\n",
                ),
                (&format!("{cpu}/cpu0/acpi_cppc/highest_perf"), "57\n"),
                (
                    &format!("{cpu}/cpu0/thermal_throttle/core_throttle_count"),
                    "12\n",
                ),
                (
                    &format!("{cpu}/cpu0/thermal_throttle/package_throttle_count"),
                    "3\n",
                ),
                (&format!("{cpu}/cpu0/cpuidle/state0/name"), "POLL\n"),
                (&format!("{cpu}/cpu0/cpuidle/state0/time"), "100\n"),
                (&format!("{cpu}/cpu0/cpuidle/state2/name"), "C10\n"),
                (&format!("{cpu}/cpu0/cpuidle/state2/time"), "90000\n"),
                (&format!("{cpu}/cpu0/cpuidle/state2/latency"), "890\n"),
                (&format!("{cpu}/cpu0/cpuidle/state2/disable"), "0\n"),
                (&format!("{cpu}/cpu0/cache/index0/level"), "1\n"),
                (&format!("{cpu}/cpu0/cache/index0/type"), "Data\n"),
                (&format!("{cpu}/cpu0/cache/index0/size"), "48K\n"),
                (&format!("{cpu}/cpu0/cache/index0/shared_cpu_list"), "0-1\n"),
                (&format!("{cpu}/cpu1/cache/index0/level"), "1\n"),
                (&format!("{cpu}/cpu1/cache/index0/type"), "Data\n"),
                (&format!("{cpu}/cpu1/cache/index0/size"), "48K\n"),
                (&format!("{cpu}/cpu1/cache/index0/shared_cpu_list"), "0-1\n"),
                (&format!("{cpu}/cpu0/cache/index3/level"), "3\n"),
                (&format!("{cpu}/cpu0/cache/index3/type"), "Unified\n"),
                (&format!("{cpu}/cpu0/cache/index3/size"), "12M\n"),
                (&format!("{cpu}/cpu0/cache/index3/shared_cpu_list"), "0-1\n"),
                (&format!("{cpu}/vulnerabilities/meltdown"), "Not affected\n"),
                (
                    &format!("{cpu}/vulnerabilities/retbleed"),
                    "Vulnerable: no microcode\n",
                ),
                (
                    &format!("{cpu}/vulnerabilities/spectre_v2"),
                    "Mitigation: Enhanced / Automatic IBRS\n",
                ),
                ("/sys/class/powercap/intel-rapl:0/name", "package-0\n"),
                ("/sys/class/powercap/intel-rapl:0/enabled", "1\n"),
                (
                    "/sys/class/powercap/intel-rapl:0/constraint_0_name",
                    "long_term\n",
                ),
                (
                    "/sys/class/powercap/intel-rapl:0/constraint_0_power_limit_uw",
                    "28000000\n",
                ),
                (
                    "/sys/class/powercap/intel-rapl:0/constraint_0_time_window_us",
                    "27983872\n",
                ),
                ("/sys/class/powercap/intel-rapl:0:0/name", "core\n"),
                (
                    "/sys/class/powercap/intel-rapl:0:0/constraint_0_name",
                    "long_term\n",
                ),
                (
                    "/sys/class/powercap/intel-rapl:0:0/constraint_0_power_limit_uw",
                    "0\n",
                ),
            ],
        );
        let d = read(&root);
        assert_eq!(d.driver.as_deref(), Some("intel_pstate"));
        assert_eq!(d.turbo, Some(true));
        assert_eq!(
            (d.family, d.model, d.stepping),
            (Some(6), Some(186), Some(3))
        );
        assert_eq!(
            d.microcode_on_disk, None,
            "no microcode file in the fixture"
        );
        assert_eq!(d.cores.len(), 2);
        let c0 = &d.cores[0];
        assert_eq!(c0.now_khz, Some(1_800_000));
        assert_eq!(c0.energy_preference.as_deref(), Some("balance_power"));
        assert_eq!(c0.throttle_count, Some(12));
        assert_eq!(
            c0.idle.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            ["POLL", "C10"]
        );
        assert_eq!(c0.idle[1].latency_us, Some(890));
        assert!(d.cores[1].now_khz.is_none() && d.cores[1].idle.is_empty());
        assert_eq!(d.package_throttle_count, Some(3));
        assert_eq!(d.caches.len(), 2, "the shared L1 is listed once");
        assert_eq!(d.caches[1].bytes, 12 * 1024 * 1024);
        assert_eq!(d.caches[0].copies, 2);
        assert_eq!(d.power_limits.len(), 1, "parts of a zone are left out");
        assert_eq!(d.power_limits[0].watts, 28.0);
        assert_eq!(
            d.flaws.iter().map(|f| f.state).collect::<Vec<_>>(),
            ["not_affected", "vulnerable", "mitigated"]
        );
    }
}
