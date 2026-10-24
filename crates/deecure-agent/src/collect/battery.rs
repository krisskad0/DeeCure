// SPDX-License-Identifier: Apache-2.0

//! Laptop batteries from `/sys/class/power_supply`.

use std::io;
use std::path::Path;

use deecure_core::{Timestamp, Unit};

use super::{Collector, Sample, SystemRoot, metric, read_trimmed};

/// Charge, health, and power draw of each system battery.
///
/// Batteries of mice and keyboards (`scope` = `Device`) are skipped. A value the
/// hardware does not report is left out, never guessed.
#[derive(Debug, Default)]
pub struct Battery;

impl Collector for Battery {
    fn name(&self) -> &'static str {
        "battery"
    }

    fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let mut sample = Sample::default();
        let dir = root.path("/sys/class/power_supply");
        let mut entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries.collect::<Result<Vec<_>, _>>()?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(sample), // a desktop PC
            Err(e) => return Err(e),
        };
        entries.sort_by_key(|entry| entry.file_name());
        // On mains: a charger (mains or USB, not a device's own) is online. Stored only on
        // computers with a system battery, for "on mains most of the last 14 days" (Optimize).
        let (mut has_battery, mut on_mains) = (false, false);
        for entry in &entries {
            let text = |file: &str| read_trimmed(&entry.path().join(file));
            if text("scope").as_deref() == Some("Device") {
                continue;
            }
            match text("type").as_deref() {
                Some("Battery") if text("present").as_deref() != Some("0") => has_battery = true,
                Some("Mains" | "USB") => on_mains |= text("online").as_deref() == Some("1"),
                _ => {}
            }
        }
        if has_battery {
            sample.metrics.push(metric(
                "power.on_mains",
                None,
                if on_mains { 1.0 } else { 0.0 },
                Unit::Count,
                now,
            ));
        }

        for entry in entries {
            let supply = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let text = |file: &str| read_trimmed(&supply.join(file));
            if text("type").as_deref() != Some("Battery")
                || text("scope").as_deref() == Some("Device")
                || text("present").as_deref() == Some("0")
            {
                continue;
            }
            let subject = Some(name.as_str());
            if let Some(charge) = number(&supply, "capacity") {
                sample.metrics.push(metric(
                    "battery.charge",
                    subject,
                    charge,
                    Unit::Percent,
                    now,
                ));
            }
            if let Some(health) = health(&supply) {
                sample.metrics.push(metric(
                    "battery.health",
                    subject,
                    health,
                    Unit::Percent,
                    now,
                ));
            }
            if let Some(watts) = power(&supply) {
                sample
                    .metrics
                    .push(metric("battery.power", subject, watts, Unit::Watts, now));
            }
            if text("status").as_deref() == Some("Discharging")
                && let Some(seconds) = time_left(&supply)
            {
                sample.metrics.push(metric(
                    "battery.time_left",
                    subject,
                    seconds,
                    Unit::Seconds,
                    now,
                ));
            }
        }
        Ok(sample)
    }
}

fn number(supply: &Path, file: &str) -> Option<f64> {
    read_trimmed(&supply.join(file))?.parse().ok()
}

/// Capacity left compared with when new, in percent. It can be a little over 100 for a new battery.
fn health(supply: &Path) -> Option<f64> {
    [
        ("energy_full", "energy_full_design"),
        ("charge_full", "charge_full_design"),
    ]
    .into_iter()
    .find_map(|(full, design)| {
        let (full, design) = (number(supply, full)?, number(supply, design)?);
        (design > 0.0).then(|| full * 100.0 / design)
    })
}

/// Time until empty at the current draw, in seconds.
fn time_left(supply: &Path) -> Option<f64> {
    let hours = [("energy_now", "power_now"), ("charge_now", "current_now")]
        .into_iter()
        .find_map(|(stored, rate)| {
            let (stored, rate) = (number(supply, stored)?, number(supply, rate)?);
            (rate > 0.0).then(|| stored / rate)
        })?;
    Some(hours * 3600.0)
}

/// Power going in or out right now, in watts.
fn power(supply: &Path) -> Option<f64> {
    if let Some(microwatts) = number(supply, "power_now") {
        return Some(microwatts / 1e6);
    }
    let microamps = number(supply, "current_now")?;
    let microvolts = number(supply, "voltage_now")?;
    Some(microamps * microvolts / 1e12)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::{fixture_root, value};

    #[test]
    fn reads_system_batteries_and_skips_the_rest() {
        let root = fixture_root(
            "battery",
            &[
                // A laptop battery that reports charge in µAh (like many ThinkPads).
                ("/sys/class/power_supply/BAT0/type", "Battery\n"),
                ("/sys/class/power_supply/BAT0/capacity", "64\n"),
                ("/sys/class/power_supply/BAT0/charge_full", "3550000\n"),
                (
                    "/sys/class/power_supply/BAT0/charge_full_design",
                    "5000000\n",
                ),
                ("/sys/class/power_supply/BAT0/current_now", "1000000\n"),
                ("/sys/class/power_supply/BAT0/voltage_now", "12000000\n"),
                // A mouse battery: skipped.
                ("/sys/class/power_supply/hidpp_battery_0/type", "Battery\n"),
                ("/sys/class/power_supply/hidpp_battery_0/scope", "Device\n"),
                ("/sys/class/power_supply/hidpp_battery_0/capacity", "90\n"),
                // The charger: skipped.
                ("/sys/class/power_supply/AC/type", "Mains\n"),
            ],
        );
        let sample = Battery.collect(&root, Timestamp(0)).unwrap();
        assert_eq!(value(&sample, "battery.charge", Some("BAT0")), Some(64.0));
        assert_eq!(value(&sample, "battery.health", Some("BAT0")), Some(71.0));
        assert_eq!(value(&sample, "battery.power", Some("BAT0")), Some(12.0));
        assert_eq!(
            value(&sample, "battery.charge", Some("hidpp_battery_0")),
            None
        );
        // The laptop has a battery, so on-mains is stored too: AC reports no "online", so 0.
        assert_eq!(value(&sample, "power.on_mains", None), Some(0.0));
        assert_eq!(sample.metrics.len(), 4);
    }

    #[test]
    fn energy_based_batteries_use_power_now() {
        let root = fixture_root(
            "battery-energy",
            &[
                ("/sys/class/power_supply/BAT1/type", "Battery\n"),
                ("/sys/class/power_supply/BAT1/energy_full", "45000000\n"),
                (
                    "/sys/class/power_supply/BAT1/energy_full_design",
                    "50000000\n",
                ),
                ("/sys/class/power_supply/BAT1/power_now", "7500000\n"),
                ("/sys/class/power_supply/BAT1/energy_now", "15000000\n"),
                ("/sys/class/power_supply/BAT1/status", "Discharging\n"),
            ],
        );
        let sample = Battery.collect(&root, Timestamp(0)).unwrap();
        assert_eq!(value(&sample, "battery.health", Some("BAT1")), Some(90.0));
        assert_eq!(value(&sample, "battery.power", Some("BAT1")), Some(7.5));
        assert_eq!(
            value(&sample, "battery.charge", Some("BAT1")),
            None,
            "no capacity file"
        );
        // 15 Wh at 7.5 W = 2 hours.
        assert_eq!(
            value(&sample, "battery.time_left", Some("BAT1")),
            Some(7200.0)
        );
    }

    #[test]
    fn desktop_without_power_supplies_reports_nothing() {
        let root = fixture_root("desktop", &[("/proc/version", "Linux\n")]);
        assert_eq!(
            Battery.collect(&root, Timestamp(0)).unwrap(),
            Sample::default()
        );
    }
}
