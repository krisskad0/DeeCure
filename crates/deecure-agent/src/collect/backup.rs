// SPDX-License-Identifier: Apache-2.0

//! Whether backups are set up and how old the newest one is, for rules. Reads the same
//! settings and folders as the `backups` inventory, once an hour.

use std::io;
use std::time::{Duration, Instant};

use deecure_core::{Timestamp, Unit};

use super::{Sample, SystemRoot, metric};
use crate::inventory::backups::{self, BackupsInfo};

const EVERY: Duration = Duration::from_secs(3600);

/// `backup.configured` (0 or 1) and `backup.age_days`.
#[derive(Debug, Default)]
pub struct Backups {
    last: Option<(Instant, BackupsInfo)>,
}

impl Backups {
    /// The name used in logs.
    pub const NAME: &'static str = "backups";

    /// Reads at most once an hour and repeats the last answer in between.
    pub async fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        if self
            .last
            .as_ref()
            .is_none_or(|(at, _)| at.elapsed() >= EVERY)
        {
            self.last = Some((Instant::now(), backups::read(root).await));
        }
        let mut sample = Sample::default();
        if let Some((_, info)) = &self.last {
            sample.metrics.push(metric(
                "backup.configured",
                None,
                f64::from(u8::from(info.configured())),
                Unit::Count,
                now,
            ));
            if let Some(days) = backups::age_days(info) {
                sample
                    .metrics
                    .push(metric("backup.age_days", None, days, Unit::Count, now));
            }
        }
        Ok(sample)
    }
}
