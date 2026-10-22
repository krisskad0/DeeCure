// SPDX-License-Identifier: Apache-2.0

//! Graphics cards from `/sys/class/drm`, named with the PCI ID list.

use serde::Serialize;

use crate::collect::SystemRoot;

/// Where Ubuntu keeps the PCI ID list (packages `pci.ids` and `hwdata`).
pub(crate) const PCI_IDS: &[&str] = &["/usr/share/misc/pci.ids", "/usr/share/hwdata/pci.ids"];

/// One graphics card.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Gpu {
    /// The DRM card, such as `card1`.
    pub card: String,
    /// Maker, such as "Intel Corporation".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    /// Model, such as "Raptor Lake-P [Iris Xe Graphics]".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// PCI ID, such as `8086:a7a1`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pci_id: Option<String>,
    /// Kernel driver, such as `i915`, `amdgpu`, or `nvidia`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    /// The card the computer started with (it shows the boot screen).
    pub boot_vga: bool,
    /// Busy time in percent. Only some drivers (such as `amdgpu`) report it without admin rights.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub busy_percent: Option<f64>,
    /// Own video memory, in bytes. Missing for cards that share system memory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vram_total: Option<u64>,
    /// Video memory in use, in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vram_used: Option<u64>,
    /// Temperature in °C, when the driver reports one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
}

/// Reads every graphics card.
pub fn read(root: &SystemRoot) -> Vec<Gpu> {
    let ids = PCI_IDS
        .iter()
        .find_map(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    let mut cards: Vec<Gpu> = std::fs::read_dir(root.path("/sys/class/drm"))
        .map(|dir| {
            dir.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                // `card1` is a card; `card1-eDP-1` is one of its screen connectors.
                .filter(|n| {
                    n.starts_with("card")
                        && n[4..].chars().all(|c| c.is_ascii_digit())
                        && n.len() > 4
                })
                .map(|card| gpu(root, card, &ids))
                .collect()
        })
        .unwrap_or_default();
    cards.sort_by(|a, b| b.boot_vga.cmp(&a.boot_vga).then(a.card.cmp(&b.card)));
    cards
}

fn gpu(root: &SystemRoot, card: String, ids: &str) -> Gpu {
    let dev = format!("/sys/class/drm/{card}/device");
    let text = |file: &str| {
        root.read(&format!("{dev}/{file}"))
            .ok()
            .map(|s| s.trim().to_owned())
    };
    let hex = |file: &str| text(file).map(|v| v.trim_start_matches("0x").to_lowercase());
    let (vendor_id, device_id) = (hex("vendor"), hex("device"));
    let (vendor, model) = match (&vendor_id, &device_id) {
        (Some(v), Some(d)) => pci_names(ids, v, d),
        _ => (None, None),
    };
    let driver = std::fs::read_link(root.path(&format!("{dev}/driver")))
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()));
    let temperature = std::fs::read_dir(root.path(&format!("{dev}/hwmon")))
        .ok()
        .and_then(|mut d| d.next())
        .and_then(|e| e.ok())
        .and_then(|e| std::fs::read_to_string(e.path().join("temp1_input")).ok())
        .and_then(|t| t.trim().parse::<f64>().ok())
        .map(|milli| milli / 1000.0);
    Gpu {
        pci_id: vendor_id.zip(device_id).map(|(v, d)| format!("{v}:{d}")),
        vendor,
        model,
        driver,
        boot_vga: text("boot_vga").as_deref() == Some("1"),
        busy_percent: text("gpu_busy_percent").and_then(|v| v.parse().ok()),
        vram_total: text("mem_info_vram_total").and_then(|v| v.parse().ok()),
        vram_used: text("mem_info_vram_used").and_then(|v| v.parse().ok()),
        temperature,
        card,
    }
}

/// Looks up the vendor and device names in the PCI ID list.
fn pci_names(ids: &str, vendor: &str, device: &str) -> (Option<String>, Option<String>) {
    let mut vendor_name = None;
    for line in ids.lines() {
        if vendor_name.is_none() {
            if let Some(name) = line.strip_prefix(vendor).and_then(|r| r.strip_prefix("  ")) {
                vendor_name = Some(name.to_owned());
            }
            continue;
        }
        // Devices of this vendor are indented by one tab; the next vendor is not indented.
        if !line.starts_with('\t') && !line.starts_with('#') && !line.is_empty() {
            break;
        }
        if let Some(name) = line
            .strip_prefix('\t')
            .and_then(|r| r.strip_prefix(device))
            .and_then(|r| r.strip_prefix("  "))
        {
            return (vendor_name, Some(name.to_owned()));
        }
    }
    (vendor_name, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDS: &str = "# comment
1002  Advanced Micro Devices, Inc. [AMD/ATI]
\t1638  Cezanne [Radeon Vega Series / Radeon Vega Mobile Series]
8086  Intel Corporation
\t0001  Something else
\t\ta7a1 1028 0c97  Subsystem line
\ta7a1  Raptor Lake-P [Iris Xe Graphics]
8087  Intel Corporation
\ta7a2  Wrong vendor
";

    #[test]
    fn finds_names_in_the_pci_list() {
        assert_eq!(
            pci_names(IDS, "8086", "a7a1"),
            (
                Some("Intel Corporation".into()),
                Some("Raptor Lake-P [Iris Xe Graphics]".into())
            )
        );
        assert_eq!(
            pci_names(IDS, "8086", "a7a2"),
            (Some("Intel Corporation".into()), None)
        );
        assert_eq!(pci_names(IDS, "abcd", "0001"), (None, None));
    }

    #[test]
    fn reads_cards_but_not_connectors() {
        let root = crate::collect::test_support::fixture_root(
            "gpu",
            &[
                ("/sys/class/drm/card1/device/vendor", "0x1002\n"),
                ("/sys/class/drm/card1/device/device", "0x1638\n"),
                ("/sys/class/drm/card1/device/boot_vga", "1\n"),
                ("/sys/class/drm/card1/device/gpu_busy_percent", "7\n"),
                (
                    "/sys/class/drm/card1/device/hwmon/hwmon3/temp1_input",
                    "45000\n",
                ),
                ("/sys/class/drm/card1-eDP-1/status", "connected\n"),
            ],
        );
        let cards = read(&root);
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].busy_percent, Some(7.0));
        assert_eq!(cards[0].temperature, Some(45.0));
        assert_eq!(cards[0].pci_id.as_deref(), Some("1002:1638"));
        assert!(cards[0].boot_vga);
    }
}
