// SPDX-License-Identifier: Apache-2.0

//! Measured values over time.

use serde::{Deserialize, Serialize};

use crate::Timestamp;

/// The unit of a measured value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    /// 0 to 100.
    Percent,
    /// Bytes.
    Bytes,
    /// Bytes per second.
    BytesPerSecond,
    /// Degrees Celsius. The UI converts to Fahrenheit when the user asks.
    Celsius,
    /// Watts.
    Watts,
    /// Seconds.
    Seconds,
    /// A plain count.
    Count,
}

/// One measured value at one moment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metric {
    /// What was measured, as `<area>.<what>`, such as `cpu.usage` or `disk.used`.
    pub name: String,
    /// Which one, when there are many: a mount point, a CPU core, a sensor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The value, in `unit`.
    pub value: f64,
    /// The unit of `value`.
    pub unit: Unit,
    /// When it was measured.
    pub at: Timestamp,
}

/// The shortest bucket [`thin`] uses: points are stored one minute apart.
const MINUTE_MS: i64 = 60_000;

/// Averages points (oldest first) into time buckets, so about `max` cover `span_ms`.
///
/// Buckets are whole minutes long and start at whole multiples of their length, so two
/// metrics thinned over the same span get the same times and line up in one chart.
/// Each result point carries the start time of its bucket.
pub fn thin(points: Vec<Metric>, span_ms: i64, max: usize) -> Vec<Metric> {
    let max = i64::try_from(max.max(1)).unwrap_or(i64::MAX);
    let per_point = span_ms.max(0) / max;
    let step = (per_point + MINUTE_MS - 1) / MINUTE_MS * MINUTE_MS;
    if step <= MINUTE_MS {
        return points;
    }
    let mut out: Vec<Metric> = Vec::new();
    let mut count = 0.0;
    for point in points {
        let bucket = Timestamp(point.at.as_millis().div_euclid(step) * step);
        match out.last_mut() {
            Some(last) if last.at == bucket => {
                count += 1.0;
                last.value += (point.value - last.value) / count;
            }
            _ => {
                count = 1.0;
                out.push(Metric {
                    at: bucket,
                    ..point
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn point(minute: i64, value: f64) -> Metric {
        Metric {
            name: "cpu.usage".into(),
            subject: None,
            value,
            unit: Unit::Percent,
            at: Timestamp(minute * MINUTE_MS),
        }
    }

    #[test]
    fn thin_averages_into_aligned_buckets() {
        let points: Vec<Metric> = (0..60).map(|m| point(m, (m % 2) as f64 * 10.0)).collect();
        // One hour into 6 buckets of 10 minutes.
        let thinned = thin(points.clone(), 60 * MINUTE_MS, 6);
        assert_eq!(thinned.len(), 6);
        assert_eq!(thinned[1].at, Timestamp(10 * MINUTE_MS));
        assert!(thinned.iter().all(|p| (p.value - 5.0).abs() < 1e-9));
        // Short spans keep every point.
        assert_eq!(thin(points, 60 * MINUTE_MS, 500).len(), 60);
    }

    #[test]
    fn json_shape_is_stable() {
        let metric = Metric {
            name: "disk.used".into(),
            subject: Some("/".into()),
            value: 94.2,
            unit: Unit::Percent,
            at: Timestamp(5),
        };
        assert_eq!(
            serde_json::to_value(&metric).unwrap(),
            json!({"name": "disk.used", "subject": "/", "value": 94.2, "unit": "percent", "at": 5})
        );
        assert_eq!(
            serde_json::to_value(Unit::BytesPerSecond).unwrap(),
            "bytes_per_second"
        );
    }
}
