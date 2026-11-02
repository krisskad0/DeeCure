// SPDX-License-Identifier: Apache-2.0

//! `gpu.busy` per graphics card (subject `card1`): the share of time it was working, in
//! percent. AMD reports it (`gpu_busy_percent`). Intel reports the time spent in its power
//! saving state instead (RC6 on `i915`, `gtidle` on `xe`), so busy is the rest of the time
//! between two readings. NVIDIA's own driver reports neither without running `nvidia-smi`,
//! which can wake a sleeping card, so it is left out.

use std::collections::HashMap;
use std::io;

use deecure_core::{Timestamp, Unit};

use super::{Collector, Sample, SystemRoot, metric, read_trimmed};

/// Where Intel drivers count time in power saving, in ms, under `/sys/class/drm/<card>`.
const IDLE_FILES: &[&str] = &[
    "gt/gt0/rc6_residency_ms",
    "device/tile0/gt0/gtidle/idle_residency_ms",
];

/// Reads `gpu.busy` for every graphics card.
#[derive(Debug, Default)]
pub struct GpuBusy {
    /// The last (time, ms in power saving) per Intel card.
    idle: HashMap<String, (Timestamp, f64)>,
}

/// Graphics cards: `card1`, not its screen connectors (`card1-eDP-1`).
fn cards(root: &SystemRoot) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(root.path("/sys/class/drm"))
        .map(|d| {
            d.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| {
                    n.len() > 4
                        && n.starts_with("card")
                        && n[4..].chars().all(|c| c.is_ascii_digit())
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

impl Collector for GpuBusy {
    fn name(&self) -> &'static str {
        "gpu"
    }

    fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let mut sample = Sample::default();
        let cards = cards(root);
        self.idle.retain(|c, _| cards.contains(c));
        for card in cards {
            let number = |file: &str| {
                read_trimmed(&root.path(&format!("/sys/class/drm/{card}/{file}")))
                    .and_then(|v| v.parse::<f64>().ok())
            };
            let busy = if let Some(b) = number("device/gpu_busy_percent") {
                Some(b)
            } else if let Some(idle) = IDLE_FILES.iter().find_map(|f| number(f)) {
                let before = self.idle.insert(card.clone(), (now, idle));
                before.and_then(|(at, was)| {
                    let wall = (now.as_millis() - at.as_millis()) as f64;
                    // A counter that went back (a driver reload) gives no reading this time.
                    (wall > 0.0 && idle >= was).then(|| 100.0 - (idle - was) * 100.0 / wall)
                })
            } else {
                None
            };
            if let Some(b) = busy {
                sample.metrics.push(metric(
                    "gpu.busy",
                    Some(&card),
                    b.clamp(0.0, 100.0),
                    Unit::Percent,
                    now,
                ));
            }
        }
        Ok(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::{fixture_root, value};

    #[test]
    fn reads_amd_busy_and_works_out_intel_busy_from_power_saving_time() {
        let root = fixture_root(
            "gpu",
            &[
                ("/sys/class/drm/card0/device/gpu_busy_percent", "37\n"),
                ("/sys/class/drm/card1/gt/gt0/rc6_residency_ms", "10000\n"),
                ("/sys/class/drm/card1-eDP-1/status", "connected\n"),
                ("/sys/class/drm/card2/device/vendor", "0x10de\n"),
            ],
        );
        let mut g = GpuBusy::default();
        let s = g.collect(&root, Timestamp(0)).unwrap();
        assert_eq!(value(&s, "gpu.busy", Some("card0")), Some(37.0));
        assert_eq!(
            value(&s, "gpu.busy", Some("card1")),
            None,
            "Intel needs two readings"
        );
        assert_eq!(
            value(&s, "gpu.busy", Some("card2")),
            None,
            "NVIDIA reports nothing"
        );
        // 60 s later, 45 s of it in power saving: busy 25%.
        std::fs::write(
            root.path("/sys/class/drm/card1/gt/gt0/rc6_residency_ms"),
            "55000\n",
        )
        .unwrap();
        let s = g.collect(&root, Timestamp(60_000)).unwrap();
        assert_eq!(value(&s, "gpu.busy", Some("card1")), Some(25.0));
        assert_eq!(s.metrics.iter().filter(|m| m.name == "gpu.busy").count(), 2);
        // A counter that went back gives no reading, not a wrong one.
        std::fs::write(
            root.path("/sys/class/drm/card1/gt/gt0/rc6_residency_ms"),
            "5\n",
        )
        .unwrap();
        let s = g.collect(&root, Timestamp(120_000)).unwrap();
        assert_eq!(value(&s, "gpu.busy", Some("card1")), None);
    }

    #[test]
    fn no_graphics_card_means_no_reading() {
        let root = fixture_root("gpu-none", &[("/proc/stat", "cpu 1 2 3\n")]);
        let s = GpuBusy::default().collect(&root, Timestamp(0)).unwrap();
        assert!(s.metrics.is_empty());
    }
}
