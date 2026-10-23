// SPDX-License-Identifier: Apache-2.0

//! Hardware errors in one place for the Expert level (docs/01-product/expert-coverage.md,
//! section 19): what the kernel logged in the last 7 days about CPU and memory faults, firmware
//! that failed to load or crashed, USB and disk link errors, PCIe errors, graphics hangs, and
//! heat events; driver crash dumps; and the drivers Ubuntu recommends. PCIe error counters and
//! memory error counts come from `pci_detail` and `memory_detail`. No admin rights.

use std::time::Duration;

use serde::Serialize;

use super::command;
use crate::collect::SystemRoot;

const LIMIT: Duration = Duration::from_secs(20);

/// What `journalctl --grep` looks for (lowercase, so the match ignores case).
const PATTERN: &str = "mce:|machine check|hardware error|direct firmware load|firmware crash|fw crash|firmware has crashed|device descriptor read|not accepting address|unable to enumerate|cannot reset|disabled by hub|over-current|hard resetting link|failed command|exception emask|nvme.*(timeout|reset)|i/o error|aer:|gpu hang|gpu reset|ring .*timeout|temperature above threshold|clock throttled";

/// The kinds, in the order the app shows them, with the words that put a line in each.
const KINDS: &[(&str, &[&str])] = &[
    ("cpu_memory", &["mce:", "machine check", "hardware error"]),
    ("firmware_missing", &["direct firmware load"]),
    (
        "firmware_crash",
        &["firmware crash", "fw crash", "firmware has crashed"],
    ),
    (
        "usb",
        &[
            "device descriptor read",
            "not accepting address",
            "unable to enumerate",
            "cannot reset",
            "disabled by hub",
            "over-current",
        ],
    ),
    (
        "disk",
        &[
            "hard resetting link",
            "failed command",
            "exception emask",
            "nvme",
            "i/o error",
        ],
    ),
    ("pcie", &["aer:"]),
    ("gpu", &["gpu hang", "gpu reset", "timeout"]),
    ("heat", &["temperature above threshold", "clock throttled"]),
];

/// One kind of hardware error.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ErrorKind {
    /// One of the names in `KINDS`.
    pub kind: &'static str,
    pub count: usize,
    /// When the last one happened, in ms since 1970.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_at: Option<u64>,
    /// Up to three different recent messages, newest first.
    pub examples: Vec<String>,
}

/// A device and the drivers Ubuntu offers for it.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct DriverOffer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Package names; `recommended` marks Ubuntu's pick.
    pub drivers: Vec<(String, bool)>,
}

/// What `hw_errors` answers.
#[derive(Debug, Default, Serialize)]
pub struct HwErrors {
    /// Missing when the kernel log cannot be read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kinds: Option<Vec<ErrorKind>>,
    /// Devices that left a crash dump (`devcoredump`) since start.
    pub crash_dumps: Vec<String>,
    /// Missing when `ubuntu-drivers` is not installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver_offers: Option<Vec<DriverOffer>>,
}

/// Sorts `journalctl -k -o short-unix` lines into kinds. A line goes into the first kind it fits.
pub fn classify(text: &str) -> Vec<ErrorKind> {
    let mut out: Vec<ErrorKind> = KINDS
        .iter()
        .map(|(kind, _)| ErrorKind {
            kind,
            ..Default::default()
        })
        .collect();
    for line in text.lines() {
        let Some((stamp, rest)) = line.split_once(' ') else {
            continue;
        };
        let Ok(seconds) = stamp.parse::<f64>() else {
            continue;
        };
        // "host kernel: message"
        let message = rest
            .split_once("kernel: ")
            .map(|(_, m)| m)
            .unwrap_or(rest)
            .trim();
        let lower = message.to_lowercase();
        let Some(i) = KINDS.iter().position(|(kind, words)| {
            words.iter().any(|w| lower.contains(w))
                // Only graphics timeouts are graphics hangs, and only disk-like I/O errors are disk errors.
                && (*kind != "gpu" || ["i915", "amdgpu", "nouveau", "xe ", "radeon", "gpu"].iter().any(|g| lower.contains(g)))
                && (*kind != "disk" || !lower.contains("usb "))
        }) else {
            continue;
        };
        let k = &mut out[i];
        k.count += 1;
        let at = (seconds * 1000.0) as u64;
        k.last_at = Some(k.last_at.map_or(at, |l| l.max(at)));
        if !k.examples.iter().any(|e| e == message) {
            k.examples.insert(0, message.to_owned());
            k.examples.truncate(3);
        }
    }
    out
}

/// Parses `ubuntu-drivers devices`: blocks that start with `== /sys/… ==`.
pub fn parse_ubuntu_drivers(text: &str) -> Vec<DriverOffer> {
    text.split("== ")
        .skip(1)
        .map(|block| {
            let get = |k: &str| {
                block.lines().find_map(|l| {
                    l.strip_prefix(k)
                        .and_then(|r| r.trim_start().strip_prefix(':'))
                        .map(|v| v.trim().to_owned())
                })
            };
            let drivers = block
                .lines()
                .filter_map(|l| l.strip_prefix("driver"))
                .filter_map(|r| r.trim_start().strip_prefix(':'))
                .map(|d| {
                    let name = d.split(" - ").next().unwrap_or(d).trim().to_owned();
                    (name, d.contains("recommended"))
                })
                .collect();
            DriverOffer {
                vendor: get("vendor"),
                model: get("model"),
                drivers,
            }
        })
        .filter(|o| !o.drivers.is_empty())
        .collect()
}

/// Reads `hw_errors`.
pub async fn read(root: &SystemRoot) -> HwErrors {
    let log = command::output_any(
        "journalctl",
        &[
            "-k",
            "--since",
            "-7d",
            "-o",
            "short-unix",
            "--no-pager",
            "--grep",
            PATTERN,
        ],
        LIMIT,
    )
    .await;
    let crash_dumps = std::fs::read_dir(root.path("/sys/class/devcoredump"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("devcd"))
        .filter_map(|e| std::fs::read_link(e.path().join("failing_device")).ok())
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    HwErrors {
        kinds: log.map(|t| classify(&t)),
        crash_dumps,
        driver_offers: command::output_any("ubuntu-drivers", &["devices"], LIMIT)
            .await
            .map(|t| parse_ubuntu_drivers(&t)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_lines_are_sorted_into_kinds() {
        let t = "-- Boot 1 --\n1700000000.5 host kernel: mce: [Hardware Error]: Machine check events logged\n1700000100.0 host kernel: iwlwifi 0000:00:14.3: Direct firmware load for iwlwifi-so-a0-gf-a0-89.ucode failed with error -2\n1700000200.0 host kernel: usb 3-2: device descriptor read/64, error -71\n1700000300.0 host kernel: usb 3-2: device descriptor read/64, error -71\n1700000400.0 host kernel: ata1.00: failed command: READ FPDMA QUEUED\n1700000500.0 host kernel: i915 0000:00:02.0: [drm] GPU HANG: ecode 12:1:85dffffb\n1700000600.0 host kernel: CPU0: Core temperature above threshold, cpu clock throttled\n1700000700.0 host kernel: pcieport 0000:00:1c.0: AER: Corrected error message received\n1700000800.0 host kernel: watchdog: timeout\n";
        let k = classify(t);
        let get = |name: &str| k.iter().find(|x| x.kind == name).unwrap();
        assert_eq!(get("cpu_memory").count, 1);
        assert_eq!(get("firmware_missing").count, 1);
        assert_eq!(get("usb").count, 2);
        assert_eq!(
            get("usb").examples.len(),
            1,
            "the same message is kept once"
        );
        assert_eq!(get("usb").last_at, Some(1_700_000_300_000));
        assert_eq!(get("disk").count, 1);
        assert_eq!(
            get("gpu").count,
            1,
            "a timeout that is not graphics is not a graphics hang"
        );
        assert_eq!(get("heat").count, 1);
        assert_eq!(get("pcie").count, 1);
        assert!(classify("-- No entries --\n").iter().all(|x| x.count == 0));
    }

    #[test]
    fn ubuntu_drivers_offers() {
        let t = "== /sys/devices/pci0000:00/0000:01:00.0 ==\nmodalias : pci:v000010DE\nvendor   : NVIDIA Corporation\nmodel    : GA107M [GeForce RTX 3050 Mobile]\ndriver   : nvidia-driver-535 - distro non-free recommended\ndriver   : nvidia-driver-470 - distro non-free\ndriver   : xserver-xorg-video-nouveau - distro free builtin\n";
        let o = parse_ubuntu_drivers(t);
        assert_eq!(o.len(), 1);
        assert_eq!(
            o[0].model.as_deref(),
            Some("GA107M [GeForce RTX 3050 Mobile]")
        );
        assert_eq!(o[0].drivers[0], ("nvidia-driver-535".into(), true));
        assert_eq!(o[0].drivers.len(), 3);
        assert!(parse_ubuntu_drivers("").is_empty());
    }
}
