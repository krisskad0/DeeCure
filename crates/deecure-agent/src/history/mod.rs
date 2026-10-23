// SPDX-License-Identifier: Apache-2.0

//! History: what changed on the computer, saved as events for the Activity pages,
//! `deecure history`, and the weekly report.
//!
//! Every 5 minutes this task reads package logs, the snap and Flatpak app lists, the
//! journal, and config files, and saves what is new. Each source is read with a key per
//! event, so reading it twice (after a restart, or a log that did not change) stores
//! nothing twice. A source that fails is skipped and logged once; the others still run.
//! Sources and event details: `docs/02-architecture/collectors.md#history-events`.

pub(crate) mod config;
pub(crate) mod diff;
pub(crate) mod journal;
pub(crate) mod packages;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use deecure_core::fix::RunStatus;
use deecure_core::{Event, EventKind, Timestamp};
use serde_json::json;

use crate::collect::SystemRoot;
use crate::inventory::{command, logs, processes};
use crate::store::{self, SharedStore};

/// How often history is read.
pub const EVERY: Duration = Duration::from_secs(5 * 60);
/// Boots change only at a boot, so the boot list is read at start and every 6 hours.
const BOOTS_EVERY: Duration = Duration::from_secs(6 * 60 * 60);
/// The first journal read goes back this far.
const FIRST_LOOK_BACK_MS: i64 = 30 * 86_400_000;
/// At most this many journal entries per read (the newest), so a log flood stays cheap.
const MAX_JOURNAL_ENTRIES: usize = 20_000;
const LIMIT: Duration = Duration::from_secs(30);
/// Setting: where the next journal read starts (ms, the start of an hour).
const JOURNAL_SINCE: &str = "history.journal_since";
/// Settings: the last snap and Flatpak lists.
const SNAPS: &str = "history.snaps";
const FLATPAKS: &str = "history.flatpaks";

/// Package logs and their newest rotated copy.
const APT_LOGS: &[&str] = &[
    "/var/log/apt/history.log.1.gz",
    "/var/log/apt/history.log.1",
    "/var/log/apt/history.log",
];
const DPKG_LOGS: &[&str] = &[
    "/var/log/dpkg.log.1.gz",
    "/var/log/dpkg.log.1",
    "/var/log/dpkg.log",
];

/// Time ranges when something known was changing the system, to say who changed a
/// config file: apt runs and DeeCure fixes.
#[derive(Debug, Clone, Default)]
pub struct Windows(pub Vec<(i64, i64, String)>);

impl Windows {
    /// Who was changing the system at `at` (with a minute's margin), if anyone.
    pub fn who(&self, at: i64) -> Option<&str> {
        self.0
            .iter()
            .rev()
            .find(|(start, end, _)| at >= start - 60_000 && at <= end + 60_000)
            .map(|(_, _, who)| who.as_str())
    }
}

/// A stable fingerprint (FNV-1a, 64 bits) as 16 hex digits. Not for security: it only
/// tells whether a file changed, and it must stay the same across Rust versions.
pub fn fnv1a(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// The history task's memory between runs.
pub struct History {
    root: SystemRoot,
    home: Option<PathBuf>,
    /// (inode, size, mtime) of each package log at the last read.
    seen_logs: HashMap<String, (u64, u64, i64)>,
    /// apt runs from the last read of the logs.
    windows: Windows,
    boots_read: Option<Instant>,
    /// Sources failing right now, so each failure is logged once.
    failing: HashSet<&'static str>,
}

impl History {
    /// History for the user whose home is `home`.
    pub fn new(root: SystemRoot, home: Option<PathBuf>) -> Self {
        History {
            root,
            home,
            seen_logs: HashMap::new(),
            windows: Windows::default(),
            boots_read: None,
            failing: HashSet::new(),
        }
    }

    fn outcome(&mut self, name: &'static str, result: Result<(), String>) {
        match result {
            Ok(()) => {
                if self.failing.remove(name) {
                    eprintln!("deecure-agent: history: {name} works again");
                }
            }
            Err(e) => {
                if self.failing.insert(name) {
                    eprintln!("deecure-agent: history: {name} failed: {e}");
                }
            }
        }
    }

    /// Reads every source once and saves what is new.
    pub async fn collect(&mut self, store: &SharedStore, now: Timestamp) {
        let r = self.packages(store).await;
        self.outcome("package logs", r);
        let r = self.apps(store, "snap", SNAPS, now).await;
        self.outcome("snap list", r);
        let r = self.apps(store, "flatpak", FLATPAKS, now).await;
        self.outcome("flatpak list", r);
        let r = self.journal(store, now).await;
        self.outcome("journal", r);
        if self.boots_read.is_none_or(|t| t.elapsed() >= BOOTS_EVERY) {
            let r = self.boots(store).await;
            self.outcome("boot list", r);
        }
        let r = self.config(store, now).await;
        self.outcome("config files", r);
    }

    /// Reads the package logs when one of them changed.
    async fn packages(&mut self, store: &SharedStore) -> Result<(), String> {
        let mut changed = false;
        let mut signatures = HashMap::new();
        for path in APT_LOGS.iter().chain(DPKG_LOGS) {
            if let Ok(m) = std::fs::metadata(self.root.path(path)) {
                let sig = (m.ino(), m.size(), m.mtime());
                changed |= self.seen_logs.get(*path) != Some(&sig);
                signatures.insert((*path).to_owned(), sig);
            }
        }
        changed |= signatures.len() != self.seen_logs.len();
        if !changed {
            return Ok(());
        }
        let mut apt_texts = Vec::new();
        for path in APT_LOGS {
            if let Some(text) = self.read_log(path).await {
                apt_texts.push(text);
            }
        }
        let mut dpkg_texts = Vec::new();
        for path in DPKG_LOGS {
            if let Some(text) = self.read_log(path).await {
                dpkg_texts.push(text);
            }
        }
        // Parsing a big log takes a moment: do it on a worker thread, so the agent keeps
        // answering the app and the CLI meanwhile.
        let (events, windows) = tokio::task::spawn_blocking(move || {
            let tz = deecure_core::local::zone();
            let runs: Vec<packages::AptRun> = apt_texts
                .iter()
                .flat_map(|t| packages::parse_apt(t, &tz))
                .collect();
            let dpkg = dpkg_texts
                .iter()
                .flat_map(|t| packages::parse_dpkg(t, &tz))
                .collect();
            let own = packages::outside_apt(dpkg, &runs);
            let mut events = packages::apt_events(&runs);
            events.extend(packages::dpkg_events(&own));
            // "apt install -y a b c" is too long to say who changed a file; "apt" is enough.
            let who = |source: &str| {
                if source.starts_with("apt") {
                    "apt".to_owned()
                } else {
                    source.to_owned()
                }
            };
            let windows = Windows(
                runs.iter()
                    .map(|r| (r.start, r.end, who(&r.source)))
                    .collect(),
            );
            (events, windows)
        })
        .await
        .map_err(|e| e.to_string())?;
        self.windows = windows;
        save_events(store, events, false).await?;
        self.seen_logs = signatures;
        Ok(())
    }

    /// A log's text; `.gz` files are unpacked with `gzip -dc`. `None` if missing or unreadable.
    async fn read_log(&self, path: &str) -> Option<String> {
        let real = self.root.path(path);
        if path.ends_with(".gz") {
            let real = real.to_string_lossy().into_owned();
            if !std::path::Path::new(&real).exists() {
                return None;
            }
            command::output("gzip", &["-dc", &real], LIMIT).await
        } else {
            std::fs::read(&real)
                .ok()
                .map(|b| String::from_utf8_lossy(&b).into_owned())
        }
    }

    /// Compares the snap or Flatpak app list with the last one. A missing or failing
    /// program changes nothing: only a list that was read can show a removal.
    async fn apps(
        &self,
        store: &SharedStore,
        format: &'static str,
        setting: &str,
        now: Timestamp,
    ) -> Result<(), String> {
        let text = match format {
            "snap" => {
                if !self.root.path("/run/snapd.socket").exists() {
                    return Ok(()); // snapd is not installed or not running
                }
                command::output("snap", &["list"], LIMIT).await
            }
            _ => {
                if !self.root.path("/usr/bin/flatpak").exists() {
                    return Ok(());
                }
                command::output(
                    "flatpak",
                    &[
                        "list",
                        "--app",
                        "--columns=application,version,branch,installation,active",
                    ],
                    LIMIT,
                )
                .await
            }
        };
        let Some(text) = text else {
            return Err(format!("could not read the {format} list"));
        };
        let new = if format == "snap" {
            packages::parse_snaps(&text)
        } else {
            packages::parse_flatpaks(&text)
        };
        let s = store::lock(store);
        let old: Option<BTreeMap<String, packages::App>> =
            s.setting(setting).map_err(|e| e.to_string())?;
        s.batch(|s| {
            if let Some(old) = &old {
                for item in packages::compare(old, &new) {
                    let at = self.snap_time(format, &item, now);
                    let key = format!("{format}:{}:{}:{}", item.name, item.action, now.as_millis());
                    s.add_event(
                        &key,
                        &packages::event(&item, at, format, format, None, false),
                    )?;
                }
            }
            s.set_setting(setting, &new)
        })
        .map_err(|e| e.to_string())
    }

    /// When a snap revision arrived: its folder's time, if it is recent. Otherwise now.
    fn snap_time(&self, format: &str, item: &packages::Item, now: Timestamp) -> i64 {
        let now = now.as_millis();
        if format != "snap" || item.action == "remove" {
            return now;
        }
        let current = self.root.path(&format!("/snap/{}/current", item.name));
        // The `current` link is replaced when a new revision becomes active.
        std::fs::symlink_metadata(current)
            .ok()
            .map(|m| m.mtime() * 1000)
            .filter(|t| *t <= now && *t > now - 2 * EVERY.as_millis() as i64)
            .unwrap_or(now)
    }

    /// Reads the journal from the start of the hour of the last read.
    async fn journal(&self, store: &SharedStore, now: Timestamp) -> Result<(), String> {
        let since = store::lock(store)
            .setting::<i64>(JOURNAL_SINCE)
            .map_err(|e| e.to_string())?
            .unwrap_or(now.as_millis() - FIRST_LOOK_BACK_MS);
        let since = since.div_euclid(journal::HOUR_MS) * journal::HOUR_MS;
        let since_arg = format!("@{}", since / 1000);
        let max_arg = MAX_JOURNAL_ENTRIES.to_string();
        // `--grep` exits with 1 when nothing matches, so any finished run counts.
        let text = command::output_any(
            "journalctl",
            &[
                "-o",
                "json",
                "--no-pager",
                "-q",
                "--output-fields=PRIORITY,SYSLOG_IDENTIFIER,_SYSTEMD_UNIT,_COMM,MESSAGE",
                "--since",
                &since_arg,
                "-n",
                &max_arg,
                "--grep",
                journal::GREP,
            ],
            LIMIT,
        )
        .await
        .ok_or("journalctl is missing or took too long")?;
        let (events, from) = tokio::task::spawn_blocking(move || {
            let entries = logs::parse_entries(&text);
            // When the entry limit cut the read short, only the hours after the oldest
            // entry read are complete; older hours keep what was stored before.
            let full = entries.len() >= MAX_JOURNAL_ENTRIES;
            let from = match entries.first() {
                Some(first) if full => {
                    (first.at.div_euclid(journal::HOUR_MS) + 1) * journal::HOUR_MS
                }
                _ => since,
            };
            let events: Vec<_> = journal::events(&entries)
                .into_iter()
                .filter(|(_, e)| e.at.as_millis() >= from)
                .collect();
            (events, from)
        })
        .await
        .map_err(|e| e.to_string())?;
        // The hours from `from` on were read in full: what is stored for them is replaced,
        // so a failure that turned out to be part of a shutdown goes away. The next read
        // starts here only after everything is saved, so a crash in between is harmless.
        store::lock(store)
            .delete_events_from(journal::KEY_PREFIXES, Timestamp(from))
            .map_err(|e| e.to_string())?;
        save_events(store, events, true).await?;
        let next = now.as_millis().div_euclid(journal::HOUR_MS) * journal::HOUR_MS;
        store::lock(store)
            .set_setting(JOURNAL_SINCE, &next)
            .map_err(|e| e.to_string())
    }

    async fn boots(&mut self, store: &SharedStore) -> Result<(), String> {
        let boots = logs::read_boots().await;
        if boots.is_empty() {
            return Err("the journal lists no boots".into());
        }
        save_events(store, journal::boot_events(&boots), false).await?;
        self.boots_read = Some(Instant::now());
        Ok(())
    }

    async fn config(&self, store: &SharedStore, now: Timestamp) -> Result<(), String> {
        let mut roots = vec![config::Root {
            real: self.root.path("/etc"),
            shown: "/etc".into(),
        }];
        if let Some(home) = &self.home {
            for dir in [".config/autostart", ".config/systemd/user"] {
                roots.push(config::Root {
                    real: home.join(dir),
                    shown: format!("~/{dir}"),
                });
            }
        }
        let users = processes::user_names(&self.root);
        let uid = rustix::process::getuid().as_raw();
        let crontab = match (users.get(&uid), self.root.path("/usr/bin/crontab").exists()) {
            (Some(user), true) => {
                command::output_any("crontab", &["-l"], LIMIT)
                    .await
                    .map(|text| config::Crontab {
                        shown: format!("/var/spool/cron/crontabs/{user}"),
                        text: (!text.trim().is_empty()).then_some(text),
                        folder_mtime: std::fs::metadata(self.root.path("/var/spool/cron/crontabs"))
                            .ok()
                            .map(|m| m.mtime() * 1000),
                        uid,
                    })
            }
            _ => None,
        };
        // Fixes DeeCure made count as "changed by DeeCure".
        let mut windows = self.windows.clone();
        let runs = store::lock(store)
            .fix_runs(200)
            .map_err(|e| e.to_string())?;
        windows.0.extend(runs.iter().map(|r| {
            (
                r.started_at.as_millis(),
                r.finished_at.as_millis(),
                "DeeCure".to_owned(),
            )
        }));
        let store = store.clone();
        tokio::task::spawn_blocking(move || {
            config::check(&store, &roots, crontab, &windows, &users, now)
        })
        .await
        .map_err(|e| e.to_string())?
        .map(|_| ())
        .map_err(|e| e.to_string())
    }
}

/// Events saved per transaction. The store is unlocked between chunks, so the app and
/// the CLI get their answers even while a big first import is saved.
const CHUNK: usize = 500;

/// Saves events on a worker thread, [`CHUNK`] at a time. `replace` updates events whose
/// key is already stored (grouped events that grow); otherwise they are left as they are.
async fn save_events(
    store: &SharedStore,
    events: Vec<(String, Event)>,
    replace: bool,
) -> Result<(), String> {
    if events.is_empty() {
        return Ok(());
    }
    let store = store.clone();
    tokio::task::spawn_blocking(move || {
        for chunk in events.chunks(CHUNK) {
            store::lock(&store).batch(|s| {
                for (key, event) in chunk {
                    if replace {
                        s.put_event(key, event)?;
                    } else {
                        s.add_event(key, event)?;
                    }
                }
                Ok(())
            })?;
        }
        Ok::<(), store::StoreError>(())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

/// The event for a fix DeeCure ran, or undid.
pub fn fix_event(run: &deecure_core::fix::FixRun, undone_at: Option<Timestamp>) -> (String, Event) {
    let undoable = run.status != RunStatus::Undone && !run.undo_calls().is_empty();
    let data = json!({
        "run_id": run.id,
        "status": run.status,
        "undoable": undoable,
    });
    match undone_at {
        None => (
            format!("fix:{}", run.id),
            Event {
                id: None,
                kind: EventKind::FixApplied,
                at: run.finished_at,
                title: run.title.clone(),
                detail: Some(match run.status {
                    RunStatus::Failed => "DeeCure · did not finish".into(),
                    _ if undoable => "DeeCure · can be undone".into(),
                    _ => "DeeCure".into(),
                }),
                subject: run.subject.clone(),
                source: "DeeCure".into(),
                data: Some(data),
            },
        ),
        Some(at) => (
            format!("undo:{}", run.id),
            Event {
                id: None,
                kind: EventKind::FixUndone,
                at,
                title: format!("Undid: {}", run.title),
                detail: Some("DeeCure".into()),
                subject: run.subject.clone(),
                source: "DeeCure".into(),
                data: Some(data),
            },
        ),
    }
}

/// Saves events for fixes made before history existed (once; keys stop repeats).
pub fn backfill_fixes(store: &SharedStore) -> store::Result<()> {
    let s = store::lock(store);
    let runs = s.fix_runs(1000)?;
    s.batch(|s| {
        for run in &runs {
            let (key, event) = fix_event(run, None);
            s.add_event(&key, &event)?;
        }
        Ok(())
    })
}

/// Reads history now, then every [`EVERY`], forever.
pub async fn run(store: SharedStore, root: SystemRoot) {
    if let Err(e) = backfill_fixes(&store) {
        eprintln!("deecure-agent: history: could not add past fixes: {e}");
    }
    let home = std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .map(PathBuf::from);
    let mut history = History::new(root, home);
    let mut timer = tokio::time::interval(EVERY);
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        timer.tick().await;
        history.collect(&store, Timestamp::now()).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprints_are_stable() {
        assert_eq!(fnv1a(b""), "cbf29ce484222325");
        assert_eq!(fnv1a(b"a"), "af63dc4c8601ec8c");
    }

    #[test]
    fn windows_name_who_was_changing_things() {
        let w = Windows(vec![
            (1_000_000, 1_100_000, "apt".into()),
            (2_000_000, 2_000_000, "DeeCure".into()),
        ]);
        assert_eq!(w.who(1_050_000), Some("apt"));
        assert_eq!(w.who(1_150_000), Some("apt"), "a minute's margin");
        assert_eq!(w.who(1_500_000), None);
        assert_eq!(w.who(2_030_000), Some("DeeCure"));
    }
}
