// SPDX-License-Identifier: Apache-2.0

//! Desktop notifications (`docs/02-architecture/notifications.md`).
//!
//! Every minute: which open problems should be shown now? Critical ones right away,
//! warnings only when the person asked for them, nothing during quiet hours (they wait
//! until the quiet hours end), and never the same alert twice within 24 hours, even
//! across restarts (the `notification` table remembers). A burst of more than three is
//! shown as one summary. The desktop's notification service is asked only when it is
//! already running, so DeeCure never starts one; when it is missing (a server, or the
//! desktop is still starting), nothing is recorded and the next minute tries again.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use deecure_core::notify::NotifySettings;
use deecure_core::{APP_ID, Finding, Severity, Timestamp, local};

use crate::context::{Agent, Bus};
use crate::store::{self, Store};

/// The setting that holds [`NotifySettings`].
pub const SETTING: &str = "notify";
/// The same alert is not shown again within this time.
pub const REPEAT_AFTER_MS: i64 = 24 * 60 * 60 * 1000;
/// More alerts than this at once become one summary.
const MAX_AT_ONCE: usize = 3;
/// Bodies longer than this are cut at a sentence end.
const MAX_BODY: usize = 220;
const SERVICE: &str = "org.freedesktop.Notifications";

/// One notification to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    /// What is shown, for the record.
    pub title: String,
    /// The text below the title.
    pub body: String,
    /// The keys it covers, such as `storage.root_almost_full|/|critical`.
    pub keys: Vec<String>,
    /// Shown as urgent.
    pub critical: bool,
}

/// The notification settings, or the defaults.
pub fn settings(store: &Store) -> store::Result<NotifySettings> {
    Ok(store.setting(SETTING)?.unwrap_or_default())
}

/// The key of one alert: the same problem at the same severity.
pub fn key(f: &Finding) -> String {
    let severity = match f.severity {
        Severity::Critical => "critical",
        Severity::Warning => "warning",
        Severity::Info => "info",
    };
    format!(
        "{}|{}|{severity}",
        f.rule,
        f.subject.as_deref().unwrap_or("")
    )
}

/// The first sentences of `text` that fit in [`MAX_BODY`] characters.
pub fn short(text: &str) -> String {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() <= MAX_BODY {
        return text;
    }
    let cut: String = text.chars().take(MAX_BODY).collect();
    match cut.rfind(". ") {
        Some(end) if end >= 20 => cut[..=end].to_owned(),
        _ => format!("{}…", cut.trim_end()),
    }
}

/// True when a record says the alert was shown less than 24 hours ago. A record from
/// the future (the clock was set back) does not block alerts for long.
fn recent(sent: Option<Timestamp>, now: Timestamp) -> bool {
    sent.is_some_and(|s| {
        let age = now.as_millis() - s.as_millis();
        (0..REPEAT_AFTER_MS).contains(&age) || (-3_600_000..0).contains(&age)
    })
}

/// What to show now, given the open findings and when each key was last shown.
pub fn due(
    findings: &[Finding],
    settings: &NotifySettings,
    last_sent: &HashMap<String, Timestamp>,
    now: Timestamp,
    minute_of_day: u16,
) -> Vec<Alert> {
    if settings.quiet_hours.contains(minute_of_day) {
        return Vec::new();
    }
    let wanted = |f: &&Finding| match f.severity {
        Severity::Critical => settings.critical,
        Severity::Warning => settings.warnings,
        Severity::Info => false,
    };
    let alerts: Vec<Alert> = findings
        .iter()
        .filter(wanted)
        .filter(|f| !recent(last_sent.get(&key(f)).copied(), now))
        .map(|f| Alert {
            title: f.title.clone(),
            body: short(&f.explain),
            keys: vec![key(f)],
            critical: f.severity == Severity::Critical,
        })
        .collect();
    if alerts.len() <= MAX_AT_ONCE {
        return alerts;
    }
    let mut body: Vec<String> = alerts
        .iter()
        .take(MAX_AT_ONCE)
        .map(|a| format!("• {}", a.title))
        .collect();
    body.push(format!(
        "and {} more. Open DeeCure to see them.",
        alerts.len() - MAX_AT_ONCE
    ));
    vec![Alert {
        title: format!("{} problems need your attention", alerts.len()),
        body: body.join("\n"),
        critical: alerts.iter().any(|a| a.critical),
        keys: alerts.into_iter().flat_map(|a| a.keys).collect(),
    }]
}

/// Shows a notification through the desktop's notification service.
pub async fn show(agent: &Agent, title: &str, body: &str, critical: bool) -> Result<(), String> {
    if !agent.is_running(Bus::Session, SERVICE).await {
        return Err("no notification service is running (is a desktop session open?)".into());
    }
    let conn = agent.bus(Bus::Session).await.map_err(|e| e.to_string())?;
    let mut hints: HashMap<&str, zbus::zvariant::Value<'_>> = HashMap::new();
    hints.insert(
        "urgency",
        zbus::zvariant::Value::U8(if critical { 2 } else { 1 }),
    );
    // Lets the desktop show DeeCure's icon and open the app when the notification is clicked.
    hints.insert("desktop-entry", zbus::zvariant::Value::from(APP_ID));
    let actions: Vec<&str> = Vec::new();
    let reply = conn
        .call_method(
            Some(SERVICE),
            "/org/freedesktop/Notifications",
            Some(SERVICE),
            "Notify",
            &("DeeCure", 0u32, APP_ID, title, body, actions, hints, -1i32),
        )
        .await;
    if let Err(e) = reply {
        agent.drop_bus(Bus::Session).await;
        return Err(e.to_string());
    }
    Ok(())
}

/// Finds what is due and shows it. Returns a reason when nothing could be shown.
pub async fn check(agent: &Agent, now: Timestamp) -> Result<(), String> {
    let minute = local::parts(now).map_or(12 * 60, |p| p.minute_of_day());
    let alerts = {
        let s = store::lock(&agent.store);
        let settings = settings(&s).map_err(|e| e.to_string())?;
        let findings = s.open_findings().map_err(|e| e.to_string())?;
        let mut last = HashMap::new();
        for f in &findings {
            let k = key(f);
            if let Some(at) = s.last_notified(&k).map_err(|e| e.to_string())? {
                last.insert(k, at);
            }
        }
        due(&findings, &settings, &last, now, minute)
    };
    for alert in alerts {
        show(agent, &alert.title, &alert.body, alert.critical).await?;
        let s = store::lock(&agent.store);
        s.batch(|s| {
            for k in &alert.keys {
                s.add_notification(k, now, &alert.title, &alert.body)?;
            }
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Checks every minute, forever. A failure is logged once until it works again.
pub async fn run(agent: Arc<Agent>) {
    let mut timer = tokio::time::interval(Duration::from_secs(60));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut failing = false;
    loop {
        timer.tick().await;
        match check(&agent, Timestamp::now()).await {
            Ok(()) => {
                if failing {
                    eprintln!("deecure-agent: notifications work again");
                }
                failing = false;
            }
            Err(e) => {
                if !failing {
                    eprintln!("deecure-agent: could not show a notification: {e}");
                }
                failing = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deecure_core::notify::QuietHours;
    use deecure_core::{FindingState, RuleId};

    const NOON: u16 = 12 * 60;
    const NOW: Timestamp = Timestamp(1_800_000_000_000);

    fn finding(rule: &str, severity: Severity) -> Finding {
        Finding {
            rule: RuleId::new(rule).unwrap(),
            subject: Some("/".into()),
            severity,
            state: FindingState::Open,
            title: format!("{rule} title"),
            explain: "Only 3 GB left. Apps can crash.".into(),
            first_seen: Timestamp(0),
            last_seen: NOW,
            fixes: vec![],
        }
    }

    #[test]
    fn critical_now_warnings_only_when_asked_info_never() {
        let findings = [
            finding("storage.root_almost_full", Severity::Critical),
            finding("services.unit_failed", Severity::Warning),
            finding("power.battery_worn", Severity::Info),
        ];
        let defaults = NotifySettings::default();
        let alerts = due(&findings, &defaults, &HashMap::new(), NOW, NOON);
        assert_eq!(alerts.len(), 1);
        assert!(alerts[0].critical);
        assert_eq!(alerts[0].keys, ["storage.root_almost_full|/|critical"]);
        let all = NotifySettings {
            warnings: true,
            ..defaults
        };
        assert_eq!(due(&findings, &all, &HashMap::new(), NOW, NOON).len(), 2);
        let none = NotifySettings {
            critical: false,
            ..defaults
        };
        assert!(due(&findings, &none, &HashMap::new(), NOW, NOON).is_empty());
    }

    #[test]
    fn no_alert_repeats_within_24_hours() {
        let findings = [finding("storage.root_almost_full", Severity::Critical)];
        let s = NotifySettings::default();
        let k = key(&findings[0]);
        let sent = |ago: i64| HashMap::from([(k.clone(), Timestamp(NOW.as_millis() - ago))]);
        let hour = 3_600_000;
        assert!(due(&findings, &s, &sent(hour), NOW, NOON).is_empty());
        assert!(due(&findings, &s, &sent(REPEAT_AFTER_MS - 1), NOW, NOON).is_empty());
        assert_eq!(
            due(&findings, &s, &sent(REPEAT_AFTER_MS), NOW, NOON).len(),
            1,
            "after 24 hours it may remind"
        );
        // Becoming critical after a warning is a new alert.
        let mut warned = findings[0].clone();
        warned.severity = Severity::Warning;
        let warned_key = HashMap::from([(key(&warned), Timestamp(NOW.as_millis() - hour))]);
        assert_eq!(due(&findings, &s, &warned_key, NOW, NOON).len(), 1);
        // A clock set back a little still counts as recent; a record a year ahead does not block.
        assert!(due(&findings, &s, &sent(-60_000), NOW, NOON).is_empty());
        assert_eq!(
            due(&findings, &s, &sent(-365 * 24 * hour), NOW, NOON).len(),
            1
        );
    }

    #[test]
    fn quiet_hours_hold_everything_back() {
        let findings = [finding("storage.root_almost_full", Severity::Critical)];
        let s = NotifySettings::default();
        assert!(due(&findings, &s, &HashMap::new(), NOW, 23 * 60).is_empty());
        assert!(due(&findings, &s, &HashMap::new(), NOW, 6 * 60).is_empty());
        assert_eq!(
            due(&findings, &s, &HashMap::new(), NOW, 7 * 60).len(),
            1,
            "shown when they end"
        );
        let off = NotifySettings {
            quiet_hours: QuietHours::Off,
            ..s
        };
        assert_eq!(due(&findings, &off, &HashMap::new(), NOW, 23 * 60).len(), 1);
    }

    #[test]
    fn a_burst_becomes_one_summary() {
        let findings: Vec<Finding> = ["a.one", "b.two", "c.three", "d.four", "e.five"]
            .iter()
            .map(|r| {
                let mut f = finding("storage.root_almost_full", Severity::Critical);
                f.rule = RuleId::new(format!("storage.{}", r.split('.').nth(1).unwrap())).unwrap();
                f
            })
            .collect();
        let alerts = due(
            &findings,
            &NotifySettings::default(),
            &HashMap::new(),
            NOW,
            NOON,
        );
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].title, "5 problems need your attention");
        assert_eq!(
            alerts[0].keys.len(),
            5,
            "each is remembered, so none repeats"
        );
        assert!(
            alerts[0]
                .body
                .ends_with("and 2 more. Open DeeCure to see them.")
        );
    }

    #[test]
    fn long_texts_are_cut_at_a_sentence() {
        assert_eq!(short("Short  text.\n More."), "Short text. More.");
        let long = format!("{} Second sentence here.", "word ".repeat(50));
        let cut = short(&long);
        assert!(cut.chars().count() <= MAX_BODY + 1, "{cut}");
        assert!(cut.ends_with('…'));
        let sentences = format!(
            "First sentence is here and long enough. {}",
            "more ".repeat(60)
        );
        assert_eq!(short(&sentences), "First sentence is here and long enough.");
    }
}
