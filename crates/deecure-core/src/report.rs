// SPDX-License-Identifier: Apache-2.0

//! A report of one period: health and its trend, open problems, what changed, disks,
//! and one tip. The agent builds it; this module turns it into text (`deecure report`)
//! or one self-contained HTML file (`deecure report --html`, the app's Export button).
//! Nothing in the HTML loads from the network.

use serde::{Deserialize, Serialize};

use crate::{Event, EventKind, Finding, Severity, Timestamp, local};

/// How many changes of each type happened.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChangeCounts {
    /// Packages installed, upgraded, or removed.
    pub packages: u32,
    /// Config files added, changed, or removed.
    pub config: u32,
    /// Devices plugged in or out.
    pub devices: u32,
    /// Crashes and out-of-memory kills.
    pub crashes: u32,
    /// Services that failed.
    pub failed_services: u32,
    /// Starts of the computer.
    pub boots: u32,
    /// Stops without an orderly shutdown.
    pub unclean_shutdowns: u32,
    /// Hardware and kernel problems the kernel logged.
    pub hardware_errors: u32,
    /// Fixes DeeCure applied.
    pub fixes: u32,
}

impl ChangeCounts {
    /// Counts events by kind. Grouped events ("crashed 14 times") count each time.
    pub fn of(events: &[Event]) -> Self {
        let mut c = ChangeCounts::default();
        for e in events {
            let times = e
                .data
                .as_ref()
                .and_then(|d| d.get("count")?.as_u64())
                .map_or(1, |n| u32::try_from(n).unwrap_or(u32::MAX).max(1));
            let slot = match e.kind {
                EventKind::PackageInstalled
                | EventKind::PackageUpgraded
                | EventKind::PackageRemoved => &mut c.packages,
                EventKind::ConfigChanged => &mut c.config,
                EventKind::DeviceAdded | EventKind::DeviceRemoved => &mut c.devices,
                EventKind::Crash | EventKind::OutOfMemoryKill => &mut c.crashes,
                EventKind::ServiceFailed => &mut c.failed_services,
                EventKind::Boot => &mut c.boots,
                EventKind::UncleanShutdown => &mut c.unclean_shutdowns,
                EventKind::HardwareError => &mut c.hardware_errors,
                EventKind::FixApplied => &mut c.fixes,
                EventKind::FixUndone | EventKind::LoginFailed => continue,
            };
            *slot = slot.saturating_add(times);
        }
        c
    }

    /// All changes together.
    pub fn total(&self) -> u32 {
        [
            self.packages,
            self.config,
            self.devices,
            self.crashes,
            self.failed_services,
            self.boots,
            self.unclean_shutdowns,
            self.hardware_errors,
            self.fixes,
        ]
        .iter()
        .fold(0u32, |a, b| a.saturating_add(*b))
    }
}

/// One disk in the report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiskLine {
    /// Such as `/`.
    pub mount_point: String,
    /// Space used, percent.
    pub used_percent: f64,
    /// Space left, bytes.
    pub free_bytes: f64,
    /// Days until full at the last week's rate; missing when it is not filling or unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub days_until_full: Option<f64>,
}

/// The report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    /// When it was made.
    pub created_at: Timestamp,
    /// The period it covers.
    pub since: Timestamp,
    /// The period it covers.
    pub until: Timestamp,
    /// The computer's name.
    pub host: String,
    /// Such as `Ubuntu 24.04.1 LTS`.
    pub os: String,
    /// The health score now; missing before the first check.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<u8>,
    /// Daily average health scores over the period, oldest first.
    #[serde(default)]
    pub score_trend: Vec<(Timestamp, f64)>,
    /// Open problems, most serious first.
    #[serde(default)]
    pub findings: Vec<Finding>,
    /// What changed.
    #[serde(default)]
    pub changes: ChangeCounts,
    /// The most important changes, newest first (at most 12).
    #[serde(default)]
    pub highlights: Vec<Event>,
    /// Local disks.
    #[serde(default)]
    pub disks: Vec<DiskLine>,
    /// One thing worth doing.
    pub tip: String,
}

impl Report {
    /// How much the score moved over the period (now minus the first day's average).
    pub fn score_change(&self) -> Option<i32> {
        let first = self.score_trend.first()?.1;
        Some(i32::from(self.score?) - first.round() as i32)
    }

    /// One line for a notification: "Health 82 (+3 this week) · 18 changes · 2 warnings".
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        let days = (self.until.as_millis() - self.since.as_millis()) / 86_400_000;
        let over = if (6..=8).contains(&days) {
            "this week".to_owned()
        } else {
            format!("in {days} days")
        };
        if let Some(score) = self.score {
            match self.score_change() {
                Some(d) if d != 0 => parts.push(format!("Health {score} ({d:+} {over})")),
                _ => parts.push(format!("Health {score}")),
            }
        }
        parts.push(plural(self.changes.total(), "change"));
        let (crit, warn) = counts(&self.findings);
        if crit > 0 {
            parts.push(plural(crit, "critical problem"));
        }
        if warn > 0 {
            parts.push(plural(warn, "warning"));
        }
        parts.join(" · ")
    }
}

fn counts(findings: &[Finding]) -> (u32, u32) {
    let n = |s: Severity| findings.iter().filter(|f| f.severity == s).count() as u32;
    (n(Severity::Critical), n(Severity::Warning))
}

fn plural(n: u32, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

fn gb(bytes: f64) -> String {
    format!("{:.1} GB", bytes / 1e9)
}

fn severity_word(s: Severity) -> &'static str {
    match s {
        Severity::Critical => "Critical",
        Severity::Warning => "Warning",
        Severity::Info => "Info",
    }
}

fn period(r: &Report, tz: &local::TimeZone) -> String {
    format!(
        "{} to {}",
        local::format_in(r.since, "%a %d %b %Y", tz),
        local::format_in(r.until, "%a %d %b %Y", tz)
    )
}

/// The report as plain text, for the terminal.
pub fn text(r: &Report, tz: &local::TimeZone) -> String {
    let mut out = format!(
        "DeeCure report · {} · {}\n{}\n\n",
        r.host,
        r.os,
        period(r, tz)
    );
    out.push_str(&format!("{}\n\n", r.summary()));
    out.push_str("Problems\n");
    if r.findings.is_empty() {
        out.push_str("  None. Nothing needs your attention.\n");
    }
    for f in &r.findings {
        out.push_str(&format!("  {:<9} {}\n", severity_word(f.severity), f.title));
    }
    let c = &r.changes;
    out.push_str(&format!(
        "\nWhat changed: {} packages, {} config files, {} device plugs, {} crashes, {} failed services, {} starts ({} unclean), {} DeeCure fixes\n",
        c.packages, c.config, c.devices, c.crashes, c.failed_services, c.boots, c.unclean_shutdowns, c.fixes
    ));
    for e in &r.highlights {
        out.push_str(&format!(
            "  {}  {}\n",
            local::format_in(e.at, "%a %d %b %H:%M", tz),
            e.title
        ));
    }
    if !r.disks.is_empty() {
        out.push_str("\nDisks\n");
        for d in &r.disks {
            let full = d
                .days_until_full
                .map(|n| format!(" · full in about {n:.0} days"))
                .unwrap_or_default();
            out.push_str(&format!(
                "  {:<12} {:>3.0}% used · {} free{full}\n",
                d.mount_point,
                d.used_percent,
                gb(d.free_bytes)
            ));
        }
    }
    out.push_str(&format!("\nTip: {}\n", r.tip));
    out
}

/// Escapes text for HTML.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// A small line chart of the score trend, as inline SVG.
fn sparkline(points: &[(Timestamp, f64)]) -> String {
    if points.len() < 2 {
        return String::new();
    }
    let (w, h) = (320.0, 60.0);
    let step = w / (points.len() - 1) as f64;
    let path: Vec<String> = points
        .iter()
        .enumerate()
        .map(|(i, (_, v))| {
            let y = h - (v.clamp(0.0, 100.0) / 100.0) * (h - 4.0) - 2.0;
            format!(
                "{}{:.1},{:.1}",
                if i == 0 { "M" } else { "L" },
                i as f64 * step,
                y
            )
        })
        .collect();
    format!(
        "<svg class=\"spark\" viewBox=\"0 0 {w} {h}\" role=\"img\" aria-label=\"Health score over the period\"><path d=\"{}\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\"/></svg>",
        path.join("")
    )
}

/// The report as one HTML file that works offline and prints well.
pub fn html(r: &Report, tz: &local::TimeZone) -> String {
    let e = escape;
    let mut body = String::new();
    body.push_str(&format!(
        "<header><h1>DeeCure report</h1><p class=\"muted\">{} · {} · {}</p></header>",
        e(&r.host),
        e(&r.os),
        e(&period(r, tz))
    ));
    let score = r.score.map_or("-".to_owned(), |s| s.to_string());
    let change = match r.score_change() {
        Some(d) if d != 0 => format!("<span class=\"muted\">{d:+} this period</span>"),
        _ => String::new(),
    };
    body.push_str(&format!(
        "<section class=\"score\"><div><div class=\"big\">{score}<small>/100</small></div><div>Health score {change}</div></div>{}</section>",
        sparkline(&r.score_trend)
    ));
    body.push_str(&format!("<p class=\"summary\">{}</p>", e(&r.summary())));
    body.push_str("<h2>Problems</h2>");
    if r.findings.is_empty() {
        body.push_str("<p>None. Nothing needs your attention.</p>");
    } else {
        body.push_str("<ul class=\"problems\">");
        for f in &r.findings {
            let word = severity_word(f.severity);
            body.push_str(&format!(
                "<li><span class=\"badge {}\">{word}</span> <strong>{}</strong><br><span class=\"muted\">{}</span></li>",
                word.to_lowercase(),
                e(&f.title),
                e(&f.explain)
            ));
        }
        body.push_str("</ul>");
    }
    let c = &r.changes;
    body.push_str("<h2>What changed</h2><table class=\"counts\"><tr>");
    for (n, label) in [
        (c.packages, "Packages"),
        (c.config, "Config files"),
        (c.devices, "Device plugs"),
        (c.crashes, "Crashes"),
        (c.failed_services, "Failed services"),
        (c.boots, "Starts"),
        (c.unclean_shutdowns, "Unclean stops"),
        (c.fixes, "DeeCure fixes"),
    ] {
        body.push_str(&format!("<td><b>{n}</b><br>{label}</td>"));
    }
    body.push_str("</tr></table>");
    if !r.highlights.is_empty() {
        body.push_str("<ul class=\"timeline\">");
        for ev in &r.highlights {
            body.push_str(&format!(
                "<li><time>{}</time> {}</li>",
                e(&local::format_in(ev.at, "%a %d %b %H:%M", tz)),
                e(&ev.title)
            ));
        }
        body.push_str("</ul>");
    }
    if !r.disks.is_empty() {
        body.push_str("<h2>Disks</h2><table class=\"disks\"><tr><th>Disk</th><th>Used</th><th>Free</th><th>Full in</th></tr>");
        for d in &r.disks {
            let full = d
                .days_until_full
                .map_or("Not filling".to_owned(), |n| format!("about {n:.0} days"));
            body.push_str(&format!(
                "<tr><td><code>{}</code></td><td>{:.0}%</td><td>{}</td><td>{}</td></tr>",
                e(&d.mount_point),
                d.used_percent,
                gb(d.free_bytes),
                e(&full)
            ));
        }
        body.push_str("</table>");
    }
    body.push_str(&format!("<h2>Tip</h2><p>{}</p>", e(&r.tip)));
    body.push_str(&format!(
        "<footer class=\"muted\">Made by DeeCure on {}. Everything in this report was read on this computer.</footer>",
        e(&local::format_in(r.created_at, "%a %d %b %Y %H:%M", tz))
    ));
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>DeeCure report · {}</title><style>{STYLE}</style></head><body><main>{body}</main></body></html>\n",
        e(&r.host)
    )
}

const STYLE: &str = ":root{--bg:#f3f4f6;--panel:#fff;--text:#1d2027;--muted:#5d6472;--line:#e3e6eb;--crit:#c62828;--warn:#a86400;--info:#1f5fbf;--good:#1e7d32}\
@media (prefers-color-scheme:dark){:root{--bg:#0b0d11;--panel:#14171c;--text:#e8eaee;--muted:#9aa3b2;--line:#262c35;--crit:#ef6b6b;--warn:#e0a84a;--info:#6ea2f2;--good:#5cc278}}\
body{margin:0;background:var(--bg);color:var(--text);font:15px/1.5 system-ui,'Ubuntu Sans',sans-serif}\
main{max-width:760px;margin:24px auto;padding:24px 28px;background:var(--panel);border:1px solid var(--line);border-radius:10px}\
h1{margin:0;font-size:24px}h2{font-size:17px;margin:28px 0 8px;border-bottom:1px solid var(--line);padding-bottom:4px}\
.muted{color:var(--muted)}.score{display:flex;align-items:center;justify-content:space-between;gap:16px;margin-top:16px}\
.big{font-size:44px;font-weight:700;line-height:1}.big small{font-size:16px;color:var(--muted)}.spark{width:320px;max-width:50%;height:60px;color:var(--info)}\
.summary{font-weight:600}ul{padding-left:18px}.problems li{margin:8px 0}\
.badge{display:inline-block;padding:1px 8px;border-radius:9px;font-size:12px;font-weight:700;color:#fff}.critical{background:var(--crit)}.warning{background:var(--warn)}.info{background:var(--info)}\
table{border-collapse:collapse;width:100%}td,th{padding:6px 8px;border-bottom:1px solid var(--line);text-align:left;vertical-align:top}.counts td{text-align:center;font-size:13px}\
.timeline{list-style:none;padding:0}.timeline li{padding:4px 0;border-bottom:1px solid var(--line)}time{display:inline-block;min-width:130px;color:var(--muted);font-variant-numeric:tabular-nums}\
code{font-family:ui-monospace,'Ubuntu Sans Mono',monospace}footer{margin-top:28px;font-size:13px}\
@media print{body{background:#fff}main{border:0;margin:0}}";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FindingState, RuleId};
    use serde_json::json;

    fn report() -> Report {
        Report {
            created_at: Timestamp(1_792_728_000_000),
            since: Timestamp(1_792_123_200_000),
            until: Timestamp(1_792_728_000_000),
            host: "riya-laptop".into(),
            os: "Ubuntu 24.04.1 LTS".into(),
            score: Some(82),
            score_trend: vec![
                (Timestamp(1), 79.4),
                (Timestamp(2), 80.0),
                (Timestamp(3), 82.0),
            ],
            findings: vec![Finding {
                rule: RuleId::new("storage.root_almost_full").unwrap(),
                subject: Some("/".into()),
                severity: Severity::Warning,
                state: FindingState::Open,
                title: "Main disk is <almost> full".into(),
                explain: "Only 30 GB left.".into(),
                first_seen: Timestamp(0),
                last_seen: Timestamp(0),
                fixes: vec![],
            }],
            changes: ChangeCounts {
                packages: 12,
                config: 3,
                crashes: 14,
                ..ChangeCounts::default()
            },
            highlights: vec![Event {
                id: Some(1),
                kind: EventKind::Boot,
                at: Timestamp(1_792_700_000_000),
                title: "Started with a new kernel 6.11 (was 6.8)".into(),
                detail: None,
                subject: None,
                source: "journal".into(),
                data: None,
            }],
            disks: vec![DiskLine {
                mount_point: "/".into(),
                used_percent: 86.0,
                free_bytes: 30e9,
                days_until_full: Some(12.4),
            }],
            tip: "Free up space: Storage › Cleanup shows what can go.".into(),
        }
    }

    #[test]
    fn summary_line_for_the_notification() {
        assert_eq!(
            report().summary(),
            "Health 82 (+3 this week) · 29 changes · 1 warning"
        );
    }

    #[test]
    fn counts_grouped_events_each_time() {
        let ev = |kind, data| Event {
            id: None,
            kind,
            at: Timestamp(0),
            title: String::new(),
            detail: None,
            subject: None,
            source: String::new(),
            data,
        };
        let c = ChangeCounts::of(&[
            ev(EventKind::Crash, Some(json!({"count": 14}))),
            ev(EventKind::PackageUpgraded, None),
            ev(EventKind::FixUndone, None),
            ev(EventKind::UncleanShutdown, None),
        ]);
        assert_eq!(
            (c.crashes, c.packages, c.unclean_shutdowns, c.total()),
            (14, 1, 1, 16)
        );
    }

    #[test]
    fn text_and_html_show_everything_and_html_is_escaped() {
        let tz = local::TimeZone::UTC;
        let t = text(&report(), &tz);
        assert!(t.contains("Warning   Main disk is <almost> full"));
        assert!(t.contains("full in about 12 days"));
        assert!(t.contains("Tip: Free up space"));
        let h = html(&report(), &tz);
        assert!(h.starts_with("<!doctype html>"));
        assert!(h.contains("Main disk is &lt;almost&gt; full"));
        assert!(!h.contains("<almost>"));
        assert!(h.contains("<svg class=\"spark\""));
        assert!(
            !h.contains("http://") && !h.contains("https://"),
            "nothing loads from the network"
        );
    }

    #[test]
    fn an_empty_report_still_reads_well() {
        let mut r = report();
        r.findings.clear();
        r.score = None;
        r.score_trend.clear();
        r.highlights.clear();
        r.disks.clear();
        r.changes = ChangeCounts::default();
        assert_eq!(r.summary(), "0 changes");
        assert!(text(&r, &local::TimeZone::UTC).contains("None. Nothing needs your attention."));
        assert!(html(&r, &local::TimeZone::UTC).contains("-<small>/100</small>"));
    }
}
