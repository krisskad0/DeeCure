// SPDX-License-Identifier: Apache-2.0

//! Pressure stall information (PSI) from `/proc/pressure/`: how much of the time work
//! was waiting for CPU, memory, or disk.

use std::io;

use deecure_core::{Timestamp, Unit};

use super::{Collector, Sample, SystemRoot, metric};

/// The `some avg10` value of a `/proc/pressure/*` file, in percent.
pub fn parse_some_avg10(text: &str) -> Option<f64> {
    let line = text.lines().find(|l| l.starts_with("some "))?;
    line.split_whitespace()
        .find_map(|field| field.strip_prefix("avg10="))?
        .parse()
        .ok()
}

/// `pressure.cpu`, `pressure.memory`, `pressure.io`. Kernels without PSI report nothing.
#[derive(Debug, Default)]
pub struct Pressure;

impl Collector for Pressure {
    fn name(&self) -> &'static str {
        "pressure"
    }

    fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let mut sample = Sample::default();
        for resource in ["cpu", "memory", "io"] {
            if let Some(value) = root
                .read(&format!("/proc/pressure/{resource}"))
                .ok()
                .as_deref()
                .and_then(parse_some_avg10)
            {
                sample.metrics.push(metric(
                    &format!("pressure.{resource}"),
                    None,
                    value,
                    Unit::Percent,
                    now,
                ));
            }
        }
        Ok(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_some_avg10() {
        let text = "some avg10=1.25 avg60=0.50 avg300=0.03 total=170018077\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=0\n";
        assert_eq!(parse_some_avg10(text), Some(1.25));
        assert_eq!(parse_some_avg10("nothing"), None);
    }
}
