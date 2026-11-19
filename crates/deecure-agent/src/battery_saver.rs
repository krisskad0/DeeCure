// SPDX-License-Identifier: Apache-2.0

//! "Power saver on battery": when the user turned it on (Power › Power Profile › Power controls, or Optimize),
//! the power mode changes to Power saver when the charger is unplugged, and back to the mode
//! before it when the charger is plugged in again.
//!
//! It runs as the user, like the "Change the power mode" fix, so it needs no password. It only
//! acts when the power source changes (and once when it starts or is turned on), so a mode the
//! user picks by hand stays until the next unplug. The mode to go back to is kept in the
//! database, so it survives a restart of the agent.

use std::path::Path;
use std::time::Duration;

use crate::collect::SystemRoot;
use crate::context::Agent;
use crate::inventory::power;
use crate::store;

/// How often the power source is read: a few small files under `/sys`.
const EVERY: Duration = Duration::from_secs(10);
/// The mode to go back to on mains, while DeeCure has Power saver on (database setting).
const BEFORE: &str = "battery_saver.before";
/// The power mode DeeCure switches to on battery.
pub const SAVER: &str = "power-saver";

/// Is the setting on? A missing or damaged file means off.
pub fn is_on(file: Option<&Path>) -> bool {
    file.and_then(|f| std::fs::read_to_string(f).ok())
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v["on"].as_bool())
        .unwrap_or(false)
}

/// Whether this computer has a system battery, and whether a charger is plugged in. `None` on
/// a computer without a battery. A charger is a `Mains` or `USB` supply that is online, the
/// same as the `power.on_mains` reading.
pub fn power_source(root: &SystemRoot) -> Option<bool> {
    let names: Vec<String> = std::fs::read_dir(root.path("/sys/class/power_supply"))
        .ok()?
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let (mut battery, mut mains) = (false, false);
    for name in names {
        let f = |k: &str| {
            root.read(&format!("/sys/class/power_supply/{name}/{k}"))
                .ok()
                .map(|v| v.trim().to_owned())
        };
        if f("scope").as_deref() == Some("Device") {
            continue;
        }
        match f("type").as_deref() {
            Some("Battery") if f("present").as_deref() != Some("0") => battery = true,
            Some("Mains" | "USB") => mains |= f("online").as_deref() == Some("1"),
            _ => {}
        }
    }
    battery.then_some(mains)
}

/// What to do now.
#[derive(Debug, PartialEq, Eq)]
pub enum Step {
    /// Leave everything as it is.
    Nothing,
    /// Switch to Power saver, and remember this mode to go back to.
    Save {
        /// The mode now.
        before: String,
    },
    /// Go back to this mode, and forget it.
    Restore {
        /// The mode before Power saver.
        to: String,
    },
    /// Forget the mode to go back to (the user changed the mode, or turned the setting off).
    Forget,
}

/// Decides what to do. `last` is the power source at the last look (`None` right after the
/// start, or right after the setting was turned on), `mains` the power source now, `mode` the
/// power mode now, and `before` the mode DeeCure will go back to, if it switched.
pub fn decide(on: bool, last: Option<bool>, mains: bool, mode: &str, before: Option<&str>) -> Step {
    if !on {
        return match before {
            Some(b) if mode == SAVER => Step::Restore { to: b.to_owned() },
            Some(_) => Step::Forget,
            None => Step::Nothing,
        };
    }
    if last == Some(mains) {
        return Step::Nothing;
    }
    match (mains, before) {
        (false, _) if mode != SAVER => Step::Save {
            before: mode.to_owned(),
        },
        (true, Some(b)) if mode == SAVER => Step::Restore { to: b.to_owned() },
        (true, Some(_)) => Step::Forget,
        _ => Step::Nothing,
    }
}

/// Runs forever: reads the setting and the power source every few seconds, and switches the
/// power mode when the power source changes.
pub async fn run(agent: std::sync::Arc<Agent>) {
    let file = deecure_core::fix::battery_saver_file();
    let mut last: Option<bool> = None;
    let mut timer = tokio::time::interval(EVERY);
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        timer.tick().await;
        let on = is_on(file.as_deref());
        let Some(mains) = power_source(&agent.root) else {
            // No battery: nothing to do, ever. Look again rarely, in case one appears.
            last = None;
            tokio::time::sleep(Duration::from_secs(600)).await;
            continue;
        };
        let before = store::lock(&agent.store)
            .setting::<Option<String>>(BEFORE)
            .ok()
            .flatten()
            .flatten();
        if !on && before.is_none() {
            last = None;
            continue;
        }
        let Some((mode, _)) = power::profile(&agent).await else {
            continue;
        };
        let step = decide(on, last, mains, &mode, before.as_deref());
        let done = match &step {
            Step::Nothing => true,
            Step::Save { before } => switch(&agent, SAVER, Some(before)).await,
            Step::Restore { to } => switch(&agent, to, None).await,
            Step::Forget => forget(&agent),
        };
        // A failed switch is tried again at the next look.
        if done {
            last = on.then_some(mains);
        }
    }
}

/// Changes the mode and remembers (or forgets) the mode to go back to.
async fn switch(agent: &Agent, to: &str, before: Option<&str>) -> bool {
    match power::set_profile(agent, to).await {
        Ok(()) => {
            eprintln!("deecure-agent: power saver on battery: the power mode is now {to}");
            let s = store::lock(&agent.store);
            let saved = match before {
                Some(b) => s.set_setting(BEFORE, b),
                None => s.set_setting(BEFORE, &Option::<String>::None),
            };
            if let Err(e) = saved {
                eprintln!("deecure-agent: could not save the power mode to go back to: {e}");
            }
            true
        }
        Err(e) => {
            eprintln!(
                "deecure-agent: power saver on battery: could not change the power mode: {e}"
            );
            false
        }
    }
}

fn forget(agent: &Agent) -> bool {
    store::lock(&agent.store)
        .set_setting(BEFORE, &Option::<String>::None)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unplugging_switches_to_power_saver_and_plugging_in_goes_back() {
        // Unplugged while Balanced.
        assert_eq!(
            decide(true, Some(true), false, "balanced", None),
            Step::Save {
                before: "balanced".into()
            }
        );
        // Still on battery: nothing more.
        assert_eq!(
            decide(true, Some(false), false, SAVER, Some("balanced")),
            Step::Nothing
        );
        // Plugged in: back to Balanced.
        assert_eq!(
            decide(true, Some(false), true, SAVER, Some("balanced")),
            Step::Restore {
                to: "balanced".into()
            }
        );
    }

    #[test]
    fn a_mode_the_user_picks_is_left_alone() {
        // On battery, the user picked Performance: no switch back to Power saver.
        assert_eq!(
            decide(true, Some(false), false, "performance", Some("balanced")),
            Step::Nothing
        );
        // Then plugged in: the user's choice stays, and the old mode is forgotten.
        assert_eq!(
            decide(true, Some(false), true, "performance", Some("balanced")),
            Step::Forget
        );
        // Already on Power saver before unplugging: nothing to switch, nothing to go back to.
        assert_eq!(decide(true, Some(true), false, SAVER, None), Step::Nothing);
        assert_eq!(decide(true, Some(false), true, SAVER, None), Step::Nothing);
    }

    #[test]
    fn the_first_look_acts_on_the_power_source_now() {
        // Turned on (or the agent started) while on battery.
        assert_eq!(
            decide(true, None, false, "balanced", None),
            Step::Save {
                before: "balanced".into()
            }
        );
        // The agent started on mains after a restart on battery.
        assert_eq!(
            decide(true, None, true, SAVER, Some("performance")),
            Step::Restore {
                to: "performance".into()
            }
        );
        assert_eq!(decide(true, None, true, "balanced", None), Step::Nothing);
    }

    #[test]
    fn turning_it_off_goes_back_only_from_power_saver() {
        assert_eq!(
            decide(false, Some(false), false, SAVER, Some("balanced")),
            Step::Restore {
                to: "balanced".into()
            }
        );
        assert_eq!(
            decide(false, Some(false), false, "performance", Some("balanced")),
            Step::Forget
        );
        assert_eq!(decide(false, None, false, "balanced", None), Step::Nothing);
    }

    #[test]
    fn the_setting_file_is_read_safely() {
        let dir = std::env::temp_dir().join(format!("deecure-saver-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("battery-saver.json");
        assert!(!is_on(Some(&file)), "a missing file means off");
        std::fs::write(&file, r#"{"on": true}"#).unwrap();
        assert!(is_on(Some(&file)));
        std::fs::write(&file, "{bad").unwrap();
        assert!(!is_on(Some(&file)), "a damaged file means off");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_power_source_counts_usb_chargers_and_skips_device_batteries() {
        use crate::collect::test_support::fixture_root;
        let root = fixture_root(
            "saver-laptop",
            &[
                ("/sys/class/power_supply/BAT0/type", "Battery\n"),
                ("/sys/class/power_supply/hidpp_battery_0/type", "Battery\n"),
                ("/sys/class/power_supply/hidpp_battery_0/scope", "Device\n"),
                (
                    "/sys/class/power_supply/ucsi-source-psy-USBC000:001/type",
                    "USB\n",
                ),
                (
                    "/sys/class/power_supply/ucsi-source-psy-USBC000:001/online",
                    "1\n",
                ),
            ],
        );
        assert_eq!(power_source(&root), Some(true));
        let desktop = fixture_root(
            "saver-desktop",
            &[
                ("/sys/class/power_supply/hidpp_battery_0/type", "Battery\n"),
                ("/sys/class/power_supply/hidpp_battery_0/scope", "Device\n"),
            ],
        );
        assert_eq!(power_source(&desktop), None);
    }
}
