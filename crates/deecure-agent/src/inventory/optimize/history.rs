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
    if let Some(b) = detail.batteries.first() {
        let usage = match mains_share(&mains, now) {
            Some((share, days)) if days >= 1.0 => format!(
                " On mains {:.0}% of the time over the last {} days.",
                share * 100.0,
                days.round().min(14.0)
            ),
            _ => String::new(),
        };
        if let Some(stop) = b.charge_stop {
            let subject = format!(
                "{}:{}:80",
                b.name,
                if b.charge_start.is_some() { "75" } else { "-" }
            );
            out.push(fix(
                &format!("power.charge_limit:{}", b.name),
                "Limit battery charge to 80%",
                if stop <= 80 {
                    format!("Battery {} already stops at {stop}%.", b.name)
                } else {
                    format!(
                        "Battery {} charges to {stop}%. Stopping at 80% slows its wear; each charge then lasts about a fifth less.{usage}",
                        b.name
                    )
                },
                stop > 80,
                "power.set_charge_limits",
                Some(subject),
                "admin",
            ));
        }
        if let Some(mode) = &b.charge_mode {
            let gentle = ["Adaptive", "Standard"]
                .into_iter()
                .find(|m| mode.offered.iter().any(|o| o == m));
            let fast = mode.current.as_deref() == Some("Fast");
            out.push(fix(
                &format!("power.charge_mode:{}", b.name),
                "Use a gentler charge mode",
                if fast {
                    "The charge mode is Fast: it charges quickly but wears the battery more."
                        .to_owned()
                } else {
                    format!(
                        "The charge mode is {}.",
                        mode.current.as_deref().unwrap_or("not reported")
                    )
                },
                fast && gentle.is_some(),
                "power.set_charge_mode",
                gentle.map(|g| format!("{}:{g}", b.name)),
                "admin",
            ));
        }
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
        out.push(battery_saver(
            &info,
            minutes_on_battery(&week),
            detail.battery_saver,
        ));
        out.push(battery_drain(agent, &b.name, &info, &week, now));
        out.push(power_mode(&info, &mains, now));
        out.push(lid_wakes(&detail).await);
    }
    out.push(zswap(agent));
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

/// How long a full charge lasts at the measured draw before Optimize offers help, in hours.
pub const SHORT_HOURS: f64 = 8.0;

fn battery_saver(info: &power::PowerInfo, minutes: usize, on: bool) -> Check {
    let title = "Use Power saver on battery";
    let (offered, measured) = if info.profile.is_none() {
        (
            false,
            "The power mode is not available (power-profiles-daemon).".to_owned(),
        )
    } else if on {
        (
            false,
            "Power saver already turns on by itself when you unplug.".to_owned(),
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
                "{} on battery in the last 7 days, and the power mode stays the same when you unplug.",
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

fn battery_drain(
    agent: &Agent,
    battery: &str,
    info: &power::PowerInfo,
    mains: &[(i64, f64)],
    now: i64,
) -> Check {
    let title = "Make the battery last longer";
    let what = "Turn on Power saver on battery and turbo off on battery (Power › Power Profile › Power controls). Lower the screen brightness: the screen is often the biggest single user. On battery, close the apps at the top of Power › Battery › Battery use by app when you do not need them.";
    let since = deecure_core::Timestamp(now - 7 * DAY);
    let (watts, top) = {
        let store = crate::store::lock(&agent.store);
        let watts = store
            .metrics(
                "battery.power",
                Some(battery),
                since,
                deecure_core::Timestamp(now),
            )
            .unwrap_or_default()
            .into_iter()
            .map(|m| (m.at.0, m.value))
            .collect::<Vec<_>>();
        let top = store
            .metric_sums(
                crate::collect::app_battery::METRIC,
                since,
                deecure_core::Timestamp(now),
            )
            .unwrap_or_default();
        (watts, top)
    };
    let full = info
        .batteries
        .iter()
        .find(|b| b.name == battery)
        .and_then(|b| b.full_wh)
        .filter(|f| *f > 0.0);
    let (Some((w, minutes)), Some(full)) = (battery_draw(mains, &watts), full) else {
        return advice(
            "power.battery_drain",
            title,
            "No readings on battery in the last 7 days.".into(),
            false,
            what,
        );
    };
    if minutes < 60 {
        return advice(
            "power.battery_drain",
            title,
            format!(
                "{} on battery in the last 7 days: not enough to judge.",
                count(minutes, "minute")
            ),
            false,
            what,
        );
    }
    let hours = full / w;
    let measured = format!(
        "On battery, the laptop used {w:.1} W on average in the last 7 days, about {:.0}% of the battery an hour. A full charge ({full:.0} Wh now) lasts about {}.{}",
        w / full * 100.0,
        hours_text(hours),
        top_users(&top)
    );
    advice(
        "power.battery_drain",
        title,
        measured,
        hours < SHORT_HOURS,
        what,
    )
}

/// " Most of it: Google Chrome 30%, the screen and system at rest 25%, …" from the per-app
/// estimate (Power › Battery), or nothing when there is none.
pub fn top_users(sums: &[(Option<String>, f64)]) -> String {
    let total: f64 = sums.iter().map(|(_, v)| v.max(0.0)).sum();
    if total <= 0.0 {
        return String::new();
    }
    let mut list: Vec<(String, f64)> = sums
        .iter()
        .filter_map(|(s, v)| {
            let (kind, name) = super::super::app_battery::parse(s.as_deref()?);
            let label = match kind.as_str() {
                "rest" => "the screen, Wi-Fi, and the system at rest".to_owned(),
                "other" => "small apps together".to_owned(),
                _ => name,
            };
            Some((label, *v))
        })
        .collect();
    list.sort_by(|a, b| b.1.total_cmp(&a.1));
    let parts: Vec<String> = list
        .iter()
        .take(3)
        .map(|(n, v)| format!("{n} {:.0}%", v / total * 100.0))
        .collect();
    format!(" Most of it (an estimate): {}.", parts.join(", "))
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
    fn the_top_users_are_named_with_their_share() {
        let sums = [
            (Some("app:Google Chrome".to_owned()), 300.0),
            (Some("rest".to_owned()), 500.0),
            (Some("app:Visual Studio Code".to_owned()), 150.0),
            (Some("other".to_owned()), 50.0),
        ];
        assert_eq!(
            top_users(&sums),
            " Most of it (an estimate): the screen, Wi-Fi, and the system at rest 50%, Google Chrome 30%, Visual Studio Code 15%."
        );
        assert_eq!(top_users(&[]), "");
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
