// SPDX-License-Identifier: Apache-2.0

//! Firmware, platform, and ports in depth for the Expert level (docs/01-product/expert-coverage.md,
//! sections 12 and 16): the firmware security level and every check behind it (fwupd), UEFI boot
//! entries and order, Secure Boot and the keys the user enrolled (MOK), the management engine's
//! version, the TPM and whether disk encryption uses it, protection against attacks through ports
//! (IOMMU, Thunderbolt), the hardware clock, virtualization, the USB tree with each device's speed
//! and power, USB-C ports with what is plugged in, and Thunderbolt devices. No admin rights.

use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use super::command;
use crate::collect::SystemRoot;

const LIMIT: Duration = Duration::from_secs(10);
const EFI_GLOBAL: &str = "8be4df61-93ca-11d2-aa0d-00e098032b8c";

/// One firmware security check from fwupd.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecurityCheck {
    pub id: String,
    pub name: String,
    /// fwupd's result word, such as `enabled`, `not-encrypted`, `valid`.
    pub result: String,
    pub passed: bool,
    /// The problem is in the running system (such as swap not encrypted), not the firmware.
    pub runtime: bool,
}

/// One UEFI boot entry.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct BootEntry {
    /// The number, such as `0001`.
    pub number: String,
    pub name: String,
    pub active: bool,
    /// The file it starts, such as `\EFI\ubuntu\shimx64.efi`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Its place in the boot order (0 first); missing when it is not in the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<usize>,
    /// The entry this boot started from.
    pub current: bool,
}

/// One USB device.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct UsbDevice {
    pub port: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maker: Option<String>,
    /// The USB version the device supports, such as `3.20`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The speed it runs at now, in Mbit/s.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_mbps: Option<f64>,
    /// The most power it asked for, in mA.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_power_ma: Option<u64>,
    pub hub: bool,
    pub drivers: Vec<String>,
}

/// An alternate mode a USB-C device offers, such as DisplayPort.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AltMode {
    /// `DisplayPort`, `Thunderbolt`, or the hex standard ID.
    pub name: String,
    pub active: bool,
}

/// One USB-C port.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct TypecPort {
    pub name: String,
    /// The roles in use: `host` or `device` for data, `source` or `sink` for power.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_role: Option<String>,
    /// Such as `default`, `1.5A`, `3.0A`, or `usb_power_delivery`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    /// Something is plugged in.
    pub connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partner_type: Option<String>,
    pub partner_power_delivery: bool,
    pub alt_modes: Vec<AltMode>,
}

/// One Thunderbolt or USB4 device.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct TbDevice {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    /// Approved to use the port (Thunderbolt security).
    pub authorized: bool,
}

/// What `platform_detail` answers.
#[derive(Debug, Default, Serialize)]
pub struct PlatformDetail {
    /// Such as `HSI:1`, from `fwupdmgr security`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hsi: Option<String>,
    pub security_checks: Vec<SecurityCheck>,
    pub boot_entries: Vec<BootEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secure_boot: Option<String>,
    /// Keys the user added for Secure Boot (MOK), by subject.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mok_keys: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub management_engine: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tpm_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tpm_driver: Option<String>,
    /// `/etc/crypttab` asks the TPM to unlock a disk.
    pub disk_unlocked_by_tpm: bool,
    /// A disk is encrypted at all (a line in `/etc/crypttab`).
    pub disk_encrypted: bool,
    /// IOMMU units the kernel uses (memory protection against devices).
    pub iommu_units: usize,
    /// Thunderbolt security level and DMA protection, when there is a Thunderbolt or USB4 port.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thunderbolt_security: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thunderbolt_dma_protection: Option<bool>,
    /// Hardware clock minus system time, in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rtc_drift_s: Option<i64>,
    /// `vmx` (Intel) or `svm` (AMD) when the CPU offers it and the firmware has it turned on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtualization: Option<String>,
    pub kvm_ready: bool,
    pub usb: Vec<UsbDevice>,
    pub typec: Vec<TypecPort>,
    pub thunderbolt: Vec<TbDevice>,
}

fn text(root: &SystemRoot, path: &str) -> Option<String> {
    root.read(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn names(root: &SystemRoot, dir: &str) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(root.path(dir))
        .map(|d| {
            d.flatten()
                .filter_map(|e| e.file_name().to_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// The checks in `fwupdmgr security --json`.
pub fn parse_hsi(json: &str) -> Vec<SecurityCheck> {
    let Ok(v) = serde_json::from_str::<Value>(json) else {
        return Vec::new();
    };
    v["SecurityAttributes"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| {
                    let flags: Vec<&str> = x["Flags"]
                        .as_array()
                        .map(|f| f.iter().filter_map(Value::as_str).collect())
                        .unwrap_or_default();
                    Some(SecurityCheck {
                        id: x["AppstreamId"].as_str()?.to_owned(),
                        name: x["Name"].as_str().unwrap_or("?").to_owned(),
                        result: x["HsiResult"].as_str().unwrap_or("?").to_owned(),
                        passed: flags.contains(&"success"),
                        runtime: flags.contains(&"runtime-issue"),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The data of an efivarfs file, after its 4-byte attribute header.
fn efivar(root: &SystemRoot, name: &str) -> Option<Vec<u8>> {
    let bytes =
        std::fs::read(root.path(&format!("/sys/firmware/efi/efivars/{name}-{EFI_GLOBAL}"))).ok()?;
    bytes.get(4..).map(<[u8]>::to_vec)
}

fn utf16(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|u| *u != 0)
        .collect();
    String::from_utf16_lossy(&units)
}

/// One `Boot####` load option: attributes, device path length, UTF-16 name, device path.
pub fn parse_load_option(data: &[u8]) -> Option<(bool, String, Option<String>)> {
    let attributes = u32::from_le_bytes(data.get(0..4)?.try_into().ok()?);
    let path_len = u16::from_le_bytes(data.get(4..6)?.try_into().ok()?) as usize;
    let rest = data.get(6..)?;
    let name_end = rest.as_chunks::<2>().0.iter().position(|c| *c == [0, 0])? * 2;
    let name = utf16(&rest[..name_end]);
    let path = rest.get(name_end + 2..name_end + 2 + path_len)?;
    // Walk the device path for a file path node (type 4, subtype 4).
    let mut at = 0;
    let mut file = None;
    while at + 4 <= path.len() {
        let (kind, sub) = (path[at], path[at + 1]);
        let len = u16::from_le_bytes([path[at + 2], path[at + 3]]) as usize;
        if len < 4 || at + len > path.len() {
            break;
        }
        if kind == 4 && sub == 4 {
            file = Some(utf16(&path[at + 4..at + len]));
        }
        if kind == 0x7f {
            break;
        }
        at += len;
    }
    Some((attributes & 1 == 1, name, file))
}

fn boot_entries(root: &SystemRoot) -> Vec<BootEntry> {
    let order: Vec<String> = efivar(root, "BootOrder")
        .map(|d| {
            d.as_chunks::<2>()
                .0
                .iter()
                .map(|c| format!("{:04X}", u16::from_le_bytes([c[0], c[1]])))
                .collect()
        })
        .unwrap_or_default();
    let current = efivar(root, "BootCurrent").and_then(|d| {
        d.get(0..2)
            .map(|c| format!("{:04X}", u16::from_le_bytes([c[0], c[1]])))
    });
    names(root, "/sys/firmware/efi/efivars")
        .into_iter()
        .filter_map(|n| {
            let number = n
                .strip_prefix("Boot")?
                .strip_suffix(&format!("-{EFI_GLOBAL}"))?;
            if number.len() != 4 || !number.chars().all(|c| c.is_ascii_hexdigit()) {
                return None;
            }
            let (active, name, file) = parse_load_option(&efivar(root, &format!("Boot{number}"))?)?;
            Some(BootEntry {
                order: order.iter().position(|o| o == number),
                current: current.as_deref() == Some(number),
                number: number.to_owned(),
                name,
                active,
                file,
            })
        })
        .collect()
}

fn usb(root: &SystemRoot) -> Vec<UsbDevice> {
    let base = "/sys/bus/usb/devices";
    let all = names(root, base);
    all.iter()
        .filter(|n| !n.contains(':') && !n.starts_with("usb"))
        .map(|port| {
            let at = |f: &str| format!("{base}/{port}/{f}");
            let mut drivers: Vec<String> = all
                .iter()
                .filter(|i| i.starts_with(&format!("{port}:")))
                .filter_map(|i| super::hardware::driver_of(&root.path(&format!("{base}/{i}"))))
                .collect();
            drivers.sort();
            drivers.dedup();
            UsbDevice {
                port: port.clone(),
                product: text(root, &at("product")),
                maker: text(root, &at("manufacturer")),
                version: text(root, &at("version")),
                speed_mbps: text(root, &at("speed")).and_then(|s| s.parse().ok()),
                max_power_ma: text(root, &at("bMaxPower"))
                    .and_then(|p| p.trim_end_matches("mA").parse().ok()),
                hub: text(root, &at("bDeviceClass")).as_deref() == Some("09"),
                drivers,
            }
        })
        .collect()
}

/// The setting in brackets, as in `[host] device`.
fn bracketed(text: &str) -> Option<String> {
    let start = text.find('[')? + 1;
    Some(text[start..start + text[start..].find(']')?].to_owned())
}

fn typec(root: &SystemRoot) -> Vec<TypecPort> {
    let base = "/sys/class/typec";
    let all = names(root, base);
    all.iter()
        .filter(|n| n.starts_with("port") && !n.contains('-') && !n.contains('.'))
        .map(|port| {
            let at = |f: &str| format!("{base}/{port}/{f}");
            let partner = format!("{port}-partner");
            let connected = all.contains(&partner);
            let alt_modes = all
                .iter()
                .filter(|n| n.starts_with(&format!("{partner}.")))
                .filter_map(|m| {
                    let svid = text(root, &format!("{base}/{m}/svid"))?.to_ascii_lowercase();
                    let name = match svid.as_str() {
                        "ff01" => "DisplayPort".to_owned(),
                        "8087" => "Thunderbolt".to_owned(),
                        other => other.to_owned(),
                    };
                    Some(AltMode {
                        name,
                        active: text(root, &format!("{base}/{m}/active")).as_deref() == Some("yes"),
                    })
                })
                .collect();
            TypecPort {
                name: port.clone(),
                data_role: text(root, &at("data_role")).and_then(|t| bracketed(&t).or(Some(t))),
                power_role: text(root, &at("power_role")).and_then(|t| bracketed(&t).or(Some(t))),
                power_mode: text(root, &at("power_operation_mode")),
                revision: text(root, &at("usb_typec_revision")),
                connected,
                partner_type: connected
                    .then(|| text(root, &format!("{base}/{partner}/type")))
                    .flatten(),
                partner_power_delivery: text(
                    root,
                    &format!("{base}/{partner}/supports_usb_power_delivery"),
                )
                .as_deref()
                    == Some("yes"),
                alt_modes,
            }
        })
        .collect()
}

fn thunderbolt(root: &SystemRoot) -> (Vec<TbDevice>, Option<String>, Option<bool>) {
    let base = "/sys/bus/thunderbolt/devices";
    let all = names(root, base);
    let domain = all.iter().find(|n| n.starts_with("domain"));
    let security = domain.and_then(|d| text(root, &format!("{base}/{d}/security")));
    let dma = domain
        .and_then(|d| text(root, &format!("{base}/{d}/iommu_dma_protection")))
        .map(|v| v == "1");
    let devices = all
        .iter()
        .filter(|n| {
            !n.starts_with("domain") && root.path(&format!("{base}/{n}/device_name")).exists()
        })
        // The computer's own controller is `0-0`; devices plugged in come after it.
        .filter(|n| !n.ends_with("-0") || n.contains('.'))
        .map(|n| TbDevice {
            id: n.clone(),
            name: text(root, &format!("{base}/{n}/device_name")),
            vendor: text(root, &format!("{base}/{n}/vendor_name")),
            authorized: text(root, &format!("{base}/{n}/authorized")).is_some_and(|a| a != "0"),
        })
        .collect();
    (devices, security, dma)
}

/// Reads `platform_detail`.
pub async fn read(root: &SystemRoot) -> PlatformDetail {
    let hsi_json = command::output(
        "fwupdmgr",
        &["security", "--json", "--no-unreported-check"],
        LIMIT,
    )
    .await;
    let hsi = command::output("fwupdmgr", &["security", "--no-unreported-check"], LIMIT)
        .await
        .and_then(|t| {
            t.lines().find_map(|l| {
                l.split("HSI:")
                    .nth(1)
                    .map(|r| format!("HSI:{}", r.split_whitespace().next().unwrap_or("")))
            })
        });
    let mok_keys = command::output("mokutil", &["--list-enrolled"], LIMIT)
        .await
        .map(|t| {
            t.lines()
                .filter_map(|l| l.trim().strip_prefix("Subject:"))
                .map(|s| s.trim().to_owned())
                .collect()
        });
    let cpuinfo = root.read("/proc/cpuinfo").unwrap_or_default();
    let flags = cpuinfo
        .lines()
        .find(|l| l.starts_with("flags"))
        .unwrap_or("");
    let virtualization = [" vmx", " svm"]
        .iter()
        .find(|f| flags.contains(**f))
        .map(|f| f.trim().to_owned());
    let crypttab = root.read("/etc/crypttab").unwrap_or_default();
    let rows: Vec<&str> = crypttab
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    let (thunderbolt, thunderbolt_security, thunderbolt_dma_protection) = thunderbolt(root);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    PlatformDetail {
        hsi,
        security_checks: hsi_json.map(|j| parse_hsi(&j)).unwrap_or_default(),
        boot_entries: boot_entries(root),
        secure_boot: command::output("mokutil", &["--sb-state"], LIMIT)
            .await
            .map(|t| t.trim().to_owned())
            .filter(|t| !t.is_empty()),
        mok_keys,
        management_engine: names(root, "/sys/class/mei")
            .first()
            .and_then(|m| text(root, &format!("/sys/class/mei/{m}/fw_ver")))
            .and_then(|v| v.lines().next().map(str::to_owned)),
        tpm_version: text(root, "/sys/class/tpm/tpm0/tpm_version_major"),
        tpm_driver: super::hardware::driver_of(&root.path("/sys/class/tpm/tpm0/device")),
        disk_unlocked_by_tpm: rows.iter().any(|l| l.contains("tpm2-device")),
        disk_encrypted: !rows.is_empty(),
        iommu_units: names(root, "/sys/class/iommu").len(),
        thunderbolt_security,
        thunderbolt_dma_protection,
        rtc_drift_s: text(root, "/sys/class/rtc/rtc0/since_epoch")
            .and_then(|v| v.parse::<i64>().ok())
            .map(|rtc| rtc - now),
        kvm_ready: virtualization.is_some() && root.path("/dev/kvm").exists(),
        virtualization,
        usb: usb(root),
        typec: typec(root),
        thunderbolt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    fn utf16z(s: &str) -> Vec<u8> {
        s.encode_utf16()
            .chain([0])
            .flat_map(u16::to_le_bytes)
            .collect()
    }

    #[test]
    fn load_options_give_name_and_file() {
        let mut path = vec![4u8, 4];
        let file = utf16z("\\EFI\\ubuntu\\shimx64.efi");
        path.extend(((file.len() + 4) as u16).to_le_bytes());
        path.extend(&file);
        path.extend([0x7f, 0xff, 4, 0]);
        let mut data = 1u32.to_le_bytes().to_vec();
        data.extend((path.len() as u16).to_le_bytes());
        data.extend(utf16z("ubuntu"));
        data.extend(&path);
        assert_eq!(
            parse_load_option(&data),
            Some((
                true,
                "ubuntu".into(),
                Some("\\EFI\\ubuntu\\shimx64.efi".into())
            ))
        );
        assert_eq!(parse_load_option(&[1, 2]), None);
    }

    #[test]
    fn hsi_checks() {
        let c = parse_hsi(
            r#"{"SecurityAttributes":[{"AppstreamId":"org.fwupd.hsi.Kernel.Swap","Name":"Linux swap","HsiResult":"not-encrypted","Flags":["runtime-issue"]},{"AppstreamId":"org.fwupd.hsi.Uefi.SecureBoot","Name":"UEFI secure boot","HsiResult":"enabled","Flags":["success"]}]}"#,
        );
        assert_eq!(c.len(), 2);
        assert!(!c[0].passed && c[0].runtime);
        assert!(c[1].passed);
        assert!(parse_hsi("not json").is_empty());
    }

    #[test]
    fn usb_typec_and_thunderbolt() {
        let root = fixture_root(
            "platform-detail",
            &[
                (
                    "/sys/bus/usb/devices/3-6/product",
                    "Integrated_Webcam_FHD\n",
                ),
                ("/sys/bus/usb/devices/3-6/version", " 2.01\n"),
                ("/sys/bus/usb/devices/3-6/speed", "480\n"),
                ("/sys/bus/usb/devices/3-6/bMaxPower", "500mA\n"),
                ("/sys/bus/usb/devices/4-1/product", "SSD\n"),
                ("/sys/bus/usb/devices/4-1/version", " 3.20\n"),
                ("/sys/bus/usb/devices/4-1/speed", "480\n"),
                ("/sys/bus/usb/devices/usb3/product", "xHCI\n"),
                ("/sys/class/typec/port0/data_role", "[host] device\n"),
                ("/sys/class/typec/port0/power_role", "[source] sink\n"),
                (
                    "/sys/class/typec/port0/power_operation_mode",
                    "usb_power_delivery\n",
                ),
                ("/sys/class/typec/port0-partner/type", "\n"),
                (
                    "/sys/class/typec/port0-partner/supports_usb_power_delivery",
                    "yes\n",
                ),
                ("/sys/class/typec/port0-partner.0/svid", "ff01\n"),
                ("/sys/class/typec/port0-partner.0/active", "yes\n"),
                ("/sys/bus/thunderbolt/devices/domain0/security", "user\n"),
                (
                    "/sys/bus/thunderbolt/devices/domain0/iommu_dma_protection",
                    "1\n",
                ),
                ("/sys/bus/thunderbolt/devices/0-0/device_name", "Laptop\n"),
                ("/sys/bus/thunderbolt/devices/0-1/device_name", "Dock\n"),
                ("/sys/bus/thunderbolt/devices/0-1/authorized", "0\n"),
            ],
        );
        let u = usb(&root);
        assert_eq!(u.len(), 2);
        assert_eq!(
            (u[1].version.as_deref(), u[1].speed_mbps),
            (Some("3.20"), Some(480.0)),
            "a USB 3 device on a USB 2 link"
        );
        assert_eq!(u[0].max_power_ma, Some(500));
        let t = typec(&root);
        assert_eq!(t.len(), 1);
        assert!(t[0].connected && t[0].partner_power_delivery);
        assert_eq!(
            (t[0].data_role.as_deref(), t[0].power_role.as_deref()),
            (Some("host"), Some("source"))
        );
        assert_eq!(
            t[0].alt_modes,
            [AltMode {
                name: "DisplayPort".into(),
                active: true
            }]
        );
        let (devices, security, dma) = thunderbolt(&root);
        assert_eq!(
            devices.len(),
            1,
            "the computer's own controller is not a device"
        );
        assert!(!devices[0].authorized);
        assert_eq!((security.as_deref(), dma), (Some("user"), Some(true)));
    }
}
