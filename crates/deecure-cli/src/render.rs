// SPDX-License-Identifier: Apache-2.0

//! Turns agent answers into plain-English text for the terminal.

use std::fmt::Write;
use std::io::IsTerminal;

use deecure_core::rpc::StatusReport;
use deecure_core::{Finding, Severity, Status, Timestamp};

/// How wide the text may be.
const WIDTH: usize = 78;
/// Where the title and explanation start, after the badge.
const INDENT: usize = 14;

/// Whether to use colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    /// Use ANSI colors.
    pub color: bool,
}

impl Style {
    /// Colors only on a real terminal, and never when `NO_COLOR` is set (no-color.org).
    pub fn detect() -> Self {
        let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
        Style {
            color: std::io::stdout().is_terminal() && !no_color,
        }
    }

    fn paint(self, code: &str, text: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_owned()
        }
    }
}

/// The text for `deecure status`.
pub fn status(report: &StatusReport, now: Timestamp, style: Style) -> String {
    let mut out = String::new();
    let Some(checked_at) = report.checked_at else {
        out.push_str("DeeCure has not checked this computer yet. It does so a few seconds\n");
        out.push_str("after the agent starts. Try again in a minute.\n");
        return out;
    };
    let _ = writeln!(
        out,
        "{} · checked {}\n",
        style.paint("1", "DeeCure status"),
        ago(checked_at, now)
    );

    if report.findings.is_empty() {
        let _ = writeln!(out, "{} No problems found.", style.paint("32", "✓"));
        return out;
    }
    for finding in &report.findings {
        finding_block(&mut out, finding, style);
    }
    out.push_str(&summary(&report.findings));
    out
}

fn finding_block(out: &mut String, finding: &Finding, style: Style) {
    let (icon, color) = match finding.severity {
        Severity::Critical => ("✗", "31"),
        Severity::Warning => ("!", "33"),
        Severity::Info => ("i", "34"),
    };
    // Icon and word together: the meaning never depends on color alone.
    let label = format!("{icon} {}", Status::from(finding.severity).label());
    let padded = format!("  {label:<width$}", width = INDENT - 2);
    let _ = writeln!(
        out,
        "{}{}",
        style.paint(color, &padded),
        style.paint("1", &finding.title)
    );
    let margin = " ".repeat(INDENT);
    for line in wrap(&finding.explain, WIDTH - INDENT) {
        let _ = writeln!(out, "{margin}{line}");
    }
    let mut source = finding.rule.to_string();
    if let Some(subject) = &finding.subject {
        let _ = write!(source, " · {subject}");
    }
    let _ = writeln!(out, "{margin}{}\n", style.paint("2", &source));
}

/// "3 problems: 1 critical, 1 warning, 1 info."
fn summary(findings: &[Finding]) -> String {
    let count = |severity| findings.iter().filter(|f| f.severity == severity).count();
    let (critical, warnings, info) = (
        count(Severity::Critical),
        count(Severity::Warning),
        count(Severity::Info),
    );
    let plural = |n: usize, word: &str| {
        if n == 1 {
            word.to_owned()
        } else {
            format!("{word}s")
        }
    };
    format!(
        "{} {}: {critical} critical, {warnings} {}, {info} info.\n",
        findings.len(),
        plural(findings.len(), "problem"),
        plural(warnings, "warning"),
    )
}

/// "just now", "5 min ago", "3 h ago", "2 days ago".
fn ago(then: Timestamp, now: Timestamp) -> String {
    let minutes = (now.as_millis() - then.as_millis()) / 60_000;
    match minutes {
        ..1 => "just now".to_owned(),
        1..60 => format!("{minutes} min ago"),
        60..1440 => format!("{} h ago", minutes / 60),
        _ => {
            let days = minutes / 1440;
            format!("{days} {} ago", if days == 1 { "day" } else { "days" })
        }
    }
}

/// Splits text into lines of at most `width` characters, at spaces.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use deecure_core::{FindingState, FixActionId, RuleId};

    const PLAIN: Style = Style { color: false };
    const MINUTE: i64 = 60_000;

    fn finding(
        rule: &str,
        severity: Severity,
        title: &str,
        explain: &str,
        subject: Option<&str>,
    ) -> Finding {
        Finding {
            rule: RuleId::new(rule).unwrap(),
            subject: subject.map(str::to_owned),
            severity,
            state: FindingState::Open,
            title: title.into(),
            explain: explain.into(),
            first_seen: Timestamp(0),
            last_seen: Timestamp(0),
            fixes: vec![FixActionId::new("storage.clean_apt_cache").unwrap()],
        }
    }

    fn report(checked_at: Option<i64>, findings: Vec<Finding>) -> StatusReport {
        StatusReport {
            generated_at: Timestamp(10 * MINUTE),
            checked_at: checked_at.map(Timestamp),
            checked: vec![],
            findings,
        }
    }

    #[test]
    fn findings_are_listed_with_icon_word_and_source() {
        let report = report(
            Some(8 * MINUTE),
            vec![
                finding(
                    "storage.root_almost_full",
                    Severity::Critical,
                    "Main disk is almost full",
                    "Your main disk has only 13.8 GB left. When it is full, apps can crash and updates can fail.",
                    None,
                ),
                finding(
                    "power.battery_worn",
                    Severity::Info,
                    "Battery has lost some of its capacity",
                    "It holds 79.3%.",
                    Some("BAT0"),
                ),
            ],
        );
        let text = status(&report, Timestamp(10 * MINUTE), PLAIN);
        assert_eq!(
            text,
            "\
DeeCure status · checked 2 min ago

  ✗ Critical  Main disk is almost full
              Your main disk has only 13.8 GB left. When it is full, apps can
              crash and updates can fail.
              storage.root_almost_full

  i Info      Battery has lost some of its capacity
              It holds 79.3%.
              power.battery_worn · BAT0

2 problems: 1 critical, 0 warnings, 1 info.
"
        );
    }

    #[test]
    fn no_problems_and_not_checked_yet_are_different() {
        let healthy = status(
            &report(Some(10 * MINUTE), vec![]),
            Timestamp(10 * MINUTE),
            PLAIN,
        );
        assert!(healthy.ends_with("✓ No problems found.\n"), "{healthy}");
        let starting = status(&report(None, vec![]), Timestamp(10 * MINUTE), PLAIN);
        assert!(starting.starts_with("DeeCure has not checked this computer yet."));
    }

    #[test]
    fn colors_wrap_text_only_when_asked() {
        let report = report(
            Some(0),
            vec![finding(
                "services.unit_failed",
                Severity::Warning,
                "t",
                "e",
                None,
            )],
        );
        let colored = status(&report, Timestamp(0), Style { color: true });
        assert!(
            colored.contains("\x1b[33m  ! Warning   \x1b[0m"),
            "{colored:?}"
        );
        assert!(!status(&report, Timestamp(0), PLAIN).contains('\x1b'));
    }

    #[test]
    fn ago_reads_naturally() {
        let at = |minutes: i64| ago(Timestamp(0), Timestamp(minutes * MINUTE));
        assert_eq!(at(0), "just now");
        assert_eq!(at(-5), "just now", "a clock that moved back");
        assert_eq!(at(5), "5 min ago");
        assert_eq!(at(150), "2 h ago");
        assert_eq!(at(1440), "1 day ago");
        assert_eq!(at(4000), "2 days ago");
    }

    #[test]
    fn wrap_breaks_at_spaces_and_keeps_long_words() {
        assert_eq!(wrap("aa bb cc dd", 5), ["aa bb", "cc dd"]);
        assert_eq!(
            wrap("averyveryverylongword x", 5),
            ["averyveryverylongword", "x"]
        );
        assert!(wrap("   ", 5).is_empty());
    }
}
