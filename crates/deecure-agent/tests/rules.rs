// SPDX-License-Identifier: Apache-2.0

//! Checks every rule shipped in `rules/`: it loads, and it gives the right answer
//! for a healthy computer and for a problem.

#![allow(clippy::unwrap_used)] // test-only file: a failed unwrap is a failed test

use std::collections::BTreeMap;
use std::path::Path;

use deecure_agent::rules::expr::Value;
use deecure_agent::rules::facts::Item;
use deecure_agent::rules::{Facts, Outcome, Rule, evaluate, load_dir};
use deecure_core::Severity;

fn shipped_rules() -> Vec<Rule> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rules");
    let (rules, errors) = load_dir(&dir);
    assert!(errors.is_empty(), "broken rule files: {errors:#?}");
    rules
}

fn rule(id: &str) -> Rule {
    shipped_rules()
        .into_iter()
        .find(|r| r.id.as_str() == id)
        .unwrap()
}

fn list(name: &str, items: &[(&str, &[(&str, Value)])]) -> Facts {
    let items = items
        .iter()
        .map(|(subject, fields)| Item {
            subject: (*subject).into(),
            fields: fields
                .iter()
                .map(|(k, v)| ((*k).to_owned(), v.clone()))
                .collect(),
        })
        .collect();
    Facts {
        lists: BTreeMap::from([(name.to_owned(), items)]),
        ..Facts::default()
    }
}

fn severity(results: &[(Option<String>, Outcome)]) -> Vec<Option<Severity>> {
    results
        .iter()
        .map(|(_, outcome)| match outcome {
            Outcome::Fired { severity, .. } => Some(*severity),
            Outcome::Clear => None,
            other => panic!("unexpected outcome {other:?}"),
        })
        .collect()
}

#[test]
fn every_shipped_rule_loads() {
    assert!(shipped_rules().len() >= 3);
}

#[test]
fn root_almost_full() {
    let rule = rule("storage.root_almost_full");
    let disk = |used, free| {
        let mut facts = Facts::default();
        facts
            .values
            .insert("root.used_percent".into(), Value::Number(used));
        facts
            .values
            .insert("root.free_gb".into(), Value::Number(free));
        facts
    };
    assert_eq!(
        severity(&evaluate(&rule, &disk(41.0, 290.0)).unwrap()),
        [None]
    );
    assert_eq!(
        severity(&evaluate(&rule, &disk(88.0, 28.0)).unwrap()),
        [Some(Severity::Warning)]
    );
    let results = evaluate(&rule, &disk(94.2, 13.8)).unwrap();
    assert_eq!(severity(&results), [Some(Severity::Critical)]);
    let Outcome::Fired { explain, .. } = &results[0].1 else {
        unreachable!()
    };
    assert!(
        explain.starts_with("Your main disk has only 13.8 GB left."),
        "{explain}"
    );
}

#[test]
fn unit_failed() {
    let rule = rule("services.unit_failed");
    let text = |s: &str| Value::Text(s.into());
    let facts = list(
        "services",
        &[
            (
                "snapd.service",
                &[("name", text("snapd.service")), ("active", text("active"))],
            ),
            (
                "bluetooth.service",
                &[
                    ("name", text("bluetooth.service")),
                    ("description", text("Bluetooth service")),
                    ("active", text("failed")),
                ],
            ),
        ],
    );
    let results = evaluate(&rule, &facts).unwrap();
    assert_eq!(severity(&results), [None, Some(Severity::Warning)]);
    let Outcome::Fired { title, explain, .. } = &results[1].1 else {
        unreachable!()
    };
    assert_eq!(title, "bluetooth.service has stopped working");
    assert!(
        explain.starts_with("The service \"Bluetooth service\" failed"),
        "{explain}"
    );
}

#[test]
fn battery_worn() {
    let rule = rule("power.battery_worn");
    let battery = |health| {
        list(
            "batteries",
            &[(
                "BAT0",
                &[
                    ("name", Value::Text("BAT0".into())),
                    ("health", Value::Number(health)),
                ],
            )],
        )
    };
    assert_eq!(severity(&evaluate(&rule, &battery(95.0)).unwrap()), [None]);
    assert_eq!(
        severity(&evaluate(&rule, &battery(79.3)).unwrap()),
        [Some(Severity::Info)]
    );
    assert_eq!(
        severity(&evaluate(&rule, &battery(55.0)).unwrap()),
        [Some(Severity::Warning)]
    );
    // A desktop with no battery: the rule has nothing to say.
    assert!(evaluate(&rule, &Facts::default()).is_none());
}

#[test]
fn filling_up() {
    let rule = rule("storage.filling_up");
    let disk = |mount: &str, used: f64, filling: bool, days: Option<f64>| {
        let mut fields: Vec<(&str, Value)> = vec![
            ("mount_point", Value::Text(mount.into())),
            ("used_percent", Value::Number(used)),
            ("free_gb", Value::Number(20.0)),
            ("percent_per_day", Value::Number(2.0)),
            ("filling", Value::Bool(filling)),
        ];
        if let Some(d) = days {
            fields.push(("days_until_full", Value::Number(d)));
        }
        fields
    };
    let check = |fields: Vec<(&str, Value)>| {
        let facts = list("filesystems", &[("x", &fields)]);
        severity(&evaluate(&rule, &facts).unwrap())[0]
    };
    assert_eq!(
        check(disk("/home", 80.0, true, Some(3.0))),
        Some(Severity::Critical)
    );
    assert_eq!(
        check(disk("/home", 80.0, true, Some(10.0))),
        Some(Severity::Warning)
    );
    assert_eq!(check(disk("/home", 80.0, true, Some(40.0))), None);
    // Not filling: days are missing, and the rule is clear (an open finding closes).
    assert_eq!(check(disk("/", 60.0, false, None)), None);
    // The main disk at 93% already has "almost full"; this one stays quiet.
    assert_eq!(check(disk("/", 95.0, true, Some(2.0))), None);
    assert_eq!(
        check(disk("/", 88.0, true, Some(2.0))),
        Some(Severity::Critical)
    );

    let facts = list(
        "filesystems",
        &[("/home", &disk("/home", 80.0, true, Some(3.0)))],
    );
    let Outcome::Fired { title, explain, .. } = &evaluate(&rule, &facts).unwrap()[0].1 else {
        panic!("expected a finding");
    };
    assert_eq!(title, "/home will be full in about 3 days");
    assert!(explain.contains("about 2% a day"), "{explain}");
}
