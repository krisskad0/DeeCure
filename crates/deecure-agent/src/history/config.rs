// SPDX-License-Identifier: Apache-2.0

//! Config changes: files in `/etc`, the user's autostart and systemd user folders, and
//! the user's crontab.
//!
//! Every few minutes the folders are listed and each file's time and size compared with
//! the last scan (`watched_file`). Only files that look different are read. A file whose
//! content is the same (only touched) gives no event. Small text files are kept, so the
//! next change can show a diff. This needs no inotify watches, so it never uses up the
//! watches other apps need, and it also sees changes made while the agent was not running.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use deecure_core::{Event, EventKind, Timestamp};
use serde_json::json;

use super::{Windows, diff, fnv1a};
use crate::store::{self, Store, WatchedFile};

/// Files bigger than this are compared by time and size only.
const MAX_HASHED: u64 = 8 * 1024 * 1024;
/// Text files up to this size are kept, so a later change can show a diff.
const MAX_KEPT: u64 = 64 * 1024;
/// The most diff text kept per event.
const MAX_DIFF: usize = 8 * 1024;
/// A scan stops after this many entries, so a strange folder cannot make it slow.
const MAX_ENTRIES: usize = 20_000;
/// At most this many events per scan (a release upgrade changes hundreds of files);
/// the rest are counted in one summary event.
const MAX_EVENTS: usize = 200;
/// On the very first scan, files changed in the last 30 days are shown as changes.
const FIRST_SCAN_DAYS_MS: i64 = 30 * 86_400_000;
const DAY_MS: i64 = 86_400_000;
/// Setting: when the last scan finished.
pub const SCANNED_AT: &str = "history.config_scanned_at";

/// Paths left out: caches and files programs rewrite all the time.
const SKIP: &[&str] = &[
    "/etc/ssl/certs",
    "/etc/ld.so.cache",
    "/etc/apparmor.d/cache",
    "/etc/mtab",
    "/etc/.pwd.lock",
    "/etc/cups/subscriptions.conf",
    "/etc/cups/subscriptions.conf.O",
    "/etc/cups/printers.conf.O",
];

/// One folder to watch: where it is on disk, and how its paths are shown.
#[derive(Debug, Clone)]
pub struct Root {
    /// The real folder, such as `/etc` (or a fixture copy in tests).
    pub real: PathBuf,
    /// How paths under it are shown, such as `/etc` or `~/.config/autostart`.
    pub shown: String,
}

/// What a scan saw of one file.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Seen {
    real: PathBuf,
    mtime: i64,
    size: i64,
    uid: u32,
    /// For a symbolic link: where it points.
    link: Option<String>,
}

#[derive(Debug, Default)]
struct Scan {
    files: BTreeMap<String, Seen>,
    /// Folders that could be listed, so files missing from them are really gone.
    listed: HashSet<String>,
    /// False when the entry limit was hit; then nothing is called removed.
    complete: bool,
}

fn ms(meta: &std::fs::Metadata) -> i64 {
    meta.mtime() * 1000 + meta.mtime_nsec() / 1_000_000
}

fn skipped(shown: &str) -> bool {
    SKIP.iter()
        .any(|s| shown == *s || shown.strip_prefix(s).is_some_and(|r| r.starts_with('/')))
}

fn scan(roots: &[Root]) -> Scan {
    let mut out = Scan {
        complete: true,
        ..Scan::default()
    };
    let mut entries = 0;
    for root in roots {
        let mut stack = vec![(root.real.clone(), root.shown.clone())];
        while let Some((dir, shown_dir)) = stack.pop() {
            let Ok(list) = std::fs::read_dir(&dir) else {
                continue;
            };
            out.listed.insert(shown_dir.clone());
            for entry in list.flatten() {
                entries += 1;
                if entries > MAX_ENTRIES {
                    out.complete = false;
                    return out;
                }
                let name = entry.file_name().to_string_lossy().into_owned();
                let shown = format!("{shown_dir}/{name}");
                if skipped(&shown) {
                    continue;
                }
                let path = entry.path();
                // Never follow links: a link is recorded as where it points.
                let Ok(meta) = path.symlink_metadata() else {
                    continue;
                };
                let kind = meta.file_type();
                if kind.is_dir() {
                    stack.push((path, shown));
                } else if kind.is_file() || kind.is_symlink() {
                    let link = kind.is_symlink().then(|| {
                        std::fs::read_link(&path)
                            .map(|t| t.to_string_lossy().into_owned())
                            .unwrap_or_default()
                    });
                    out.files.insert(
                        shown,
                        Seen {
                            real: path,
                            mtime: ms(&meta),
                            size: i64::try_from(meta.len()).unwrap_or(i64::MAX),
                            uid: meta.uid(),
                            link,
                        },
                    );
                }
            }
        }
    }
    out
}

/// Reads a file's fingerprint and, for small text, its text.
fn read(seen: &Seen) -> (Option<String>, Option<String>) {
    if let Some(target) = &seen.link {
        let text = format!("→ {target}\n");
        return (Some(fnv1a(text.as_bytes())), Some(text));
    }
    if seen.size.unsigned_abs() > MAX_HASHED {
        return (None, None);
    }
    let Ok(bytes) = std::fs::read(&seen.real) else {
        return (None, None);
    };
    let hash = Some(fnv1a(&bytes));
    let text = (bytes.len() as u64 <= MAX_KEPT && !bytes.contains(&0))
        .then(|| String::from_utf8(bytes).ok())
        .flatten();
    (hash, text)
}

/// When a change most likely happened. A file's own time is used when it is newer than
/// the last scan. Package managers keep the package's old file times, so then the
/// folder's time (which changes when a file is added, replaced, or removed) is used,
/// and if that is old too, the time of this scan.
fn when(own: Option<i64>, folder: Option<i64>, last_scan: i64, now: i64) -> i64 {
    let fresh = |t: i64| t > last_scan && t <= now + 60_000;
    match (own.filter(|t| fresh(*t)), folder.filter(|t| fresh(*t))) {
        (Some(t), _) | (None, Some(t)) => t.min(now),
        (None, None) => now,
    }
}

fn folder_time(path: &Path) -> Option<i64> {
    path.parent()
        .and_then(|p| p.symlink_metadata().ok())
        .map(|m| ms(&m))
}

/// The user's crontab, read with `crontab -l`, as a watched file.
#[derive(Debug, Clone)]
pub struct Crontab {
    /// Shown as `/var/spool/cron/crontabs/<user>`.
    pub shown: String,
    /// The text, or `None` when the user has no crontab.
    pub text: Option<String>,
    /// The spool folder's time, if it can be read (it changes when a crontab is saved).
    pub folder_mtime: Option<i64>,
    /// The user's ID, who owns it.
    pub uid: u32,
}

/// One change found by a scan.
struct Change {
    path: String,
    what: &'static str,
    at: i64,
    owner: Option<u32>,
    diff: Option<diff::Diff>,
    first_scan: bool,
}

/// Scans the folders, compares with the last scan, and saves the changes as events.
/// Runs on a blocking thread; the store is locked only to read the last scan and to save.
pub fn check(
    store: &store::SharedStore,
    roots: &[Root],
    crontab: Option<Crontab>,
    windows: &Windows,
    users: &HashMap<u32, String>,
    now: Timestamp,
) -> store::Result<usize> {
    let now = now.as_millis();
    let (baseline, last_scan) = {
        let s = store::lock(store);
        (s.watched_files()?, s.setting::<i64>(SCANNED_AT)?)
    };
    let first = last_scan.is_none();
    let last_scan = last_scan.unwrap_or(0);
    let found = scan(roots);

    let mut rows: Vec<(String, WatchedFile, Option<String>)> = Vec::new();
    let mut changes: Vec<Change> = Vec::new();
    let mut gone: Vec<String> = Vec::new();
    // Old texts are needed only for files that changed; read them in one go below.
    let mut need_old: Vec<String> = Vec::new();

    for (path, seen) in &found.files {
        let old = baseline.get(path);
        if old.is_some_and(|o| o.mtime == seen.mtime && o.size == seen.size) {
            continue;
        }
        let (hash, text) = read(seen);
        let row = WatchedFile {
            mtime: seen.mtime,
            size: seen.size,
            hash: hash.clone(),
        };
        match old {
            None if first => {
                if seen.mtime >= now - FIRST_SCAN_DAYS_MS {
                    changes.push(Change {
                        path: path.clone(),
                        what: "changed",
                        at: seen.mtime.min(now),
                        owner: Some(seen.uid),
                        diff: None,
                        first_scan: true,
                    });
                }
            }
            None => changes.push(Change {
                path: path.clone(),
                what: "added",
                at: when(Some(seen.mtime), folder_time(&seen.real), last_scan, now),
                owner: Some(seen.uid),
                diff: text.as_deref().map(|t| diff::lines("", t, MAX_DIFF)),
                first_scan: false,
            }),
            Some(o) if o.hash.is_some() && o.hash == hash => {} // touched only
            Some(_) => {
                need_old.push(path.clone());
                changes.push(Change {
                    path: path.clone(),
                    what: "changed",
                    at: when(Some(seen.mtime), folder_time(&seen.real), last_scan, now),
                    owner: Some(seen.uid),
                    diff: None, // filled in below, from the text saved last time
                    first_scan: false,
                });
            }
        }
        rows.push((path.clone(), row, text));
    }

    if found.complete {
        for path in baseline.keys() {
            if found.files.contains_key(path) || path.starts_with("/var/spool/cron/") {
                continue;
            }
            let parent = path.rsplit_once('/').map_or("", |(p, _)| p);
            // Only when the folder was listed (or is gone too) is the file really gone.
            let real_parent = roots
                .iter()
                .find_map(|r| {
                    parent
                        .strip_prefix(&r.shown)
                        .map(|rest| r.real.join(rest.trim_start_matches('/')))
                })
                .unwrap_or_else(|| PathBuf::from(parent));
            let parent_gone = matches!(real_parent.symlink_metadata(), Err(e) if e.kind() == std::io::ErrorKind::NotFound);
            if found.listed.contains(parent) || parent_gone {
                let folder = real_parent.symlink_metadata().ok().map(|m| ms(&m));
                changes.push(Change {
                    path: path.clone(),
                    what: "removed",
                    at: when(None, folder, last_scan, now),
                    owner: None,
                    diff: None,
                    first_scan: false,
                });
                gone.push(path.clone());
            }
        }
    }

    if let Some(cron) = crontab {
        let old = baseline.get(&cron.shown);
        let hash = cron.text.as_deref().map(|t| fnv1a(t.as_bytes()));
        let at = when(None, cron.folder_mtime, last_scan, now);
        match (old, &cron.text) {
            (None, Some(text)) => {
                if !first {
                    changes.push(Change {
                        path: cron.shown.clone(),
                        what: "added",
                        at,
                        owner: Some(cron.uid),
                        diff: Some(diff::lines("", text, MAX_DIFF)),
                        first_scan: false,
                    });
                }
                let size = i64::try_from(text.len()).unwrap_or(i64::MAX);
                let row = WatchedFile {
                    mtime: 0,
                    size,
                    hash,
                };
                rows.push((cron.shown.clone(), row, Some(text.clone())));
            }
            (Some(o), Some(text)) if o.hash != hash => {
                need_old.push(cron.shown.clone());
                changes.push(Change {
                    path: cron.shown.clone(),
                    what: "changed",
                    at,
                    owner: Some(cron.uid),
                    diff: None,
                    first_scan: false,
                });
                let size = i64::try_from(text.len()).unwrap_or(i64::MAX);
                let row = WatchedFile {
                    mtime: 0,
                    size,
                    hash,
                };
                rows.push((cron.shown.clone(), row, Some(text.clone())));
            }
            (Some(_), None) => {
                changes.push(Change {
                    path: cron.shown.clone(),
                    what: "removed",
                    at,
                    owner: Some(cron.uid),
                    diff: None,
                    first_scan: false,
                });
                gone.push(cron.shown.clone());
            }
            _ => {}
        }
    }

    let store = store::lock(store);
    store.batch(|s| {
        // Diffs of changed files need the text saved by the last scan.
        let new_text: HashMap<&str, Option<&str>> = rows
            .iter()
            .map(|(p, _, t)| (p.as_str(), t.as_deref()))
            .collect();
        for path in &need_old {
            let old = s.watched_content(path)?;
            if let (Some(old), Some(Some(new))) = (old, new_text.get(path.as_str()))
                && let Some(change) = changes
                    .iter_mut()
                    .find(|c| &c.path == path && c.what == "changed")
            {
                change.diff = Some(diff::lines(&old, new, MAX_DIFF));
            } else if let Some(change) = changes
                .iter_mut()
                .find(|c| &c.path == path && c.what == "changed")
            {
                change.diff = None;
            }
        }
        changes.sort_by_key(|c| std::cmp::Reverse(c.at));
        let extra = changes.len().saturating_sub(MAX_EVENTS);
        for c in changes.iter().take(MAX_EVENTS) {
            save(s, c, windows, users)?;
        }
        if extra > 0 {
            s.add_event(
                &format!("config:more:{now}"),
                &Event {
                    id: None,
                    kind: EventKind::ConfigChanged,
                    at: Timestamp(now),
                    title: format!("{extra} more configuration files changed"),
                    detail: None,
                    subject: None,
                    source: windows.who(now).unwrap_or("system").to_owned(),
                    data: Some(json!({ "change": "changed", "count": extra })),
                },
            )?;
        }
        for (path, row, text) in &rows {
            s.put_watched(path, row, text.as_deref())?;
        }
        for path in &gone {
            s.forget_watched(path)?;
        }
        s.set_setting(SCANNED_AT, &now)?;
        Ok(changes.len())
    })
}

/// Saves one change. Changes to the same file on the same day become one event that
/// counts them, so a file a program rewrites all day cannot flood the timeline.
fn save(
    s: &Store,
    c: &Change,
    windows: &Windows,
    users: &HashMap<u32, String>,
) -> store::Result<()> {
    let key = format!("config:{}:{}", c.path, c.at.div_euclid(DAY_MS));
    let count = s
        .event_by_key(&key)?
        .and_then(|e| e.data?.get("count")?.as_u64())
        .unwrap_or(0)
        + 1;
    let verb = match c.what {
        "added" => "Added",
        "removed" => "Removed",
        _ => "Changed",
    };
    let times = if count > 1 {
        format!(" ({count} times today)")
    } else {
        String::new()
    };
    let who = windows
        .who(c.at)
        .map(str::to_owned)
        .or_else(|| {
            c.owner
                .map(|u| users.get(&u).cloned().unwrap_or_else(|| u.to_string()))
        })
        .unwrap_or_else(|| "unknown".to_owned());
    let mut data = json!({ "change": c.what, "count": count });
    if let Some(d) = &c.diff {
        data["diff"] = json!(d.text);
        data["added"] = json!(d.added);
        data["removed"] = json!(d.removed);
    }
    if c.first_scan {
        data["first_scan"] = json!(true);
    }
    s.put_event(
        &key,
        &Event {
            id: None,
            kind: EventKind::ConfigChanged,
            at: Timestamp(c.at),
            title: format!("{verb} {}{times}", c.path),
            detail: None,
            subject: Some(c.path.clone()),
            source: who,
            data: Some(data),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn temp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("deecure-config-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("etc/apt/sources.list.d")).unwrap();
        dir
    }

    fn set_mtime(path: &Path, secs: i64) {
        // touch -d is the simplest way to set a time without unsafe code.
        let status = std::process::Command::new("touch")
            .args(["-h", "-d", &format!("@{secs}")])
            .arg(path)
            .status()
            .unwrap();
        assert!(status.success());
    }

    fn events(store: &store::SharedStore) -> Vec<Event> {
        store::lock(store).events_since(Timestamp(0), 100).unwrap()
    }

    #[test]
    fn first_scan_is_quiet_then_changes_show_with_diffs() {
        let dir = temp("flow");
        let etc = dir.join("etc");
        std::fs::write(etc.join("hosts"), "127.0.0.1 localhost\n").unwrap();
        std::fs::write(etc.join("old.conf"), "x\n").unwrap();
        set_mtime(&etc.join("old.conf"), 1_000_000); // 1970: not "recent" on the first scan
        let roots = [Root {
            real: etc.clone(),
            shown: "/etc".into(),
        }];
        let store: store::SharedStore = Arc::new(Mutex::new(Store::open_in_memory().unwrap()));
        let users = HashMap::new();
        let windows = Windows::default();
        let now = Timestamp::now();

        check(&store, &roots, None, &windows, &users, now).unwrap();
        let first = events(&store);
        assert_eq!(first.len(), 1, "only the recently changed file: {first:?}");
        assert_eq!(first[0].data.as_ref().unwrap()["first_scan"], true);

        // Change one file, add one, remove one, touch one.
        std::fs::write(etc.join("hosts"), "127.0.0.1 localhost\n10.0.0.5 nas\n").unwrap();
        std::fs::write(
            etc.join("apt/sources.list.d/vscode.list"),
            "deb https://example/ stable main\n",
        )
        .unwrap();
        std::fs::remove_file(etc.join("old.conf")).unwrap();
        let later = Timestamp(now.as_millis() + 5 * 60_000);
        check(&store, &roots, None, &windows, &users, later).unwrap();
        let all = events(&store);
        let find = |p: &str| {
            all.iter()
                .find(|e| e.subject.as_deref() == Some(p))
                .unwrap()
        };
        let hosts = find("/etc/hosts");
        assert_eq!(hosts.data.as_ref().unwrap()["change"], "changed");
        assert_eq!(
            hosts.data.as_ref().unwrap()["diff"],
            "  127.0.0.1 localhost\n+ 10.0.0.5 nas\n"
        );
        assert!(
            hosts.title.contains("2 times today") || hosts.title.starts_with("Changed /etc/hosts")
        );
        let added = find("/etc/apt/sources.list.d/vscode.list");
        assert_eq!(added.title, "Added /etc/apt/sources.list.d/vscode.list");
        assert_eq!(
            find("/etc/old.conf").data.as_ref().unwrap()["change"],
            "removed"
        );

        // Touching without changing the content gives nothing new.
        let count = all.len();
        set_mtime(
            &etc.join("apt/sources.list.d/vscode.list"),
            now.as_millis() / 1000 + 600,
        );
        check(
            &store,
            &roots,
            None,
            &windows,
            &users,
            Timestamp(later.as_millis() + 60_000),
        )
        .unwrap();
        assert_eq!(events(&store).len(), count);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_unlistable_folder_does_not_look_removed() {
        let dir = temp("unlisted");
        let etc = dir.join("etc");
        std::fs::write(etc.join("apt/sources.list.d/a.list"), "deb x\n").unwrap();
        let roots = [Root {
            real: etc.clone(),
            shown: "/etc".into(),
        }];
        let store: store::SharedStore = Arc::new(Mutex::new(Store::open_in_memory().unwrap()));
        let (users, windows) = (HashMap::new(), Windows::default());
        check(&store, &roots, None, &windows, &users, Timestamp::now()).unwrap();
        let before = events(&store).len();
        // The folder still exists but cannot be listed now.
        let folder = etc.join("apt/sources.list.d");
        std::process::Command::new("chmod")
            .arg("000")
            .arg(&folder)
            .status()
            .unwrap();
        let listable = std::fs::read_dir(&folder).is_ok(); // true when tests run as root
        check(
            &store,
            &roots,
            None,
            &windows,
            &users,
            Timestamp(Timestamp::now().as_millis() + 1),
        )
        .unwrap();
        std::process::Command::new("chmod")
            .arg("755")
            .arg(&folder)
            .status()
            .unwrap();
        if !listable {
            assert_eq!(events(&store).len(), before, "nothing is called removed");
            assert!(
                store::lock(&store)
                    .watched_files()
                    .unwrap()
                    .contains_key("/etc/apt/sources.list.d/a.list")
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn package_runs_and_crontabs_are_named() {
        let dir = temp("who");
        let etc = dir.join("etc");
        let roots = [Root {
            real: etc.clone(),
            shown: "/etc".into(),
        }];
        let store: store::SharedStore = Arc::new(Mutex::new(Store::open_in_memory().unwrap()));
        let users = HashMap::new();
        let now = Timestamp::now().as_millis();
        let windows = Windows(vec![(now - 120_000, now + 600_000, "apt".into())]);
        let cron = |text: Option<&str>| Crontab {
            shown: "/var/spool/cron/crontabs/riya".into(),
            text: text.map(str::to_owned),
            folder_mtime: None,
            uid: 1000,
        };
        check(
            &store,
            &roots,
            Some(cron(Some("0 9 * * * true\n"))),
            &windows,
            &users,
            Timestamp(now),
        )
        .unwrap();
        assert!(
            events(&store).is_empty(),
            "the first crontab is the starting point"
        );
        std::fs::write(etc.join("new.conf"), "a\n").unwrap();
        check(
            &store,
            &roots,
            Some(cron(Some("0 9 * * * true\n0 21 * * * backup\n"))),
            &windows,
            &users,
            Timestamp(now + 60_000),
        )
        .unwrap();
        let all = events(&store);
        let new = all
            .iter()
            .find(|e| e.subject.as_deref() == Some("/etc/new.conf"))
            .unwrap();
        assert_eq!(new.source, "apt", "changed during an apt run");
        let tab = all
            .iter()
            .find(|e| e.subject.as_deref() == Some("/var/spool/cron/crontabs/riya"))
            .unwrap();
        assert_eq!(
            tab.data.as_ref().unwrap()["diff"],
            "  0 9 * * * true\n+ 0 21 * * * backup\n"
        );
        check(
            &store,
            &roots,
            Some(cron(None)),
            &Windows::default(),
            &users,
            Timestamp(now + 120_000),
        )
        .unwrap();
        // Changes to one file on one day are one event; the newest says what happened last.
        let tab = events(&store);
        let tab = tab
            .iter()
            .find(|e| e.subject.as_deref() == Some("/var/spool/cron/crontabs/riya"))
            .unwrap();
        assert_eq!(
            tab.title,
            "Removed /var/spool/cron/crontabs/riya (2 times today)"
        );
        assert_eq!(tab.source, "1000");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn change_times_prefer_fresh_file_times() {
        assert_eq!(when(Some(150), Some(160), 100, 200), 150);
        assert_eq!(
            when(Some(50), Some(160), 100, 200),
            160,
            "old package time: use the folder"
        );
        assert_eq!(
            when(Some(50), Some(60), 100, 200),
            200,
            "both old: the scan time"
        );
        assert_eq!(
            when(Some(90_000), None, 100, 200),
            200,
            "far future: the scan time"
        );
        assert!(skipped("/etc/ssl/certs/abc.pem"));
        assert!(!skipped("/etc/ssl/certsabc"));
    }
}
