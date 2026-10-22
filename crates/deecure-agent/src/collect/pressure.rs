// SPDX-License-Identifier: Apache-2.0

//! Pressure stall information (PSI) from `/proc/pressure/`: how much of the time work
//! was waiting for CPU, memory, or disk.

use std::io;

use deecure_core::{Timestamp, Unit};

use super::{Collector, Sample, SystemRoot, metric};

/// The `avg10` value of the `some` or `full` line of a `/proc/pressure/*` file, in percent.
/// `some`: at least one task waited. `full`: all tasks waited at once.
pub fn parse_avg10(text: &str, kind: &str) -> Option<f64> {
    let line = text
        .lines()
        .find(|l| l.split_whitespace().next() == Some(kind))?;
    line.split_whitespace()
        .find_map(|field| field.strip_prefix("avg10="))?
        .parse()
        .ok()
}

/// `pressure.cpu`, `pressure.memory`, `pressure.io` (some), and `pressure.memory_full`,
/// `pressure.io_full`. Kernels without PSI report nothing.
#[derive(Debug, Default)]
pub struct Pressure;

impl Collector for Pressure {
    fn name(&self) -> &'static str {
        "pressure"
    }

    fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let mut sample = Sample::default();
        for resource in ["cpu", "memory", "io"] {
            let Ok(text) = root.read(&format!("/proc/pressure/{resource}")) else {
                continue;
            };
            let mut push = |name: String, value: Option<f64>| {
                if let Some(value) = value {
                    sample
                        .metrics
                        .push(metric(&name, None, value, Unit::Percent, now));
                }
            };
            push(format!("pressure.{resource}"), parse_avg10(&text, "some"));
            // The system-wide CPU "full" line is always zero, so it is left out.
            if resource != "cpu" {
                push(
                    format!("pressure.{resource}_full"),
                    parse_avg10(&text, "full"),
                );
            }
        }
        Ok(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_some_and_full_avg10() {
        let text = "some avg10=1.25 avg60=0.50 avg300=0.03 total=170018077\nfull avg10=0.40 avg60=0.00 avg300=0.00 total=0\n";
        assert_eq!(parse_avg10(text, "some"), Some(1.25));
        assert_eq!(parse_avg10(text, "full"), Some(0.40));
        assert_eq!(parse_avg10("nothing", "some"), None);
    }
}
