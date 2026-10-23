// SPDX-License-Identifier: Apache-2.0

//! Thermal zones and cooling devices for the Expert level (docs/01-product/expert-coverage.md,
//! section 8): each zone's temperature and the temperatures where the system acts (`passive`:
//! slows down, `active`: fans, `hot`: warns, `critical`: shuts down), and each cooling device's
//! level. From `/sys/class/thermal`, no admin rights.

use serde::Serialize;

use crate::collect::SystemRoot;

/// A temperature where the system acts.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Trip {
    /// `passive`, `active`, `hot`, or `critical`.
    pub kind: String,
    pub celsius: f64,
}

/// One thermal zone.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Zone {
    pub zone: String,
    /// The firmware's name, such as `TCPU`, `TSKN` (skin), or `x86_pkg_temp`.
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub celsius: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,
    /// Trip points that are in use, lowest first.
    pub trips: Vec<Trip>,
}

/// Cooling devices of one kind, such as the CPU's speed limit or a fan.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Cooling {
    /// Such as `Processor`, `dell-smm-fan1`, or `TCC Offset`.
    pub kind: String,
    /// How many devices of this kind (one per CPU for `Processor`).
    pub count: usize,
    /// The highest level now among them, and the most they allow.
    pub level: u64,
    pub max_level: u64,
}

/// What `thermal` answers.
#[derive(Debug, Default, Serialize)]
pub struct Thermal {
    pub zones: Vec<Zone>,
    pub cooling: Vec<Cooling>,
}

fn text(root: &SystemRoot, path: &str) -> Option<String> {
    root.read(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn names(root: &SystemRoot, prefix: &str) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(root.path("/sys/class/thermal"))
        .map(|d| {
            d.flatten()
                .filter_map(|e| {
                    e.file_name()
                        .to_str()
                        .filter(|n| n.starts_with(prefix))
                        .map(str::to_owned)
                })
                .collect()
        })
        .unwrap_or_default();
    // thermal_zone10 after thermal_zone9.
    v.sort_by_key(|n| {
        n.trim_start_matches(prefix)
            .parse::<u32>()
            .unwrap_or(u32::MAX)
    });
    v
}

/// Reads `thermal`.
pub fn read(root: &SystemRoot) -> Thermal {
    let zones = names(root, "thermal_zone")
        .into_iter()
        .filter_map(|z| {
            let at = |f: &str| format!("/sys/class/thermal/{z}/{f}");
            let kind = text(root, &at("type"))?;
            let mut trips: Vec<Trip> = (0..32)
                .map_while(|n| {
                    let kind = text(root, &at(&format!("trip_point_{n}_type")))?;
                    Some((kind, text(root, &at(&format!("trip_point_{n}_temp")))))
                })
                .filter_map(|(kind, temp)| {
                    let milli: i64 = temp?.parse().ok()?;
                    // -274 °C and below means the trip point is turned off.
                    (milli > -273_000).then(|| Trip {
                        kind,
                        celsius: milli as f64 / 1000.0,
                    })
                })
                .collect();
            trips.sort_by(|a, b| a.celsius.total_cmp(&b.celsius));
            trips.dedup();
            Some(Zone {
                celsius: text(root, &at("temp"))
                    .and_then(|t| t.parse::<i64>().ok())
                    .filter(|t| *t > -273_000)
                    .map(|t| t as f64 / 1000.0),
                policy: text(root, &at("policy")),
                trips,
                zone: z,
                kind,
            })
        })
        .collect();
    let mut cooling: Vec<Cooling> = Vec::new();
    for c in names(root, "cooling_device") {
        let at = |f: &str| format!("/sys/class/thermal/{c}/{f}");
        let (Some(kind), Some(level), Some(max)) = (
            text(root, &at("type")),
            text(root, &at("cur_state")).and_then(|v| v.parse::<u64>().ok()),
            text(root, &at("max_state")).and_then(|v| v.parse::<u64>().ok()),
        ) else {
            continue;
        };
        match cooling.iter_mut().find(|x| x.kind == kind) {
            Some(x) => {
                x.count += 1;
                x.level = x.level.max(level);
                x.max_level = x.max_level.max(max);
            }
            None => cooling.push(Cooling {
                kind,
                count: 1,
                level,
                max_level: max,
            }),
        }
    }
    Thermal { zones, cooling }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn zones_keep_trips_in_use_and_cooling_is_grouped() {
        let t = "/sys/class/thermal";
        let root = fixture_root(
            "thermal",
            &[
                (&format!("{t}/thermal_zone1/type"), "TSKN\n"),
                (&format!("{t}/thermal_zone1/temp"), "49050\n"),
                (&format!("{t}/thermal_zone1/policy"), "step_wise\n"),
                (&format!("{t}/thermal_zone1/trip_point_0_type"), "passive\n"),
                (&format!("{t}/thermal_zone1/trip_point_0_temp"), "-274000\n"),
                (
                    &format!("{t}/thermal_zone1/trip_point_1_type"),
                    "critical\n",
                ),
                (&format!("{t}/thermal_zone1/trip_point_1_temp"), "127050\n"),
                (&format!("{t}/thermal_zone1/trip_point_2_type"), "passive\n"),
                (&format!("{t}/thermal_zone1/trip_point_2_temp"), "65050\n"),
                (&format!("{t}/thermal_zone10/type"), "x86_pkg_temp\n"),
                (&format!("{t}/thermal_zone10/temp"), "57000\n"),
                (&format!("{t}/thermal_zone2/type"), "TMEM\n"),
                (&format!("{t}/cooling_device0/type"), "Processor\n"),
                (&format!("{t}/cooling_device0/cur_state"), "0\n"),
                (&format!("{t}/cooling_device0/max_state"), "3\n"),
                (&format!("{t}/cooling_device1/type"), "Processor\n"),
                (&format!("{t}/cooling_device1/cur_state"), "2\n"),
                (&format!("{t}/cooling_device1/max_state"), "3\n"),
                (&format!("{t}/cooling_device2/type"), "TCC Offset\n"),
                (&format!("{t}/cooling_device2/cur_state"), "8\n"),
                (&format!("{t}/cooling_device2/max_state"), "63\n"),
            ],
        );
        let th = read(&root);
        assert_eq!(
            th.zones.iter().map(|z| z.kind.as_str()).collect::<Vec<_>>(),
            ["TSKN", "TMEM", "x86_pkg_temp"]
        );
        let skin = &th.zones[0];
        assert_eq!(skin.celsius, Some(49.05));
        assert_eq!(
            skin.trips
                .iter()
                .map(|t| (t.kind.as_str(), t.celsius))
                .collect::<Vec<_>>(),
            [("passive", 65.05), ("critical", 127.05)]
        );
        assert_eq!(th.zones[1].celsius, None);
        assert_eq!(
            th.cooling,
            [
                Cooling {
                    kind: "Processor".into(),
                    count: 2,
                    level: 2,
                    max_level: 3
                },
                Cooling {
                    kind: "TCC Offset".into(),
                    count: 1,
                    level: 8,
                    max_level: 63
                }
            ]
        );
    }
}
