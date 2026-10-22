// SPDX-License-Identifier: Apache-2.0

//! Replays every machine in `tests/fixtures/` through the real collectors and rules,
//! and checks that DeeCure reports exactly the expected findings.
//!
//! Each fixture holds saved copies of `/proc` and `/sys` files, disk space
//! (`statvfs.json`), the service list (`services.json`), and `expected-findings.yaml`.
//! Capture a new one with `scripts/capture-fixture.py`.

#![allow(clippy::unwrap_used, clippy::expect_used)] // test-only file

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use deecure_agent::collect::{
    Battery, Collector, Cpu, Filesystems, FsUsage, Memory, Sample, SystemRoot, UnitState,
    summarize_services,
};
use deecure_agent::rules::{self, Rule};
use deecure_agent::store::Store;
use deecure_core::{Severity, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct Expected {
    rule: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    subject: Option<String>,
    severity: Severity,
}

#[derive(Deserialize)]
struct Usage {
    size: u64,
    used: u64,
    free: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Info {
    description: String,
    source: String,
    captured: String,
    machine: String,
    os: String,
    kernel: String,
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixtures() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(repo().join("tests/fixtures"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    dirs
}

fn shipped_rules() -> Vec<Rule> {
    let (rules, errors) = rules::load_dir(&repo().join("rules"));
    assert!(errors.is_empty(), "{errors:#?}");
    rules
}

/// Runs one fixture and returns the open findings, sorted.
fn run(dir: &Path, rules: &[Rule]) -> Vec<Expected> {
    let name = dir.file_name().unwrap().to_string_lossy().into_owned();
    let now = Timestamp::now();
    let store = Store::open_in_memory().unwrap();
    let save = |sample: Sample| {
        store.add_metrics(&sample.metrics).unwrap();
        for snapshot in &sample.snapshots {
            store.add_snapshot_if_changed(snapshot).unwrap();
        }
    };

    let usage: HashMap<String, Usage> = serde_json::from_str(
        &std::fs::read_to_string(dir.join("statvfs.json")).expect("statvfs.json"),
    )
    .unwrap();
    let stat = move |path: &Path| {
        usage
            .get(path.to_str().unwrap_or_default())
            .map(|u| FsUsage {
                size: u.size,
                used: u.used,
                free: u.free,
                ..FsUsage::default()
            })
            .ok_or_else(|| io::Error::from(io::ErrorKind::PermissionDenied))
    };
    let root = SystemRoot::at(dir);
    let collectors: Vec<Box<dyn Collector>> = vec![
        Box::new(Cpu::default()),
        Box::new(Memory),
        Box::new(Battery),
        Box::new(Filesystems::with_stat(stat)),
    ];
    for mut collector in collectors {
        let sample = collector
            .collect(&root, now)
            .unwrap_or_else(|e| panic!("{name}: the {} collector failed: {e}", collector.name()));
        save(sample);
    }
    let units: Vec<UnitState> = serde_json::from_str(
        &std::fs::read_to_string(dir.join("services.json")).expect("services.json"),
    )
    .unwrap();
    save(summarize_services(units, now));

    let facts = rules::gather(&store, now).unwrap();
    let mistakes = rules::apply(&store, rules, &facts, now).unwrap();
    assert!(mistakes.is_empty(), "{name}: {mistakes:?}");

    let mut found: Vec<Expected> = store
        .open_findings()
        .unwrap()
        .into_iter()
        .map(|f| Expected {
            rule: f.rule.to_string(),
            subject: f.subject,
            severity: f.severity,
        })
        .collect();
    found.sort();
    found
}

#[test]
fn every_fixture_gives_exactly_the_expected_findings() {
    let rules = shipped_rules();
    let mut failures = Vec::new();
    for dir in fixtures() {
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        let mut expected: Vec<Expected> = serde_saphyr::from_str(
            &std::fs::read_to_string(dir.join("expected-findings.yaml"))
                .expect("expected-findings.yaml"),
        )
        .unwrap();
        expected.sort();
        let found = run(&dir, &rules);
        if found != expected {
            failures.push(format!(
                "{name}:\n  expected {expected:?}\n  found    {found:?}\n  If the new result is right, put this in expected-findings.yaml:\n{}",
                serde_json::to_string_pretty(&found).unwrap()
            ));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n\n"));
}

#[test]
fn there_are_enough_fixtures_and_each_is_described() {
    let dirs = fixtures();
    assert!(
        dirs.len() >= 3,
        "keep at least 3 machines in tests/fixtures/"
    );
    for dir in dirs {
        let info: Info =
            serde_saphyr::from_str(&std::fs::read_to_string(dir.join("fixture.yaml")).unwrap())
                .unwrap_or_else(|e| panic!("{}: fixture.yaml: {e}", dir.display()));
        assert!(
            matches!(info.source.as_str(), "real" | "synthetic"),
            "{}: source",
            dir.display()
        );
        assert!(!info.description.is_empty() && !info.os.is_empty());
        let _ = (info.captured, info.machine, info.kernel);
    }
}
