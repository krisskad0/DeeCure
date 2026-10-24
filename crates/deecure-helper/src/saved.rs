// SPDX-License-Identifier: Apache-2.0

//! Power settings kept after restart (docs/02-architecture/helper.md, "Kept after restart").
//!
//! Linux forgets charge limits, turbo, and device power settings at every start. After a power
//! change works, the helper saves it in `/etc/deecure/power.json`, one entry per setting.
//! Setting a value back to the one it had before the first change removes the entry.
//! At boot, `deecure-power.service` runs `deecure-helper --apply-saved`: each entry goes through
//! the same checks as a change made in the app ([`power::plan`]), and a device setting is only
//! applied when the same device (vendor and product) is still at that place. The results go to
//! `/run/deecure/power-boot.json`, so the app can show what happened at this start.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use deecure_core::fix::Call;
use serde::{Deserialize, Serialize};

use crate::power;

/// The saved settings, under the system root.
pub const SAVED: &str = "etc/deecure/power.json";
/// What happened at this start, under the system root (`/run` is empty after each restart).
pub const RESULTS: &str = "run/deecure/power-boot.json";
/// Batteries and devices can show up a few seconds into boot.
const WAIT_ROUNDS: u32 = 20;

/// One kept setting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kept {
    /// The change to make at each start.
    pub call: Call,
    /// The setting as it was before the first change.
    pub original: Call,
    /// The device's vendor and product (`8086:a0ed`), or the battery's model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Saved {
    pub settings: BTreeMap<String, Kept>,
}

/// One line of the boot results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Outcome {
    pub setting: String,
    /// The change tried, so a later change of the same setting is not shown as done at boot.
    pub call: Call,
    pub ok: bool,
    pub message: String,
}

/// The setting a power call changes, such as `charge_limits:BAT0`. `None` for other calls.
pub fn setting(call: &Call) -> Option<String> {
    Some(match call {
        Call::SetChargeLimits { battery, .. } => format!("charge_limits:{battery}"),
        Call::SetChargeMode { battery, .. } => format!("charge_mode:{battery}"),
        Call::SetTurbo { .. } => "turbo".to_owned(),
        Call::SetDeviceWakeup { device, .. } => format!("wakeup:{device}"),
        Call::SetDevicePowerSaving { device, .. } => format!("power_saving:{device}"),
        _ => return None,
    })
}

fn read(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

/// What makes the target this exact thing: a device's vendor and product, a battery's model.
fn identity(call: &Call, root: &Path) -> Option<String> {
    match call {
        Call::SetDeviceWakeup { device, .. } | Call::SetDevicePowerSaving { device, .. } => {
            let dir = root.join("sys/devices").join(device);
            let pair =
                |a: &str, b: &str| Some(format!("{}:{}", read(&dir.join(a))?, read(&dir.join(b))?));
            pair("idVendor", "idProduct").or_else(|| pair("vendor", "device"))
        }
        Call::SetChargeLimits { battery, .. } | Call::SetChargeMode { battery, .. } => read(
            &root
                .join("sys/class/power_supply")
                .join(battery)
                .join("model_name"),
        ),
        _ => None,
    }
}

/// Is the battery, device, or CPU switch there yet?
fn present(call: &Call, root: &Path) -> bool {
    match call {
        Call::SetDeviceWakeup { device, .. } | Call::SetDevicePowerSaving { device, .. } => {
            root.join("sys/devices").join(device).exists()
        }
        Call::SetChargeLimits { battery, .. } | Call::SetChargeMode { battery, .. } => {
            root.join("sys/class/power_supply").join(battery).exists()
        }
        _ => {
            root.join("sys/devices/system/cpu/intel_pstate/no_turbo")
                .exists()
                || root.join("sys/devices/system/cpu/cpufreq/boost").exists()
        }
    }
}

fn load(path: &Path) -> Saved {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

/// Writes in one step, so a crash never leaves the file half written.
fn store(path: &Path, text: &str) -> Result<(), String> {
    let dir = path.parent().ok_or("no folder")?;
    std::fs::create_dir_all(dir).map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    let tmp = path.with_extension("deecure-tmp");
    std::fs::write(&tmp, text)
        .and_then(|()| std::fs::rename(&tmp, path))
        .map_err(|e| format!("could not write {}: {e}", path.display()))
}

/// Saves a power change that worked. `undo` is the call that puts back what was there.
pub fn remember(root: &Path, call: &Call, undo: &Call) -> Result<(), String> {
    let Some(key) = setting(call) else {
        return Ok(());
    };
    let path = root.join(SAVED);
    let mut saved = load(&path);
    let original = saved
        .settings
        .get(&key)
        .map_or_else(|| undo.clone(), |k| k.original.clone());
    if *call == original {
        saved.settings.remove(&key);
    } else {
        saved.settings.insert(
            key,
            Kept {
                call: call.clone(),
                original,
                id: identity(call, root),
            },
        );
    }
    if saved.settings.is_empty() {
        return match std::fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                Err(format!("could not remove {}: {e}", path.display()))
            }
            _ => Ok(()),
        };
    }
    let text = serde_json::to_string_pretty(&saved).map_err(|e| e.to_string())?;
    store(&path, &text)
}

/// Only a file root alone can change is trusted at boot.
pub fn trusted(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::MetadataExt;
    let meta = std::fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !meta.file_type().is_file() || meta.uid() != 0 || meta.mode() & 0o022 != 0 {
        return Err(format!(
            "{} must be a plain file owned by root that others cannot change",
            path.display()
        ));
    }
    Ok(())
}

fn apply_one(root: &Path, key: &str, kept: &Kept) -> Result<String, String> {
    if setting(&kept.call).as_deref() != Some(key) {
        return Err("the entry does not match its setting".into());
    }
    // The checks come first, so no file outside the allowed places is even read.
    let (writes, _) = power::plan(&kept.call, root).ok_or("not a power setting")??;
    if let Some(id) = &kept.id
        && identity(&kept.call, root).as_ref() != Some(id)
    {
        return Err("a different device is there now, so nothing was changed".into());
    }
    power::apply(&writes)
}

/// Applies every saved setting and returns what happened to each. Waits up to 20 seconds
/// for batteries and devices that are not there yet.
pub fn apply_saved(root: &Path, wait: Duration) -> Vec<Outcome> {
    let saved = load(&root.join(SAVED));
    let mut pending: Vec<(&String, &Kept)> = saved.settings.iter().collect();
    let mut done = Vec::new();
    for round in 0..WAIT_ROUNDS {
        let (ready, later): (Vec<_>, Vec<_>) = pending
            .into_iter()
            .partition(|(_, k)| present(&k.call, root) || round + 1 == WAIT_ROUNDS);
        for (key, kept) in ready {
            let result = apply_one(root, key, kept);
            done.push(Outcome {
                setting: key.clone(),
                call: kept.call.clone(),
                ok: result.is_ok(),
                message: result.unwrap_or_else(|e| e),
            });
        }
        pending = later;
        if pending.is_empty() {
            break;
        }
        std::thread::sleep(wait);
    }
    done
}

/// Saves the boot results for the app.
pub fn write_results(root: &Path, time_ms: u128, outcomes: &[Outcome]) -> Result<(), String> {
    let text = serde_json::json!({"time": time_ms, "results": outcomes}).to_string();
    store(&root.join(RESULTS), &text)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    /// The folder a test uses as the system root.
    fn test_root(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "deecure-helper-saved-{}-{name}",
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

    const BAT: &[(&str, &str)] = &[
        ("/sys/class/power_supply/BAT0/type", "Battery\n"),
        ("/sys/class/power_supply/BAT0/model_name", "5B10W51867\n"),
        (
            "/sys/class/power_supply/BAT0/charge_control_end_threshold",
            "100\n",
        ),
        ("/sys/devices/system/cpu/intel_pstate/no_turbo", "0\n"),
    ];

    fn limits(stop: u32) -> Call {
        Call::SetChargeLimits {
            battery: "BAT0".into(),
            start: None,
            stop,
        }
    }

    fn file(root: &Path, path: &str) -> String {
        std::fs::read_to_string(root.join(path))
            .unwrap()
            .trim()
            .to_owned()
    }

    #[test]
    fn a_change_is_kept_and_going_back_removes_it() {
        let root = test_root("keep", BAT);
        remember(&root, &limits(80), &limits(100)).unwrap();
        remember(
            &root,
            &Call::SetTurbo { on: false },
            &Call::SetTurbo { on: true },
        )
        .unwrap();
        let saved = load(&root.join(SAVED));
        assert_eq!(saved.settings.len(), 2);
        let kept = &saved.settings["charge_limits:BAT0"];
        assert_eq!(kept.original, limits(100));
        assert_eq!(kept.id.as_deref(), Some("5B10W51867"));

        // A second change keeps the first original value.
        remember(&root, &limits(60), &limits(80)).unwrap();
        let saved = load(&root.join(SAVED));
        assert_eq!(saved.settings["charge_limits:BAT0"].call, limits(60));
        assert_eq!(saved.settings["charge_limits:BAT0"].original, limits(100));

        // Back to the original: nothing to keep, and the last one removes the file.
        remember(&root, &limits(100), &limits(60)).unwrap();
        remember(
            &root,
            &Call::SetTurbo { on: true },
            &Call::SetTurbo { on: false },
        )
        .unwrap();
        assert!(!root.join(SAVED).exists());

        // Other calls are never saved.
        remember(&root, &Call::CleanAptCache, &Call::CleanAptCache).unwrap();
        assert!(!root.join(SAVED).exists());
    }

    #[test]
    fn boot_applies_with_the_same_checks_and_reports_each_result() {
        let root = test_root("boot", BAT);
        remember(&root, &limits(80), &limits(100)).unwrap();
        remember(
            &root,
            &Call::SetTurbo { on: false },
            &Call::SetTurbo { on: true },
        )
        .unwrap();
        let out = apply_saved(&root, Duration::ZERO);
        assert!(out.iter().all(|o| o.ok), "{out:?}");
        assert_eq!(
            file(
                &root,
                "sys/class/power_supply/BAT0/charge_control_end_threshold"
            ),
            "80"
        );
        assert_eq!(
            file(&root, "sys/devices/system/cpu/intel_pstate/no_turbo"),
            "1"
        );
        write_results(&root, 1, &out).unwrap();
        assert!(file(&root, RESULTS).contains("\"setting\":\"turbo\""));

        // An edited file still cannot get past the checks.
        let bad = r#"{"settings":{"charge_limits:BAT0":{"call":{"op":"set_charge_limits","battery":"BAT0","start":null,"stop":5},"original":{"op":"set_charge_limits","battery":"BAT0","start":null,"stop":100}},
            "turbo":{"call":{"op":"set_charge_mode","battery":"BAT0","mode":"Fast"},"original":{"op":"set_turbo","on":true}}}}"#;
        std::fs::write(root.join(SAVED), bad).unwrap();
        let out = apply_saved(&root, Duration::ZERO);
        assert!(out.iter().all(|o| !o.ok), "{out:?}");
        assert_eq!(
            file(
                &root,
                "sys/class/power_supply/BAT0/charge_control_end_threshold"
            ),
            "80"
        );
    }

    #[test]
    fn a_different_device_or_battery_is_left_alone() {
        let dev = "pci0000:00/0000:00:14.0/usb1/1-3";
        let mut files = BAT.to_vec();
        let wake = format!("/sys/devices/{dev}/power/wakeup");
        let vendor = format!("/sys/devices/{dev}/idVendor");
        let product = format!("/sys/devices/{dev}/idProduct");
        files.extend([
            (wake.as_str(), "enabled\n"),
            (vendor.as_str(), "046d\n"),
            (product.as_str(), "c52b\n"),
        ]);
        let root = test_root("other", &files);
        let off = Call::SetDeviceWakeup {
            device: dev.into(),
            enabled: false,
        };
        let on = Call::SetDeviceWakeup {
            device: dev.into(),
            enabled: true,
        };
        remember(&root, &off, &on).unwrap();
        remember(&root, &limits(80), &limits(100)).unwrap();
        assert_eq!(
            load(&root.join(SAVED)).settings[&format!("wakeup:{dev}")]
                .id
                .as_deref(),
            Some("046d:c52b")
        );

        // Another receiver in the same port, and a new battery.
        std::fs::write(root.join(&product[1..]), "c534\n").unwrap();
        std::fs::write(
            root.join("sys/class/power_supply/BAT0/model_name"),
            "OTHER\n",
        )
        .unwrap();
        let out = apply_saved(&root, Duration::ZERO);
        assert!(
            out.iter().all(|o| !o.ok && o.message.contains("different")),
            "{out:?}"
        );
        assert_eq!(file(&root, &wake[1..]), "enabled");
    }

    #[test]
    fn a_missing_device_is_waited_for_then_reported() {
        let root = test_root("missing", BAT);
        let dev = Call::SetDevicePowerSaving {
            device: "pci0000:00/0000:00:1f.3".into(),
            on: true,
        };
        let saved = Saved {
            settings: BTreeMap::from([(
                "power_saving:pci0000:00/0000:00:1f.3".to_owned(),
                Kept {
                    call: dev.clone(),
                    original: dev,
                    id: None,
                },
            )]),
        };
        std::fs::create_dir_all(root.join("etc/deecure")).unwrap();
        std::fs::write(root.join(SAVED), serde_json::to_string(&saved).unwrap()).unwrap();
        let out = apply_saved(&root, Duration::ZERO);
        assert_eq!(out.len(), 1);
        assert!(
            !out[0].ok && out[0].message.contains("not present"),
            "{out:?}"
        );
    }

    #[test]
    fn only_a_root_only_file_is_trusted() {
        let root = test_root("trust", &[("/etc/deecure/power.json", "{}")]);
        // Tests do not run as root, so the file belongs to someone else: refused.
        if !rustix::process::geteuid().is_root() {
            assert!(trusted(&root.join(SAVED)).is_err());
        }
        assert!(trusted(&root.join("etc/deecure/none.json")).is_err());
    }
}
