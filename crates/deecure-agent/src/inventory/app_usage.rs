// SPDX-License-Identifier: Apache-2.0

//! Use per app over the last 24 hours, 7 days, and 30 days, from the history the `AppUsage`
//! collector stores (CPU time and memory) and the traffic per app.

use std::collections::BTreeMap;

use deecure_core::Timestamp;
use serde::Serialize;

use super::app_battery::parse;
use crate::collect::app_usage::{CPU, MEMORY};
use crate::context::Agent;
use crate::store::{self, SubjectStats};

/// The metrics answered, each per app.
pub const METRICS: &[&str] = &[CPU, MEMORY];
/// The time windows: name and length.
const WINDOWS: &[(&str, i64)] = &[
    ("24h", 86_400_000),
    ("7d", 7 * 86_400_000),
    ("30d", 30 * 86_400_000),
];
/// Apps listed per metric and window.
const TOP: usize = 30;

/// One app, service, or `other`, over one window.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Entry {
    /// `app`, `service`, `user-service`, or `other`.
    pub kind: String,
    pub name: String,
    /// The stored subject, to ask for its history.
    pub subject: String,
    /// All readings added up (CPU seconds, bytes of traffic).
    pub total: f64,
    /// The average reading, and the highest (memory).
    pub average: f64,
    pub peak: f64,
}

/// One time window: per metric name, the biggest first.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Window {
    /// `24h`, `7d`, or `30d`, as the app's time ranges.
    pub range: &'static str,
    pub metrics: BTreeMap<&'static str, Vec<Entry>>,
}

/// What `app_usage` answers.
#[derive(Debug, Default, Serialize)]
pub struct AppUsageInfo {
    pub windows: Vec<Window>,
}

pub fn read(agent: &Agent) -> AppUsageInfo {
    read_in(&store::lock(&agent.store), Timestamp::now())
}

fn entry(stats: SubjectStats) -> Option<Entry> {
    let subject = stats.subject?;
    let (kind, name) = parse(&subject);
    Some(Entry {
        kind,
        name,
        subject,
        total: stats.sum,
        average: stats.sum / stats.readings.max(1) as f64,
        peak: stats.max,
    })
}

fn read_in(store: &store::Store, now: Timestamp) -> AppUsageInfo {
    let windows = WINDOWS
        .iter()
        .map(|&(range, length)| {
            let since = Timestamp(now.as_millis() - length);
            let metrics = METRICS
                .iter()
                .map(|&name| {
                    let mut list: Vec<Entry> = store
                        .metric_stats(name, since, now)
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(entry)
                        .collect();
                    // Memory is a level: the biggest on average first.
                    if name == MEMORY {
                        list.sort_by(|a, b| b.average.total_cmp(&a.average));
                    }
                    list.truncate(TOP);
                    (name, list)
                })
                .collect();
            Window { range, metrics }
        })
        .collect();
    AppUsageInfo { windows }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deecure_core::{Metric, Unit};

    #[test]
    fn windows_rank_apps_by_total_or_average() {
        let store = store::Store::open_in_memory().unwrap();
        let now = Timestamp(40 * 86_400_000);
        let point = |name: &str, subject: &str, ago: i64, value: f64| Metric {
            name: name.into(),
            subject: Some(subject.into()),
            value,
            unit: Unit::Count,
            at: Timestamp(now.as_millis() - ago),
        };
        let day = 86_400_000;
        store
            .add_metrics(&[
                point(CPU, "app:Files", 60_000, 10.0),
                point(CPU, "app:Files", 400_000, 5.0),
                point(CPU, "service:cups", 3 * day, 100.0),
                // Chrome is big once, Code all the time: Code is first on average.
                point(MEMORY, "app:Chrome", 60_000, 900.0),
                point(MEMORY, "app:Chrome", 400_000, 100.0),
                point(MEMORY, "app:Chrome", 700_000, 100.0),
                point(MEMORY, "app:Code", 60_000, 500.0),
            ])
            .unwrap();
        let info = read_in(&store, now);
        assert_eq!(info.windows.len(), 3);
        let day_cpu = &info.windows[0].metrics[CPU];
        assert_eq!(day_cpu.len(), 1);
        assert_eq!(
            (day_cpu[0].name.as_str(), day_cpu[0].total),
            ("Files", 15.0)
        );
        let week_cpu = &info.windows[1].metrics[CPU];
        assert_eq!(week_cpu[0].name, "cups");
        let mem = &info.windows[0].metrics[MEMORY];
        assert_eq!(mem[0].name, "Code");
        assert_eq!((mem[1].average, mem[1].peak), (1100.0 / 3.0, 900.0));
        assert_eq!(mem[1].subject, "app:Chrome");
    }
}
