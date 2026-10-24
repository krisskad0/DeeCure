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
    let disk = |free_percent, free| {
        let mut facts = Facts::default();
        facts
            .values
            .insert("root.free_percent".into(), Value::Number(free_percent));
        facts
            .values
            .insert("root.free_gb".into(), Value::Number(free));
        facts
    };
    assert_eq!(
        severity(&evaluate(&rule, &disk(59.0, 290.0)).unwrap()),
        [None]
    );
    // A 2 TB disk at 92% still has 160 GB: not low. A small disk with 8% and 19 GB is.
    assert_eq!(
        severity(&evaluate(&rule, &disk(8.0, 160.0)).unwrap()),
        [None]
    );
    assert_eq!(
        severity(&evaluate(&rule, &disk(8.0, 19.0)).unwrap()),
        [Some(Severity::Warning)]
    );
    let results = evaluate(&rule, &disk(5.8, 4.2)).unwrap();
    assert_eq!(severity(&results), [Some(Severity::Critical)]);
    let Outcome::Fired { explain, .. } = &results[0].1 else {
        unreachable!()
    };
    assert!(
        explain.starts_with("Your main disk has 4.2 GB (5.8%) left."),
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
                    ("timer", Value::Bool(false)),
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
            ("free_percent", Value::Number(100.0 - used)),
            (
                "free_gb",
                Value::Number(if used >= 95.0 { 3.0 } else { 20.0 }),
            ),
            ("percent_per_day", Value::Number(2.0)),
            ("gb_per_day", Value::Number(4.6)),
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
    assert!(
        explain.contains("about 4.6 GB a day (2% of the disk)"),
        "{explain}"
    );
    assert!(
        explain.contains("last 3 days and still in the last day"),
        "{explain}"
    );
}

fn values(pairs: &[(&str, f64)]) -> Facts {
    let mut facts = Facts::default();
    for (k, v) in pairs {
        facts.values.insert((*k).into(), Value::Number(*v));
    }
    facts
}

fn one(id: &str, facts: &Facts) -> Option<Severity> {
    severity(&evaluate(&rule(id), facts).unwrap())[0]
}

#[test]
fn hardware_network_packages_and_accounts() {
    // The CPU's own limit: 93 °C under load is normal for a CPU whose limit is 100 °C.
    let cpu = |temp, minutes| {
        values(&[
            ("temp.cpu", temp),
            ("temp.cpu_limit", 95.0),
            ("temp.cpu_critical", 100.0),
            ("temp.cpu_hot_minutes", minutes),
        ])
    };
    assert_eq!(one("hardware.cpu_hot", &cpu(93.0, 0.0)), None);
    assert_eq!(
        one("hardware.cpu_hot", &cpu(97.0, 2.0)),
        None,
        "a short spike"
    );
    assert_eq!(
        one("hardware.cpu_hot", &cpu(97.0, 5.0)),
        Some(Severity::Warning)
    );
    assert_eq!(
        one("hardware.cpu_hot", &cpu(100.0, 1.0)),
        Some(Severity::Critical)
    );

    assert_eq!(
        one("network.offline", &values(&[("net.default_route", 1.0)])),
        None
    );
    assert_eq!(
        one("network.offline", &values(&[("net.default_route", 0.0)])),
        Some(Severity::Info)
    );

    let updates = |p, s| values(&[("updates.pending", p), ("updates.security", s)]);
    assert_eq!(one("packages.updates_available", &updates(0.0, 0.0)), None);
    assert_eq!(
        one("packages.updates_available", &updates(4.0, 4.0)),
        None,
        "all security: one warning is enough"
    );
    assert_eq!(
        one("packages.updates_available", &updates(12.0, 4.0)),
        Some(Severity::Info)
    );

    assert_eq!(
        one(
            "accounts.extra_root",
            &values(&[("accounts.uid0_extra", 0.0)])
        ),
        None
    );
    assert_eq!(
        one(
            "accounts.extra_root",
            &values(&[("accounts.uid0_extra", 1.0)])
        ),
        Some(Severity::Critical)
    );

    // A computer without a sensor: the rule is unknown, so nothing changes.
    let Outcome::Unknown = &evaluate(&rule("hardware.cpu_hot"), &Facts::default()).unwrap()[0].1
    else {
        panic!("a missing sensor must not clear or open anything");
    };
}

#[test]
fn jobs_containers_and_privacy() {
    let unit = |active: &str, timer: bool| {
        list(
            "services",
            &[(
                "apt-daily.service",
                &[
                    ("name", Value::Text("apt-daily.service".into())),
                    (
                        "description",
                        Value::Text("Daily apt download activities".into()),
                    ),
                    ("active", Value::Text(active.into())),
                    ("timer", Value::Bool(timer)),
                ],
            )],
        )
    };
    let job = rule("jobs.job_failed");
    let service = rule("services.unit_failed");
    // A failed job is reported once, under Scheduled Jobs, not also under Services.
    assert_eq!(
        severity(&evaluate(&job, &unit("failed", true)).unwrap()),
        [Some(Severity::Warning)]
    );
    assert_eq!(
        severity(&evaluate(&service, &unit("failed", true)).unwrap()),
        [None]
    );
    assert_eq!(
        severity(&evaluate(&job, &unit("failed", false)).unwrap()),
        [None]
    );
    assert_eq!(
        severity(&evaluate(&service, &unit("failed", false)).unwrap()),
        [Some(Severity::Warning)]
    );
    assert_eq!(
        severity(&evaluate(&job, &unit("inactive", true)).unwrap()),
        [None]
    );

    assert_eq!(
        one(
            "containers.port_exposed",
            &values(&[("containers.exposed", 0.0)])
        ),
        None
    );
    assert_eq!(
        one(
            "containers.port_exposed",
            &values(&[("containers.exposed", 2.0)])
        ),
        Some(Severity::Warning)
    );
    assert_eq!(
        one("privacy.popcon", &values(&[("privacy.popcon", 0.0)])),
        None
    );
    assert_eq!(
        one("privacy.popcon", &values(&[("privacy.popcon", 1.0)])),
        Some(Severity::Info)
    );
}

#[test]
fn user_limits_change_when_a_warning_fires() {
    use deecure_agent::rules::evaluate_with;
    use deecure_core::rpc::WarningSettings;
    let rule = rule("hardware.cpu_hot");
    let facts = values(&[
        ("temp.cpu", 97.0),
        ("temp.cpu_limit", 95.0),
        ("temp.cpu_critical", 100.0),
        ("temp.cpu_hot_minutes", 8.0),
    ]);
    let fired = |s: &WarningSettings| severity(&evaluate_with(&rule, &facts, s).unwrap())[0];
    assert_eq!(fired(&WarningSettings::default()), Some(Severity::Warning));
    let mut s = WarningSettings::default();
    s.limits
        .entry("hardware.cpu_hot".into())
        .or_default()
        .insert("minutes".into(), 20.0);
    assert_eq!(fired(&s), None, "the user waits 20 minutes");
    // Out of range values are kept inside the limit's range (1 to 60 minutes).
    s.limits
        .get_mut("hardware.cpu_hot")
        .unwrap()
        .insert("minutes".into(), -5.0);
    assert_eq!(fired(&s), Some(Severity::Warning));
}

#[test]
fn health_warnings_fire_on_harm_not_on_use() {
    // Memory full of cache and a busy CPU are use: nothing fires.
    assert_eq!(
        one(
            "performance.memory_short",
            &values(&[("pressure.memory_stall_minutes", 0.0)])
        ),
        None
    );
    assert_eq!(
        one(
            "performance.memory_short",
            &values(&[("pressure.memory_stall_minutes", 6.0)])
        ),
        Some(Severity::Warning)
    );
    assert_eq!(
        one("storage.disk_errors", &values(&[("log.disk_io_24h", 3.0)])),
        Some(Severity::Critical)
    );
    assert_eq!(
        one(
            "system.kernel_crash",
            &values(&[("log.kernel_crash_7d", 0.0)])
        ),
        None
    );
    let battery = |temp, charging| {
        list(
            "batteries",
            &[(
                "BAT0",
                &[
                    ("name", Value::Text("BAT0".into())),
                    ("temp", Value::Number(temp)),
                    ("charging", Value::Number(charging)),
                ],
            )],
        )
    };
    let hot = rule("power.battery_hot");
    assert_eq!(
        severity(&evaluate(&hot, &battery(46.0, 0.0)).unwrap()),
        [None]
    );
    assert_eq!(
        severity(&evaluate(&hot, &battery(46.0, 1.0)).unwrap()),
        [Some(Severity::Warning)]
    );
    assert_eq!(
        severity(&evaluate(&hot, &battery(61.0, 0.0)).unwrap()),
        [Some(Severity::Critical)]
    );
    let sensor = |cpu: bool, celsius, minutes| {
        list(
            "sensors",
            &[(
                "nvme/Composite",
                &[
                    ("part", Value::Text("The NVMe drive (Composite)".into())),
                    ("cpu", Value::Bool(cpu)),
                    ("celsius", Value::Number(celsius)),
                    ("limit", Value::Number(84.85)),
                    ("critical", Value::Number(89.85)),
                    ("hot_minutes", Value::Number(minutes)),
                ],
            )],
        )
    };
    let part = rule("hardware.part_hot");
    assert_eq!(
        severity(&evaluate(&part, &sensor(false, 86.0, 6.0)).unwrap()),
        [Some(Severity::Warning)]
    );
    assert_eq!(
        severity(&evaluate(&part, &sensor(true, 95.0, 9.0)).unwrap()),
        [None],
        "the CPU has its own warning"
    );
}
