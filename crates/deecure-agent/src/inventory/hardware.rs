// SPDX-License-Identifier: Apache-2.0

//! Hardware: machine and firmware, processor, memory, disks, and devices. Read from
//! `/sys` and `/proc`. Values only root can read (serial number, memory modules,
//! SMART data) are left out; the app says they need Full access.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::gpu::PCI_IDS;
use crate::collect::SystemRoot;

/// The machine and its platform firmware.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Machine {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    /// The friendly model name, such as "Latitude 3450" or "IdeaPad 5 14IAL7".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Laptop, Desktop, Tablet, …
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chassis: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bios_vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bios_version: Option<String>,
    /// As the firmware writes it, `MM/DD/YYYY`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bios_date: Option<String>,
    /// `UEFI` or `Legacy BIOS`.
    pub boot_mode: String,
    /// Missing when the firmware does not say (legacy BIOS).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secure_boot: Option<bool>,
    /// TPM version, such as "2.0". Missing when there is no TPM.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tpm: Option<String>,
}

/// One logical CPU.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LogicalCpu {
    /// Its number, as in `cpu.core` metrics.
    pub cpu: u32,
    /// The physical core it belongs to; matches `coretemp` "Core N" sensors.
    pub core_id: u32,
    /// `Performance` or `Efficient` on hybrid CPUs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Current clock speed, in megahertz.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mhz: Option<f64>,
}

/// The processor.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Processor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Physical cores.
    pub cores: usize,
    /// Performance and efficient cores, on hybrid CPUs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hybrid: Option<(usize, usize)>,
    pub threads: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_hz: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_hz: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l3_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub microcode: Option<String>,
    /// Plain names of notable features, such as "VT-x" and "AVX2".
    pub features: Vec<String>,
    pub cpus: Vec<LogicalCpu>,
}

/// One whole disk.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Disk {
    /// Such as `nvme0n1` or `sda`.
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// "SSD (NVMe)", "SSD", "Hard disk", or "USB drive".
    pub kind: String,
    /// Bytes.
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware: Option<String>,
    pub removable: bool,
}

/// One device on a bus.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Device {
    pub name: String,
    /// `USB`, `PCI`, or `Input`.
    pub bus: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    /// Kernel drivers in use; empty when none is bound.
    pub drivers: Vec<String>,
    /// Plain class, such as "Network" or "Audio".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    /// `vendor:device` IDs, for USB and PCI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// The `hardware` inventory.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct HardwareInfo {
    pub machine: Machine,
    pub processor: Processor,
    /// All memory, in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_total: Option<u64>,
    /// True when the kernel reports memory errors (EDAC), which needs ECC memory.
    pub ecc_reporting: bool,
    pub disks: Vec<Disk>,
    pub devices: Vec<Device>,
    /// Bluetooth adapters the kernel knows.
    pub bluetooth_adapters: usize,
}

fn text(root: &SystemRoot, path: &str) -> Option<String> {
    root.read(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty() && s != "Default string" && s != "To be filled by O.E.M.")
}

/// SMBIOS chassis types, in plain words.
fn chassis(code: u32) -> &'static str {
    match code {
        3 | 4 | 5 | 6 | 7 | 15 | 16 | 24 => "Desktop",
        8 | 9 | 10 | 14 => "Laptop",
        11 => "Handheld",
        13 => "All-in-one",
        17 | 23 | 28 | 29 => "Server",
        30 => "Tablet",
        31 => "Convertible",
        32 => "Detachable",
        35 | 36 => "Mini PC",
        _ => "Other",
    }
}

fn machine(root: &SystemRoot) -> Machine {
    let dmi = |f: &str| text(root, &format!("/sys/class/dmi/id/{f}"));
    // Lenovo keeps the friendly name in product_version; others in product_name.
    let model = match (dmi("sys_vendor").as_deref(), dmi("product_version")) {
        (Some("LENOVO"), Some(version)) => Some(version),
        _ => dmi("product_name"),
    };
    let efi = root.path("/sys/firmware/efi").exists();
    let secure_boot = std::fs::read_dir(root.path("/sys/firmware/efi/efivars"))
        .ok()
        .and_then(|d| {
            d.flatten()
                .find(|e| e.file_name().to_string_lossy().starts_with("SecureBoot-"))
        })
        // efivars files start with 4 attribute bytes; the 5th is the value.
        .and_then(|e| std::fs::read(e.path()).ok())
        .and_then(|bytes| bytes.get(4).map(|b| *b == 1));
    Machine {
        vendor: dmi("sys_vendor"),
        model,
        chassis: dmi("chassis_type")
            .and_then(|c| c.parse().ok())
            .map(|c| chassis(c).to_owned()),
        bios_vendor: dmi("bios_vendor"),
        bios_version: dmi("bios_version"),
        bios_date: dmi("bios_date"),
        boot_mode: if efi { "UEFI" } else { "Legacy BIOS" }.into(),
        secure_boot,
        tpm: text(root, "/sys/class/tpm/tpm0/tpm_version_major").map(|v| format!("{v}.0")),
    }
}

/// Reads a kernel CPU list such as `0-3,8`.
pub fn cpu_list(text: &str) -> BTreeSet<u32> {
    text.trim()
        .split(',')
        .filter(|p| !p.is_empty())
        .flat_map(|part| match part.split_once('-') {
            Some((a, b)) => match (a.parse::<u32>(), b.parse::<u32>()) {
                (Ok(a), Ok(b)) if a <= b => (a..=b).collect::<Vec<_>>(),
                _ => Vec::new(),
            },
            None => part.parse().ok().into_iter().collect(),
        })
        .collect()
}

const FEATURES: &[(&str, &str)] = &[
    ("vmx", "VT-x"),
    ("svm", "AMD-V"),
    ("aes", "AES-NI"),
    ("avx2", "AVX2"),
    ("avx512f", "AVX-512"),
    ("sha_ni", "SHA"),
    ("hypervisor", "Running in a VM"),
];

fn processor(root: &SystemRoot) -> Processor {
    let cpuinfo = root.read("/proc/cpuinfo").unwrap_or_default();
    let first = |key: &str| {
        cpuinfo.lines().find_map(|l| {
            l.strip_prefix(key)?
                .split_once(':')
                .map(|(_, v)| v.trim().to_owned())
        })
    };
    let mut cores = BTreeSet::new();
    let mut physical = String::new();
    for line in cpuinfo.lines() {
        if let Some((key, value)) = line.split_once(':') {
            match key.trim() {
                "physical id" => physical = value.trim().to_owned(),
                "core id" => {
                    cores.insert((physical.clone(), value.trim().to_owned()));
                }
                _ => {}
            }
        }
    }
    let flags: BTreeSet<&str> = cpuinfo
        .lines()
        .find_map(|l| l.strip_prefix("flags"))
        .map(|f| {
            f.trim_start_matches([' ', '\t', ':'])
                .split_whitespace()
                .collect()
        })
        .unwrap_or_default();
    let performance = text(root, "/sys/devices/cpu_core/cpus")
        .map(|l| cpu_list(&l))
        .unwrap_or_default();
    let efficient = text(root, "/sys/devices/cpu_atom/cpus")
        .map(|l| cpu_list(&l))
        .unwrap_or_default();
    let base = "/sys/devices/system/cpu";
    let online = text(root, &format!("{base}/online"))
        .map(|l| cpu_list(&l))
        .unwrap_or_default();
    let khz = |cpu: u32, file: &str| {
        text(root, &format!("{base}/cpu{cpu}/cpufreq/{file}")).and_then(|v| v.parse::<f64>().ok())
    };
    let cpus: Vec<LogicalCpu> = online
        .iter()
        .map(|&cpu| LogicalCpu {
            cpu,
            core_id: text(root, &format!("{base}/cpu{cpu}/topology/core_id"))
                .and_then(|v| v.parse().ok())
                .unwrap_or(cpu),
            kind: if performance.contains(&cpu) {
                Some("Performance".into())
            } else if efficient.contains(&cpu) {
                Some("Efficient".into())
            } else {
                None
            },
            mhz: khz(cpu, "scaling_cur_freq").map(|k| k / 1000.0),
        })
        .collect();
    let core_count = |set: &BTreeSet<u32>| {
        cpus.iter()
            .filter(|c| set.contains(&c.cpu))
            .map(|c| c.core_id)
            .collect::<BTreeSet<_>>()
            .len()
    };
    Processor {
        model: first("model name"),
        cores: cores.len().max(1),
        hybrid: (!performance.is_empty() && !efficient.is_empty())
            .then(|| (core_count(&performance), core_count(&efficient))),
        threads: cpus.len().max(
            cpuinfo
                .lines()
                .filter(|l| l.starts_with("processor"))
                .count(),
        ),
        base_hz: khz(0, "base_frequency")
            .or_else(|| khz(0, "cpuinfo_min_freq"))
            .map(|k| k * 1000.0),
        max_hz: khz(0, "cpuinfo_max_freq").map(|k| k * 1000.0),
        l3_bytes: text(root, &format!("{base}/cpu0/cache/index3/size"))
            .and_then(|s| s.trim_end_matches('K').parse::<u64>().ok())
            .map(|k| k * 1024),
        microcode: first("microcode"),
        features: FEATURES
            .iter()
            .filter(|(flag, _)| flags.contains(flag))
            .map(|(_, name)| (*name).to_owned())
            .collect(),
        cpus,
    }
}

fn disks(root: &SystemRoot) -> Vec<Disk> {
    let mut out: Vec<Disk> = std::fs::read_dir(root.path("/sys/block"))
        .map(|d| {
            d.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
        .into_iter()
        // Only real disks have a device; loop, dm, zram, and RAM disks do not.
        .filter(|n| root.path(&format!("/sys/block/{n}/device")).exists())
        .map(|name| {
            let b = |f: &str| text(root, &format!("/sys/block/{name}/{f}"));
            let path = std::fs::canonicalize(root.path(&format!("/sys/block/{name}")))
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();
            let rotational = b("queue/rotational").as_deref() == Some("1");
            let kind = if path.contains("/usb") {
                "USB drive"
            } else if name.starts_with("nvme") {
                "SSD (NVMe)"
            } else if name.starts_with("mmcblk") {
                "Memory card"
            } else if rotational {
                "Hard disk"
            } else {
                "SSD"
            };
            Disk {
                model: b("device/model"),
                kind: kind.into(),
                size: b("size").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0) * 512,
                firmware: b("device/firmware_rev").or_else(|| b("device/rev")),
                removable: b("removable").as_deref() == Some("1"),
                name,
            }
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Vendor and device names for the IDs asked for, found in one pass over `pci.ids`.
pub fn pci_lookup(
    ids: &str,
    wanted: &BTreeSet<(String, String)>,
) -> BTreeMap<(String, String), (Option<String>, Option<String>)> {
    let vendors: BTreeSet<&str> = wanted.iter().map(|(v, _)| v.as_str()).collect();
    let mut out = BTreeMap::new();
    let mut current: Option<(String, String)> = None; // (vendor id, vendor name)
    for line in ids.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with("C ") {
            break; // the class list follows the vendors
        }
        if !line.starts_with('\t') {
            current = line
                .split_once("  ")
                .filter(|(id, _)| vendors.contains(id))
                .map(|(id, name)| (id.to_owned(), name.to_owned()));
            if let Some((id, name)) = &current {
                for (v, d) in wanted.iter().filter(|(v, _)| v == id) {
                    out.entry((v.clone(), d.clone()))
                        .or_insert((Some(name.clone()), None));
                }
            }
            continue;
        }
        let Some((vendor, _)) = &current else {
            continue;
        };
        if let Some((device, name)) = line
            .strip_prefix('\t')
            .filter(|l| !l.starts_with('\t'))
            .and_then(|l| l.split_once("  "))
            && let Some(entry) = out.get_mut(&(vendor.clone(), device.to_owned()))
        {
            entry.1 = Some(name.to_owned());
        }
    }
    out
}

/// PCI class codes, in plain words.
fn pci_class(code: &str) -> Option<&'static str> {
    let code = code.trim_start_matches("0x");
    Some(match code.get(..2)? {
        "01" => "Storage",
        "02" => "Network",
        "03" => "Display",
        "04" => "Audio and video",
        "05" => "Memory",
        "06" => "Bridge",
        "07" => "Communication",
        "08" => "System",
        "09" => "Input",
        "0c" => match code.get(2..4)? {
            "03" => "USB controller",
            "05" => "SMBus",
            "80" | "0a" => "Thunderbolt",
            _ => "Serial bus",
        },
        "0d" => "Wireless",
        "10" => "Encryption",
        "11" => "Signal processing",
        _ => "Other",
    })
}

fn driver_of(path: &std::path::Path) -> Option<String> {
    std::fs::read_link(path.join("driver"))
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
}

fn devices(root: &SystemRoot) -> Vec<Device> {
    let mut out = Vec::new();
    // PCI
    let pci: Vec<_> = std::fs::read_dir(root.path("/sys/bus/pci/devices"))
        .map(|d| d.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    let hex = |p: &std::path::Path, f: &str| {
        std::fs::read_to_string(p.join(f))
            .ok()
            .map(|v| v.trim().trim_start_matches("0x").to_lowercase())
    };
    let wanted: BTreeSet<(String, String)> = pci
        .iter()
        .filter_map(|p| Some((hex(p, "vendor")?, hex(p, "device")?)))
        .collect();
    let ids = PCI_IDS
        .iter()
        .find_map(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    let names = pci_lookup(&ids, &wanted);
    for p in &pci {
        let (Some(v), Some(d)) = (hex(p, "vendor"), hex(p, "device")) else {
            continue;
        };
        let (vendor, model) = names
            .get(&(v.clone(), d.clone()))
            .cloned()
            .unwrap_or((None, None));
        out.push(Device {
            name: model.unwrap_or_else(|| format!("PCI device {v}:{d}")),
            bus: "PCI".into(),
            vendor,
            drivers: driver_of(p).into_iter().collect(),
            class: hex(p, "class")
                .as_deref()
                .and_then(pci_class)
                .map(str::to_owned),
            id: Some(format!("{v}:{d}")),
        });
    }
    // USB: devices such as `3-1`; root hubs (`usb1`) and interfaces (`3-1:1.0`) are skipped.
    let usb: Vec<_> = std::fs::read_dir(root.path("/sys/bus/usb/devices"))
        .map(|d| d.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    for p in usb.iter().filter(|p| {
        p.file_name().is_some_and(|n| {
            let n = n.to_string_lossy();
            !n.starts_with("usb") && !n.contains(':')
        })
    }) {
        let read = |f: &str| {
            std::fs::read_to_string(p.join(f))
                .ok()
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
        };
        let (Some(v), Some(d)) = (read("idVendor"), read("idProduct")) else {
            continue;
        };
        let base = p
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut drivers: Vec<String> = usb
            .iter()
            .filter(|i| {
                i.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with(&format!("{base}:")))
            })
            .filter_map(|i| driver_of(i))
            .collect();
        drivers.sort();
        drivers.dedup();
        out.push(Device {
            name: read("product").unwrap_or_else(|| format!("USB device {v}:{d}")),
            bus: "USB".into(),
            vendor: read("manufacturer"),
            drivers,
            class: None,
            id: Some(format!("{v}:{d}")),
        });
    }
    // Input devices with a keyboard or pointer handler.
    let input = root.read("/proc/bus/input/devices").unwrap_or_default();
    for block in input.split("\n\n") {
        let name = block
            .lines()
            .find_map(|l| l.strip_prefix("N: Name="))
            .map(|n| n.trim_matches('"').to_owned());
        let handlers = block
            .lines()
            .find_map(|l| l.strip_prefix("H: Handlers="))
            .unwrap_or("");
        let class = if handlers.contains("kbd") && handlers.contains("leds") {
            "Keyboard"
        } else if handlers.contains("mouse") {
            "Pointer"
        } else {
            continue;
        };
        if let Some(name) = name {
            out.push(Device {
                name,
                bus: "Input".into(),
                vendor: None,
                drivers: Vec::new(),
                class: Some(class.into()),
                id: None,
            });
        }
    }
    out
}

/// Reads the hardware inventory.
pub fn read(root: &SystemRoot) -> HardwareInfo {
    HardwareInfo {
        machine: machine(root),
        processor: processor(root),
        memory_total: root
            .read("/proc/meminfo")
            .ok()
            .and_then(|m| {
                m.lines()
                    .find_map(|l| l.strip_prefix("MemTotal:"))
                    .and_then(|v| v.trim().trim_end_matches("kB").trim().parse::<u64>().ok())
            })
            .map(|kb| kb * 1024),
        ecc_reporting: std::fs::read_dir(root.path("/sys/devices/system/edac/mc")).is_ok_and(
            |mut d| d.any(|e| e.is_ok_and(|e| e.file_name().to_string_lossy().starts_with("mc"))),
        ),
        disks: disks(root),
        devices: devices(root),
        bluetooth_adapters: std::fs::read_dir(root.path("/sys/class/bluetooth"))
            .map(|d| {
                d.flatten()
                    .filter(|e| !e.file_name().to_string_lossy().contains(':'))
                    .count()
            })
            .unwrap_or(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn reads_cpu_lists() {
        assert_eq!(
            cpu_list("0-3,8\n").into_iter().collect::<Vec<_>>(),
            [0, 1, 2, 3, 8]
        );
        assert!(cpu_list("").is_empty());
        assert!(cpu_list("5-2").is_empty());
    }

    #[test]
    fn looks_up_many_pci_names_in_one_pass() {
        let ids = "8086  Intel Corporation\n\ta7a1  Iris Xe\n\t\ta7a1 1028 0c97  Sub\n\t51f0  Wi-Fi 6E\n10ec  Realtek\n\t5765  RTL NVMe\nC 00  Unclassified\n";
        let wanted: BTreeSet<_> = [("8086", "51f0"), ("10ec", "9999")]
            .iter()
            .map(|(v, d)| ((*v).to_owned(), (*d).to_owned()))
            .collect();
        let names = pci_lookup(ids, &wanted);
        assert_eq!(
            names[&("8086".into(), "51f0".into())],
            (Some("Intel Corporation".into()), Some("Wi-Fi 6E".into()))
        );
        assert_eq!(
            names[&("10ec".into(), "9999".into())],
            (Some("Realtek".into()), None)
        );
    }

    #[test]
    fn reads_a_hybrid_laptop() {
        let root = fixture_root(
            "hardware",
            &[
                ("/sys/class/dmi/id/sys_vendor", "LENOVO\n"),
                ("/sys/class/dmi/id/product_name", "82SD\n"),
                ("/sys/class/dmi/id/product_version", "IdeaPad 5 14IAL7\n"),
                ("/sys/class/dmi/id/chassis_type", "10\n"),
                (
                    "/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c",
                    "\x06\x00\x00\x00\x01",
                ),
                ("/sys/class/tpm/tpm0/tpm_version_major", "2\n"),
                (
                    "/proc/cpuinfo",
                    "processor\t: 0\nmodel name\t: Intel i5-1235U\nmicrocode\t: 0x4123\nflags\t\t: fpu vmx aes avx2\nphysical id\t: 0\ncore id\t\t: 0\n\nprocessor\t: 1\nphysical id\t: 0\ncore id\t\t: 0\n\nprocessor\t: 2\nphysical id\t: 0\ncore id\t\t: 8\n",
                ),
                ("/sys/devices/system/cpu/online", "0-2\n"),
                ("/sys/devices/system/cpu/cpu0/topology/core_id", "0\n"),
                ("/sys/devices/system/cpu/cpu1/topology/core_id", "0\n"),
                ("/sys/devices/system/cpu/cpu2/topology/core_id", "8\n"),
                (
                    "/sys/devices/system/cpu/cpu2/cpufreq/scaling_cur_freq",
                    "1180000\n",
                ),
                ("/sys/devices/cpu_core/cpus", "0-1\n"),
                ("/sys/devices/cpu_atom/cpus", "2\n"),
                ("/sys/block/nvme0n1/device/model", "Samsung PM991a\n"),
                ("/sys/block/nvme0n1/size", "500118192\n"),
                ("/sys/block/loop0/size", "8\n"),
                (
                    "/proc/bus/input/devices",
                    "I: Bus=0011\nN: Name=\"AT Translated Set 2 keyboard\"\nH: Handlers=sysrq kbd leds event3\n\nI: Bus=0019\nN: Name=\"Power Button\"\nH: Handlers=kbd event0\n",
                ),
            ],
        );
        let hw = read(&root);
        assert_eq!(hw.machine.model.as_deref(), Some("IdeaPad 5 14IAL7"));
        assert_eq!(hw.machine.chassis.as_deref(), Some("Laptop"));
        assert_eq!(hw.machine.secure_boot, Some(true));
        assert_eq!(hw.machine.tpm.as_deref(), Some("2.0"));
        assert_eq!(hw.machine.boot_mode, "UEFI");
        let p = &hw.processor;
        assert_eq!((p.cores, p.threads, p.hybrid), (2, 3, Some((1, 1))));
        assert_eq!(p.features, ["VT-x", "AES-NI", "AVX2"]);
        assert_eq!(p.cpus[2].kind.as_deref(), Some("Efficient"));
        assert_eq!(p.cpus[2].mhz, Some(1180.0));
        assert_eq!(hw.disks.len(), 1, "loop devices are not disks");
        assert_eq!(hw.disks[0].kind, "SSD (NVMe)");
        assert_eq!(hw.disks[0].size, 500_118_192 * 512);
        let input: Vec<&str> = hw
            .devices
            .iter()
            .filter(|d| d.bus == "Input")
            .map(|d| d.name.as_str())
            .collect();
        assert_eq!(
            input,
            ["AT Translated Set 2 keyboard"],
            "the power button is not a keyboard"
        );
    }
}
