// SPDX-License-Identifier: Apache-2.0

//! Predictions from history: "the disk will be full in about N days".
//!
//! A straight line (least squares) through the last 7 days of hourly averages. It needs
//! at least a day of data, so a fresh install predicts nothing instead of guessing. A
//! disk that grows less than 0.05% a day, or would take over a year, is "not filling".

use deecure_core::{Metric, thin};

/// How far back the trend looks.
pub const WINDOW_MS: i64 = 7 * 86_400_000;
const DAY_MS: f64 = 86_400_000.0;
/// Too little history to say anything.
const MIN_SPAN_MS: i64 = 86_400_000;
const MIN_POINTS: usize = 12;
/// Slower than this is "not filling": noise, logs, caches.
const MIN_PERCENT_PER_DAY: f64 = 0.05;
const MAX_DAYS: f64 = 365.0;

/// What the trend says about one disk.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Forecast {
    /// How fast it fills, in percent of the disk per day (negative: it is emptying).
    pub percent_per_day: f64,
    /// Days until 100% at this rate; `None` when it is not filling.
    pub days_until_full: Option<f64>,
}

/// The trend of `disk.used_percent` points (oldest first), or `None` with too little data.
pub fn disk(points: Vec<Metric>) -> Option<Forecast> {
    let span = points.last()?.at.as_millis() - points.first()?.at.as_millis();
    if span < MIN_SPAN_MS {
        return None;
    }
    // Hourly averages, so a busy minute does not outweigh a quiet day.
    let points = thin(points, span, 7 * 24);
    if points.len() < MIN_POINTS {
        return None;
    }
    let xs: Vec<f64> = points
        .iter()
        .map(|p| p.at.as_millis() as f64 / DAY_MS)
        .collect();
    let ys: Vec<f64> = points.iter().map(|p| p.value).collect();
    let n = xs.len() as f64;
    let (mx, my) = (xs.iter().sum::<f64>() / n, ys.iter().sum::<f64>() / n);
    let num: f64 = xs.iter().zip(&ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    let den: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum();
    if den == 0.0 {
        return None;
    }
    let slope = num / den;
    let last = *ys.last()?;
    let days = (100.0 - last).max(0.0) / slope;
    Some(Forecast {
        percent_per_day: slope,
        days_until_full: (slope >= MIN_PERCENT_PER_DAY && days <= MAX_DAYS).then_some(days),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use deecure_core::{Timestamp, Unit};

    fn series(days: f64, start: f64, per_day: f64) -> Vec<Metric> {
        let steps = (days * 24.0) as i64;
        (0..=steps)
            .map(|h| Metric {
                name: "disk.used_percent".into(),
                subject: Some("/".into()),
                value: start + per_day * h as f64 / 24.0,
                unit: Unit::Percent,
                at: Timestamp(1_800_000_000_000 + h * 3_600_000),
            })
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
    fn too_little_history_says_nothing() {
        assert_eq!(disk(series(0.5, 80.0, 5.0)), None);
        assert_eq!(disk(Vec::new()), None);
    }

    #[test]
    fn a_full_disk_is_full_now() {
        let f = disk(series(3.0, 95.0, 2.0)).unwrap();
        assert_eq!(f.days_until_full, Some(0.0));
    }
}
