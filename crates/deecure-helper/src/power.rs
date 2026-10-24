// SPDX-License-Identifier: Apache-2.0

//! Power settings the helper writes to `/sys` (docs/01-product/expert-coverage.md, section 7c).
//!
//! Only these files can be written, each after strict checks against the live system:
//!
//! - `/sys/class/power_supply/<battery>/charge_control_{start,end}_threshold` and `charge_types`
//! - `/sys/devices/system/cpu/intel_pstate/no_turbo` or `/sys/devices/system/cpu/cpufreq/boost`
//! - `/sys/devices/<device>/power/wakeup` and `/sys/devices/<device>/power/control`
//!
//! A device path must resolve inside `/sys/devices`. The lid, power and sleep buttons, and
//! keyboards are never stopped from waking the computer; keyboards, mice, and sound devices never
//! get power saving (they can miss keys or crackle). Every write returns the call that puts the
//! old value back. All paths go through `root`, so tests use a folder instead of `/sys`.

use std::path::{Path, PathBuf};

use deecure_core::fix::Call;

/// One file and the value to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Write {
    pub path: PathBuf,
    pub value: String,
}

fn read(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_owned())
}

/// `BAT0`, `BAT1`, `BATT`, `CMB0`: the names Linux gives system batteries.
fn is_battery_name(name: &str) -> bool {
    let digits = name
        .trim_start_matches("BATT")
        .trim_start_matches("BAT")
        .trim_start_matches("CMB");
    (name.starts_with("BAT") || name.starts_with("CMB"))
        && digits.len() <= 2
        && digits.chars().all(|c| c.is_ascii_digit())
}

fn battery_dir(root: &Path, battery: &str) -> Result<PathBuf, String> {
    if !is_battery_name(battery) {
        return Err("not a battery name".into());
    }
    let dir = root.join("sys/class/power_supply").join(battery);
    if read(&dir.join("type")).as_deref() != Some("Battery") {
        return Err(format!("{battery} is not a battery"));
    }
    Ok(dir)
}

/// The device folder for a path under `/sys/devices`, after resolving links.
fn device_dir(root: &Path, device: &str) -> Result<PathBuf, String> {
    let allowed = |c: char| c.is_ascii_alphanumeric() || ":._-/".contains(c);
    if device.is_empty()
        || device.starts_with('/')
        || device.contains("..")
        || !device.chars().all(allowed)
        || device.len() > 256
    {
        return Err("not a device path".into());
    }
    let base = root
        .join("sys/devices")
        .canonicalize()
        .map_err(|_| "no /sys/devices".to_owned())?;
    let dir = base
        .join(device)
        .canonicalize()
        .map_err(|_| format!("device {device} is not present"))?;
    if !dir.starts_with(&base) {
        return Err("the device is outside /sys/devices".into());
    }
    Ok(dir)
}

/// Interface classes and protocols of a USB device: `(class, protocol)` per interface.
fn usb_interfaces(dir: &Path) -> Vec<(String, String)> {
    let Some(name) = dir.file_name().and_then(|n| n.to_str()) else {
        return Vec::new();
    };
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.starts_with(&format!("{name}:")))
        })
        .filter_map(|e| {
            Some((
                read(&e.path().join("bInterfaceClass"))?,
                read(&e.path().join("bInterfaceProtocol")).unwrap_or_default(),
            ))
        })
        .collect()
}

/// The lid, a power or sleep button, or a keyboard: these always keep waking the computer.
fn must_keep_waking(dir: &Path) -> bool {
    let path = dir.to_string_lossy();
    [
        "PNP0C0D", "PNP0C0C", "PNP0C0E", "LNXPWRBN", "LNXSLPBN", "serio", "i8042",
    ]
    .iter()
    .any(|k| path.contains(k))
        || usb_interfaces(dir)
            .iter()
            .any(|(class, protocol)| class == "03" && protocol == "01")
}

/// A keyboard, mouse, or other input device, or a sound device: these never get power saving.
fn must_stay_awake(dir: &Path) -> bool {
    let usb = usb_interfaces(dir);
    let class = read(&dir.join("class")).unwrap_or_default();
    usb.iter().any(|(c, _)| c == "03" || c == "01")
        || read(&dir.join("bDeviceClass")).as_deref() == Some("01")
        || class.trim_start_matches("0x").starts_with("04")
}

/// Checks a power call and returns the writes, in order, and the call that undoes them.
/// `None` when the call is not a power call.
pub fn plan(call: &Call, root: &Path) -> Option<Result<(Vec<Write>, Call), String>> {
    Some(match call {
        Call::SetChargeLimits {
            battery,
            start,
            stop,
        } => (|| {
            let dir = battery_dir(root, battery)?;
            let stop_file = dir.join("charge_control_end_threshold");
            let start_file = dir.join("charge_control_start_threshold");
            let old_stop: u32 = read(&stop_file)
                .and_then(|v| v.parse().ok())
                .ok_or(format!("{battery} has no charge limit"))?;
            let old_start: Option<u32> = read(&start_file).and_then(|v| v.parse().ok());
            if !(20..=100).contains(stop) {
                return Err("the stop limit must be between 20 and 100%".into());
            }
            let mut writes = Vec::new();
            match (start, old_start) {
                (Some(s), Some(_)) => {
                    if *s >= *stop {
                        return Err("charging must start below the stop limit".into());
                    }
                    // Write in an order that never has start at or above stop.
                    let start_w = Write {
                        path: start_file,
                        value: s.to_string(),
                    };
                    let stop_w = Write {
                        path: stop_file,
                        value: stop.to_string(),
                    };
                    if *s < old_stop {
                        writes.extend([start_w, stop_w]);
                    } else {
                        writes.extend([stop_w, start_w]);
                    }
                }
                (Some(_), None) => return Err(format!("{battery} has no start limit")),
                (None, _) => writes.push(Write {
                    path: stop_file,
                    value: stop.to_string(),
                }),
            }
            Ok((
                writes,
                Call::SetChargeLimits {
                    battery: battery.clone(),
                    start: old_start.filter(|_| start.is_some()),
                    stop: old_stop,
                },
            ))
        })(),
        Call::SetChargeMode { battery, mode } => (|| {
            let file = battery_dir(root, battery)?.join("charge_types");
            let text = read(&file).ok_or(format!("{battery} has no charge modes"))?;
            let offered: Vec<&str> = text
                .split_whitespace()
                .map(|m| m.trim_matches(['[', ']']))
                .collect();
            let current = text
                .split_whitespace()
                .find(|m| m.starts_with('['))
                .map(|m| m.trim_matches(['[', ']']).to_owned());
            if !offered.contains(&mode.as_str()) {
                return Err(format!("{battery} does not offer the {mode} mode"));
            }
            let old = current.ok_or("the current charge mode is not reported")?;
            Ok((
                vec![Write {
                    path: file,
                    value: mode.clone(),
                }],
                Call::SetChargeMode {
                    battery: battery.clone(),
                    mode: old,
                },
            ))
        })(),
        Call::SetTurbo { on } => {
            let no_turbo = root.join("sys/devices/system/cpu/intel_pstate/no_turbo");
            let boost = root.join("sys/devices/system/cpu/cpufreq/boost");
            if let Some(v) = read(&no_turbo) {
                let was_on = v == "0";
                Ok((
                    vec![Write {
                        path: no_turbo,
                        value: if *on { "0" } else { "1" }.into(),
                    }],
                    Call::SetTurbo { on: was_on },
                ))
            } else if let Some(v) = read(&boost) {
                let was_on = v == "1";
                Ok((
                    vec![Write {
                        path: boost,
                        value: if *on { "1" } else { "0" }.into(),
                    }],
                    Call::SetTurbo { on: was_on },
                ))
            } else {
                Err("this CPU has no turbo switch".into())
            }
        }
        Call::SetDeviceWakeup { device, enabled } => (|| {
            let dir = device_dir(root, device)?;
            let file = dir.join("power/wakeup");
            let old = read(&file).ok_or("this device cannot wake the computer")?;
            if !*enabled && must_keep_waking(&dir) {
                return Err(
                    "the lid, power buttons, and keyboards always keep waking the computer".into(),
                );
            }
            Ok((
                vec![Write {
                    path: file,
                    value: if *enabled { "enabled" } else { "disabled" }.into(),
                }],
                Call::SetDeviceWakeup {
                    device: device.clone(),
                    enabled: old == "enabled",
                },
            ))
        })(),
        Call::SetDevicePowerSaving { device, on } => (|| {
            let dir = device_dir(root, device)?;
            let file = dir.join("power/control");
            let old = read(&file).ok_or("this device has no power saving")?;
            if *on && must_stay_awake(&dir) {
                return Err("keyboards, mice, and sound devices stay awake, so they never miss input or crackle".into());
            }
            Ok((
                vec![Write {
                    path: file,
                    value: if *on { "auto" } else { "on" }.into(),
                }],
                Call::SetDevicePowerSaving {
                    device: device.clone(),
                    on: old == "auto",
                },
            ))
        })(),
        _ => return None,
    })
}

/// Writes each value. Stops at the first failure and says which file failed.
pub fn apply(writes: &[Write]) -> Result<String, String> {
    for (i, w) in writes.iter().enumerate() {
        std::fs::write(&w.path, &w.value).map_err(|e| {
            let done = if i > 0 {
                format!(
                    " ({i} earlier change{} stayed)",
                    if i == 1 { "" } else { "s" }
                )
            } else {
                String::new()
            };
            format!(
                "could not set {} to {}: {e}{done}",
                w.path.display(),
                w.value
            )
        })?;
    }
    Ok(writes
        .iter()
        .map(|w| {
            format!(
                "{} = {}",
                w.path
                    .file_name()
                    .map_or_else(String::new, |n| n.to_string_lossy().into_owned()),
                w.value
            )
        })
        .collect::<Vec<_>>()
        .join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "deecure-helper-power-{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        for (path, content) in files {
            let f = dir.join(path.trim_start_matches('/'));
            std::fs::create_dir_all(f.parent().unwrap()).unwrap();
            std::fs::write(f, content).unwrap();
        }
        std::fs::create_dir_all(dir.join("sys/devices")).unwrap();
        dir
    }

    fn values(writes: &[Write]) -> Vec<(String, String)> {
        writes
            .iter()
            .map(|w| {
                (
                    w.path.file_name().unwrap().to_string_lossy().into_owned(),
                    w.value.clone(),
                )
            })
            .collect()
    }

    #[test]
    fn charge_limits_are_checked_and_written_in_a_safe_order() {
        let b = "/sys/class/power_supply/BAT0";
        let r = root(
            "charge",
            &[
                (&format!("{b}/type"), "Battery\n"),
                (&format!("{b}/charge_control_start_threshold"), "50\n"),
                (&format!("{b}/charge_control_end_threshold"), "80\n"),
            ],
        );
        let (w, undo) = plan(
            &Call::SetChargeLimits {
                battery: "BAT0".into(),
                start: Some(85),
                stop: 90,
            },
            &r,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            values(&w),
            [
                ("charge_control_end_threshold".into(), "90".into()),
                ("charge_control_start_threshold".into(), "85".into())
            ],
            "stop first when the new start is above the old stop"
        );
        assert_eq!(
            undo,
            Call::SetChargeLimits {
                battery: "BAT0".into(),
                start: Some(50),
                stop: 80
            }
        );
        let (w, _) = plan(
            &Call::SetChargeLimits {
                battery: "BAT0".into(),
                start: Some(40),
                stop: 60,
            },
            &r,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            w[0].path.file_name().unwrap(),
            "charge_control_start_threshold"
        );
        for bad in [
            Call::SetChargeLimits {
                battery: "BAT0".into(),
                start: Some(80),
                stop: 80,
            },
            Call::SetChargeLimits {
                battery: "BAT0".into(),
                start: None,
                stop: 10,
            },
            Call::SetChargeLimits {
                battery: "../../etc".into(),
                start: None,
                stop: 80,
            },
            Call::SetChargeLimits {
                battery: "AC".into(),
                start: None,
                stop: 80,
            },
            Call::SetChargeLimits {
                battery: "BAT1".into(),
                start: None,
                stop: 80,
            },
        ] {
            assert!(plan(&bad, &r).unwrap().is_err(), "{bad:?}");
        }
    }

    #[test]
    fn charge_modes_must_be_offered() {
        let b = "/sys/class/power_supply/BAT0";
        let r = root(
            "mode",
            &[
                (&format!("{b}/type"), "Battery\n"),
                (
                    &format!("{b}/charge_types"),
                    "Trickle Fast Standard Adaptive [Custom]\n",
                ),
            ],
        );
        let (w, undo) = plan(
            &Call::SetChargeMode {
                battery: "BAT0".into(),
                mode: "Adaptive".into(),
            },
            &r,
        )
        .unwrap()
        .unwrap();
        assert_eq!(values(&w), [("charge_types".into(), "Adaptive".into())]);
        assert_eq!(
            undo,
            Call::SetChargeMode {
                battery: "BAT0".into(),
                mode: "Custom".into()
            }
        );
        assert!(
            plan(
                &Call::SetChargeMode {
                    battery: "BAT0".into(),
                    mode: "Turbo".into()
                },
                &r
            )
            .unwrap()
            .is_err()
        );
    }

    #[test]
    fn turbo_uses_the_switch_the_cpu_has() {
        let r = root(
            "turbo",
            &[("/sys/devices/system/cpu/intel_pstate/no_turbo", "0\n")],
        );
        let (w, undo) = plan(&Call::SetTurbo { on: false }, &r).unwrap().unwrap();
        assert_eq!(values(&w), [("no_turbo".into(), "1".into())]);
        assert_eq!(undo, Call::SetTurbo { on: true });
        let r = root("boost", &[("/sys/devices/system/cpu/cpufreq/boost", "0\n")]);
        assert_eq!(
            values(&plan(&Call::SetTurbo { on: true }, &r).unwrap().unwrap().0),
            [("boost".into(), "1".into())]
        );
        let r = root("none", &[]);
        assert!(plan(&Call::SetTurbo { on: true }, &r).unwrap().is_err());
    }

    #[test]
    fn devices_stay_inside_sys_devices_and_protected_ones_are_refused() {
        let d = "/sys/devices/pci0000:00/0000:00:14.0";
        let r = root(
            "devices",
            &[
                (&format!("{d}/power/wakeup"), "enabled\n"),
                (&format!("{d}/power/control"), "on\n"),
                (&format!("{d}/usb3/3-1/power/wakeup"), "enabled\n"),
                (&format!("{d}/usb3/3-1/power/control"), "on\n"),
                (&format!("{d}/usb3/3-1/3-1:1.0/bInterfaceClass"), "03\n"),
                (&format!("{d}/usb3/3-1/3-1:1.0/bInterfaceProtocol"), "01\n"),
                (
                    "/sys/devices/LNXSYSTM:00/LNXSYBUS:00/PNP0C0D:00/power/wakeup",
                    "enabled\n",
                ),
                ("/etc/shadow", "secret\n"),
            ],
        );
        let (w, undo) = plan(
            &Call::SetDeviceWakeup {
                device: "pci0000:00/0000:00:14.0".into(),
                enabled: false,
            },
            &r,
        )
        .unwrap()
        .unwrap();
        assert_eq!(values(&w), [("wakeup".into(), "disabled".into())]);
        assert_eq!(
            undo,
            Call::SetDeviceWakeup {
                device: "pci0000:00/0000:00:14.0".into(),
                enabled: true
            }
        );
        let (w, _) = plan(
            &Call::SetDevicePowerSaving {
                device: "pci0000:00/0000:00:14.0".into(),
                on: true,
            },
            &r,
        )
        .unwrap()
        .unwrap();
        assert_eq!(values(&w), [("control".into(), "auto".into())]);
        let refused = [
            Call::SetDeviceWakeup {
                device: "pci0000:00/0000:00:14.0/usb3/3-1".into(),
                enabled: false,
            },
            Call::SetDeviceWakeup {
                device: "LNXSYSTM:00/LNXSYBUS:00/PNP0C0D:00".into(),
                enabled: false,
            },
            Call::SetDevicePowerSaving {
                device: "pci0000:00/0000:00:14.0/usb3/3-1".into(),
                on: true,
            },
            Call::SetDeviceWakeup {
                device: "../../etc".into(),
                enabled: false,
            },
            Call::SetDeviceWakeup {
                device: "/etc".into(),
                enabled: false,
            },
            Call::SetDeviceWakeup {
                device: "pci0000:00/missing".into(),
                enabled: false,
            },
            Call::SetDeviceWakeup {
                device: "pci0000:00/0000:00:14.0; rm -rf /".into(),
                enabled: false,
            },
        ];
        for bad in refused {
            assert!(plan(&bad, &r).unwrap().is_err(), "{bad:?}");
        }
        // A keyboard may be allowed to wake the computer again, and kept awake.
        assert!(
            plan(
                &Call::SetDeviceWakeup {
                    device: "pci0000:00/0000:00:14.0/usb3/3-1".into(),
                    enabled: true
                },
                &r
            )
            .unwrap()
            .is_ok()
        );
        assert!(
            plan(
                &Call::SetDevicePowerSaving {
                    device: "pci0000:00/0000:00:14.0/usb3/3-1".into(),
                    on: false
                },
                &r
            )
            .unwrap()
            .is_ok()
        );
    }

    #[test]
    fn a_link_out_of_sys_devices_is_refused() {
        let r = root("escape", &[("/outside/power/wakeup", "enabled\n")]);
        std::os::unix::fs::symlink(r.join("outside"), r.join("sys/devices/escape")).unwrap();
        assert!(
            plan(
                &Call::SetDeviceWakeup {
                    device: "escape".into(),
                    enabled: false
                },
                &r
            )
            .unwrap()
            .is_err()
        );
    }

    #[test]
    fn other_calls_are_not_power_calls_and_writes_report_files() {
        assert!(plan(&Call::CleanAptCache, Path::new("/")).is_none());
        let r = root("apply", &[("/x/charge_control_end_threshold", "80\n")]);
        let msg = apply(&[Write {
            path: r.join("x/charge_control_end_threshold"),
            value: "75".into(),
        }])
        .unwrap();
        assert_eq!(msg, "charge_control_end_threshold = 75");
        assert_eq!(
            read(&r.join("x/charge_control_end_threshold")).as_deref(),
            Some("75")
        );
        assert!(
            apply(&[Write {
                path: r.join("missing/dir/file"),
                value: "1".into()
            }])
            .unwrap_err()
            .contains("could not set")
        );
    }
}
