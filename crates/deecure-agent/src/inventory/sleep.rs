// SPDX-License-Identifier: Apache-2.0

//! Suspend and hibernate history from the systemd journal (`journalctl`, read-only).

use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use super::command;

/// One sleep.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SleepEvent {
    /// When it went to sleep, in milliseconds since 1970.
    pub at: i64,
    /// `suspend`, `hibernate`, `hybrid-sleep`, or `suspend-then-hibernate`.
    pub kind: String,
    /// Milliseconds asleep. Missing when the resume was not logged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    pub failed: bool,
}

/// Pairs "Performing sleep operation" with "System returned from sleep" lines.
/// Each line is one `journalctl -o json` entry, oldest first.
pub fn parse(lines: &str) -> Vec<SleepEvent> {
    let mut events: Vec<SleepEvent> = Vec::new();
    for line in lines.lines() {
        let Ok(entry) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let text = |k: &str| entry.get(k).and_then(Value::as_str).unwrap_or("");
        let at = text("__REALTIME_TIMESTAMP")
            .parse::<i64>()
            .map(|us| us / 1000)
            .unwrap_or(0);
        let message = text("MESSAGE");
        let kind = message
            .split('\'')
            .nth(1)
            .unwrap_or(text("SLEEP"))
            .to_owned();
        if message.starts_with("Performing sleep operation")
            || message.starts_with("Entering sleep state")
        {
            events.push(SleepEvent {
                at,
                kind,
                duration_ms: None,
                failed: false,
            });
        } else if message.starts_with("System returned from sleep")
            || message.starts_with("System resumed")
        {
            if let Some(last) = events
                .last_mut()
                .filter(|e| e.duration_ms.is_none() && !e.failed)
            {
                last.duration_ms = Some(at - last.at);
            }
        } else if message.starts_with("Failed to put system to sleep")
            || message.contains("Failed to suspend")
        {
            match events.last_mut().filter(|e| e.duration_ms.is_none()) {
                Some(last) => last.failed = true,
                None => events.push(SleepEvent {
                    at,
                    kind: "suspend".into(),
                    duration_ms: None,
                    failed: true,
                }),
            }
        }
    }
    events.reverse();
    events
}

/// The last 30 days, newest first. Empty when the journal cannot be read.
pub async fn read() -> Vec<SleepEvent> {
    let out = command::output(
        "journalctl",
        &[
            "-o",
            "json",
            "--no-pager",
            "-q",
            "--since",
            "-30d",
            "SYSLOG_IDENTIFIER=systemd-sleep",
        ],
        Duration::from_secs(15),
    )
    .await;
    out.map(|o| parse(&o)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_sleep_and_resume() {
        let lines = [
            r#"{"__REALTIME_TIMESTAMP":"1000000000","MESSAGE":"Performing sleep operation 'suspend'...","SLEEP":"suspend"}"#,
            r#"{"__REALTIME_TIMESTAMP":"1060000000","MESSAGE":"System returned from sleep operation 'suspend'."}"#,
            r#"{"__REALTIME_TIMESTAMP":"2000000000","MESSAGE":"Performing sleep operation 'hibernate'..."}"#,
            r#"{"__REALTIME_TIMESTAMP":"2000500000","MESSAGE":"Failed to put system to sleep. System resumed again: Device or resource busy"}"#,
            "not json",
        ]
        .join("\n");
        let events = parse(&lines);
        assert_eq!(events.len(), 2);
        assert_eq!(
            (events[0].kind.as_str(), events[0].failed),
            ("hibernate", true)
        );
        assert_eq!(
            (events[1].kind.as_str(), events[1].duration_ms),
            ("suspend", Some(60_000))
        );
    }
}
