// SPDX-License-Identifier: Apache-2.0

//! Builds a [`Report`] of one period from the database: the weekly summary,
//! `deecure report`, and the app's Export button all use it.

use std::collections::HashSet;

use deecure_core::report::{ChangeCounts, DiskLine, Report};
use deecure_core::{Event, EventKind, Severity, Timestamp, health, thin};
use serde_json::json;

use crate::collect::SystemRoot;
use crate::inventory::system;
use crate::rules::LAST_CHECK_SETTING;
use crate::rules::facts::FRESH_FOR_MS;
use crate::rules::forecast;
use crate::store::{self, Store};

/// The most events a report looks at.
const MAX_EVENTS: usize = 20_000;
/// The most highlights in a report.
const MAX_HIGHLIGHTS: usize = 12;
const DAY_MS: i64 = 86_400_000;

/// How much a change matters in a summary: lower comes first.
fn rank(e: &Event) -> u8 {
    match e.kind {
        EventKind::UncleanShutdown => 0,
        EventKind::Boot if e.data.as_ref().is_some_and(|d| d["kernel_changed"] == true) => 0,
        EventKind::Crash | EventKind::OutOfMemoryKill => 1,
        EventKind::ServiceFailed => 2,
        EventKind::FixApplied | EventKind::FixUndone => 3,
        EventKind::PackageInstalled | EventKind::PackageRemoved | EventKind::PackageUpgraded => 4,
        EventKind::ConfigChanged => 5,
        _ => 9,
    }
}

/// The most important changes, newest first. The packages of one apt run are one line,
/// and repeats of the same crash or failure are one line.
pub fn highlights(events: &[Event]) -> Vec<Event> {
    let mut picked: Vec<Event> = Vec::new();
    let mut seen = HashSet::new();
    let mut runs: Vec<(String, usize, Vec<String>)> = Vec::new();
    for e in events {
        if rank(e) >= 9 || e.kind == EventKind::Boot && rank(e) != 0 {
            continue;
        }
        let run = e
            .data
            .as_ref()
            .and_then(|d| d.get("run"))
            .map(|r| format!("{r}|{}", e.source));
        if let Some(run) = run {
            if let Some((_, i, names)) = runs.iter_mut().find(|(r, _, _)| *r == run) {
                names.push(e.subject.clone().unwrap_or_default());
                let n = names.len();
                picked[*i].title = format!(
                    "{n} packages changed: {}{}",
                    names.iter().take(3).cloned().collect::<Vec<_>>().join(", "),
                    if n > 3 { " …" } else { "" }
                );
                continue;
            }
            runs.push((
                run,
                picked.len(),
                vec![e.subject.clone().unwrap_or_default()],
            ));
        } else {
            let once = format!("{:?}|{}", e.kind, e.subject.as_deref().unwrap_or(&e.title));
            if matches!(
                e.kind,
                EventKind::Crash
                    | EventKind::OutOfMemoryKill
                    | EventKind::ServiceFailed
                    | EventKind::ConfigChanged
            ) && !seen.insert(once)
            {
                continue;
            }
        }
        picked.push(e.clone());
    }
    // Each type gets a few places, so twenty failures cannot hide a big update.
    let quota = |r: u8| match r {
        0 => 3,
        4 => 4,
        5 => 2,
        _ => 3,
    };
    let mut order: Vec<usize> = (0..picked.len()).collect();
    order.sort_by_key(|&i| (rank(&picked[i]), std::cmp::Reverse(picked[i].at)));
    let mut used = std::collections::HashMap::new();
    order.retain(|&i| {
        let r = rank(&picked[i]);
        let n = used.entry(r).or_insert(0);
        *n += 1;
        *n <= quota(r)
    });
    order.truncate(MAX_HIGHLIGHTS);
    let mut out: Vec<Event> = order.into_iter().map(|i| picked[i].clone()).collect();
    out.sort_by_key(|e| std::cmp::Reverse(e.at));
    out
}

/// One thing worth doing, most urgent first.
fn tip(r: &Report, latest: &dyn Fn(&str) -> Option<f64>) -> String {
    if let Some(f) = r.findings.iter().find(|f| f.severity == Severity::Critical) {
        return format!(
            "Fix this first: {}. Open DeeCure › Overview › Issues to see how.",
            f.title
        );
    }
    if let Some(d) = r
        .disks
        .iter()
        .find(|d| d.days_until_full.is_some_and(|n| n <= 30.0))
    {
        return format!(
            "{} fills up in about {:.0} days. Storage › Cleanup shows what can go safely.",
            d.mount_point,
            d.days_until_full.unwrap_or_default()
        );
    }
    if latest("updates.security").is_some_and(|n| n > 0.0) {
        return "Install the waiting security updates: they fix known holes.".into();
    }
    if latest("backup.configured") == Some(0.0) {
        return "Set up backups (Backups › Overview), so a broken disk cannot take your files."
            .into();
    }
    if latest("system.reboot_required") == Some(1.0) {
        return "Restart when it suits you, to finish installing updates.".into();
    }
    if r.changes.crashes > 0 {
        return "Some programs crashed. Logs › Crashes shows which, and how often.".into();
    }
    "Everything looks good. Keep installing updates when DeeCure or Ubuntu offers them.".into()
}

/// The report for `since` to `until`.
pub fn build(
    store: &Store,
    root: &SystemRoot,
    since: Timestamp,
    until: Timestamp,
    now: Timestamp,
) -> store::Result<Report> {
    let info = system::read(root);
    let findings = store.open_findings()?;
    let checked = store.setting::<Timestamp>(LAST_CHECK_SETTING)?.is_some();
    let span = (until.as_millis() - since.as_millis()).max(DAY_MS);
    let days = usize::try_from(span / DAY_MS).unwrap_or(1).max(1);
    let trend = thin(
        store.metrics("health.score", None, since, until)?,
        span,
        days,
    );
    let events = store.events(since, until, &[], MAX_EVENTS)?;

    let latest = store.latest_metrics(Timestamp(now.as_millis() - FRESH_FOR_MS))?;
    let value = |name: &str, subject: Option<&str>| {
        latest
            .iter()
            .find(|m| m.name == name && m.subject.as_deref() == subject)
            .map(|m| m.value)
    };
    let mut disks = Vec::new();
    for m in latest.iter().filter(|m| m.name == "disk.used_percent") {
        let Some(mount) = m.subject.as_deref() else {
            continue;
        };
        let week = Timestamp(now.as_millis() - forecast::WINDOW_MS);
        let f = forecast::disk(store.metrics("disk.used_percent", Some(mount), week, now)?);
        disks.push(DiskLine {
            mount_point: mount.to_owned(),
            used_percent: m.value,
            free_bytes: value("disk.free", Some(mount)).unwrap_or(0.0),
            days_until_full: f.and_then(|f| f.days_until_full),
        });
    }

    let mut report = Report {
        created_at: now,
        since,
        until,
        host: info.hostname.unwrap_or_else(|| "this computer".into()),
        os: info.os.unwrap_or_else(|| "Linux".into()),
        score: checked.then(|| health::score(&findings).0),
        score_trend: trend.iter().map(|m| (m.at, m.value)).collect(),
        changes: ChangeCounts::of(&events),
        highlights: highlights(&events),
        findings,
        disks,
        tip: String::new(),
    };
    report.tip = tip(&report, &|name| value(name, None));
    Ok(report)
}

/// The report as JSON for the `report` table.
pub fn to_json(r: &Report) -> serde_json::Value {
    json!(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use deecure_core::{Finding, FindingState, Metric, RuleId, Unit};

    fn ev(kind: EventKind, at: i64, title: &str, data: Option<serde_json::Value>) -> Event {
        Event {
            id: None,
            kind,
            at: Timestamp(at),
            title: title.into(),
            detail: None,
            subject: Some(title.into()),
            source: "apt".into(),
            data,
        }
    }

    #[test]
    fn highlights_put_what_matters_first_and_join_runs() {
        let mut events: Vec<Event> = (0..5)
            .map(|i| {
                ev(
                    EventKind::PackageUpgraded,
                    100,
                    &format!("pkg{i}"),
                    Some(json!({"run": 100})),
                )
            })
            .collect();
        events.push(ev(EventKind::Crash, 90, "bluetoothd", None));
        events.push(ev(EventKind::Crash, 80, "bluetoothd", None));
        events.push(ev(EventKind::Boot, 70, "Computer started", None));
        events.push(ev(EventKind::UncleanShutdown, 60, "Unclean shutdown", None));
        let h = highlights(&events);
        let titles: Vec<&str> = h.iter().map(|e| e.title.as_str()).collect();
        assert_eq!(
            titles,
            [
                "5 packages changed: pkg0, pkg1, pkg2 …",
                "bluetoothd",
                "Unclean shutdown"
            ]
        );
    }

    #[test]
    fn many_failures_do_not_hide_an_update() {
        let mut events: Vec<Event> = (0..20)
            .map(|i| ev(EventKind::ServiceFailed, 200 - i, &format!("unit{i}"), None))
            .collect();
        events.push(ev(
            EventKind::PackageUpgraded,
            50,
            "openssl",
            Some(json!({"run": 50})),
        ));
        let h = highlights(&events);
        assert_eq!(
            h.iter()
                .filter(|e| e.kind == EventKind::ServiceFailed)
                .count(),
            3
        );
        assert!(h.iter().any(|e| e.kind == EventKind::PackageUpgraded));
    }

    #[test]
    fn builds_a_report_with_score_trend_disks_and_a_tip() {
        let store = Store::open_in_memory().unwrap();
        let now = Timestamp::now();
        let day = DAY_MS;
        store
            .add_metrics(&[Metric {
                name: "health.score".into(),
                subject: None,
                value: 80.0,
                unit: Unit::Count,
                at: Timestamp(now.as_millis() - 3 * day),
            }])
            .unwrap();
        store
            .add_metrics(&[
                Metric {
                    name: "disk.used_percent".into(),
                    subject: Some("/".into()),
                    value: 70.0,
                    unit: Unit::Percent,
                    at: now,
                },
                Metric {
                    name: "disk.free".into(),
                    subject: Some("/".into()),
                    value: 50e9,
                    unit: Unit::Bytes,
                    at: now,
                },
                Metric {
                    name: "backup.configured".into(),
                    subject: None,
                    value: 0.0,
                    unit: Unit::Count,
                    at: now,
                },
            ])
            .unwrap();
        store.set_setting(LAST_CHECK_SETTING, &now).unwrap();
        store
            .upsert_finding(&Finding {
                rule: RuleId::new("services.unit_failed").unwrap(),
                subject: Some("cups.service".into()),
                severity: Severity::Warning,
                state: FindingState::Open,
                title: "cups.service failed".into(),
                explain: "x".into(),
                first_seen: now,
                last_seen: now,
                fixes: vec![],
            })
            .unwrap();
        store
            .add_event(
                "a",
                &ev(
                    EventKind::ConfigChanged,
                    now.as_millis() - day,
                    "/etc/hosts",
                    None,
                ),
            )
            .unwrap();
        let r = build(
            &store,
            &SystemRoot::at("/nonexistent"),
            Timestamp(now.as_millis() - 7 * day),
            now,
            now,
        )
        .unwrap();
        assert_eq!(r.score, Some(95));
        assert_eq!(r.score_trend.len(), 1);
        assert_eq!(r.score_change(), Some(15));
        assert_eq!(r.changes.config, 1);
        assert_eq!(r.disks.len(), 1);
        assert_eq!(r.disks[0].days_until_full, None, "one point is not a trend");
        assert_eq!(r.host, "this computer");
        assert!(r.tip.starts_with("Set up backups"), "{}", r.tip);
    }
}
