// SPDX-License-Identifier: Apache-2.0

//! Power in depth for the Expert level (docs/01-product/expert-coverage.md, sections 7a and 7b):
//! each battery's voltage, current, power, temperature, health, manufacture date, charge limits,
//! charge modes; each charger and what a USB-C charger offers (Power Delivery); the suspend
//! types offered; every wake source with what it belongs to and how often it fired; and each
//! USB device's power saving. From `/sys`, no admin rights.

use serde::{Deserialize, Serialize};

use crate::collect::SystemRoot;

/// A setting with the choices the hardware offers, such as `Trickle Fast [Custom]`.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Choice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
    pub offered: Vec<String>,
}

/// One battery.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct BatteryDetail {
    pub name: String,
    /// Volts now and the design minimum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volts: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volts_min_design: Option<f64>,
    /// Amps now (always positive; `status` says the direction).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amps: Option<f64>,
    /// Watts now: `power_now`, or volts times amps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watts: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_c: Option<f64>,
    /// The battery's own health word, such as `Good`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<String>,
    /// `YYYY-MM-DD` (or `YYYY-MM`, `YYYY`), as far as the battery reports it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub made: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,
    /// Charging starts below `charge_start` and stops at `charge_stop`, in percent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_start: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_stop: Option<u32>,
    /// Charge modes, such as Dell's Standard, Adaptive, Fast, Trickle, Custom.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_mode: Option<Choice>,
    /// Charging behaviour, such as `auto`, `inhibit-charge`, `force-discharge`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_behaviour: Option<Choice>,
}

/// One USB Power Delivery offer: a voltage and the most current or power at it.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PdOffer {
    /// `fixed_supply`, `variable_supply`, `battery`, or `programmable_supply` (PPS).
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub millivolts: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_millivolts: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milliamps: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milliwatts: Option<u64>,
}

/// One USB-C port's Power Delivery: what the other side offers, and what this computer takes.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PowerDelivery {
    pub name: String,
    pub offered: Vec<PdOffer>,
    pub accepted: Vec<PdOffer>,
}

/// One power source other than a battery.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Charger {
    pub name: String,
    /// `Mains` or `USB`.
    pub kind: String,
    pub online: bool,
    /// USB type in use, such as `PD`, and the ones offered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usb_type: Option<Choice>,
    /// Most watts at its current voltage and current, when it reports them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_watts: Option<f64>,
}

/// One wake source.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct WakeSource {
    pub name: String,
    /// The device path under `/sys/devices`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pci_address: Option<String>,
    /// The USB product name, when it is a USB device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    /// Times it signalled, times it stopped a suspend from starting or finishing (the kernel's
    /// `wakeup_count`), and times it kept the computer awake.
    pub events: u64,
    pub wakeups: u64,
    pub active: u64,
    /// Allowed to wake the computer from suspend (`power/wakeup` of its device).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub may_wake: Option<bool>,
}

/// One USB device's power saving.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct UsbPower {
    /// The USB path, such as `3-6`.
    pub port: String,
    /// Its path under `/sys/devices`, as the helper's power controls take it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sys_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maker: Option<String>,
    /// `auto` (saves power) or `on` (always awake).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// A keyboard, mouse, or other input device, or a sound device: power saving is never
    /// offered for these, because they can miss keys or crackle.
    pub keep_awake: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub may_wake: Option<bool>,
}

/// One radio from rfkill.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Radio {
    pub name: String,
    /// `wlan`, `bluetooth`, `wwan`, …
    pub kind: String,
    /// Turned off in software (can be turned on), or by a hardware switch or key.
    pub soft_blocked: bool,
    pub hard_blocked: bool,
}

/// The keyboard backlight.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct KbdBacklight {
    pub level: u32,
    pub max: u32,
}

/// A power setting the helper sets again at each start (`/etc/deecure/power.json`), and what
/// happened to it at this start (`/run/deecure/power-boot.json`).
#[derive(Debug, Serialize)]
pub struct KeptSetting {
    /// Such as `charge_limits:BAT0`, `turbo`, or `wakeup:<device>`.
    pub setting: String,
    pub call: deecure_core::fix::Call,
    /// `None` when it was kept after this start, so it was not applied at boot yet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_ok: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_message: Option<String>,
}

#[derive(Deserialize)]
struct SavedFile {
    settings: std::collections::BTreeMap<String, SavedEntry>,
}

#[derive(Deserialize)]
struct SavedEntry {
    call: deecure_core::fix::Call,
}

#[derive(Deserialize)]
struct BootFile {
    results: Vec<BootResult>,
}

#[derive(Deserialize)]
struct BootResult {
    setting: String,
    call: Option<deecure_core::fix::Call>,
    ok: bool,
    message: String,
}

/// The kept settings with this start's results. Unreadable files count as none.
pub fn kept(root: &SystemRoot) -> Vec<KeptSetting> {
    let Some(saved) = root
        .read("/etc/deecure/power.json")
        .ok()
        .and_then(|t| serde_json::from_str::<SavedFile>(&t).ok())
    else {
        return Vec::new();
    };
    let boot = root
        .read("/run/deecure/power-boot.json")
        .ok()
        .and_then(|t| serde_json::from_str::<BootFile>(&t).ok())
        .map(|b| b.results)
        .unwrap_or_default();
    saved
        .settings
        .into_iter()
        .map(|(setting, entry)| {
            // A result for an earlier value of the same setting does not count.
            let result = boot
                .iter()
                .find(|r| r.setting == setting && r.call.as_ref() == Some(&entry.call));
            KeptSetting {
                boot_ok: result.map(|r| r.ok),
                boot_message: result.map(|r| r.message.clone()),
                setting,
                call: entry.call,
            }
        })
        .collect()
}

/// What `power_detail` answers.
#[derive(Debug, Default, Serialize)]
pub struct PowerDetail {
    pub batteries: Vec<BatteryDetail>,
    pub chargers: Vec<Charger>,
    pub power_delivery: Vec<PowerDelivery>,
    /// Suspend types: the one in use and the ones offered (`s2idle`, `deep`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspend: Option<Choice>,
    /// The PCIe power saving policy (ASPM): the one in use and the ones offered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspm_policy: Option<Choice>,
    pub wake_sources: Vec<WakeSource>,
    /// The interrupt that last woke the computer from suspend, and its device from
    /// `/proc/interrupts`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_wake_irq: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_wake_device: Option<String>,
    pub usb: Vec<UsbPower>,
    pub radios: Vec<Radio>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyboard_backlight: Option<KbdBacklight>,
    /// Power settings DeeCure sets again at each start.
    pub kept: Vec<KeptSetting>,
}

/// A device's path under `/sys/devices`, from a link such as `/sys/bus/usb/devices/3-6`.
fn sys_path(root: &SystemRoot, link: &str) -> Option<String> {
    let real = std::fs::canonicalize(root.path(link)).ok()?;
    let base = std::fs::canonicalize(root.path("/sys/devices")).ok()?;
    real.strip_prefix(&base)
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

fn text(root: &SystemRoot, path: &str) -> Option<String> {
    root.read(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn number(root: &SystemRoot, path: &str) -> Option<u64> {
    text(root, path)?
        .trim_end_matches("mV")
        .trim_end_matches("mA")
        .trim_end_matches("mW")
        .parse()
        .ok()
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

/// `Trickle Fast [Custom]`: the one in brackets is in use.
pub fn choice(text: &str) -> Choice {
    let offered: Vec<String> = text
        .split_whitespace()
        .map(|s| s.trim_matches(['[', ']']).to_owned())
        .collect();
    let current = text
        .split_whitespace()
        .find(|s| s.starts_with('['))
        .map(|s| s.trim_matches(['[', ']']).to_owned());
    // A file with one word and no brackets holds just the value in use.
    let current = current.or_else(|| (offered.len() == 1).then(|| offered[0].clone()));
    Choice { current, offered }
}

fn battery(root: &SystemRoot, name: &str) -> BatteryDetail {
    let at = |f: &str| format!("/sys/class/power_supply/{name}/{f}");
    let micro = |f: &str| number(root, &at(f)).map(|v| v as f64 / 1e6);
    let volts = micro("voltage_now");
    let amps = micro("current_now");
    let watts = micro("power_now").or(match (volts, amps) {
        (Some(v), Some(a)) => Some(v * a),
        _ => None,
    });
    let made = match (
        number(root, &at("manufacture_year")),
        number(root, &at("manufacture_month")),
        number(root, &at("manufacture_day")),
    ) {
        (Some(y), Some(m), Some(d)) if y > 0 && m > 0 && d > 0 => {
            Some(format!("{y}-{m:02}-{d:02}"))
        }
        (Some(y), Some(m), _) if y > 0 && m > 0 => Some(format!("{y}-{m:02}")),
        (Some(y), _, _) if y > 0 => Some(y.to_string()),
        _ => None,
    };
    BatteryDetail {
        name: name.to_owned(),
        volts,
        volts_min_design: micro("voltage_min_design"),
        amps,
        watts,
        status: text(root, &at("status")),
        temp_c: text(root, &at("temp"))
            .and_then(|t| t.parse::<f64>().ok())
            .map(|t| t / 10.0),
        health: text(root, &at("health")),
        made,
        serial: text(root, &at("serial_number")),
        charge_start: number(root, &at("charge_control_start_threshold")).map(|v| v as u32),
        charge_stop: number(root, &at("charge_control_end_threshold")).map(|v| v as u32),
        charge_mode: text(root, &at("charge_types")).map(|t| choice(&t)),
        charge_behaviour: text(root, &at("charge_behaviour")).map(|t| choice(&t)),
    }
}

fn pd_offers(root: &SystemRoot, dir: &str) -> Vec<PdOffer> {
    names(root, dir)
        .into_iter()
        .filter_map(|entry| {
            let (_, kind) = entry.split_once(':')?;
            let at = |f: &str| format!("{dir}/{entry}/{f}");
            Some(PdOffer {
                kind: kind.to_owned(),
                millivolts: number(root, &at("voltage"))
                    .or_else(|| number(root, &at("minimum_voltage"))),
                max_millivolts: number(root, &at("maximum_voltage")),
                milliamps: number(root, &at("maximum_current"))
                    .or_else(|| number(root, &at("operational_current"))),
                milliwatts: number(root, &at("maximum_power"))
                    .or_else(|| number(root, &at("operational_power"))),
            })
        })
        .collect()
}

/// The last PCI address in a device path, such as `0000:00:14.0`.
fn pci_in(path: &str) -> Option<String> {
    path.split('/')
        .rfind(|c| {
            c.len() == 12
                && c.as_bytes().get(4) == Some(&b':')
                && c.as_bytes().get(7) == Some(&b':')
        })
        .map(str::to_owned)
}

fn wake_sources(root: &SystemRoot) -> Vec<WakeSource> {
    names(root, "/sys/class/wakeup")
        .into_iter()
        .filter_map(|w| {
            let at = |f: &str| format!("/sys/class/wakeup/{w}/{f}");
            let real = std::fs::canonicalize(root.path(&at("device"))).ok();
            let device = real.as_ref().and_then(|p| {
                let s = p.to_string_lossy();
                s.find("/sys/devices/")
                    .map(|i| s[i + "/sys/devices/".len()..].to_owned())
            });
            let product = real
                .as_ref()
                .and_then(|p| std::fs::read_to_string(p.join("product")).ok())
                .map(|s| s.trim().to_owned());
            let may_wake = real
                .as_ref()
                .and_then(|p| std::fs::read_to_string(p.join("power/wakeup")).ok())
                .map(|s| s.trim() == "enabled")
                .filter(|_| {
                    real.as_ref()
                        .is_some_and(|p| p.join("power/wakeup").exists())
                });
            Some(WakeSource {
                name: text(root, &at("name"))?,
                pci_address: device.as_deref().and_then(pci_in),
                product,
                events: number(root, &at("event_count")).unwrap_or(0),
                wakeups: number(root, &at("wakeup_count")).unwrap_or(0),
                active: number(root, &at("active_count")).unwrap_or(0),
                may_wake,
                device,
            })
        })
        .collect()
}

/// USB interface classes that must stay awake: HID (03, keyboards and mice) and audio (01).
const KEEP_AWAKE: &[&str] = &["03", "01"];

fn usb(root: &SystemRoot) -> Vec<UsbPower> {
    let base = "/sys/bus/usb/devices";
    let all = names(root, base);
    all.iter()
        // Devices, not hubs' root (`usb1`) or interfaces (`3-6:1.0`).
        .filter(|n| !n.starts_with("usb") && !n.contains(':'))
        .map(|port| {
            let at = |f: &str| format!("{base}/{port}/{f}");
            let interfaces: Vec<String> = all
                .iter()
                .filter(|i| i.starts_with(&format!("{port}:")))
                .filter_map(|i| text(root, &format!("{base}/{i}/bInterfaceClass")))
                .collect();
            let class = text(root, &at("bDeviceClass"));
            let keep_awake = interfaces
                .iter()
                .chain(class.iter())
                .any(|c| KEEP_AWAKE.contains(&c.as_str()));
            UsbPower {
                sys_path: sys_path(root, &format!("{base}/{port}")),
                port: port.clone(),
                product: text(root, &at("product")),
                maker: text(root, &at("manufacturer")),
                control: text(root, &at("power/control")),
                state: text(root, &at("power/runtime_status")),
                keep_awake,
                may_wake: text(root, &at("power/wakeup")).map(|w| w == "enabled"),
            }
        })
        .collect()
}

/// Reads `power_detail`.
pub fn read(root: &SystemRoot) -> PowerDetail {
    let supplies = names(root, "/sys/class/power_supply");
    let kind =
        |n: &str| text(root, &format!("/sys/class/power_supply/{n}/type")).unwrap_or_default();
    // Mouse and keyboard batteries have scope `Device`; only the system's own batteries count.
    let system = |n: &str| {
        text(root, &format!("/sys/class/power_supply/{n}/scope")).is_none_or(|s| s != "Device")
    };
    let last_wake_irq = text(root, "/sys/power/pm_wakeup_irq").filter(|v| v != "0");
    let last_wake_device = last_wake_irq.as_ref().and_then(|irq| {
        let list = root.read("/proc/interrupts").ok()?;
        super::cpu_detail::parse_interrupts(&list)
            .into_iter()
            .find(|i| &i.irq == irq)
            .map(|i| i.device)
    });
    let radios = names(root, "/sys/class/rfkill")
        .into_iter()
        .filter_map(|r| {
            let at = |f: &str| format!("/sys/class/rfkill/{r}/{f}");
            Some(Radio {
                name: text(root, &at("name")).unwrap_or_else(|| r.clone()),
                kind: text(root, &at("type"))?,
                soft_blocked: text(root, &at("soft")).as_deref() == Some("1"),
                hard_blocked: text(root, &at("hard")).as_deref() == Some("1"),
            })
        })
        .collect();
    let keyboard_backlight = names(root, "/sys/class/leds")
        .into_iter()
        .find(|l| l.ends_with("kbd_backlight"))
        .and_then(|l| {
            Some(KbdBacklight {
                level: number(root, &format!("/sys/class/leds/{l}/brightness"))? as u32,
                max: number(root, &format!("/sys/class/leds/{l}/max_brightness"))? as u32,
            })
        });
    PowerDetail {
        kept: kept(root),
        radios,
        keyboard_backlight,
        last_wake_irq,
        last_wake_device,
        batteries: supplies
            .iter()
            .filter(|n| kind(n) == "Battery" && system(n))
            .map(|n| battery(root, n))
            .collect(),
        chargers: supplies
            .iter()
            .filter(|n| matches!(kind(n).as_str(), "Mains" | "USB"))
            .map(|n| {
                let at = |f: &str| format!("/sys/class/power_supply/{n}/{f}");
                let micro = |f: &str| number(root, &at(f)).map(|v| v as f64 / 1e6);
                Charger {
                    name: n.clone(),
                    kind: kind(n),
                    online: text(root, &at("online")).as_deref() == Some("1"),
                    usb_type: text(root, &at("usb_type")).map(|t| choice(&t)),
                    max_watts: match (micro("voltage_max"), micro("current_max")) {
                        (Some(v), Some(a)) if v * a > 0.0 => Some(v * a),
                        _ => None,
                    },
                }
            })
            .collect(),
        power_delivery: names(root, "/sys/class/usb_power_delivery")
            .into_iter()
            .map(|pd| PowerDelivery {
                offered: pd_offers(
                    root,
                    &format!("/sys/class/usb_power_delivery/{pd}/source-capabilities"),
                ),
                accepted: pd_offers(
                    root,
                    &format!("/sys/class/usb_power_delivery/{pd}/sink-capabilities"),
                ),
                name: pd,
            })
            .collect(),
        suspend: text(root, "/sys/power/mem_sleep").map(|t| choice(&t)),
        aspm_policy: text(root, "/sys/module/pcie_aspm/parameters/policy").map(|t| choice(&t)),
        wake_sources: wake_sources(root),
        usb: usb(root),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn choices_show_the_one_in_use() {
        assert_eq!(
            choice("Trickle Fast [Custom]"),
            Choice {
                current: Some("Custom".into()),
                offered: vec!["Trickle".into(), "Fast".into(), "Custom".into()]
            }
        );
        assert_eq!(choice("[s2idle]").current.as_deref(), Some("s2idle"));
        assert_eq!(
            choice("auto").current.as_deref(),
            Some("auto"),
            "one word is the value in use"
        );
        assert_eq!(
            pci_in("pci0000:00/0000:00:14.0/usb3/3-6"),
            Some("0000:00:14.0".into())
        );
    }

    #[test]
    fn reads_a_dell_laptop() {
        let b = "/sys/class/power_supply/BAT0";
        let root = fixture_root(
            "power-detail",
            &[
                (&format!("{b}/type"), "Battery\n"),
                (&format!("{b}/status"), "Discharging\n"),
                (&format!("{b}/voltage_now"), "11969000\n"),
                (&format!("{b}/voltage_min_design"), "11400000\n"),
                (&format!("{b}/current_now"), "1361000\n"),
                (&format!("{b}/temp"), "325\n"),
                (&format!("{b}/health"), "Good\n"),
                (&format!("{b}/manufacture_year"), "2024\n"),
                (&format!("{b}/manufacture_month"), "3\n"),
                (&format!("{b}/charge_control_start_threshold"), "75\n"),
                (&format!("{b}/charge_control_end_threshold"), "80\n"),
                (
                    &format!("{b}/charge_types"),
                    "Trickle Fast Standard Adaptive [Custom]\n",
                ),
                ("/sys/class/power_supply/hidpp_battery_0/type", "Battery\n"),
                ("/sys/class/power_supply/hidpp_battery_0/scope", "Device\n"),
                ("/sys/class/power_supply/AC/type", "Mains\n"),
                ("/sys/class/power_supply/AC/online", "0\n"),
                (
                    "/sys/class/power_supply/ucsi-source-psy-USBC000:001/type",
                    "USB\n",
                ),
                (
                    "/sys/class/power_supply/ucsi-source-psy-USBC000:001/online",
                    "1\n",
                ),
                (
                    "/sys/class/power_supply/ucsi-source-psy-USBC000:001/usb_type",
                    "C [PD] PD_PPS\n",
                ),
                (
                    "/sys/class/power_supply/ucsi-source-psy-USBC000:001/voltage_max",
                    "20000000\n",
                ),
                (
                    "/sys/class/power_supply/ucsi-source-psy-USBC000:001/current_max",
                    "3250000\n",
                ),
                (
                    "/sys/class/usb_power_delivery/pd0/source-capabilities/1:fixed_supply/voltage",
                    "20000mV\n",
                ),
                (
                    "/sys/class/usb_power_delivery/pd0/source-capabilities/1:fixed_supply/maximum_current",
                    "3250mA\n",
                ),
                (
                    "/sys/class/usb_power_delivery/pd0/sink-capabilities/1:fixed_supply/voltage",
                    "5000mV\n",
                ),
                ("/sys/power/mem_sleep", "[s2idle]\n"),
                (
                    "/sys/module/pcie_aspm/parameters/policy",
                    "[default] performance powersave powersupersave\n",
                ),
                ("/sys/power/pm_wakeup_irq", "9\n"),
                (
                    "/proc/interrupts",
                    "           CPU0\n   9:     10   IR-IO-APIC    9-fasteoi   acpi\n",
                ),
                ("/sys/class/wakeup/wakeup5/name", "device:90\n"),
                ("/sys/class/wakeup/wakeup5/event_count", "12\n"),
                ("/sys/class/wakeup/wakeup5/wakeup_count", "3\n"),
                (
                    "/sys/bus/usb/devices/3-1/product",
                    "Dell Universal Receiver\n",
                ),
                ("/sys/bus/usb/devices/3-1/power/control", "on\n"),
                ("/sys/bus/usb/devices/3-1/bDeviceClass", "00\n"),
                ("/sys/bus/usb/devices/3-1:1.0/bInterfaceClass", "03\n"),
                (
                    "/sys/bus/usb/devices/3-6/product",
                    "Integrated_Webcam_FHD\n",
                ),
                ("/sys/bus/usb/devices/3-6/power/control", "auto\n"),
                ("/sys/bus/usb/devices/3-6/bDeviceClass", "ef\n"),
                ("/sys/bus/usb/devices/3-6:1.0/bInterfaceClass", "0e\n"),
                (
                    "/sys/bus/usb/devices/usb3/product",
                    "xHCI Host Controller\n",
                ),
            ],
        );
        let p = read(&root);
        assert_eq!(p.batteries.len(), 1, "mouse batteries are left out");
        let bat = &p.batteries[0];
        assert!(
            (bat.watts.unwrap() - 11.969 * 1.361).abs() < 0.001,
            "volts times amps"
        );
        assert_eq!(
            (bat.temp_c, bat.made.as_deref(), bat.health.as_deref()),
            (Some(32.5), Some("2024-03"), Some("Good"))
        );
        assert_eq!((bat.charge_start, bat.charge_stop), (Some(75), Some(80)));
        assert_eq!(
            bat.charge_mode.as_ref().unwrap().current.as_deref(),
            Some("Custom")
        );
        assert_eq!(bat.charge_behaviour, None);
        assert_eq!(p.chargers.len(), 2);
        let usb_c = p.chargers.iter().find(|c| c.kind == "USB").unwrap();
        assert!(usb_c.online);
        assert_eq!(usb_c.max_watts, Some(65.0));
        assert_eq!(
            usb_c.usb_type.as_ref().unwrap().current.as_deref(),
            Some("PD")
        );
        assert_eq!(p.power_delivery[0].offered[0].millivolts, Some(20_000));
        assert_eq!(p.power_delivery[0].offered[0].milliamps, Some(3250));
        assert_eq!(p.suspend.unwrap().offered, ["s2idle"]);
        assert_eq!(p.aspm_policy.unwrap().current.as_deref(), Some("default"));
        assert_eq!(p.last_wake_irq.as_deref(), Some("9"));
        assert!(p.last_wake_device.unwrap().ends_with("acpi"));
        assert_eq!(
            (p.wake_sources[0].name.as_str(), p.wake_sources[0].wakeups),
            ("device:90", 3)
        );
        assert_eq!(
            p.usb.len(),
            2,
            "host controllers and interfaces are not devices"
        );
        assert!(
            p.usb[0].keep_awake,
            "a receiver for a keyboard and mouse stays awake"
        );
        assert!(!p.usb[1].keep_awake);
    }

    #[test]
    fn kept_settings_show_this_starts_result() {
        let root = fixture_root(
            "power-kept",
            &[
                (
                    "/etc/deecure/power.json",
                    r#"{"settings":{"charge_limits:BAT0":{"call":{"op":"set_charge_limits","battery":"BAT0","start":null,"stop":80},"original":{"op":"set_charge_limits","battery":"BAT0","start":null,"stop":100},"id":"5B10W51867"},
                        "turbo":{"call":{"op":"set_turbo","on":false},"original":{"op":"set_turbo","on":true}}}}"#,
                ),
                (
                    "/run/deecure/power-boot.json",
                    r#"{"time":1,"results":[{"setting":"charge_limits:BAT0","call":{"op":"set_charge_limits","battery":"BAT0","start":null,"stop":80},"ok":false,"message":"a different device is there now, so nothing was changed"},
                        {"setting":"turbo","call":{"op":"set_turbo","on":true},"ok":true,"message":"no_turbo = 0"}]}"#,
                ),
            ],
        );
        let k = kept(&root);
        assert_eq!(k.len(), 2);
        assert_eq!(k[0].setting, "charge_limits:BAT0");
        assert_eq!(k[0].boot_ok, Some(false));
        assert_eq!(k[1].call, deecure_core::fix::Call::SetTurbo { on: false });
        assert_eq!(k[1].boot_ok, None, "the boot set an earlier value");
        assert!(kept(&fixture_root("power-kept-none", &[])).is_empty());
    }
}
