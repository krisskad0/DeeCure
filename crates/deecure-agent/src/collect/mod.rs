// SPDX-License-Identifier: Apache-2.0

//! Collectors: read the system and turn what they find into metrics and inventories.
//!
//! File collectors read through a [`SystemRoot`], so tests can point them at saved
//! copies of `/proc` and `/sys` from real machines (`tests/fixtures/`).
//! Metric names and units: `docs/02-architecture/collectors.md`.

mod accounts;
mod backup;
mod battery;
mod boot;
mod containers;
mod cpu;
mod diskio;
mod filesystems;
mod health;
mod memory;
mod network;
mod pressure;
mod sensors;
mod services;
mod updates;

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use deecure_core::{Metric, Snapshot, Timestamp, Unit};
use tokio::time::{MissedTickBehavior, interval};

use crate::store::{self, SharedStore, Store};

pub use accounts::Accounts;
pub use backup::Backups;
pub use battery::Battery;
pub use boot::{Boot, BootInfo};
pub use containers::Containers;
pub use cpu::Cpu;
pub use diskio::DiskIo;
pub use filesystems::{Filesystems, FsUsage};
pub use health::{Health, HealthSlow};
pub use memory::Memory;
pub use network::Network;
pub use pressure::Pressure;
pub use sensors::Sensors;
pub use services::{Services, UnitState, summarize as summarize_services};
pub use updates::{Updates, UpdatesInfo, Upgradable};

/// How often the file collectors run in the background.
pub const EVERY: Duration = Duration::from_secs(60);
/// Services are checked on every fifth run, so every 5 minutes.
const SERVICES_EVERY_NTH_RUN: u64 = 5;

/// Where `/proc` and `/sys` are read from: `/` on a real system, a fixture folder in tests.
#[derive(Debug, Clone)]
pub struct SystemRoot(PathBuf);

impl SystemRoot {
    /// The running system.
    pub fn real() -> Self {
        SystemRoot(PathBuf::from("/"))
    }

    /// A folder that holds saved copies of `proc/…` and `sys/…`.
    pub fn at(dir: impl Into<PathBuf>) -> Self {
        SystemRoot(dir.into())
    }

    /// Maps an absolute system path such as `/proc/stat` into this root.
    pub fn path(&self, absolute: &str) -> PathBuf {
        self.0.join(absolute.trim_start_matches('/'))
    }

    /// Reads a whole text file, such as `/proc/meminfo`.
    pub fn read(&self, absolute: &str) -> io::Result<String> {
        std::fs::read_to_string(self.path(absolute))
    }
}

/// What one collector run produced.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Sample {
    /// Measured values.
    pub metrics: Vec<Metric>,
    /// Inventories. They are stored only when they change.
    pub snapshots: Vec<Snapshot>,
}

/// A collector that only reads files, so it never waits on other programs.
pub trait Collector: Send {
    /// A short name for logs, such as `cpu`.
    fn name(&self) -> &'static str;

    /// Reads the system once.
    fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample>;
}

/// Runs every collector forever and saves what they find. `after_save` runs right
/// after each save, with the store still locked (the agent checks rules there).
pub async fn run(
    store: SharedStore,
    root: SystemRoot,
    recheck: std::sync::Arc<tokio::sync::Notify>,
    mut after_save: impl FnMut(&Store, Timestamp) + Send + 'static,
) {
    let mut collectors: Vec<Box<dyn Collector>> = vec![
        Box::new(Cpu::default()),
        Box::new(Memory::default()),
        Box::new(Battery::default()),
        Box::new(Filesystems::real()),
        Box::new(Network::default()),
        Box::new(DiskIo::default()),
        Box::new(Pressure::default()),
        Box::new(Sensors::default()),
        Box::new(Accounts),
        Box::new(Health::default()),
    ];
    let mut health_slow = HealthSlow::default();
    let mut services = Services::default();
    let mut boot = Boot::default();
    let mut updates = Updates::default();
    let mut backups = Backups::default();
    let mut containers = Containers;
    // Collectors that are failing right now, so each failure is logged once, not every minute.
    let mut failing = HashSet::new();
    let mut timer = interval(EVERY);
    timer.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let mut run_number: u64 = 0;
    loop {
        // Every minute, or right away when a fix asks for a fresh check.
        tokio::select! {
            _ = timer.tick() => {}
            () = recheck.notified() => {}
        }
        let now = Timestamp::now();
        let mut sample = Sample::default();
        for collector in &mut collectors {
            let result = collector.collect(&root, now);
            merge(&mut sample, &mut failing, collector.name(), result);
        }
        if run_number.is_multiple_of(SERVICES_EVERY_NTH_RUN) {
            let result = services.collect(now).await;
            merge(&mut sample, &mut failing, Services::NAME, result);
            // Reads once per boot; until the boot has finished, it tries again here.
            let result = boot.collect(&root, now).await;
            merge(&mut sample, &mut failing, Boot::NAME, result);
            let result = containers.collect(&root, now).await;
            merge(&mut sample, &mut failing, Containers::NAME, result);
        }
        if run_number.is_multiple_of(HealthSlow::EVERY_NTH_RUN) {
            let result = health_slow.collect(&root, now).await;
            merge(&mut sample, &mut failing, HealthSlow::NAME, result);
        }
        let result = updates.collect(&root, now).await;
        merge(&mut sample, &mut failing, Updates::NAME, result);
        let result = backups.collect(&root, now).await;
        merge(&mut sample, &mut failing, Backups::NAME, result);
        save(&store, &sample, now, &mut after_save);
        run_number = run_number.wrapping_add(1);
    }
}

fn merge(
    sample: &mut Sample,
    failing: &mut HashSet<&'static str>,
    name: &'static str,
    result: io::Result<Sample>,
) {
    match result {
        Ok(part) => {
            if failing.remove(name) {
                eprintln!("deecure-agent: the {name} collector works again");
            }
            sample.metrics.extend(part.metrics);
            sample.snapshots.extend(part.snapshots);
        }
        Err(e) => {
            if failing.insert(name) {
                eprintln!("deecure-agent: the {name} collector failed: {e}");
            }
        }
    }
}

fn save(
    store: &SharedStore,
    sample: &Sample,
    now: Timestamp,
    after_save: &mut impl FnMut(&Store, Timestamp),
) {
    let store = store::lock(store);
    if let Err(e) = store.add_metrics(&sample.metrics) {
        eprintln!("deecure-agent: could not save metrics: {e}");
    }
    for snapshot in &sample.snapshots {
        if let Err(e) = store.add_snapshot_if_changed(snapshot) {
            eprintln!(
                "deecure-agent: could not save the {:?} inventory: {e}",
                snapshot.kind
            );
        }
    }
    after_save(&store, now);
}

/// Builds a metric.
fn metric(name: &str, subject: Option<&str>, value: f64, unit: Unit, at: Timestamp) -> Metric {
    Metric {
        name: name.to_owned(),
        subject: subject.map(str::to_owned),
        value,
        unit,
        at,
    }
}

/// An error for a file that does not look the way the kernel writes it.
fn invalid(file: &str, why: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("{file}: {why}"))
}

/// Reads a small `/sys` file and trims it, or `None` if it is missing or unreadable.
fn read_trimmed(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_owned())
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::SystemRoot;

    /// Writes `files` (absolute path → content) into a fresh folder and returns it as a root.
    pub fn fixture_root(name: &str, files: &[(&str, &str)]) -> SystemRoot {
        let dir =
            std::env::temp_dir().join(format!("deecure-collect-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let root = SystemRoot::at(&dir);
        for (path, content) in files {
            let file = root.path(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, content).unwrap();
        }
        root
    }

    /// Finds a metric value by name and subject.
    pub fn value(sample: &super::Sample, name: &str, subject: Option<&str>) -> Option<f64> {
        sample
            .metrics
            .iter()
            .find(|m| m.name == name && m.subject.as_deref() == subject)
            .map(|m| m.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_maps_absolute_paths() {
        let root = SystemRoot::at("/fixtures/laptop");
        assert_eq!(
            root.path("/proc/stat"),
            PathBuf::from("/fixtures/laptop/proc/stat")
        );
        assert_eq!(
            SystemRoot::real().path("/proc/stat"),
            PathBuf::from("/proc/stat")
        );
    }

    #[test]
    fn a_failure_is_logged_once_and_recovery_clears_it() {
        let mut sample = Sample::default();
        let mut failing = HashSet::new();
        merge(
            &mut sample,
            &mut failing,
            "cpu",
            Err(io::Error::other("boom")),
        );
        assert!(failing.contains("cpu"));
        merge(
            &mut sample,
            &mut failing,
            "cpu",
            Err(io::Error::other("boom")),
        );
        assert_eq!(failing.len(), 1);
        merge(&mut sample, &mut failing, "cpu", Ok(Sample::default()));
        assert!(failing.is_empty());
    }
}
