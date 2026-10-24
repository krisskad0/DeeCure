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
    let mut out = Vec::new();
    let mains = series(agent, "power.on_mains", 14 * DAY);
    if let Some(b) = detail.batteries.first() {
        let usage = mains_share(&mains, now);
        let enough = usage.is_some_and(|(_, days)| days >= 13.5);
        let mostly = usage.is_some_and(|(share, _)| share >= 0.7);
        let text = |what: &str| match usage {
            None => "No power readings yet: DeeCure keeps one a minute.".to_owned(),
            Some((_, days)) if days < 13.5 => {
                format!("{days:.0} of 14 days of power readings so far; {what} after 14 days.")
            }
            Some((share, _)) => format!(
                "On mains {:.0}% of the time over the last 14 days.",
                share * 100.0
            ),
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
                    text("checked")
                },
                enough && mostly && stop > 80,
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
                    text("checked")
                } else {
                    format!(
                        "The charge mode is {}.",
                        mode.current.as_deref().unwrap_or("not reported")
                    )
                },
                fast && enough && mostly && gentle.is_some(),
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
        out.push(power_mode(agent, &mains, now).await);
        out.push(lid_wakes(&detail).await);
    }
    out.push(zswap(agent));
    out
}

async fn power_mode(agent: &Agent, mains: &[(i64, f64)], now: i64) -> Check {
    let info = power::read(agent).await;
    let title = "Change the power mode";
    let hour: Vec<f64> = mains
        .iter()
        .filter(|(t, _)| now - t <= HOUR)
        .map(|(_, v)| *v)
        .collect();
    let steady = hour.len() >= 50;
    let all_mains = steady && hour.iter().all(|v| *v >= 0.5);
    let all_battery = steady && hour.iter().all(|v| *v < 0.5);
    let profile = info.profile.unwrap_or_default();
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
