// SPDX-License-Identifier: Apache-2.0

//! Scheduled jobs: cron, anacron, at, and how often jobs ran in the last 7 days.
//! Timers come from `units.rs`. Schedules are sent as written; the app works out the
//! next run in local time.

use std::collections::HashSet;
use std::time::{Duration, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;

use super::command;
use super::units::{self, Timer};
use crate::collect::SystemRoot;
use crate::context::Agent;

/// One cron line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CronJob {
    /// Five fields such as `*/30 * * * *`, or a nickname such as `@daily`.
    pub schedule: String,
    pub command: String,
    pub user: String,
    /// `crontab (riya)`, `/etc/crontab`, or `/etc/cron.d/<file>`.
    pub source: String,
}

/// One anacron job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnacronJob {
    pub id: String,
    /// Days, or `@monthly`.
    pub period: String,
    /// Minutes to wait after start.
    pub delay: u32,
    pub command: String,
    /// When it last ran (the stamp file's time), milliseconds since 1970.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run: Option<i64>,
}

/// One `at` job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AtJob {
    pub id: String,
    /// As `atq` prints it, such as `Fri Oct 23 09:00:00 2026`.
    pub when: String,
    pub queue: String,
    pub user: String,
}

/// One past run, for the "runs by hour" heatmap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Run {
    pub at: i64,
    pub job: String,
    pub failed: bool,
}

/// The `jobs` inventory.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct JobsInfo {
    pub cron: Vec<CronJob>,
    pub timers: Vec<Timer>,
    pub anacron: Vec<AnacronJob>,
    pub at: Vec<AtJob>,
    /// True when the `at` tools are installed.
    pub at_installed: bool,
    /// Runs in the last 7 days, oldest first.
    pub runs: Vec<Run>,
}

/// Reads a crontab. `user` is `None` for system files, whose lines carry a user field.
pub fn parse_crontab(text: &str, user: Option<&str>, source: &str) -> Vec<CronJob> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let first = words.next()?;
            // Lines like SHELL=/bin/sh set variables; they are not jobs.
            if first.contains('=') && !first.starts_with('@') {
                return None;
            }
            let fields = if first.starts_with('@') { 1 } else { 5 };
            let mut parts: Vec<&str> = std::iter::once(first).chain(words).collect();
            if parts.len() < fields + 1 + usize::from(user.is_none()) {
                return None;
            }
            let schedule = parts.drain(..fields).collect::<Vec<_>>().join(" ");
            let who = match user {
                Some(u) => u.to_owned(),
                None => parts.remove(0).to_owned(),
            };
            Some(CronJob {
                schedule,
                command: parts.join(" "),
                user: who,
                source: source.to_owned(),
            })
        })
        .collect()
}

/// Reads `/etc/anacrontab`: `period delay id command`.
pub fn parse_anacrontab(text: &str) -> Vec<AnacronJob> {
    text.lines()
        .map(str::trim)
        .filter(|l| {
            !l.is_empty()
                && !l.starts_with('#')
                && !l.split_whitespace().next().is_some_and(|w| w.contains('='))
        })
        .filter_map(|l| {
            let mut f = l.split_whitespace();
            Some(AnacronJob {
                period: f.next()?.to_owned(),
                delay: f.next()?.parse().ok()?,
                id: f.next()?.to_owned(),
                command: f.collect::<Vec<_>>().join(" "),
                last_run: None,
            })
        })
        .collect()
}

/// Reads `atq`: `4\tFri Oct 23 09:00:00 2026 a riya`.
pub fn parse_atq(text: &str) -> Vec<AtJob> {
    text.lines()
        .filter_map(|l| {
            let (id, rest) = l.split_once('\t')?;
            let words: Vec<&str> = rest.split_whitespace().collect();
            let n = words.len();
            (n >= 7).then(|| AtJob {
                id: id.trim().to_owned(),
                when: words[..n - 2].join(" "),
                queue: words[n - 2].to_owned(),
                user: words[n - 1].to_owned(),
            })
        })
        .collect()
}

/// systemd's "unit started" and "unit failed" message IDs.
const STARTED: &str = "39f53479d3a045ac8e11786248231fbf";
const FAILED: &str = "d9b373ed55a64feb8242e02dbe79a49c";

/// Runs of cron jobs and of the services timers start, from the journal.
pub fn parse_runs(cron_lines: &str, unit_lines: &str, timer_units: &HashSet<String>) -> Vec<Run> {
    let mut runs = Vec::new();
    let time = |e: &Value| {
        e.get("__REALTIME_TIMESTAMP")
            .and_then(Value::as_str)
            .and_then(|t| t.parse::<i64>().ok())
            .map(|us| us / 1000)
    };
    for e in cron_lines
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
    {
        let message = e.get("MESSAGE").and_then(Value::as_str).unwrap_or("");
        if let (Some(at), Some(cmd)) = (
            time(&e),
            message
                .split_once(" CMD (")
                .map(|(_, c)| c.trim_end_matches(')')),
        ) {
            runs.push(Run {
                at,
                job: cmd.to_owned(),
                failed: false,
            });
        }
    }
    for e in unit_lines
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
    {
        let unit = e
            .get("UNIT")
            .or_else(|| e.get("USER_UNIT"))
            .and_then(Value::as_str)
            .unwrap_or("");
        if !timer_units.contains(unit) {
            continue;
        }
        let failed = e.get("MESSAGE_ID").and_then(Value::as_str) == Some(FAILED);
        if let Some(at) = time(&e) {
            runs.push(Run {
                at,
                job: unit.to_owned(),
                failed,
            });
        }
    }
    runs.sort_by_key(|r| r.at);
    runs
}

fn mtime_ms(path: &std::path::Path) -> Option<i64> {
    let t = std::fs::metadata(path).ok()?.modified().ok()?;
    i64::try_from(t.duration_since(UNIX_EPOCH).ok()?.as_millis()).ok()
}

/// Reads all jobs.
pub async fn read(agent: &Agent) -> JobsInfo {
    let root: &SystemRoot = &agent.root;
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_default();
    let mut cron = Vec::new();
    // `crontab -l` exits non-zero when the user has none; that is an empty list.
    if let Some(text) = command::output("crontab", &["-l"], Duration::from_secs(5)).await {
        cron.extend(parse_crontab(
            &text,
            Some(&user),
            &format!("crontab ({user})"),
        ));
    }
    cron.extend(parse_crontab(
        &root.read("/etc/crontab").unwrap_or_default(),
        None,
        "/etc/crontab",
    ));
    let mut files: Vec<_> = std::fs::read_dir(root.path("/etc/cron.d"))
        .map(|d| d.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    files.sort();
    for f in files {
        let name = f
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        // run-parts skips names with dots and hidden files, like `.placeholder`.
        if name.contains('.') {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(&f) {
            cron.extend(parse_crontab(&text, None, &format!("/etc/cron.d/{name}")));
        }
    }
    let mut anacron = parse_anacrontab(&root.read("/etc/anacrontab").unwrap_or_default());
    for job in &mut anacron {
        job.last_run = mtime_ms(&root.path(&format!("/var/spool/anacron/{}", job.id)));
    }
    let at_installed = root.path("/usr/bin/atq").exists();
    let at = if at_installed {
        command::output("atq", &[], Duration::from_secs(5))
            .await
            .map(|t| parse_atq(&t))
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let timers = units::timers(agent).await;
    let timer_units: HashSet<String> = timers.iter().map(|t| t.activates.clone()).collect();
    let journal = |filter: String| async move {
        let args = [
            "-o",
            "json",
            "--no-pager",
            "-q",
            "--since",
            "-7d",
            "--output-fields=MESSAGE,MESSAGE_ID,UNIT,USER_UNIT",
            filter.as_str(),
        ];
        command::output("journalctl", &args, Duration::from_secs(20))
            .await
            .unwrap_or_default()
    };
    let cron_lines = journal("SYSLOG_IDENTIFIER=CRON".into()).await;
    let started = journal(format!("MESSAGE_ID={STARTED}")).await;
    let failed = journal(format!("MESSAGE_ID={FAILED}")).await;
    let runs = parse_runs(&cron_lines, &format!("{started}\n{failed}"), &timer_units);
    JobsInfo {
        cron,
        timers,
        anacron,
        at,
        at_installed,
        runs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_user_and_system_crontabs() {
        let user = parse_crontab(
            "# m h dom mon dow command\n*/30 * * * * ~/bin/sync-notes.sh\nMAILTO=riya\n@reboot ~/bin/start.sh\n",
            Some("riya"),
            "crontab (riya)",
        );
        assert_eq!(user.len(), 2);
        assert_eq!(
            (user[0].schedule.as_str(), user[0].command.as_str()),
            ("*/30 * * * *", "~/bin/sync-notes.sh")
        );
        assert_eq!(user[1].schedule, "@reboot");
        let system = parse_crontab(
            "SHELL=/bin/sh\n17 *\t* * *\troot\tcd / && run-parts --report /etc/cron.hourly\n",
            None,
            "/etc/crontab",
        );
        assert_eq!(
            (system[0].user.as_str(), system[0].command.as_str()),
            ("root", "cd / && run-parts --report /etc/cron.hourly")
        );
    }

    #[test]
    fn parses_anacron_and_atq() {
        let a = parse_anacrontab(
            "SHELL=/bin/sh\n1\t5\tcron.daily\trun-parts --report /etc/cron.daily\n@monthly\t15\tcron.monthly\trun-parts /etc/cron.monthly\n",
        );
        assert_eq!(
            (a[0].period.as_str(), a[0].delay, a[0].id.as_str()),
            ("1", 5, "cron.daily")
        );
        assert_eq!(a[1].period, "@monthly");
        let at = parse_atq("4\tFri Oct 23 09:00:00 2026 a riya\n");
        assert_eq!(
            (at[0].id.as_str(), at[0].when.as_str(), at[0].user.as_str()),
            ("4", "Fri Oct 23 09:00:00 2026", "riya")
        );
    }

    #[test]
    fn counts_cron_and_timer_runs() {
        let cron = r#"{"__REALTIME_TIMESTAMP":"2000000","MESSAGE":"(root) CMD (cd / && run-parts --report /etc/cron.hourly)"}
{"__REALTIME_TIMESTAMP":"2500000","MESSAGE":"pam_unix(cron:session): session opened"}"#;
        let units = format!(
            "{{\"__REALTIME_TIMESTAMP\":\"1000000\",\"MESSAGE_ID\":\"{STARTED}\",\"UNIT\":\"apt-daily.service\"}}\n{{\"__REALTIME_TIMESTAMP\":\"3000000\",\"MESSAGE_ID\":\"{FAILED}\",\"UNIT\":\"apt-daily.service\"}}\n{{\"__REALTIME_TIMESTAMP\":\"4000000\",\"MESSAGE_ID\":\"{STARTED}\",\"UNIT\":\"docker-x.scope\"}}"
        );
        let set: HashSet<String> = ["apt-daily.service".to_owned()].into();
        let runs = parse_runs(cron, &units, &set);
        assert_eq!(
            runs.iter().map(|r| (r.at, r.failed)).collect::<Vec<_>>(),
            [(1000, false), (2000, false), (3000, true)]
        );
    }
}
