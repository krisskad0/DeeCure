// SPDX-License-Identifier: Apache-2.0

//! Temperatures and fans from `/sys/class/hwmon`, with each sensor's own limits.
//!
//! Health warnings (docs/01-product/health-warnings.md) need a problem to last, so this
//! collector also counts minutes in a row: near a sensor's own limit, and a fan stopped while
//! the CPU is hot. It keeps the last hour of heat slowdowns, too.

use std::collections::{HashMap, VecDeque};
use std::io;

use deecure_core::{Timestamp, Unit};

use super::{Collector, Sample, SystemRoot, metric, minutes_since, read_trimmed};

/// Chips whose first reading is the CPU temperature, best first.
const CPU_CHIPS: &[(&str, &[&str])] = &[
    ("coretemp", &["Package id 0"]),
    ("k10temp", &["Tctl", "Tdie"]),
    ("zenpower", &["Tdie", "Tctl"]),
    ("cpu_thermal", &[]),
    ("acpitz", &[]),
];
/// A limit outside this range is a chip reporting nonsense (0, 255, 511 °C), not a limit.
const SANE_LIMIT: std::ops::RangeInclusive<f64> = 40.0..=150.0;
/// "Near the limit": this close to the critical temperature (when no lower warning is reported).
const MARGIN: f64 = 5.0;
/// For a CPU that reports no limit (some AMD chips): above every current CPU's normal maximum
/// (AMD 95 °C, Intel 100 °C), so heavy use never warns (docs/01-product/health-warnings.md).
const CPU_FALLBACK: (f64, f64) = (100.0, 105.0);
/// A fan at 0 RPM is a problem only when the CPU is at least this hot.
const FAN_HOT_C: f64 = 75.0;
/// A heat slowdown counts toward "the cooling cannot keep up" only when the CPU is this close
/// to its own limit. Turbo bursts touch the limit for a split second at a cool 55-65 °C; that is
/// the chip working as designed, not weak cooling.
const SLOWDOWN_NEAR_LIMIT_C: f64 = 15.0;

/// One `temp` metric per sensor (subject `<chip>/<label>`) with `temp.limit` (where it starts
/// to be too hot: the sensor's own `max`, or 5 °C under its `crit`), `temp.critical`, and
/// `temp.hot_minutes`; one `fan` per fan; `temp.cpu` with the same three for the CPU; and
/// `cpu.throttle_total` with `cpu.throttle_minutes_hour`.
#[derive(Debug, Default)]
pub struct Sensors {
    /// Minutes in a row at or over the limit, per sensor.
    hot: HashMap<String, f64>,
    fan_stopped_hot: f64,
    /// Minutes in a row at the fallback limit, for a CPU without its own.
    cpu_hot_fallback: f64,
    /// The throttle counter at each reading of the last hour, and whether the CPU was hot then.
    throttle: VecDeque<(Timestamp, f64, bool)>,
    last: Option<Timestamp>,
}

/// One temperature: chip, label, °C, where it starts to be too hot, and its critical limit.
type Reading = (String, String, f64, Option<f64>, Option<f64>);

/// Where a sensor starts to be too hot, and its critical temperature, from its own files.
pub fn limits(max: Option<f64>, crit: Option<f64>) -> (Option<f64>, Option<f64>) {
    let max = max.filter(|v| SANE_LIMIT.contains(v));
    let crit = crit.filter(|v| SANE_LIMIT.contains(v));
    let warn = match (max, crit) {
        (Some(m), Some(c)) => Some(m.min(c - MARGIN)),
        (Some(m), None) => Some(m),
        (None, Some(c)) => Some(c - MARGIN),
        (None, None) => None,
    };
    (warn, crit)
}

impl Sensors {
    /// `cpu.throttle_minutes_hour`: minutes of the last hour in which the chip slowed itself
    /// down because of heat while it was hot (`hot`, see [`SLOWDOWN_NEAR_LIMIT_C`]).
    fn slowdowns(&mut self, sample: &mut Sample, now: Timestamp, count: Option<f64>, hot: bool) {
        let Some(count) = count else {
            return;
        };
        self.throttle.push_back((now, count, hot));
        while self
            .throttle
            .front()
            .is_some_and(|(t, ..)| now.as_millis() - t.as_millis() > 60 * 60_000)
        {
            self.throttle.pop_front();
        }
        // The time between two readings counts when the counter went up (a restart sets it
        // back to 0) and the CPU was hot at the end of it, at most a minute each, so an extra
        // reading adds nothing.
        let slowed: f64 = self
            .throttle
            .iter()
            .zip(self.throttle.iter().skip(1))
            .filter(|((_, a, _), (_, b, hot))| b > a && *hot)
            .map(|((ta, ..), (tb, ..))| {
                ((tb.as_millis() - ta.as_millis()) as f64 / 60_000.0).clamp(0.0, 1.0)
            })
            .sum();
        sample.metrics.push(metric(
            "cpu.throttle_minutes_hour",
            None,
            slowed.round(),
            Unit::Count,
            now,
        ));
    }
}

impl Collector for Sensors {
    fn name(&self) -> &'static str {
        "sensors"
    }

    fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let mut sample = Sample::default();
        let step = minutes_since(&mut self.last, now, 1.0);
        let throttle = read_trimmed(
            &root.path("/sys/devices/system/cpu/cpu0/thermal_throttle/package_throttle_count"),
        )
        .and_then(|v| v.parse::<f64>().ok());
        if let Some(count) = throttle {
            sample
                .metrics
                .push(metric("cpu.throttle_total", None, count, Unit::Count, now));
        }
        let dir = root.path("/sys/class/hwmon");
        let Ok(entries) = std::fs::read_dir(&dir) else {
            // No sensors exposed: every slowdown counts, as the CPU's heat is unknown.
            self.slowdowns(&mut sample, now, throttle, true);
            return Ok(sample); // no sensors exposed
        };
        let mut chips: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        chips.sort();
        let mut readings: Vec<Reading> = Vec::new();
        let mut fans: Vec<f64> = Vec::new();
        let mut seen_hot = Vec::new();
        for chip in &chips {
            let Some(chip_name) = read_trimmed(&chip.join("name")) else {
                continue;
            };
            let mut files: Vec<String> = std::fs::read_dir(chip)
                .map(|d| {
                    d.flatten()
                        .filter_map(|e| e.file_name().into_string().ok())
                        .collect()
                })
                .unwrap_or_default();
            files.sort();
            for file in files {
                let Some(prefix) = file.strip_suffix("_input") else {
                    continue;
                };
                let label = read_trimmed(&chip.join(format!("{prefix}_label")))
                    .unwrap_or_else(|| prefix.to_owned());
                let Some(raw) = read_trimmed(&chip.join(&file)).and_then(|v| v.parse::<f64>().ok())
                else {
                    continue;
                };
                let subject = format!("{chip_name}/{label}");
                if prefix.starts_with("temp") {
                    let celsius = raw / 1000.0;
                    let milli = |end: &str| {
                        read_trimmed(&chip.join(format!("{prefix}_{end}")))
                            .and_then(|v| v.parse::<f64>().ok())
                            .map(|v| v / 1000.0)
                    };
                    let (limit, critical) = limits(milli("max"), milli("crit"));
                    let at = |name: &str, value: f64, unit: Unit| {
                        metric(name, Some(&subject), value, unit, now)
                    };
                    sample.metrics.push(at("temp", celsius, Unit::Celsius));
                    if let Some(l) = limit {
                        sample.metrics.push(at("temp.limit", l, Unit::Celsius));
                        let minutes = self.hot.entry(subject.clone()).or_insert(0.0);
                        *minutes = if celsius >= l { *minutes + step } else { 0.0 };
                        sample
                            .metrics
                            .push(at("temp.hot_minutes", *minutes, Unit::Count));
                        seen_hot.push(subject.clone());
                    }
                    if let Some(c) = critical {
                        sample.metrics.push(at("temp.critical", c, Unit::Celsius));
                    }
                    readings.push((chip_name.clone(), label, celsius, limit, critical));
                } else if prefix.starts_with("fan") {
                    sample
                        .metrics
                        .push(metric("fan", Some(&subject), raw, Unit::Count, now));
                    fans.push(raw);
                }
            }
        }
        // Sensors that went away keep no count.
        self.hot.retain(|k, _| seen_hot.contains(k));
        let cpu = CPU_CHIPS.iter().find_map(|(chip, labels)| {
            let on_chip = readings.iter().filter(|(name, ..)| name == chip);
            let preferred = labels.iter().find_map(|l| {
                readings
                    .iter()
                    .find(|(name, label, ..)| name == chip && label == l)
            });
            preferred.or_else(|| on_chip.clone().next()).cloned()
        });
        let cpu_hot = cpu.as_ref().map(|(_, _, celsius, limit, _)| {
            *celsius >= limit.unwrap_or(CPU_FALLBACK.0) - SLOWDOWN_NEAR_LIMIT_C
        });
        self.slowdowns(&mut sample, now, throttle, cpu_hot.unwrap_or(true));
        if let Some((chip, label, celsius, limit, critical)) = cpu {
            let mut push = |name: &str, value: f64, unit: Unit| {
                sample.metrics.push(metric(name, None, value, unit, now));
            };
            push("temp.cpu", celsius, Unit::Celsius);
            let (limit, critical) = match (limit, critical) {
                (Some(l), c) => (l, c.unwrap_or(l + MARGIN)),
                (None, _) => CPU_FALLBACK,
            };
            push("temp.cpu_limit", limit, Unit::Celsius);
            push("temp.cpu_critical", critical, Unit::Celsius);
            let key = format!("{chip}/{label}");
            let minutes = match self.hot.get(&key) {
                Some(m) => *m,
                // No limit of its own: count here against the fallback.
                None => {
                    let m = self.cpu_hot_fallback;
                    self.cpu_hot_fallback = if celsius >= limit { m + step } else { 0.0 };
                    self.cpu_hot_fallback
                }
            };
            push("temp.cpu_hot_minutes", minutes, Unit::Count);
            self.fan_stopped_hot = if celsius >= FAN_HOT_C && fans.contains(&0.0) {
                self.fan_stopped_hot + step
            } else {
                0.0
            };
            if !fans.is_empty() {
                push("fan.stopped_hot_minutes", self.fan_stopped_hot, Unit::Count);
            }
        }
        Ok(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::{fixture_root, value};

    #[test]
    fn reads_temperatures_fans_and_picks_the_cpu_sensor() {
        let root = fixture_root(
            "sensors",
            &[
                ("/sys/class/hwmon/hwmon2/name", "nvme\n"),
                ("/sys/class/hwmon/hwmon2/temp1_input", "41850\n"),
                ("/sys/class/hwmon/hwmon2/temp1_label", "Composite\n"),
                ("/sys/class/hwmon/hwmon2/temp1_max", "84850\n"),
                ("/sys/class/hwmon/hwmon2/temp1_crit", "89850\n"),
                ("/sys/class/hwmon/hwmon5/name", "dell_smm\n"),
                ("/sys/class/hwmon/hwmon5/fan1_input", "2400\n"),
                ("/sys/class/hwmon/hwmon6/name", "coretemp\n"),
                ("/sys/class/hwmon/hwmon6/temp1_input", "54000\n"),
                ("/sys/class/hwmon/hwmon6/temp1_label", "Package id 0\n"),
                ("/sys/class/hwmon/hwmon6/temp1_max", "100000\n"),
                ("/sys/class/hwmon/hwmon6/temp1_crit", "100000\n"),
                ("/sys/class/hwmon/hwmon6/temp2_input", "51000\n"),
                ("/sys/class/hwmon/hwmon6/temp2_label", "Core 0\n"),
                ("/sys/class/hwmon/hwmon6/temp2_crit", "0\n"),
            ],
        );
        let sample = Sensors::default().collect(&root, Timestamp(0)).unwrap();
        assert_eq!(value(&sample, "temp", Some("nvme/Composite")), Some(41.85));
        assert_eq!(value(&sample, "fan", Some("dell_smm/fan1")), Some(2400.0));
        assert_eq!(value(&sample, "temp.cpu", None), Some(54.0));
        // The drive's own warning temperature; the CPU: 5 °C under its critical.
        assert_eq!(
            value(&sample, "temp.limit", Some("nvme/Composite")),
            Some(84.85)
        );
        assert_eq!(value(&sample, "temp.cpu_limit", None), Some(95.0));
        assert_eq!(value(&sample, "temp.cpu_critical", None), Some(100.0));
        // A critical of 0 is nonsense, so that core has no limit.
        assert_eq!(value(&sample, "temp.limit", Some("coretemp/Core 0")), None);
        assert_eq!(value(&sample, "fan.stopped_hot_minutes", None), Some(0.0));
    }

    #[test]
    fn minutes_near_the_limit_and_with_heat_slowdowns_add_up() {
        let files = |temp: &str, throttle: &str| {
            fixture_root(
                "sensors-hot",
                &[
                    ("/sys/class/hwmon/hwmon6/name", "coretemp\n"),
                    ("/sys/class/hwmon/hwmon6/temp1_input", temp),
                    ("/sys/class/hwmon/hwmon6/temp1_label", "Package id 0\n"),
                    ("/sys/class/hwmon/hwmon6/temp1_crit", "100000\n"),
                    ("/sys/class/hwmon/hwmon5/name", "dell_smm\n"),
                    ("/sys/class/hwmon/hwmon5/fan1_input", "0\n"),
                    (
                        "/sys/devices/system/cpu/cpu0/thermal_throttle/package_throttle_count",
                        throttle,
                    ),
                ],
            )
        };
        let mut s = Sensors::default();
        let minute = |n: i64| Timestamp(n * 60_000);
        for (n, (temp, throttle)) in [
            ("97000\n", "10\n"),
            ("98000\n", "12\n"),
            ("96000\n", "12\n"),
        ]
        .into_iter()
        .enumerate()
        {
            s.collect(&files(temp, throttle), minute(n as i64)).unwrap();
        }
        let sample = s.collect(&files("99000\n", "15\n"), minute(3)).unwrap();
        assert_eq!(value(&sample, "temp.cpu_hot_minutes", None), Some(4.0));
        assert_eq!(value(&sample, "cpu.throttle_minutes_hour", None), Some(2.0));
        assert_eq!(value(&sample, "fan.stopped_hot_minutes", None), Some(4.0));
        // An extra reading 6 seconds later (after a fix) adds only those 6 seconds.
        let sample = s
            .collect(&files("99000\n", "16\n"), Timestamp(3 * 60_000 + 6_000))
            .unwrap();
        assert_eq!(value(&sample, "temp.cpu_hot_minutes", None), Some(4.1));
        assert_eq!(value(&sample, "cpu.throttle_minutes_hour", None), Some(2.0));
        // Cooler: the count starts again.
        let sample = s.collect(&files("60000\n", "16\n"), minute(4)).unwrap();
        assert_eq!(value(&sample, "temp.cpu_hot_minutes", None), Some(0.0));
        assert_eq!(value(&sample, "fan.stopped_hot_minutes", None), Some(0.0));
        // Slowdowns at a cool 60 °C (turbo bursts touching the limit) do not count.
        for n in 5..=9 {
            let sample = s
                .collect(&files("60000\n", &format!("{}\n", 20 + n)), minute(n))
                .unwrap();
            assert_eq!(value(&sample, "cpu.throttle_minutes_hour", None), Some(2.0));
        }
    }

    #[test]
    fn limits_come_from_the_sensor_itself() {
        assert_eq!(limits(Some(84.0), Some(89.0)), (Some(84.0), Some(89.0)));
        assert_eq!(limits(Some(100.0), Some(100.0)), (Some(95.0), Some(100.0)));
        assert_eq!(limits(None, Some(105.0)), (Some(100.0), Some(105.0)));
        assert_eq!(limits(Some(0.0), Some(255.0)), (None, None));
    }
}
