// SPDX-License-Identifier: Apache-2.0

//! Battery use per app, by local day: today and the 7 days before, from the estimate the
//! `AppBattery` collector stores each minute on battery (`app.battery_mwh`).

use deecure_core::{Timestamp, local};
use serde::Serialize;

use crate::collect::app_battery::METRIC;
use crate::context::Agent;
use crate::store;

/// Days counted, today included.
const DAYS: i32 = 8;

/// One app, service, or part, on one day.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Use {
    /// `app`, `service`, `user-service`, `other` (small ones together), or `rest` (the screen,
    /// Wi-Fi, and the system at rest).
    pub kind: String,
    pub name: String,
    /// Energy from the battery, in milliwatt hours.
    pub mwh: f64,
}

/// One local day.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Day {
    /// Local midnight, in ms since 1970.
    pub start: i64,
    /// Biggest first.
    pub uses: Vec<Use>,
}

/// What `app_battery` answers.
#[derive(Debug, Default, Serialize)]
pub struct AppBatteryInfo {
    /// Today first.
    pub days: Vec<Day>,
}

/// Splits a stored subject such as `app:Files` into its kind and name.
pub fn parse(subject: &str) -> (String, String) {
    match subject.split_once(':') {
        Some((kind, name)) => (kind.to_owned(), name.to_owned()),
        None => (subject.to_owned(), String::new()),
    }
}

pub fn read(agent: &Agent) -> AppBatteryInfo {
    read_in(&store::lock(&agent.store), Timestamp::now(), &local::zone())
}

fn read_in(store: &store::Store, now: Timestamp, tz: &local::TimeZone) -> AppBatteryInfo {
    let mut days = Vec::new();
    for back in 0..DAYS {
        let (Some(start), Some(end)) = (
            local::day_start_in(now, -back, tz),
            local::day_start_in(now, 1 - back, tz),
        ) else {
            continue;
        };
        let uses = store
            .metric_sums(METRIC, start, Timestamp(end.as_millis() - 1))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(subject, mwh)| {
                let (kind, name) = parse(&subject?);
                Some(Use { kind, name, mwh })
            })
            .collect();
        days.push(Day {
            start: start.as_millis(),
            uses,
        });
    }
    AppBatteryInfo { days }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deecure_core::{Metric, Unit};

    #[test]
    fn days_add_up_per_app_in_local_time() {
        let store = store::Store::open_in_memory().unwrap();
        let tz = local::fixed(0);
        let now = Timestamp(10 * 86_400_000 + 12 * 3_600_000); // noon on day 10
        let used = |subject: &str, at: i64, value: f64| Metric {
            name: METRIC.into(),
            subject: Some(subject.into()),
            value,
            unit: Unit::Count,
            at: Timestamp(at),
        };
        store
            .add_metrics(&[
                used("app:Files", now.as_millis() - 60_000, 30.0),
                used("app:Files", now.as_millis(), 20.0),
                used("rest", now.as_millis(), 70.0),
                // Yesterday, one minute before midnight.
                used("service:cups", 10 * 86_400_000 - 60_000, 5.0),
            ])
            .unwrap();
        let info = read_in(&store, now, &tz);
        assert_eq!(info.days.len(), 8);
        assert_eq!(info.days[0].start, 10 * 86_400_000);
        assert_eq!(
            info.days[0].uses,
            [
                Use {
                    kind: "rest".into(),
                    name: String::new(),
                    mwh: 70.0
                },
                Use {
                    kind: "app".into(),
                    name: "Files".into(),
                    mwh: 50.0
                },
            ]
        );
        assert_eq!(info.days[1].uses.len(), 1);
        assert_eq!(info.days[1].uses[0].name, "cups");
        assert!(info.days[2].uses.is_empty());
        assert_eq!(
            parse("user-service:Desktop (GNOME Shell)"),
            ("user-service".into(), "Desktop (GNOME Shell)".into())
        );
    }
}
