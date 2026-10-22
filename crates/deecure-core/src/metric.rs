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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
