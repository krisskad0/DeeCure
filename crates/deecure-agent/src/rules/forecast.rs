// SPDX-License-Identifier: Apache-2.0

//! Predictions from history: "the disk will be full in about N days".
//!
//! A disk often jumps once (an install, a download, a build) and then stays flat. A straight
//! line through all the history turns such a jump into "fills every day". So the rate is
//! measured twice, over the last day and over the last 3 days (a straight line through hourly
//! averages each), and a disk is filling only when both say so; the slower of the two is used.
//! A jump that stopped shows up only in the 3-day rate, so it does not count; a disk that keeps
//! filling shows up in both.
//!
//! When the last day alone is flat or emptying, the disk is not filling, whatever the 3 days
//! say. It needs 3 days of history, and each window needs readings spread over most of it with
//! some in at least 4 of every 10 hours (a laptop sleeps at night), so it says nothing instead
//! of guessing. Growth under 0.05% a day, or a date over a year away, is "not filling".

use deecure_core::{Metric, thin};
use serde::Serialize;

/// How far back the trend looks.
pub const WINDOW_MS: i64 = 7 * 86_400_000;
const HOUR_MS: i64 = 3_600_000;
const DAY_MS: f64 = 86_400_000.0;
/// The two windows the rate is measured over.
const SHORT_MS: i64 = 86_400_000;
const LONG_MS: i64 = 3 * 86_400_000;
/// History needed before anything is said.
pub const MIN_HISTORY_DAYS: f64 = 3.0;
/// A window's readings must span this share of it...
const COVERAGE: f64 = 0.7;
/// ...and fall in at least this share of its hours.
const HOURS_COVERED: f64 = 0.4;
/// Slower than this is "not filling": noise, logs, caches.
const MIN_PERCENT_PER_DAY: f64 = 0.05;
const MAX_DAYS: f64 = 365.0;

/// What the trend says about one disk.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Forecast {
    /// How fast it fills now, in percent of the disk per day (negative: it is emptying).
    pub percent_per_day: f64,
    /// Days until 100% at this rate; `None` when it is not filling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub days_until_full: Option<f64>,
}

/// Days the points cover, from the first to the last.
pub fn history_days(points: &[Metric]) -> f64 {
    match (points.first(), points.last()) {
        (Some(a), Some(b)) => (b.at.as_millis() - a.at.as_millis()) as f64 / DAY_MS,
        _ => 0.0,
    }
}

/// The least-squares slope of hourly points inside the last `window_ms`, in percent per day;
/// `None` when readings cover too little of the window.
fn rate(hourly: &[Metric], window_ms: i64) -> Option<f64> {
    let end = hourly.last()?.at.as_millis();
    let pts: Vec<&Metric> = hourly
        .iter()
        .filter(|p| p.at.as_millis() >= end - window_ms)
        .collect();
    let needed = (window_ms as f64 / HOUR_MS as f64 * HOURS_COVERED).ceil() as usize;
    let first = pts.first()?.at.as_millis();
    if pts.len() < needed || ((end - first) as f64) < window_ms as f64 * COVERAGE {
        return None;
    }
    let xs: Vec<f64> = pts
        .iter()
        .map(|p| p.at.as_millis() as f64 / DAY_MS)
        .collect();
    let ys: Vec<f64> = pts.iter().map(|p| p.value).collect();
    let n = xs.len() as f64;
    let (mx, my) = (xs.iter().sum::<f64>() / n, ys.iter().sum::<f64>() / n);
    let num: f64 = xs.iter().zip(&ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    let den: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum();
    (den > 0.0).then(|| num / den)
}

/// The trend of `disk.used_percent` points (oldest first), or `None` with too little data.
pub fn disk(points: Vec<Metric>) -> Option<Forecast> {
    if history_days(&points) < MIN_HISTORY_DAYS {
        return None;
    }
    let last = points.last()?.value;
    let span = points.last()?.at.as_millis() - points.first()?.at.as_millis();
    // Hourly averages, so a busy minute does not outweigh a quiet day.
    let hourly = thin(
        points,
        span,
        usize::try_from(span / HOUR_MS).unwrap_or(1).max(1),
    );
    let short = rate(&hourly, SHORT_MS)?;
    if short < MIN_PERCENT_PER_DAY {
        // Flat or emptying over the last day: not filling, so the 3 days are not needed.
        return Some(Forecast {
            percent_per_day: short,
            days_until_full: None,
        });
    }
    let long = rate(&hourly, LONG_MS)?;
    let filling = long >= MIN_PERCENT_PER_DAY;
    let slope = if filling { short.min(long) } else { short };
    let days = (100.0 - last).max(0.0) / slope;
    Some(Forecast {
        percent_per_day: slope,
        days_until_full: (filling && days <= MAX_DAYS).then_some(days),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use deecure_core::{Timestamp, Unit};

    const START: i64 = 1_800_000_000_000;

    fn point(h: i64, value: f64) -> Metric {
        Metric {
            name: "disk.used_percent".into(),
            subject: Some("/".into()),
            value,
            unit: Unit::Percent,
            at: Timestamp(START + h * HOUR_MS),
        }
    }

    fn series(days: f64, start: f64, per_day: f64) -> Vec<Metric> {
        let steps = (days * 24.0) as i64;
        (0..=steps)
            .map(|h| point(h, start + per_day * h as f64 / 24.0))
            .collect()
    }

    #[test]
    fn a_filling_disk_gets_a_date() {
        let f = disk(series(7.0, 80.0, 2.0)).unwrap();
        assert!((f.percent_per_day - 2.0).abs() < 1e-6);
        // 80 + 14 = 94% now, 6 points left at 2 a day.
        assert!((f.days_until_full.unwrap() - 3.0).abs() < 1e-6);
    }

    #[test]
    fn a_jump_that_stopped_is_not_filling() {
        // 42% for a day, a build adds 8% in a day, then flat at 50% for 2 days (with a little
        // noise): the old straight line said "6.6% a day".
        let mut points: Vec<Metric> = (0..24).map(|h| point(h, 42.0)).collect();
        points.extend((24..48).map(|h| point(h, 42.0 + 8.0 * (h - 24) as f64 / 24.0)));
        points.extend((48..=96).map(|h| point(h, 50.0 + 0.002 * (h - 48) as f64)));
        let f = disk(points).unwrap();
        assert_eq!(f.days_until_full, None);
        assert!(f.percent_per_day < 0.1, "{f:?}");
    }

    #[test]
    fn a_disk_that_keeps_filling_uses_the_slower_rate() {
        // 1% a day for 3 days, then 3% a day in the last day: the 3-day rate is the slower.
        let mut points: Vec<Metric> = (0..72).map(|h| point(h, 60.0 + h as f64 / 24.0)).collect();
        points.extend((72..=96).map(|h| point(h, 63.0 + 3.0 * (h - 72) as f64 / 24.0)));
        let f = disk(points).unwrap();
        assert!(f.percent_per_day > 1.0 && f.percent_per_day < 3.0, "{f:?}");
        assert!(f.days_until_full.is_some());
    }

    #[test]
    fn steady_emptying_or_slow_disks_are_not_filling() {
        assert_eq!(disk(series(7.0, 50.0, 0.0)).unwrap().days_until_full, None);
        assert_eq!(disk(series(7.0, 50.0, -1.0)).unwrap().days_until_full, None);
        assert_eq!(disk(series(7.0, 50.0, 0.01)).unwrap().days_until_full, None);
        assert_eq!(
            disk(series(7.0, 10.0, 0.1)).unwrap().days_until_full,
            None,
            "over a year"
        );
    }

    #[test]
    fn too_little_history_or_long_gaps_say_nothing() {
        assert_eq!(disk(series(2.5, 80.0, 5.0)), None);
        assert_eq!(disk(Vec::new()), None);
        // 4 days, but the agent was off for the whole last day but two hours.
        let mut gap = series(3.0, 50.0, 2.0);
        gap.push(point(94, 56.0));
        gap.push(point(96, 56.1));
        assert_eq!(disk(gap), None);
    }

    #[test]
    fn a_laptop_that_sleeps_still_gets_an_answer() {
        // Readings only from 8:00 to 20:00 each day (asleep at night), for 4 days.
        let awake = |h: i64| (h % 24) >= 8 && (h % 24) < 20;
        let flat: Vec<Metric> = (0..=96)
            .filter(|h| awake(*h))
            .map(|h| point(h, 67.0))
            .collect();
        assert_eq!(disk(flat).unwrap().days_until_full, None);
        let filling: Vec<Metric> = (0..=96)
            .filter(|h| awake(*h))
            .map(|h| point(h, 60.0 + 2.0 * h as f64 / 24.0))
            .collect();
        assert!(disk(filling).unwrap().days_until_full.is_some());
    }

    #[test]
    fn emptying_in_the_last_day_is_not_filling_even_with_gaps_before() {
        // Two jumps up (as before the warning), the agent mostly off on days 2 and 3, then a
        // clean-up in the last day: the last day decides.
        let mut points: Vec<Metric> = (0..24)
            .map(|h| point(h, 52.0 + 15.0 * h as f64 / 24.0))
            .collect();
        points.extend([30, 40, 50].map(|h| point(h, 67.0)));
        points.extend((72..=96).map(|h| point(h, 67.0 - 3.0 * (h - 72) as f64 / 24.0)));
        let f = disk(points).unwrap();
        assert_eq!(f.days_until_full, None);
        assert!(f.percent_per_day < 0.0, "{f:?}");
    }

    #[test]
    fn a_full_disk_is_full_now() {
        let f = disk(series(4.0, 92.0, 2.0)).unwrap();
        assert_eq!(f.days_until_full, Some(0.0));
    }
}
