// SPDX-License-Identifier: Apache-2.0

//! PCI devices in depth for the Expert level (docs/01-product/expert-coverage.md, sections 3,
//! 7b, and 19): each device's PCIe link speed and lanes against the most it supports, the link
//! errors it reported (AER), and its power saving. From `/sys/bus/pci/devices`, no admin rights.

use std::collections::BTreeSet;

use serde::Serialize;

use super::gpu::PCI_IDS;
use super::hardware::{driver_of, pci_class, pci_lookup};
use crate::collect::SystemRoot;

/// Link errors one device reported since start, by kind.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct LinkErrors {
    /// Fixed by the hardware (a retry).
    pub corrected: u64,
    /// Not fixed, but the device kept working.
    pub nonfatal: u64,
    /// The link failed.
    pub fatal: u64,
}

/// One PCI device.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PciDevice {
    /// The address, such as `0000:01:00.0`.
    pub address: String,
    /// In plain words, such as `Storage` or `Display`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    /// Link speed now and the most it supports, such as `16.0 GT/s PCIe`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_speed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_link_speed: Option<String>,
    /// Lanes now and the most it supports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_link_width: Option<u32>,
    /// The most the slot (the bridge above the device) supports. The link cannot be faster than
    /// the slower of the two, so a device below its own maximum is not a fault by itself.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot_max_link_speed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot_max_link_width: Option<u32>,
    /// Missing when the device or kernel does not report link errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<LinkErrors>,
    /// Its path under `/sys/devices`, as the helper's power controls take it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sys_path: Option<String>,
    /// Power saving: `auto` (on) or `on` (always awake).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_control: Option<String>,
    /// `active`, `suspended`, or `unsupported`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_state: Option<String>,
}

fn text(root: &SystemRoot, path: &str) -> Option<String> {
    root.read(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty() && s != "Unknown")
}

/// The `TOTAL_ERR_*` line of an `aer_dev_*` file, or the sum of its counters.
pub fn aer_total(text: &str) -> u64 {
    let mut sum = 0;
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let (Some(name), Some(value)) = (
            parts.next(),
            parts.next().and_then(|v| v.parse::<u64>().ok()),
        ) else {
            continue;
        };
        if name.starts_with("TOTAL_ERR") {
            return value;
        }
        sum += value;
    }
    sum
}

/// Reads every PCI device.
pub fn read(root: &SystemRoot) -> Vec<PciDevice> {
    let base = "/sys/bus/pci/devices";
    let mut addresses: Vec<String> = std::fs::read_dir(root.path(base))
        .map(|d| {
            d.flatten()
                .filter_map(|e| e.file_name().to_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    addresses.sort();
    let ids: Vec<(String, (String, String))> = addresses
        .iter()
        .filter_map(|a| {
            let id = |f: &str| {
                text(root, &format!("{base}/{a}/{f}"))
                    .map(|v| v.trim_start_matches("0x").to_owned())
            };
            Some((a.clone(), (id("vendor")?, id("device")?)))
        })
        .collect();
    let wanted: BTreeSet<(String, String)> = ids.iter().map(|(_, id)| id.clone()).collect();
    let names = PCI_IDS
        .iter()
        .find_map(|p| std::fs::read_to_string(p).ok())
        .map(|list| pci_lookup(&list, &wanted))
        .unwrap_or_default();
    addresses
        .into_iter()
        .map(|address| {
            let at = |f: &str| format!("{base}/{address}/{f}");
            let errors = match (
                text(root, &at("aer_dev_correctable")),
                text(root, &at("aer_dev_nonfatal")),
                text(root, &at("aer_dev_fatal")),
            ) {
                (None, None, None) => None,
                (c, n, f) => Some(LinkErrors {
                    corrected: c.map(|t| aer_total(&t)).unwrap_or(0),
                    nonfatal: n.map(|t| aer_total(&t)).unwrap_or(0),
                    fatal: f.map(|t| aer_total(&t)).unwrap_or(0),
                }),
            };
            // The bridge above: the parent folder of the device's real path, when it is a PCI address.
            let bridge = std::fs::canonicalize(root.path(&format!("{base}/{address}")))
                .ok()
                .and_then(|p| p.parent().map(|q| q.to_path_buf()))
                .filter(|q| {
                    q.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.len() == 12 && n.chars().nth(4) == Some(':'))
                });
            let from_bridge = |f: &str| {
                bridge
                    .as_ref()
                    .and_then(|b| std::fs::read_to_string(b.join(f)).ok())
                    .map(|v| v.trim().to_owned())
                    .filter(|v| !v.is_empty() && v != "Unknown")
            };
            let (vendor, model) = ids
                .iter()
                .find(|(a, _)| a == &address)
                .and_then(|(_, id)| names.get(id).cloned())
                .unwrap_or((None, None));
            PciDevice {
                class: text(root, &at("class"))
                    .and_then(|c| pci_class(&c))
                    .map(str::to_owned),
                vendor,
                model,
                driver: driver_of(&root.path(&format!("{base}/{address}"))),
                link_speed: text(root, &at("current_link_speed")),
                max_link_speed: text(root, &at("max_link_speed")),
                link_width: text(root, &at("current_link_width")).and_then(|w| w.parse().ok()),
                max_link_width: text(root, &at("max_link_width")).and_then(|w| w.parse().ok()),
                slot_max_link_speed: from_bridge("max_link_speed"),
                slot_max_link_width: from_bridge("max_link_width").and_then(|w| w.parse().ok()),
                errors,
                sys_path: std::fs::canonicalize(root.path(&format!("{base}/{address}")))
                    .ok()
                    .zip(std::fs::canonicalize(root.path("/sys/devices")).ok())
                    .and_then(|(real, devices)| {
                        real.strip_prefix(&devices)
                            .ok()
                            .map(|p| p.to_string_lossy().into_owned())
                    }),
                power_control: text(root, &at("power/control")),
                power_state: text(root, &at("power/runtime_status")),
                address,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn aer_files_give_their_total() {
        assert_eq!(aer_total("RxErr 2\nBadTLP 1\nTOTAL_ERR_COR 3\n"), 3);
        assert_eq!(aer_total("RxErr 2\nBadTLP 1\n"), 3);
        assert_eq!(aer_total(""), 0);
    }

    #[test]
    fn reads_links_errors_and_power() {
        let d = "/sys/bus/pci/devices/0000:01:00.0";
        let root = fixture_root(
            "pci-detail",
            &[
                (&format!("{d}/vendor"), "0x144d\n"),
                (&format!("{d}/device"), "0xa80a\n"),
                (&format!("{d}/class"), "0x010802\n"),
                (&format!("{d}/current_link_speed"), "8.0 GT/s PCIe\n"),
                (&format!("{d}/max_link_speed"), "16.0 GT/s PCIe\n"),
                (&format!("{d}/current_link_width"), "4\n"),
                (&format!("{d}/max_link_width"), "4\n"),
                (
                    &format!("{d}/aer_dev_correctable"),
                    "RxErr 5\nTOTAL_ERR_COR 5\n",
                ),
                (&format!("{d}/aer_dev_fatal"), "TOTAL_ERR_FATAL 0\n"),
                (&format!("{d}/power/control"), "on\n"),
                (&format!("{d}/power/runtime_status"), "active\n"),
                ("/sys/bus/pci/devices/0000:00:00.0/vendor", "0x8086\n"),
                ("/sys/bus/pci/devices/0000:00:00.0/device", "0xa706\n"),
                (
                    "/sys/bus/pci/devices/0000:00:00.0/current_link_speed",
                    "Unknown\n",
                ),
            ],
        );
        let devices = read(&root);
        assert_eq!(devices.len(), 2);
        let host = &devices[0];
        assert_eq!(
            (host.link_speed.as_ref(), host.errors.as_ref()),
            (None, None),
            "Unknown is left out"
        );
        let ssd = &devices[1];
        assert_eq!(ssd.class.as_deref(), Some("Storage"));
        assert_eq!(ssd.link_speed.as_deref(), Some("8.0 GT/s PCIe"));
        assert_eq!(ssd.max_link_speed.as_deref(), Some("16.0 GT/s PCIe"));
        assert_eq!(
            ssd.errors,
            Some(LinkErrors {
                corrected: 5,
                nonfatal: 0,
                fatal: 0
            })
        );
        assert_eq!(ssd.power_control.as_deref(), Some("on"));
    }

    #[test]
    fn the_slot_maximum_comes_from_the_bridge_above() {
        let root = fixture_root(
            "pci-slot",
            &[
                (
                    "/sys/devices/pci0000:00/0000:00:06.0/max_link_speed",
                    "8.0 GT/s PCIe\n",
                ),
                ("/sys/devices/pci0000:00/0000:00:06.0/max_link_width", "4\n"),
                (
                    "/sys/devices/pci0000:00/0000:00:06.0/0000:01:00.0/max_link_speed",
                    "16.0 GT/s PCIe\n",
                ),
            ],
        );
        std::fs::create_dir_all(root.path("/sys/bus/pci/devices")).unwrap();
        std::os::unix::fs::symlink(
            root.path("/sys/devices/pci0000:00/0000:00:06.0/0000:01:00.0"),
            root.path("/sys/bus/pci/devices/0000:01:00.0"),
        )
        .unwrap();
        let d = read(&root);
        let ssd = d.iter().find(|p| p.address == "0000:01:00.0").unwrap();
        assert_eq!(ssd.max_link_speed.as_deref(), Some("16.0 GT/s PCIe"));
        assert_eq!(ssd.slot_max_link_speed.as_deref(), Some("8.0 GT/s PCIe"));
        assert_eq!(ssd.slot_max_link_width, Some(4));
    }
}
