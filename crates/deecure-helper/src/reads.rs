// SPDX-License-Identifier: Apache-2.0

//! Admin read-outs: things only root may read. Each returns JSON or text, never secrets:
//! password hashes stay in /etc/shadow, only their status is reported, and BIOS passwords
//! are never read, only whether one is set.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;
use serde_json::{Value, json};

/// One account's password status, from `/etc/shadow` (the hash itself is never returned).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AccountStatus {
    pub user: String,
    /// True when the password is locked (`!` or `*`).
    pub locked: bool,
    /// True when there is no password at all (an empty field).
    pub empty: bool,
    /// Days since 1970 when the password last changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changed_day: Option<u64>,
    /// Days a password may be kept, when set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_days: Option<u64>,
}

/// Reads `/etc/shadow` lines for the given users.
pub fn shadow_status(shadow: &str, users: &[String]) -> Vec<AccountStatus> {
    shadow
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split(':').collect();
            let user = (*f.first()?).to_owned();
            if !users.contains(&user) {
                return None;
            }
            let hash = f.get(1).copied().unwrap_or("");
            Some(AccountStatus {
                locked: hash.starts_with('!') || hash.starts_with('*'),
                empty: hash.is_empty(),
                changed_day: f.get(2).and_then(|v| v.parse().ok()),
                max_days: f
                    .get(4)
                    .and_then(|v| v.parse().ok())
                    .filter(|d| *d < 99_999),
                user,
            })
        })
        .collect()
}

/// Human users (UID 0 and 1000-59999) from `/etc/passwd`.
pub fn human_users(passwd: &str) -> Vec<String> {
    passwd
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split(':').collect();
            let uid: u32 = f.get(2)?.parse().ok()?;
            let name = (*f.first()?).to_owned();
            (uid == 0 || (1000..60_000).contains(&uid)).then_some(name)
        })
        .collect()
}

/// Each user's crontab from `/var/spool/cron/crontabs`.
pub fn all_crontabs(dir: &std::path::Path) -> BTreeMap<String, String> {
    std::fs::read_dir(dir)
        .map(|d| {
            d.flatten()
                .filter(|e| e.path().is_file())
                .filter_map(|e| {
                    Some((
                        e.file_name().to_string_lossy().into_owned(),
                        std::fs::read_to_string(e.path()).ok()?,
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Disk names `smartctl` may read: `sda`, `nvme0n1`, `mmcblk0`, and nothing else.
pub fn is_disk(name: &str) -> bool {
    let digits = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
    if let Some(rest) = name.strip_prefix("sd") {
        return (1..=3).contains(&rest.len()) && rest.chars().all(|c| c.is_ascii_lowercase());
    }
    if let Some(rest) = name.strip_prefix("nvme") {
        return rest
            .split_once('n')
            .is_some_and(|(a, b)| digits(a) && digits(b));
    }
    name.strip_prefix("mmcblk").is_some_and(digits)
}

/// `dmidecode -t 16 -t 17`: the board's memory arrays (most memory, slots, error correction)
/// and every slot, with the rated speed next to the one it runs at and its bank (channel).
pub fn memory_slots(text: &str) -> Value {
    let array_keys = [
        "Use",
        "Location",
        "Maximum Capacity",
        "Number Of Devices",
        "Error Correction Type",
    ];
    let module_keys = [
        "Size",
        "Type",
        "Speed",
        "Configured Memory Speed",
        "Form Factor",
        "Manufacturer",
        "Locator",
        "Bank Locator",
        "Part Number",
        "Rank",
        "Data Width",
        "Total Width",
    ];
    let fields = |block: &str, keys: &[&str]| -> BTreeMap<String, String> {
        block
            .lines()
            .filter_map(|l| l.trim().split_once(": "))
            .filter(|(k, _)| keys.contains(k))
            .map(|(k, v)| (k.to_owned(), v.trim().to_owned()))
            .collect()
    };
    let mut arrays = Vec::new();
    let mut modules = Vec::new();
    for block in text.split("\n\n") {
        let title = block.lines().nth(1).unwrap_or("").trim();
        if title == "Physical Memory Array" {
            let a = fields(block, &array_keys);
            // Arrays for video or cache memory are not the RAM a person can add to.
            if a.get("Use").is_none_or(|u| u == "System Memory") {
                arrays.push(a);
            }
        } else if title == "Memory Device" {
            let mut m = fields(block, &module_keys);
            m.insert(
                "Filled".into(),
                (!m.get("Size").is_some_and(|s| s.starts_with("No Module"))).to_string(),
            );
            modules.push(m);
        }
    }
    json!({ "arrays": arrays, "modules": modules })
}

/// A BIOS settings interface (`/sys/class/firmware-attributes/<name>`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BiosInterface {
    pub name: String,
    /// Whether a BIOS admin password is set (`authentication/Admin/is_enabled`); changing
    /// settings from Linux needs it when it is. `None` when the interface does not say.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_password: Option<bool>,
    /// The same for the power-on (system) password.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_password: Option<bool>,
    pub settings: Vec<BiosSetting>,
}

/// One BIOS setting with its value now and what it may be.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BiosSetting {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// `enumeration`, `integer`, `string`, or another type the firmware names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<String>,
}

/// Names the firmware gives its folders: letters, digits, `_.-`, never a path.
fn plain_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c))
}

/// One short line from a sysfs file, or nothing (missing, unreadable, or too long).
fn short(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let line = text.trim_end_matches('\n');
    (!line.is_empty() && line.len() <= 512 && !line.contains('\n')).then(|| line.to_owned())
}

/// Every BIOS setting under `<root>/sys/class/firmware-attributes` (Dell, Lenovo, HP), with
/// whether the BIOS admin and power-on passwords are set. Password files are never read.
pub fn bios_settings(root: &Path) -> Vec<BiosInterface> {
    let names = |dir: &Path| -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(dir)
            .map(|d| {
                d.flatten()
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .filter(|n| plain_name(n))
                    .collect()
            })
            .unwrap_or_default();
        v.sort();
        v
    };
    let base = root.join("sys/class/firmware-attributes");
    names(&base)
        .into_iter()
        .map(|name| {
            let dir = base.join(&name);
            let password = |role: &str| {
                short(&dir.join("authentication").join(role).join("is_enabled")).map(|v| v == "1")
            };
            let settings = names(&dir.join("attributes"))
                .into_iter()
                .filter(|n| dir.join("attributes").join(n).is_dir())
                .map(|n| {
                    let a = dir.join("attributes").join(&n);
                    let file = |f: &str| short(&a.join(f));
                    let kind = file("type");
                    BiosSetting {
                        label: file("display_name").filter(|l| *l != n),
                        value: file("current_value"),
                        default: file("default_value"),
                        options: file("possible_values")
                            .map(|v| {
                                v.split(';')
                                    .filter(|o| !o.is_empty())
                                    .map(str::to_owned)
                                    .collect()
                            })
                            .unwrap_or_default(),
                        min: file("min_value").or_else(|| file("min_length")),
                        max: file("max_value").or_else(|| file("max_length")),
                        name: n,
                        kind,
                    }
                })
                .collect();
            BiosInterface {
                admin_password: password("Admin"),
                system_password: password("System"),
                settings,
                name,
            }
        })
        .collect()
}

/// The NVMe controller of a disk name such as `nvme0n1` (`nvme0`), checked by [`is_disk`].
pub fn nvme_controller(disk: &str) -> Option<String> {
    if !is_disk(disk) || !disk.starts_with("nvme") {
        return None;
    }
    disk.rsplit_once('n')
        .map(|(c, _)| c.to_owned())
        .filter(|c| c.len() > 4)
}

fn le(bytes: &[u8], at: usize, len: usize) -> Option<u64> {
    let b = bytes.get(at..at + len)?;
    Some(b.iter().rev().fold(0, |n, &x| (n << 8) | u64::from(x)))
}

fn ascii(bytes: &[u8], at: usize, len: usize) -> Option<String> {
    let b = bytes.get(at..at + len)?;
    let s: String = b
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| {
            if c.is_ascii_graphic() || c == b' ' {
                c as char
            } else {
                '?'
            }
        })
        .collect();
    Some(s.trim().to_owned()).filter(|s| !s.is_empty())
}

/// The parts of NVMe "Identify Controller" (`nvme id-ctrl -o binary`, 4096 bytes, offsets from
/// the NVMe base specification) a person can use. The serial number is left out.
pub fn nvme_identify_controller(b: &[u8]) -> Option<Value> {
    if b.len() < 4096 {
        return None;
    }
    let kelvin = |at| le(b, at, 2).filter(|k| *k > 0).map(|k| k as i64 - 273);
    let frmw = b[260];
    Some(json!({
        "model": ascii(b, 24, 40),
        "firmware": ascii(b, 64, 8),
        "firmware_slots": (frmw >> 1) & 0x7,
        "slot1_read_only": frmw & 1 == 1,
        "activate_without_reset": frmw & 0x10 != 0,
        "power_states": u64::from(b[263]) + 1,
        "autonomous_power_saving": b[265] & 1 == 1,
        "warning_celsius": kelvin(266),
        "critical_celsius": kelvin(268),
        "namespaces": le(b, 516, 4),
        "write_cache": b[525] & 1 == 1,
        "self_test": le(b, 256, 2).map(|oacs| oacs & 0x10 != 0),
    }))
}

/// Relative performance of an NVMe block format (`RP` field).
const PERFORMANCE: [&str; 4] = ["best", "better", "good", "degraded"];

/// NVMe "Identify Namespace" (`nvme id-ns -o binary`): its size and every block format it
/// offers, with the one in use. A drive that offers 4096-byte blocks is a bit faster with them.
pub fn nvme_identify_namespace(b: &[u8]) -> Option<Value> {
    if b.len() < 4096 {
        return None;
    }
    let count = usize::from(b[25]) + 1;
    let flbas = b[26];
    let in_use = usize::from(flbas & 0xF) | (usize::from(flbas >> 5 & 0x3) << 4);
    let formats: Vec<Value> = (0..count.min(64))
        .filter_map(|i| {
            let f = le(b, 128 + 4 * i, 4)?;
            let shift = (f >> 16) & 0xFF;
            (9..=16).contains(&shift).then(|| {
                json!({
                    "block_size": 1u64 << shift,
                    "metadata": f & 0xFFFF,
                    "performance": PERFORMANCE[((f >> 24) & 3) as usize],
                    "in_use": i == in_use,
                })
            })
        })
        .collect();
    let block = formats
        .iter()
        .find(|f| f["in_use"] == true)
        .and_then(|f| f["block_size"].as_u64());
    Some(json!({
        "blocks": le(b, 0, 8),
        "size_bytes": block.and_then(|s| le(b, 0, 8)?.checked_mul(s)),
        "formats": formats,
    }))
}

/// NVMe "Firmware Slot Information" (`nvme fw-log -o binary`, 512 bytes): the version in each
/// slot, the active slot, and the one the next reset starts.
pub fn nvme_firmware_log(b: &[u8]) -> Option<Value> {
    if b.len() < 64 {
        return None;
    }
    let slots: Vec<Value> = (1..=7)
        .filter_map(|n| ascii(b, 8 * n, 8).map(|v| json!({ "slot": n, "version": v })))
        .collect();
    let next = (b[0] >> 4) & 0x7;
    Some(json!({
        "active": b[0] & 0x7,
        "next": (next != 0).then_some(next),
        "slots": slots,
    }))
}

/// The drive's self-test log from `smartctl -j -l selftest` (ATA and NVMe, newest first),
/// and the test running now, if any.
pub fn self_tests(json: &str) -> Value {
    let v: Value = serde_json::from_str(json).unwrap_or(Value::Null);
    let text = |v: &Value| v.get("string").and_then(Value::as_str).map(str::to_owned);
    let mut tests = Vec::new();
    if let Some(table) = v
        .pointer("/ata_smart_self_test_log/standard/table")
        .and_then(Value::as_array)
    {
        for t in table {
            tests.push(json!({
                "kind": t.get("type").and_then(text),
                "result": t.get("status").and_then(text),
                "passed": t.pointer("/status/passed").and_then(Value::as_bool),
                "hours": t.get("lifetime_hours").and_then(Value::as_u64),
            }));
        }
    }
    if let Some(table) = v
        .pointer("/nvme_self_test_log/table")
        .and_then(Value::as_array)
    {
        for t in table {
            let result = t.pointer("/self_test_result/value").and_then(Value::as_u64);
            tests.push(json!({
                "kind": t.get("self_test_code").and_then(text),
                "result": t.get("self_test_result").and_then(text),
                "passed": result.map(|r| r == 0),
                "hours": t.get("power_on_hours").and_then(Value::as_u64),
            }));
        }
    }
    let running = v
        .pointer("/nvme_self_test_log/current_self_test_operation")
        .or_else(|| v.pointer("/ata_smart_data/self_test/status"))
        .and_then(text)
        .filter(|s| {
            let s = s.to_ascii_lowercase();
            !s.contains("no self-test") && !s.contains("completed") && !s.contains("never")
        });
    let logged =
        v.get("ata_smart_self_test_log").is_some() || v.get("nvme_self_test_log").is_some();
    json!({ "supported": logged, "running": running, "tests": tests })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_password_status_without_hashes() {
        let shadow = "root:!:19800:0:99999:7:::\nriya:$y$j9T$abc$def:19850:0:90:7:::\nguest::19000:0:99999:7:::\ndaemon:*:19000:0:99999:7:::\n";
        let users = human_users(
            "root:x:0:0::/root:/bin/bash\ndaemon:x:1:1::/:/usr/sbin/nologin\nriya:x:1000:1000::/home/riya:/bin/bash\nguest:x:1001:1001::/home/guest:/bin/bash\n",
        );
        let s = shadow_status(shadow, &users);
        assert_eq!(s.len(), 3);
        assert!(s[0].locked);
        assert_eq!(
            (s[1].locked, s[1].max_days, s[1].changed_day),
            (false, Some(90), Some(19850))
        );
        assert!(s[2].empty);
        let json = serde_json::to_string(&s).unwrap();
        assert!(!json.contains("$y$"), "hashes never leave the helper");
    }

    #[test]
    fn accepts_only_disk_names() {
        for good in ["sda", "nvme0n1", "mmcblk0", "sdab"] {
            assert!(is_disk(good), "{good}");
        }
        for bad in [
            "sda1",
            "../sda",
            "nvme0",
            "loop0",
            "dm-0",
            "sd",
            "nvme0n1p2",
        ] {
            assert!(!is_disk(bad), "{bad}");
        }
    }

    #[test]
    fn reads_memory_slots_and_board_limits() {
        let text = "Handle 0x0010, DMI type 16, 23 bytes\nPhysical Memory Array\n\tLocation: System Board Or Motherboard\n\tUse: System Memory\n\tError Correction Type: None\n\tMaximum Capacity: 64 GB\n\tNumber Of Devices: 2\n\nHandle 0x0011, DMI type 16, 23 bytes\nPhysical Memory Array\n\tUse: Video Memory\n\tMaximum Capacity: 1 GB\n\nHandle 0x0012, DMI type 17, 92 bytes\nMemory Device\n\tSize: 16 GB\n\tForm Factor: SODIMM\n\tLocator: DIMM A\n\tBank Locator: P0 CHANNEL A\n\tType: DDR5\n\tSpeed: 5600 MT/s\n\tConfigured Memory Speed: 4800 MT/s\n\tRank: 1\n\nHandle 0x0013, DMI type 17, 92 bytes\nMemory Device\n\tSize: No Module Installed\n\tLocator: DIMM B\n\tBank Locator: P0 CHANNEL B\n";
        let v = memory_slots(text);
        assert_eq!(
            v["arrays"].as_array().unwrap().len(),
            1,
            "video memory is left out"
        );
        assert_eq!(v["arrays"][0]["Maximum Capacity"], "64 GB");
        assert_eq!(v["modules"][0]["Configured Memory Speed"], "4800 MT/s");
        assert_eq!(v["modules"][0]["Bank Locator"], "P0 CHANNEL A");
        assert_eq!(v["modules"][1]["Filled"], "false");
    }

    #[test]
    fn reads_bios_settings_but_never_passwords() {
        let dir = std::env::temp_dir().join(format!("deecure-bios-{}", std::process::id()));
        let base = dir.join("sys/class/firmware-attributes/dell-wmi-sysman");
        let write = |p: &str, v: &str| {
            let p = base.join(p);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, v).unwrap();
        };
        write("attributes/WakeOnLan/type", "enumeration\n");
        write("attributes/WakeOnLan/display_name", "Wake on LAN\n");
        write("attributes/WakeOnLan/current_value", "Disabled\n");
        write(
            "attributes/WakeOnLan/possible_values",
            "Disabled;LanOnly;\n",
        );
        write("attributes/PrimaryBattChargeCfg/type", "integer\n");
        write("attributes/PrimaryBattChargeCfg/current_value", "80\n");
        write("attributes/PrimaryBattChargeCfg/min_value", "50\n");
        write("attributes/PrimaryBattChargeCfg/max_value", "100\n");
        write("authentication/Admin/is_enabled", "1\n");
        write("authentication/Admin/current_password", "secret\n");
        write("authentication/System/is_enabled", "0\n");
        let v = bios_settings(&dir);
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(
            (v[0].admin_password, v[0].system_password),
            (Some(true), Some(false))
        );
        let charge = &v[0].settings[0];
        assert_eq!(
            (
                charge.name.as_str(),
                charge.value.as_deref(),
                charge.min.as_deref(),
                charge.max.as_deref()
            ),
            ("PrimaryBattChargeCfg", Some("80"), Some("50"), Some("100"))
        );
        let wake = &v[0].settings[1];
        assert_eq!(wake.options, ["Disabled", "LanOnly"]);
        assert_eq!(wake.label.as_deref(), Some("Wake on LAN"));
        let json = serde_json::to_string(&v).unwrap();
        assert!(!json.contains("secret"), "passwords are never read");
    }

    #[test]
    fn finds_the_nvme_controller() {
        assert_eq!(nvme_controller("nvme0n1").as_deref(), Some("nvme0"));
        assert_eq!(nvme_controller("nvme12n3").as_deref(), Some("nvme12"));
        for bad in ["sda", "nvme0", "nvme0n1p1", "../nvme0n1"] {
            assert_eq!(nvme_controller(bad), None, "{bad}");
        }
    }

    #[test]
    fn reads_nvme_identify_and_firmware_log() {
        let mut c = vec![0u8; 4096];
        c[24..24 + 9].copy_from_slice(b"Fast SSD ");
        c[64..72].copy_from_slice(b"2B2QEXM7");
        c[256] = 0x17; // OACS: self-test supported
        c[260] = 0b0000_0111; // 3 slots, slot 1 read-only
        c[263] = 4;
        c[265] = 1;
        c[266..268].copy_from_slice(&353u16.to_le_bytes());
        c[268..270].copy_from_slice(&358u16.to_le_bytes());
        c[516..520].copy_from_slice(&1u32.to_le_bytes());
        c[525] = 1;
        let v = nvme_identify_controller(&c).unwrap();
        assert_eq!(v["model"], "Fast SSD");
        assert_eq!(
            (v["firmware_slots"].as_u64(), v["slot1_read_only"].as_bool()),
            (Some(3), Some(true))
        );
        assert_eq!(
            (
                v["warning_celsius"].as_i64(),
                v["critical_celsius"].as_i64()
            ),
            (Some(80), Some(85))
        );
        assert_eq!(
            (v["power_states"].as_u64(), v["self_test"].as_bool()),
            (Some(5), Some(true))
        );
        assert!(v.get("serial").is_none());
        assert!(nvme_identify_controller(&c[..100]).is_none());

        let mut n = vec![0u8; 4096];
        n[0..8].copy_from_slice(&1000u64.to_le_bytes());
        n[25] = 1; // two formats
        n[26] = 0; // the first is in use
        n[128..132].copy_from_slice(&(9u32 << 16 | 2 << 24).to_le_bytes());
        n[132..136].copy_from_slice(&(12u32 << 16).to_le_bytes());
        let v = nvme_identify_namespace(&n).unwrap();
        assert_eq!(v["size_bytes"].as_u64(), Some(512_000));
        assert_eq!(v["formats"][0]["block_size"].as_u64(), Some(512));
        assert_eq!(v["formats"][0]["in_use"], true);
        assert_eq!(v["formats"][1]["block_size"].as_u64(), Some(4096));
        assert_eq!(v["formats"][1]["performance"], "best");

        let mut f = vec![0u8; 512];
        f[0] = 0x21; // active slot 1, slot 2 at the next reset
        f[8..16].copy_from_slice(b"1.0.0   ");
        f[16..24].copy_from_slice(b"1.2.0   ");
        let v = nvme_firmware_log(&f).unwrap();
        assert_eq!(
            (v["active"].as_u64(), v["next"].as_u64()),
            (Some(1), Some(2))
        );
        assert_eq!(v["slots"][1]["version"], "1.2.0");
    }

    #[test]
    fn reads_self_test_logs() {
        let ata = r#"{"ata_smart_data":{"self_test":{"status":{"value":0,"string":"completed without error"}}},
            "ata_smart_self_test_log":{"standard":{"table":[{"type":{"value":1,"string":"Short offline"},
            "status":{"value":0,"string":"Completed without error","passed":true},"lifetime_hours":1200}]}}}"#;
        let v = self_tests(ata);
        assert_eq!(
            (v["supported"].as_bool(), v["running"].is_null()),
            (Some(true), true)
        );
        assert_eq!(v["tests"][0]["kind"], "Short offline");
        assert_eq!(v["tests"][0]["passed"], true);
        let nvme = r#"{"nvme_self_test_log":{"current_self_test_operation":{"value":1,"string":"Short self-test in progress"},
            "table":[{"self_test_code":{"value":2,"string":"Extended"},"self_test_result":{"value":7,"string":"Completed: failed segments"},"power_on_hours":50}]}}"#;
        let v = self_tests(nvme);
        assert_eq!(v["running"], "Short self-test in progress");
        assert_eq!(
            (
                v["tests"][0]["passed"].as_bool(),
                v["tests"][0]["hours"].as_u64()
            ),
            (Some(false), Some(50))
        );
        assert_eq!(self_tests("{}")["supported"], false);
    }
}
