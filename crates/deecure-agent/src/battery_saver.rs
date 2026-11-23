// SPDX-License-Identifier: Apache-2.0

//! "Power saver on battery": when the user turned it on (Power › Power Profile › Power controls, or Optimize),
//! the power mode changes to Power saver when the charger is unplugged, and back to the mode
//! before it when the charger is plugged in again.
//!
//! It runs as the user, like the "Change the power mode" fix, so it needs no password. It only
//! acts when the power source changes (and once when it is turned on), so a mode the user picks
//! by hand stays until the next unplug. The mode to go back to and the last power source are
//! kept in the database, so a restart of the agent (log out and in, an update) is not taken for
//! an unplug; a charger change while the agent was not running still counts.

use std::path::Path;
use std::time::Duration;

use crate::collect::SystemRoot;
use crate::context::{Agent, Bus};
use crate::inventory::power;
use crate::store;

/// How often the power source is read: a few small files under `/sys`.
const EVERY: Duration = Duration::from_secs(10);
/// The mode to go back to on mains, while DeeCure has Power saver on (database setting).
const BEFORE: &str = "battery_saver.before";
/// The power source at the last look while the setting is on (database setting): `true` on
/// mains, `false` on battery, none when it is off or nothing was seen yet.
const LAST: &str = "battery_saver.last";
/// The power mode DeeCure switches to on battery.
pub const SAVER: &str = "power-saver";

/// Is the setting on? A missing or damaged file means off.
pub fn is_on(file: Option<&Path>) -> bool {
    file.is_some_and(deecure_core::fix::battery_saver_on)
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
    // Still on battery, but the mode is no longer Power saver: someone picked it by hand. The
    // saved mode is forgotten now, not at the plug-in, so it never replaces that choice later
    // (GNOME holds Power saver at low battery; DeeCure must not then put back the old mode).
    if !mains && last == Some(false) && before.is_some() && mode != SAVER {
        return Step::Forget;
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

/// A mode to go back to that the computer no longer offers (after a driver or
/// power-profiles-daemon change) would fail at every look, forever. Balanced, which every
/// computer with power modes has, is put back instead. `offered` empty means not known.
pub fn offered_or_balanced(step: Step, offered: &[String]) -> Step {
    match step {
        Step::Restore { to } if !offered.is_empty() && !offered.contains(&to) => {
            eprintln!(
                "deecure-agent: power saver on battery: {to} is no longer offered, so Balanced is put back"
            );
            Step::Restore {
                to: "balanced".into(),
            }
        }
        other => other,
    }
}

/// Runs forever: reads the setting and the power source every few seconds, and switches the
/// power mode when the power source changes.
pub async fn run(agent: std::sync::Arc<Agent>) {
    let file = deecure_core::fix::battery_saver_file();
    let mut last: Option<bool> = store::lock(&agent.store)
        .setting::<Option<bool>>(LAST)
        .ok()
        .flatten()
        .flatten();
    // What the database holds now, so it is written only when the power source changes.
    let mut kept = last;
    let mut timer = tokio::time::interval(EVERY);
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        timer.tick().await;
        last = look(&agent, file.as_deref(), last).await;
        if last != kept && store::lock(&agent.store).set_setting(LAST, &last).is_ok() {
            kept = last;
        }
    }
}

/// One look: switches the power mode if needed, and gives the power source to remember.
///
/// The setting comes first: a small file and one database value. The power supply files are
/// ACPI or firmware calls (a few ms each), so they are read only when there is work to do: the
/// setting is on, or a mode is waiting to be put back. A user who never turns it on costs
/// nothing but this check.
async fn look(agent: &Agent, file: Option<&Path>, last: Option<bool>) -> Option<bool> {
    let on = is_on(file);
    let before = store::lock(&agent.store)
        .setting::<Option<String>>(BEFORE)
        .ok()
        .flatten()
        .flatten();
    if !on && before.is_none() {
        return None;
    }
    // No battery: nothing to switch.
    let mains = power_source(&agent.root)?;
    // The power mode is one for the whole computer. With two people signed in, only the one at
    // the screen switches it; the other waits, and acts on a change once back at the screen.
    if !at_screen(agent).await {
        return last;
    }
    let Some((mode, offered)) = power::profile(agent).await else {
        return last;
    };
    let step = offered_or_balanced(decide(on, last, mains, &mode, before.as_deref()), &offered);
    let done = match &step {
        Step::Nothing => true,
        Step::Save { before } => switch(agent, SAVER, Some(before)).await,
        Step::Restore { to } => switch(agent, to, None).await,
        Step::Forget => forget(agent),
    };
    // A failed switch is tried again at the next look.
    if done { on.then_some(mains) } else { last }
}

/// Is this user at the screen? From logind's `State` for this user: `active` when one of the
/// user's sessions is in front (a locked screen still counts, and so does a session with no
/// seat, such as SSH). `true` when logind cannot say, as on a computer with one user.
async fn at_screen(agent: &Agent) -> bool {
    let Ok(bus) = agent.bus(Bus::System).await else {
        return true;
    };
    let Ok(reply) = bus
        .call_method(
            Some("org.freedesktop.login1"),
            "/org/freedesktop/login1/user/self",
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.freedesktop.login1.User", "State"),
        )
        .await
    else {
        return true;
    };
    let state = reply
        .body()
        .deserialize::<zbus::OwnedVariant>()
        .ok()
        .and_then(|v| String::try_from(v).ok());
    user_at_screen(state.as_deref())
}

/// `false` only when logind says the user is signed in but not at the screen (another person
/// is), or is leaving.
pub fn user_at_screen(state: Option<&str>) -> bool {
    !matches!(state, Some("online" | "lingering" | "closing" | "offline"))
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
        // On battery, the user picked Performance: no switch back to Power saver, and the saved
        // Balanced is forgotten at once, so it can never replace Performance later.
        assert_eq!(
            decide(true, Some(false), false, "performance", Some("balanced")),
            Step::Forget
        );
        // After that, on battery with nothing saved: nothing to do.
        assert_eq!(
            decide(true, Some(false), false, "performance", None),
            Step::Nothing
        );
        // GNOME then holds Power saver at low battery, and the charger is plugged in: nothing
        // saved, so DeeCure leaves it, and GNOME puts back Performance.
        assert_eq!(decide(true, Some(false), true, SAVER, None), Step::Nothing);
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
        // Turned on while on battery (or the very first start: no power source kept yet).
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
    fn a_restart_on_the_same_power_source_changes_nothing() {
        // On battery the user picked Performance, then the agent restarted: the kept power
        // source is still "battery", so the user's mode stays (the old saved mode is forgotten).
        assert_eq!(
            decide(true, Some(false), false, "performance", Some("balanced")),
            Step::Forget
        );
        assert_eq!(
            decide(true, Some(false), false, "performance", None),
            Step::Nothing
        );
        // Unplugged while the agent was not running: that still counts as an unplug.
        assert_eq!(
            decide(true, Some(true), false, "balanced", None),
            Step::Save {
                before: "balanced".into()
            }
        );
    }

    #[test]
    fn a_mode_no_longer_offered_is_replaced_by_balanced() {
        let offered: Vec<String> = ["power-saver", "balanced"].map(String::from).to_vec();
        let back = |to: &str| Step::Restore { to: to.into() };
        assert_eq!(
            offered_or_balanced(back("performance"), &offered),
            back("balanced")
        );
        assert_eq!(
            offered_or_balanced(back("balanced"), &offered),
            back("balanced")
        );
        // Not known: left as it is. Other steps: untouched.
        assert_eq!(
            offered_or_balanced(back("performance"), &[]),
            back("performance")
        );
        assert_eq!(offered_or_balanced(Step::Forget, &offered), Step::Forget);
    }

    #[test]
    fn only_the_person_at_the_screen_switches() {
        assert!(user_at_screen(Some("active")));
        // Signed in, but another person is at the screen.
        assert!(!user_at_screen(Some("online")));
        assert!(!user_at_screen(Some("closing")));
        // logind cannot say: one user, so act.
        assert!(user_at_screen(None));
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
