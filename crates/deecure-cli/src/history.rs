// SPDX-License-Identifier: Apache-2.0

//! `deecure history` (what changed) and `deecure report` (a summary of a period).

use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use deecure_core::report::{self, Report};
use deecure_core::rpc::{EventList, method};
use deecure_core::{Event, EventKind, Timestamp, client, local};
use serde_json::json;

fn fail(message: impl std::fmt::Display) -> ExitCode {
    eprintln!("deecure: {message}");
    ExitCode::FAILURE
}

/// Reads `7d`, `24h`, `30m`, `2w`, or a local date `2026-10-01`, as a moment before `now`.
pub fn parse_since(text: &str, now: Timestamp, tz: &local::TimeZone) -> Result<Timestamp, String> {
    let text = text.trim();
    let unit = text.chars().last().unwrap_or(' ');
    let ms = match unit {
        'm' => Some(60_000),
        'h' => Some(3_600_000),
        'd' => Some(86_400_000),
        'w' => Some(7 * 86_400_000),
        _ => None,
    };
    if let Some(ms) = ms
        && let Ok(n) = text[..text.len() - 1].parse::<i64>()
        && n >= 0
    {
        return Ok(Timestamp(
            now.as_millis().saturating_sub(n.saturating_mul(ms)),
        ));
    }
    local::parse_in(&format!("{text} 00:00:00"), tz).ok_or_else(|| {
        format!("'{text}' is not a time: use 30m, 24h, 7d, 2w, or a date like 2026-10-01")
    })
}

/// The event kinds for a `--kind` word.
pub fn kinds(word: &str) -> Result<&'static [EventKind], String> {
    use EventKind as K;
    Ok(match word {
        "packages" => &[K::PackageInstalled, K::PackageUpgraded, K::PackageRemoved],
        "config" => &[K::ConfigChanged],
        "devices" => &[K::DeviceAdded, K::DeviceRemoved],
        "system" => &[K::Boot, K::UncleanShutdown],
        "services" => &[K::ServiceFailed],
        "crashes" => &[K::Crash, K::OutOfMemoryKill, K::HardwareError],
        "deecure" => &[K::FixApplied, K::FixUndone],
        other => {
            return Err(format!(
                "unknown kind '{other}' (use packages, config, devices, system, services, crashes, or deecure)"
            ));
        }
    })
}

fn area(kind: EventKind) -> &'static str {
    use EventKind as K;
    match kind {
        K::PackageInstalled | K::PackageUpgraded | K::PackageRemoved => "Packages",
        K::ConfigChanged => "Config",
        K::DeviceAdded | K::DeviceRemoved => "Devices",
        K::Boot | K::UncleanShutdown | K::LoginFailed => "System",
        K::ServiceFailed => "Services",
        K::Crash | K::OutOfMemoryKill | K::HardwareError => "Crashes",
        K::FixApplied | K::FixUndone => "DeeCure",
    }
}

/// The lines of `deecure history`, newest first, with a heading per local day. The
/// packages of one apt run are one line.
pub fn lines(events: &[Event], tz: &local::TimeZone) -> Vec<String> {
    let mut out = Vec::new();
    let mut day = String::new();
    let mut i = 0;
    while i < events.len() {
        let e = &events[i];
        let run = e.data.as_ref().and_then(|d| d.get("run")).cloned();
        let mut j = i + 1;
        if run.is_some() {
            while j < events.len()
                && events[j].source == e.source
                && events[j].data.as_ref().and_then(|d| d.get("run")) == run.as_ref()
            {
                j += 1;
            }
        }
        let this_day = local::format_in(e.at, "%a %d %b %Y", tz);
        if this_day != day {
            if !out.is_empty() {
                out.push(String::new());
            }
            out.push(this_day.clone());
            day = this_day;
        }
        let title = if j - i > 1 {
            let names: Vec<&str> = events[i..j]
                .iter()
                .filter_map(|e| e.subject.as_deref())
                .collect();
            let more = if names.len() > 3 {
                format!(" +{} more", names.len() - 3)
            } else {
                String::new()
            };
            format!(
                "{} packages changed: {}{more}",
                j - i,
                names[..names.len().min(3)].join(", ")
            )
        } else {
            e.title.clone()
        };
        let mut extra = Vec::new();
        if matches!(
            e.kind,
            EventKind::PackageInstalled | EventKind::PackageUpgraded | EventKind::PackageRemoved
        ) {
            extra.push(e.source.clone());
        }
        if e.kind == EventKind::ConfigChanged {
            extra.push(format!("by {}", e.source));
        }
        if let Some(d) = &e.data
            && e.kind == EventKind::FixApplied
            && let Some(id) = d.get("run_id").and_then(serde_json::Value::as_i64)
        {
            if d.get("undoable") == Some(&json!(true)) {
                extra.push(format!("#{id}, undo with: deecure undo {id}"));
            } else {
                extra.push(format!("#{id}"));
            }
        }
        let extra = if extra.is_empty() {
            String::new()
        } else {
            format!(" · {}", extra.join(" · "))
        };
        out.push(format!(
            "  {}  {:<8}  {title}{extra}",
            local::format_in(e.at, "%H:%M", tz),
            area(e.kind)
        ));
        i = j;
    }
    out
}

pub fn history(socket: &Path, since: &str, kind: &[String], json_out: bool) -> ExitCode {
    let tz = local::zone();
    let now = Timestamp::now();
    let since = match parse_since(since, now, &tz) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("deecure: {e}");
            return ExitCode::from(2);
        }
    };
    let mut wanted = Vec::new();
    for word in kind {
        match kinds(word) {
            Ok(k) => wanted.extend_from_slice(k),
            Err(e) => {
                eprintln!("deecure: {e}");
                return ExitCode::from(2);
            }
        }
    }
    let list: EventList = match client::call(
        socket,
        method::EVENTS,
        Some(json!({ "since": since, "kinds": wanted })),
    ) {
        Ok(l) => l,
        Err(e) => return fail(e),
    };
    if json_out {
        return match serde_json::to_string_pretty(&list) {
            Ok(text) => {
                println!("{text}");
                ExitCode::SUCCESS
            }
            Err(e) => fail(e),
        };
    }
    if list.events.is_empty() {
        println!("Nothing changed in that time.");
        return ExitCode::SUCCESS;
    }
    for line in lines(&list.events, &tz) {
        println!("{line}");
    }
    if list.truncated {
        println!("\nOnly the newest changes are shown. Use --since or --kind to see older ones.");
    }
    ExitCode::SUCCESS
}

pub fn report(socket: &Path, since: &str, html: Option<&Path>, json_out: bool) -> ExitCode {
    let tz = local::zone();
    let now = Timestamp::now();
    let since = match parse_since(since, now, &tz) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("deecure: {e}");
            return ExitCode::from(2);
        }
    };
    let r: Report = match client::call(
        socket,
        method::REPORT,
        Some(json!({ "since": since, "until": now })),
    ) {
        Ok(r) => r,
        Err(e) => return fail(e),
    };
    if json_out {
        return match serde_json::to_string_pretty(&r) {
            Ok(text) => {
                println!("{text}");
                ExitCode::SUCCESS
            }
            Err(e) => fail(e),
        };
    }
    let Some(path) = html else {
        print!("{}", report::text(&r, &tz));
        return ExitCode::SUCCESS;
    };
    let page = report::html(&r, &tz);
    if path == Path::new("-") {
        let _ = std::io::stdout().write_all(page.as_bytes());
        return ExitCode::SUCCESS;
    }
    match write_file(path, page.as_bytes()) {
        Ok(()) => {
            println!("Saved the report to {}", path.display());
            ExitCode::SUCCESS
        }
        Err(e) => fail(format!("could not write {}: {e}", path.display())),
    }
}

/// Writes through a temporary file next to `path`, so a failed write never leaves half a report.
fn write_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let dir = path
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = dir.join(format!(".{name}.{}.tmp", std::process::id()));
    let result = std::fs::write(&tmp, bytes).and_then(|()| std::fs::rename(&tmp, path));
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tz() -> local::TimeZone {
        local::fixed(0)
    }

    #[test]
    fn since_takes_durations_and_dates() {
        let now = Timestamp(10 * 86_400_000);
        assert_eq!(parse_since("7d", now, &tz()), Ok(Timestamp(3 * 86_400_000)));
        assert_eq!(
            parse_since("24h", now, &tz()),
            Ok(Timestamp(9 * 86_400_000))
        );
        assert_eq!(parse_since("1w", now, &tz()), Ok(Timestamp(3 * 86_400_000)));
        assert_eq!(
            parse_since("1970-01-02", now, &tz()),
            Ok(Timestamp(86_400_000))
        );
        assert!(parse_since("soon", now, &tz()).is_err());
        assert!(parse_since("-3d", now, &tz()).is_err());
        assert!(parse_since("", now, &tz()).is_err());
    }

    #[test]
    fn kinds_have_words() {
        assert_eq!(
            kinds("crashes").unwrap(),
            [
                EventKind::Crash,
                EventKind::OutOfMemoryKill,
                EventKind::HardwareError
            ]
        );
        assert!(kinds("toasters").is_err());
    }

    #[test]
    fn history_groups_runs_and_names_undo() {
        let ev = |kind, at: i64, title: &str, subject: Option<&str>, source: &str, data| Event {
            id: None,
            kind,
            at: Timestamp(at),
            title: title.into(),
            detail: None,
            subject: subject.map(str::to_owned),
            source: source.into(),
            data,
        };
        let day = 86_400_000;
        let events = [
            ev(
                EventKind::FixApplied,
                2 * day + 3_600_000,
                "Turn off cups-browsed.service",
                None,
                "DeeCure",
                Some(json!({"run_id": 12, "undoable": true})),
            ),
            ev(
                EventKind::PackageUpgraded,
                2 * day,
                "Upgraded openssl 1 → 2",
                Some("openssl"),
                "unattended-upgrades",
                Some(json!({"run": 5})),
            ),
            ev(
                EventKind::PackageUpgraded,
                2 * day,
                "Upgraded sudo 1 → 2",
                Some("sudo"),
                "unattended-upgrades",
                Some(json!({"run": 5})),
            ),
            ev(
                EventKind::ConfigChanged,
                day,
                "Changed /etc/hosts",
                Some("/etc/hosts"),
                "root",
                None,
            ),
        ];
        let out = lines(&events, &tz());
        assert_eq!(
            out,
            [
                "Sat 03 Jan 1970",
                "  01:00  DeeCure   Turn off cups-browsed.service · #12, undo with: deecure undo 12",
                "  00:00  Packages  2 packages changed: openssl, sudo · unattended-upgrades",
                "",
                "Fri 02 Jan 1970",
                "  00:00  Config    Changed /etc/hosts · by root",
            ]
        );
    }
}
