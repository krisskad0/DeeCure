// SPDX-License-Identifier: Apache-2.0

//! Checks on stored readings over time (Optimize batch 2): the power source over 14 days, heat
//! slowdowns on battery, swapping, the power mode, and wake-ups with the lid closed. Each says so
//! when there are not enough readings yet, instead of guessing.

use std::collections::HashMap;
use std::time::Duration;

use super::super::{command, power, power_detail};
use super::cleanup::series;
use super::{Check, advice, count, fix, slowdowns};
use crate::context::Agent;

const HOUR: i64 = 3_600_000;
const DAY: i64 = 24 * HOUR;

/// The share of time on mains in `points` (0 to 1; 15-minute and daily points are averages, so
/// the mean is still the share), and how many days the points cover.
pub fn mains_share(points: &[(i64, f64)], now: i64) -> Option<(f64, f64)> {
    let first = points.first()?.0;
    let share = points.iter().map(|(_, v)| v).sum::<f64>() / points.len() as f64;
    Some((share, (now - first) as f64 / DAY as f64))
}

/// Minutes on battery, and those with heat slowdowns, from the two counters taken together.
pub fn battery_slowdowns(mains: &[(i64, f64)], throttle: &[(i64, f64)]) -> (usize, usize) {
    let on: HashMap<i64, f64> = mains.iter().copied().collect();
    let mut minutes = 0;
    let mut slowed = 0;
    for w in throttle.windows(2) {
        if on.get(&w[1].0).is_some_and(|v| *v < 0.5) {
            minutes += 1;
            if slowdowns(&[w[0].1, w[1].1]) > 0 {
                slowed += 1;
            }
        }
    }
    (minutes, slowed)
}

/// The longest run of readings in a row where both swap-ins and hard faults stay high.
pub fn longest_swap_storm(swap_ins: &[(i64, f64)], faults: &[(i64, f64)]) -> usize {
    let f: HashMap<i64, f64> = faults.iter().copied().collect();
    let (mut run, mut best) = (0, 0);
    for (at, s) in swap_ins {
        if *s >= 50.0 && f.get(at).is_some_and(|v| *v >= 100.0) {
            run += 1;
            best = best.max(run);
        } else {
            run = 0;
        }
    }
    best
}

/// Wake-ups from suspend with the lid closed, from the system log: a resume (`PM: suspend
/// exit`) while the lid was closed, with no "Lid opened." in the next minute (opening the lid
/// to wake the laptop logs the resume first).
pub fn lid_closed_wakes(log: &str) -> usize {
    let lines: Vec<(f64, &str)> = log
        .lines()
        .filter_map(|l| {
            let (t, rest) = l.split_once(' ')?;
            Some((t.parse().ok()?, rest))
        })
        .collect();
    let mut closed = false;
    let mut count = 0;
    for (i, (t, text)) in lines.iter().enumerate() {
        if text.contains("Lid closed") {
            closed = true;
        } else if text.contains("Lid opened") {
            closed = false;
        } else if text.contains("PM: suspend exit") && closed {
            let opened_soon = lines[i + 1..]
                .iter()
                .take_while(|(t2, _)| t2 - t <= 60.0)
                .any(|(_, x)| x.contains("Lid opened"));
            if !opened_soon {
                count += 1;
            }
        }
    }
    count
}

pub async fn checks(agent: &Agent) -> Vec<Check> {
    let now = deecure_core::Timestamp::now().0;
    let detail = power_detail::read(&agent.root);
    let info = power::read(agent).await;
    let mut out = Vec::new();
    let mains = series(agent, "power.on_mains", 14 * DAY);
    let usage = match mains_share(&mains, now) {
        Some((share, days)) if days >= 1.0 => format!(
            " On mains {:.0}% of the time over the last {} days.",
            share * 100.0,
            days.round().min(14.0)
        ),
        _ => String::new(),
    };
    // Every battery (a ThinkPad can have two), not only the first.
    for b in &detail.batteries {
        out.extend(charge_checks(b, &usage));
    }
    if let Some(b) = detail.batteries.first() {
        let throttle = series(agent, "cpu.throttle_total", 7 * DAY);
        let (minutes, slowed) = battery_slowdowns(&mains, &throttle);
        let often = minutes >= 60 && slowed * 10 >= minutes;
        out.push(fix(
            "power.battery_turbo",
            "Turn off turbo on battery",
            if minutes < 60 {
                format!("{} on battery in the last 7 days: not enough to judge.", count(minutes, "minute"))
            } else {
                format!("The CPU slowed down from heat in {slowed} of {minutes} minutes on battery in the last 7 days.")
            },
            often,
            "power.set_battery_turbo",
            Some("off".into()),
            "admin",
        ));
        let week: Vec<(i64, f64)> = mains
            .iter()
            .filter(|(t, _)| now - t <= 7 * DAY)
            .copied()
            .collect();
        let usage = battery_usage(agent, &b.name, &info, &week, now);
        out.push(battery_saver(
            &info,
            minutes_on_battery(&week),
            detail.battery_saver,
            usage.as_deref(),
        ));
        out.push(power_mode(&info, &mains, now));
        out.push(lid_wakes(&detail).await);
    }
    out.push(away(&info, !detail.batteries.is_empty()));
    out.push(zswap(agent));
    out
}

/// The charge limit and charge mode items for one battery. `usage` is the time on mains, as a
/// sentence with a space before it, or empty.
///
/// A limit counts only when it is in use. A battery that offers a Custom charge mode follows
/// its limit only in Custom; in any other mode its limit files still show the stored Custom
/// values (Dell), which do nothing, so that battery charges to 100%.
///
/// - The 80% limit is offered when no limit is in use. A lower limit in use, such as 90% from
///   Power controls, was chosen and is left alone. When the battery needs Custom for the limit
///   to work, the text says so, and the preview also sets Custom.
/// - Fast charging gets a gentler mode, except when the 80% item will set Custom itself: a
///   gentler mode would then turn the limit off again.
pub fn charge_checks(b: &power_detail::BatteryDetail, usage: &str) -> Vec<Check> {
    let mut out = Vec::new();
    let custom = power_detail::limit_needs_custom(b.charge_mode.as_ref());
    let in_use = b.charge_stop.filter(|s| *s < 100 && custom.is_none());
    if b.charge_stop.is_some() {
        let subject = format!(
            "{}:{}:80",
            b.name,
            if b.charge_start.is_some() { "75" } else { "-" }
        );
        let wear = format!(
            "Stopping at 80% slows its wear; each charge then lasts about a fifth less.{usage}"
        );
        let measured = match (in_use, &custom) {
            (Some(stop), _) => format!("Battery {} already stops at {stop}%.", b.name),
            (None, Some(now)) => format!(
                "Battery {} charges to 100%: it follows a charge limit only in the Custom charge mode, and the mode is {now}. {wear} The fix also sets the Custom mode.",
                b.name
            ),
            (None, None) => format!("Battery {} charges to 100%. {wear}", b.name),
        };
        out.push(fix(
            &format!("power.charge_limit:{}", b.name),
            "Limit battery charge to 80%",
            measured,
            in_use.is_none(),
            "power.set_charge_limits",
            Some(subject),
            "admin",
        ));
    }
    if let Some(mode) = &b.charge_mode {
        let now = mode.current.as_deref().unwrap_or("not reported");
        let fast = mode.current.as_deref() == Some("Fast");
        let gentle = ["Adaptive", "Standard"]
            .into_iter()
            .find(|m| mode.offered.iter().any(|o| o == m));
        let (measured, offered) = if fast && custom.is_some() && b.charge_stop.is_some() {
            (
                "The charge mode is Fast. \"Limit battery charge to 80%\" also sets the Custom mode, which stops at the limit.".to_owned(),
                false,
            )
        } else if fast {
            (
                "The charge mode is Fast: it charges quickly but wears the battery more."
                    .to_owned(),
                gentle.is_some(),
            )
        } else {
            (format!("The charge mode is {now}."), false)
        };
        out.push(fix(
            &format!("power.charge_mode:{}", b.name),
            "Use a gentler charge mode",
            measured,
            offered,
            "power.set_charge_mode",
            gentle
                .filter(|_| offered)
                .map(|m| format!("{}:{m}", b.name)),
            "admin",
        ));
    }
    out
}

/// Minutes on battery in `mains` (one point a minute in the last 7 days).
pub fn minutes_on_battery(mains: &[(i64, f64)]) -> usize {
    mains.iter().filter(|(_, v)| *v < 0.5).count()
}

/// The average draw on battery in watts, and the minutes it covers: only readings taken while no
/// charger was plugged in, matched by time with the power source readings.
pub fn battery_draw(mains: &[(i64, f64)], watts: &[(i64, f64)]) -> Option<(f64, usize)> {
    let on: HashMap<i64, f64> = mains.iter().copied().collect();
    let draws: Vec<f64> = watts
        .iter()
        .filter(|(t, w)| *w > 0.0 && on.get(t).is_some_and(|v| *v < 0.5))
        .map(|(_, w)| *w)
        .collect();
    (!draws.is_empty()).then(|| (draws.iter().sum::<f64>() / draws.len() as f64, draws.len()))
}

/// "5 minutes", "1 minute".
fn minutes(seconds: u32) -> String {
    count((seconds / 60) as usize, "minute")
}

/// What is wrong with the screen and sleep settings, if anything: the screen never turns off or
/// stays on over 15 minutes; on battery the laptop never sleeps or waits over 30 minutes.
pub fn away_needs(
    screen: Option<u32>,
    sleep: Option<u32>,
    battery: bool,
) -> (Option<String>, Option<String>) {
    let screen = match screen {
        Some(0) => Some("the screen never turns off by itself".to_owned()),
        Some(s) if s > 900 => Some(format!(
            "the screen stays on for {} without use",
            minutes(s)
        )),
        _ => None,
    };
    let sleep = match sleep.filter(|_| battery) {
        Some(0) => Some("on battery the laptop never sleeps by itself".to_owned()),
        Some(s) if s > 1800 => Some(format!(
            "on battery the laptop sleeps only after {} without use",
            minutes(s)
        )),
        _ => None,
    };
    (screen, sleep)
}

/// One item for both settings that save power only while nobody uses the computer: the screen
/// turning off, and sleep on battery. The fix preview has a step for each one needed.
fn away(info: &power::PowerInfo, battery: bool) -> Check {
    let (screen, sleep) = away_needs(
        info.screen_blank_seconds,
        info.suspend_on_battery_seconds,
        battery,
    );
    let offered = screen.is_some() || sleep.is_some();
    let measured = if offered {
        let facts: Vec<String> = [screen, sleep].into_iter().flatten().collect();
        let text = facts.join(", and ");
        let mut first = text.chars();
        format!(
            "{}{}. {} power only while you are away: video, calls, and apps that ask to stay awake still keep the screen and the laptop on.",
            first
                .next()
                .map(|c| c.to_uppercase().to_string())
                .unwrap_or_default(),
            first.as_str(),
            if facts.len() == 1 {
                "The fix saves"
            } else {
                "Both fixes save"
            }
        )
    } else {
        match (info.screen_blank_seconds, info.suspend_on_battery_seconds) {
            (None, _) => "The screen and sleep settings cannot be read (GNOME only).".to_owned(),
            (Some(s), Some(b)) if battery => format!(
                "The screen turns off after {} without use, and on battery the laptop sleeps after {}.",
                minutes(s),
                minutes(b)
            ),
            (Some(s), _) => format!("The screen turns off after {} without use.", minutes(s)),
        }
    };
    fix(
        "power.away",
        "Save power when you are away",
        measured,
        offered,
        "power.save_when_away",
        None,
        "you",
    )
}

fn battery_saver(info: &power::PowerInfo, minutes: usize, on: bool, usage: Option<&str>) -> Check {
    let title = "Use Power saver on battery";
    let usage = usage.map(|u| format!(" {u}")).unwrap_or_default();
    let (offered, measured) = if info.profile.is_none() {
        (
            false,
            "The power mode is not available (power-profiles-daemon).".to_owned(),
        )
    } else if on {
        (
            false,
            format!("Power saver already turns on by itself when you unplug.{usage}"),
        )
    } else if minutes < 60 {
        (
            false,
            format!(
                "{} on battery in the last 7 days: not enough to judge.",
                count(minutes, "minute")
            ),
        )
    } else {
        (
            true,
            format!(
                "{} on battery in the last 7 days.{usage} The power mode stays the same when you unplug.",
                hours_text(minutes as f64 / 60.0)
            ),
        )
    };
    fix(
        "power.battery_saver",
        title,
        measured,
        offered,
        "power.set_battery_saver",
        Some("on".into()),
        "you",
    )
}

/// The measured use on battery in the last 7 days, in one sentence: the average draw, the share
/// of the battery an hour, and how long a full charge lasts. `None` with under 60 minutes.
fn battery_usage(
    agent: &Agent,
    battery: &str,
    info: &power::PowerInfo,
    mains: &[(i64, f64)],
    now: i64,
) -> Option<String> {
    let watts: Vec<(i64, f64)> = crate::store::lock(&agent.store)
        .metrics(
            "battery.power",
            Some(battery),
            deecure_core::Timestamp(now - 7 * DAY),
            deecure_core::Timestamp(now),
        )
        .unwrap_or_default()
        .into_iter()
        .map(|m| (m.at.0, m.value))
        .collect();
    let full = info
        .batteries
        .iter()
        .find(|b| b.name == battery)
        .and_then(|b| b.full_wh)
        .filter(|f| *f > 0.0)?;
    let (w, minutes) = battery_draw(mains, &watts)?;
    (minutes >= 60).then(|| {
        format!(
            "On battery, the laptop used {w:.1} W on average (about {:.0}% of the battery an hour), so a full charge ({full:.0} Wh now) lasts about {}.",
            w / full * 100.0,
            hours_text(full / w)
        )
    })
}

/// "7.2 hours", "45 minutes".
fn hours_text(hours: f64) -> String {
    if hours < 1.0 {
        count((hours * 60.0).round() as usize, "minute")
    } else {
        format!("{hours:.1} hours")
    }
}

fn power_mode(info: &power::PowerInfo, mains: &[(i64, f64)], now: i64) -> Check {
    let title = "Change the power mode";
    let hour: Vec<f64> = mains
        .iter()
        .filter(|(t, _)| now - t <= HOUR)
        .map(|(_, v)| *v)
        .collect();
    let steady = hour.len() >= 50;
    let all_mains = steady && hour.iter().all(|v| *v >= 0.5);
    let all_battery = steady && hour.iter().all(|v| *v < 0.5);
    let profile = info.profile.clone().unwrap_or_default();
    let (offered, why) = match profile.as_str() {
        "power-saver" if all_mains => (
            true,
            "Power saver is on, and the laptop was on mains for the last hour: it runs slower than it needs to.",
        ),
        "performance" if all_battery => (
            true,
            "Performance is on, and the laptop was on battery for the last hour: it drains the battery faster.",
        ),
        "" => (
            false,
            "The power mode is not available (power-profiles-daemon).",
        ),
        _ => (
            false,
            "The power mode fits how the laptop was powered in the last hour.",
        ),
    };
    fix(
        "power.mode",
        title,
        why.into(),
        offered,
        "power.set_profile",
        Some("balanced".into()),
        "you",
    )
}

async fn lid_wakes(detail: &power_detail::PowerDetail) -> Check {
    let log = command::output(
        "journalctl",
        &[
            "--since=-7d",
            "-o",
            "short-unix",
            "--no-pager",
            "-q",
            "-t",
            "kernel",
            "-t",
            "systemd-logind",
            "--grep",
            "Lid (closed|opened)|PM: suspend exit",
        ],
        Duration::from_secs(10),
    )
    .await;
    let title = "Stop a device from waking the computer";
    let Some(log) = log else {
        return advice(
            "power.lid_wakes",
            title,
            "The system log of the last 7 days could not be read.".into(),
            false,
            "",
        );
    };
    let n = lid_closed_wakes(&log);
    let last = detail
        .last_wake_device
        .as_deref()
        .map(|d| format!(" The last wake-up came from {d}."))
        .unwrap_or_default();
    advice(
        "power.lid_wakes",
        title,
        format!(
            "The laptop woke {} with the lid closed in the last 7 days.{}",
            count(n, "time"),
            if n > 0 { last } else { String::new() }
        ),
        n >= 3,
        "Power › Sleep lists every device allowed to wake the computer; stop the ones you do not need (often a mouse receiver, a USB hub, or the network card). The lid and power button always stay on.",
    )
}

fn zswap(agent: &Agent) -> Check {
    let title = "Turn on compressed swap (zswap)";
    let state = agent
        .root
        .read("/sys/module/zswap/parameters/enabled")
        .ok()
        .map(|s| s.trim().to_owned());
    let swap = agent
        .root
        .read("/proc/swaps")
        .ok()
        .is_some_and(|s| s.lines().count() > 1);
    let on = matches!(state.as_deref(), Some("Y" | "1"));
    if state.is_none() || !swap || on {
        return fix(
            "memory.zswap",
            title,
            if state.is_none() {
                "This kernel has no zswap.".into()
            } else if !swap {
                "There is no swap, so there is nothing to compress.".into()
            } else {
                "zswap is already on.".into()
            },
            false,
            "",
            None,
            "admin",
        );
    }
    let storm = longest_swap_storm(
        &series(agent, "memory.swap_ins_per_s", DAY),
        &series(agent, "memory.hard_faults_per_s", DAY),
    );
    fix(
        "memory.zswap",
        title,
        if storm >= 10 {
            format!(
                "Swapping stayed high for {storm} minutes in a row in the last 24 hours (over 50 pages swapped in and 100 hard faults a second). zswap keeps swapped memory compressed in RAM, which is much faster than the disk."
            )
        } else {
            format!(
                "Swapping did not stay high for 10 minutes in the last 24 hours (longest: {storm} minutes)."
            )
        },
        storm >= 10,
        "system.enable_zswap",
        Some("on".into()),
        "admin",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mains_share_and_days_covered() {
        let points: Vec<(i64, f64)> = (0..14)
            .map(|d| (d * DAY, if d < 12 { 1.0 } else { 0.0 }))
            .collect();
        let (share, days) = mains_share(&points, 14 * DAY).unwrap();
        assert!((share - 12.0 / 14.0).abs() < 1e-9);
        assert!((days - 14.0).abs() < 1e-9);
        assert!(mains_share(&[], 0).is_none());
    }

    #[test]
    fn the_draw_counts_only_readings_on_battery() {
        let mains = [(1, 1.0), (2, 0.0), (3, 0.0), (4, 0.0), (5, 1.0)];
        // Minute 1 and 5 on the charger; minute 4 has no power reading.
        let watts = [(1, 20.0), (2, 4.0), (3, 6.0), (5, 25.0)];
        assert_eq!(battery_draw(&mains, &watts), Some((5.0, 2)));
        assert_eq!(minutes_on_battery(&mains), 3);
        assert_eq!(battery_draw(&[(1, 1.0)], &[(1, 20.0)]), None);
    }

    #[test]
    fn away_offers_only_what_never_happens_or_waits_long() {
        // This laptop on 2026-11-19: the screen never turns off, and it never sleeps on battery.
        let (screen, sleep) = away_needs(Some(0), Some(0), true);
        assert_eq!(
            screen.as_deref(),
            Some("the screen never turns off by itself")
        );
        assert_eq!(
            sleep.as_deref(),
            Some("on battery the laptop never sleeps by itself")
        );
        assert_eq!(away_needs(Some(300), Some(900), true), (None, None));
        assert!(away_needs(Some(1800), Some(3600), true).1.is_some());
        // A desktop: sleep on battery is not its business.
        assert_eq!(away_needs(Some(300), Some(0), false), (None, None));
        let info = power::PowerInfo {
            screen_blank_seconds: Some(0),
            suspend_on_battery_seconds: Some(900),
            ..Default::default()
        };
        let check = away(&info, true);
        assert!(check.offered);
        assert!(
            check
                .measured
                .starts_with("The screen never turns off by itself. The fix saves"),
            "{}",
            check.measured
        );
    }

    #[test]
    fn charge_items_follow_the_limit_and_the_custom_mode() {
        let battery = |stop: u32, modes: &str| power_detail::BatteryDetail {
            name: "BAT0".into(),
            charge_stop: Some(stop),
            charge_mode: Some(power_detail::choice(modes)),
            ..Default::default()
        };
        let item = |checks: &[Check], id: &str| {
            checks
                .iter()
                .find(|c| c.id == id)
                .cloned()
                .unwrap_or_else(|| panic!("no {id}"))
        };
        // A Dell with no limit, in Fast: only the 80% item (its preview also sets Custom).
        let dell = charge_checks(&battery(100, "Standard Adaptive Custom [Fast] Trickle"), "");
        assert!(item(&dell, "power.charge_limit:BAT0").offered);
        let mode = item(&dell, "power.charge_mode:BAT0");
        assert!(!mode.offered, "{}", mode.measured);
        // A Dell in Standard whose limit files show a stored 90%: that value does nothing, so
        // it is not called a limit. The 80% item is offered and says why; no mode item.
        let stored = charge_checks(&battery(90, "[Standard] Adaptive Custom Fast"), "");
        let limit = item(&stored, "power.charge_limit:BAT0");
        assert!(limit.offered);
        assert!(
            limit.measured.starts_with("Battery BAT0 charges to 100%: it follows a charge limit only in the Custom charge mode, and the mode is Standard."),
            "{}",
            limit.measured
        );
        assert!(!item(&stored, "power.charge_mode:BAT0").offered);
        // A 90% limit the user chose, working in Custom: nothing is offered.
        let chosen = charge_checks(&battery(90, "Standard Adaptive [Custom] Fast"), "");
        assert!(chosen.iter().all(|c| !c.offered));
        // No Custom mode at all, in Fast: the gentler mode, as before.
        let other = charge_checks(&battery(80, "Standard Adaptive [Fast]"), "");
        let mode = item(&other, "power.charge_mode:BAT0");
        assert!(mode.offered);
        assert_eq!(mode.subject.as_deref(), Some("BAT0:Adaptive"));
    }

    #[test]
    fn hours_read_well() {
        assert_eq!(hours_text(7.04), "7.0 hours");
        assert_eq!(hours_text(0.75), "45 minutes");
    }

    #[test]
    fn slowdowns_count_only_minutes_on_battery() {
        let mains = [(1, 1.0), (2, 0.0), (3, 0.0), (4, 0.0)];
        let throttle = [(1, 10.0), (2, 15.0), (3, 15.0), (4, 2.0)];
        // Minute 2: +5 on battery; minute 3: none; minute 4: a restart, then 2 more.
        assert_eq!(battery_slowdowns(&mains, &throttle), (3, 2));
    }

    #[test]
    fn a_swap_storm_must_last() {
        let s: Vec<(i64, f64)> = (0..12)
            .map(|m| (m, if m == 5 { 0.0 } else { 80.0 }))
            .collect();
        let f: Vec<(i64, f64)> = (0..12).map(|m| (m, 150.0)).collect();
        assert_eq!(longest_swap_storm(&s, &f), 6);
    }

    #[test]
    fn only_wakes_with_the_lid_still_closed_count() {
        let log = "100.0 systemd-logind[1]: Lid closed.\n\
                   200.0 kernel: PM: suspend exit\n\
                   1000.0 kernel: PM: suspend exit\n\
                   1001.0 systemd-logind[1]: Lid opened.\n\
                   1100.0 systemd-logind[1]: Lid closed.\n\
                   1200.0 kernel: PM: suspend exit\n";
        // 200 and 1200: woke with the lid closed. 1000: the lid was opened right after (a normal wake).
        assert_eq!(lid_closed_wakes(log), 2);
    }
}
