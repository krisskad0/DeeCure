// SPDX-License-Identifier: Apache-2.0

//! Power details: batteries, the power profile, battery policies, and sleep settings.

use std::collections::HashMap;
use std::time::Duration;

use serde::Serialize;
use zbus::zvariant::OwnedValue;

use super::command;
use crate::collect::SystemRoot;
use crate::context::{Agent, Bus};

/// One system battery.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct BatteryInfo {
    pub name: String,
    /// Charging, Discharging, Full, Not charging, Unknown.
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge: Option<f64>,
    /// Watt-hours it holds now when full, and when new.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_wh: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub design_wh: Option<f64>,
    /// Missing when the battery does not count (many report 0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycles: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Charging stops at this percent, when the laptop supports a limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_limit: Option<u32>,
}

/// UPower's battery policy, from `/etc/UPower/UPower.conf` (UPower's defaults when unset).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Policy {
    pub low_percent: u32,
    pub critical_percent: u32,
    pub action_percent: u32,
    /// What happens at `action_percent`: HybridSleep, Hibernate, PowerOff, Ignore.
    pub critical_action: String,
}

/// The `power` inventory.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PowerInfo {
    pub batteries: Vec<BatteryInfo>,
    /// True when on mains power. Missing on machines without a mains supply entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_ac: Option<bool>,
    /// `power-saver`, `balanced`, or `performance`. Missing when power-profiles-daemon is not running.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    pub profiles: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<Policy>,
    /// Seconds idle before the screen blanks; 0 = never.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screen_blank_seconds: Option<u32>,
    /// Seconds idle on battery before suspend; 0 = never.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspend_on_battery_seconds: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspend_on_ac_seconds: Option<u32>,
    /// What closing the lid does, from logind: suspend, ignore, poweroff, …
    pub lid_action: String,
    /// The sleep kind in use: `s2idle` or `deep`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_mode: Option<String>,
}

/// The last `key=value` for `key`, so later files and lines override earlier ones.
fn value(text: &str, key: &str) -> Option<String> {
    text.lines()
        .rev()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .find_map(|l| l.strip_prefix(key)?.trim_start().strip_prefix('='))
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

/// Reads UPower's settings, falling back to its built-in defaults.
pub fn policy(conf: &str) -> Policy {
    let num = |k: &str, default: u32| {
        value(conf, k)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    };
    Policy {
        low_percent: num("PercentageLow", 20),
        critical_percent: num("PercentageCritical", 5),
        action_percent: num("PercentageAction", 2),
        critical_action: value(conf, "CriticalPowerAction").unwrap_or_else(|| "HybridSleep".into()),
    }
}

/// The active choice in `/sys/power/mem_sleep`, such as `[s2idle] deep`.
pub fn bracketed(text: &str) -> Option<String> {
    text.split_whitespace()
        .find_map(|w| w.strip_prefix('[')?.strip_suffix(']').map(str::to_owned))
}

fn batteries(root: &SystemRoot) -> (Vec<BatteryInfo>, Option<bool>) {
    let mut list = Vec::new();
    let mut on_ac = None;
    let dir = std::fs::read_dir(root.path("/sys/class/power_supply"))
        .map(|d| {
            d.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for name in dir {
        let f = |k: &str| {
            root.read(&format!("/sys/class/power_supply/{name}/{k}"))
                .ok()
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
        };
        let num = |k: &str| f(k).and_then(|v| v.parse::<f64>().ok());
        match f("type").as_deref() {
            Some("Mains") => {
                on_ac = Some(on_ac.unwrap_or(false) || f("online").as_deref() == Some("1"))
            }
            Some("Battery") if f("scope").as_deref() != Some("Device") => {
                // Energy in µWh, or charge in µAh times the design voltage in µV.
                let volts = num("voltage_min_design").map(|v| v / 1e6);
                let wh = |energy: &str, charge: &str| {
                    num(energy)
                        .map(|e| e / 1e6)
                        .or_else(|| Some(num(charge)? / 1e6 * volts?))
                };
                list.push(BatteryInfo {
                    status: f("status").unwrap_or_else(|| "Unknown".into()),
                    charge: num("capacity"),
                    full_wh: wh("energy_full", "charge_full"),
                    design_wh: wh("energy_full_design", "charge_full_design"),
                    cycles: num("cycle_count").filter(|c| *c > 0.0).map(|c| c as u32),
                    technology: f("technology"),
                    manufacturer: f("manufacturer"),
                    model: f("model_name"),
                    charge_limit: num("charge_control_end_threshold").map(|c| c as u32),
                    name,
                });
            }
            _ => {}
        }
    }
    list.sort_by(|a, b| a.name.cmp(&b.name));
    (list, on_ac)
}

async fn profile(agent: &Agent) -> Option<(String, Vec<String>)> {
    // The new name first; older releases only have the second.
    for (service, object) in [
        (
            "org.freedesktop.UPower.PowerProfiles",
            "/org/freedesktop/UPower/PowerProfiles",
        ),
        ("net.hadess.PowerProfiles", "/net/hadess/PowerProfiles"),
    ] {
        if !agent.is_running(Bus::System, service).await {
            continue;
        }
        let bus = agent.bus(Bus::System).await.ok()?;
        let reply = bus
            .call_method(
                Some(service),
                object,
                Some("org.freedesktop.DBus.Properties"),
                "GetAll",
                &(service,),
            )
            .await
            .ok()?;
        let props: HashMap<String, OwnedValue> = reply.body().deserialize().ok()?;
        let active = props
            .get("ActiveProfile")
            .and_then(|v| v.try_clone().ok())
            .and_then(|v| String::try_from(v).ok())?;
        let profiles = props
            .get("Profiles")
            .and_then(|v| v.try_clone().ok())
            .and_then(|v| Vec::<HashMap<String, OwnedValue>>::try_from(v).ok())
            .map(|list| {
                list.into_iter()
                    .filter_map(|p| {
                        p.get("Profile")
                            .and_then(|v| v.try_clone().ok())
                            .and_then(|v| String::try_from(v).ok())
                    })
                    .collect()
            })
            .unwrap_or_default();
        return Some((active, profiles));
    }
    None
}

async fn gsetting(schema: &str, key: &str) -> Option<String> {
    let out = command::output("gsettings", &["get", schema, key], Duration::from_secs(3)).await?;
    Some(
        out.trim()
            .trim_start_matches("uint32 ")
            .trim_start_matches("int32 ")
            .trim_matches('\'')
            .to_owned(),
    )
}

/// Reads the power details.
pub async fn read(agent: &Agent) -> PowerInfo {
    let root = &agent.root;
    let (batteries, on_ac) = batteries(root);
    let (profile, profiles) = profile(agent)
        .await
        .map_or((None, Vec::new()), |(a, p)| (Some(a), p));
    let power = "org.gnome.settings-daemon.plugins.power";
    // A suspend timeout only counts when the action is "suspend".
    let suspend = |timeout: Option<String>, kind: Option<String>| match kind.as_deref() {
        Some("nothing") => Some(0),
        _ => timeout.and_then(|t| t.parse().ok()),
    };
    PowerInfo {
        batteries,
        on_ac,
        profile,
        profiles,
        // No UPower config means UPower is not installed, so no policy applies.
        policy: root
            .read("/etc/UPower/UPower.conf")
            .ok()
            .map(|conf| policy(&conf)),
        screen_blank_seconds: gsetting("org.gnome.desktop.session", "idle-delay")
            .await
            .and_then(|v| v.parse().ok()),
        suspend_on_battery_seconds: suspend(
            gsetting(power, "sleep-inactive-battery-timeout").await,
            gsetting(power, "sleep-inactive-battery-type").await,
        ),
        suspend_on_ac_seconds: suspend(
            gsetting(power, "sleep-inactive-ac-timeout").await,
            gsetting(power, "sleep-inactive-ac-type").await,
        ),
        lid_action: super::logind::lid(root).on_battery,
        sleep_mode: root
            .read("/sys/power/mem_sleep")
            .ok()
            .as_deref()
            .and_then(bracketed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn reads_policy_with_defaults() {
        let p = policy(
            "[UPower]\n# PercentageLow=99\nPercentageLow=10\nCriticalPowerAction=PowerOff\n",
        );
        assert_eq!(
            (
                p.low_percent,
                p.critical_percent,
                p.critical_action.as_str()
            ),
            (10, 5, "PowerOff")
        );
        assert_eq!(policy("").critical_action, "HybridSleep");
        assert_eq!(bracketed("[s2idle] deep\n").as_deref(), Some("s2idle"));
    }

    #[test]
    fn reads_batteries_in_energy_or_charge_units() {
        let root = fixture_root(
            "power",
            &[
                ("/sys/class/power_supply/AC/type", "Mains\n"),
                ("/sys/class/power_supply/AC/online", "0\n"),
                ("/sys/class/power_supply/BAT0/type", "Battery\n"),
                ("/sys/class/power_supply/BAT0/status", "Discharging\n"),
                ("/sys/class/power_supply/BAT0/capacity", "64\n"),
                ("/sys/class/power_supply/BAT0/energy_full", "40500000\n"),
                (
                    "/sys/class/power_supply/BAT0/energy_full_design",
                    "57000000\n",
                ),
                ("/sys/class/power_supply/BAT0/cycle_count", "0\n"),
                ("/sys/class/power_supply/BAT1/type", "Battery\n"),
                ("/sys/class/power_supply/BAT1/charge_full", "4000000\n"),
                (
                    "/sys/class/power_supply/BAT1/charge_full_design",
                    "5000000\n",
                ),
                (
                    "/sys/class/power_supply/BAT1/voltage_min_design",
                    "11400000\n",
                ),
                ("/sys/class/power_supply/hidpp_battery_0/type", "Battery\n"),
                ("/sys/class/power_supply/hidpp_battery_0/scope", "Device\n"),
            ],
        );
        let (list, ac) = batteries(&root);
        assert_eq!(ac, Some(false));
        assert_eq!(list.len(), 2, "the mouse battery is left out");
        assert_eq!(
            (list[0].full_wh, list[0].design_wh, list[0].cycles),
            (Some(40.5), Some(57.0), None)
        );
        assert!((list[1].design_wh.unwrap() - 57.0).abs() < 1e-9);
    }
}
