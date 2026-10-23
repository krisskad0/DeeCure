// SPDX-License-Identifier: Apache-2.0

//! Temperatures and fans from `/sys/class/hwmon`.

use std::io;

use deecure_core::{Timestamp, Unit};

use super::{Collector, Sample, SystemRoot, metric, read_trimmed};

/// Chips whose first reading is the CPU temperature, best first.
const CPU_CHIPS: &[(&str, &[&str])] = &[
    ("coretemp", &["Package id 0"]),
    ("k10temp", &["Tctl", "Tdie"]),
    ("zenpower", &["Tdie", "Tctl"]),
    ("cpu_thermal", &[]),
    ("acpitz", &[]),
];

/// One `temp` metric per sensor (subject `<chip>/<label>`), one `fan` per fan,
/// `temp.cpu` from the best CPU sensor found, and `cpu.throttle_total`: the times heat slowed
/// the whole chip since start (Intel), so history shows slowdowns next to the temperature.
#[derive(Debug, Default)]
pub struct Sensors;

impl Collector for Sensors {
    fn name(&self) -> &'static str {
        "sensors"
    }

    fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let mut sample = Sample::default();
        if let Some(count) = read_trimmed(
            &root.path("/sys/devices/system/cpu/cpu0/thermal_throttle/package_throttle_count"),
        )
        .and_then(|v| v.parse::<f64>().ok())
        {
            sample
                .metrics
                .push(metric("cpu.throttle_total", None, count, Unit::Count, now));
        }
        let dir = root.path("/sys/class/hwmon");
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return Ok(sample); // no sensors exposed
        };
        let mut chips: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        chips.sort();
        let mut readings: Vec<(String, String, f64)> = Vec::new();
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
                    sample.metrics.push(metric(
                        "temp",
                        Some(&subject),
                        celsius,
                        Unit::Celsius,
                        now,
                    ));
                    readings.push((chip_name.clone(), label, celsius));
                } else if prefix.starts_with("fan") {
                    sample
                        .metrics
                        .push(metric("fan", Some(&subject), raw, Unit::Count, now));
                }
            }
        }
        let cpu = CPU_CHIPS.iter().find_map(|(chip, labels)| {
            let on_chip = readings.iter().filter(|(name, ..)| name == chip);
            let preferred = labels.iter().find_map(|l| {
                readings
                    .iter()
                    .find(|(name, label, _)| name == chip && label == l)
            });
            preferred
                .or_else(|| on_chip.clone().next())
                .map(|(.., c)| *c)
        });
        if let Some(celsius) = cpu {
            sample
                .metrics
                .push(metric("temp.cpu", None, celsius, Unit::Celsius, now));
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
                ("/sys/class/hwmon/hwmon5/name", "dell_smm\n"),
                ("/sys/class/hwmon/hwmon5/fan1_input", "2400\n"),
                ("/sys/class/hwmon/hwmon6/name", "coretemp\n"),
                ("/sys/class/hwmon/hwmon6/temp1_input", "54000\n"),
                ("/sys/class/hwmon/hwmon6/temp1_label", "Package id 0\n"),
                ("/sys/class/hwmon/hwmon6/temp2_input", "51000\n"),
                ("/sys/class/hwmon/hwmon6/temp2_label", "Core 0\n"),
            ],
        );
        let sample = Sensors.collect(&root, Timestamp(0)).unwrap();
        assert_eq!(value(&sample, "temp", Some("nvme/Composite")), Some(41.85));
        assert_eq!(value(&sample, "fan", Some("dell_smm/fan1")), Some(2400.0));
        assert_eq!(value(&sample, "temp.cpu", None), Some(54.0));
    }
}
