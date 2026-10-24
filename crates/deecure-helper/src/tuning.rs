// SPDX-License-Identifier: Apache-2.0

//! Settings files the helper writes for Optimize (docs/01-product/optimize-catalog.md), each one
//! file that only DeeCure writes, so undo removes or restores just that file:
//!
//! - `/etc/systemd/journald.conf.d/50-deecure-size.conf`: `SystemMaxUse=` caps the system log.
//!   journald reads it when it starts (the next restart); the log is shrunk to the cap now with
//!   `journalctl --vacuum-size`. journald is never restarted.
//! - `/etc/sysctl.d/60-deecure-inotify.conf`: `fs.inotify.max_user_watches`, set now with
//!   `sysctl -w` and kept for the next starts.

use std::path::{Path, PathBuf};

use deecure_core::fix::Call;

use crate::actions::Command;

pub const JOURNAL: &str = "etc/systemd/journald.conf.d/50-deecure-size.conf";
pub const WATCHES: &str = "etc/sysctl.d/60-deecure-inotify.conf";
const KEY: &str = "fs.inotify.max_user_watches";

/// One settings file to write (or remove: `text` is `None`), then the commands to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Planned {
    pub path: PathBuf,
    pub text: Option<String>,
    pub commands: Vec<Command>,
    pub undo: Call,
    /// Said after the result, such as when it fully applies.
    pub note: &'static str,
}

fn value_in(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let (k, v) = l.split_once('=')?;
        (k.trim() == key).then(|| v.trim().to_owned())
    })
}

/// Checks the call and plans the file and commands. `None` when the call is not one of these.
pub fn plan(call: &Call, root: &Path) -> Option<Result<Planned, String>> {
    Some(match call {
        Call::SetJournalCap { mb } => (|| {
            if mb.is_some_and(|m| !(100..=10_000).contains(&m)) {
                return Err("the cap must be 100-10000 MB".into());
            }
            let path = root.join(JOURNAL);
            let before = std::fs::read_to_string(&path).ok();
            let old = before
                .as_deref()
                .and_then(|t| value_in(t, "SystemMaxUse"))
                .and_then(|v| v.trim_end_matches('M').parse().ok());
            Ok(Planned {
                text: mb.map(|m| {
                    format!(
                        "# Written by DeeCure (Optimize). Delete this file to remove the cap.\n[Journal]\nSystemMaxUse={m}M\n"
                    )
                }),
                commands: mb
                    .map(|m| Command {
                        program: "/usr/bin/journalctl",
                        args: vec![format!("--vacuum-size={m}M")],
                    })
                    .into_iter()
                    .collect(),
                undo: Call::SetJournalCap { mb: old },
                note: "; the cap applies to new logs after the next restart",
                path,
            })
        })(),
        Call::SetFileWatchLimit { value, keep } => (|| {
            if !(8192..=4_194_304).contains(value) {
                return Err("the limit must be 8192-4194304".into());
            }
            let path = root.join(WATCHES);
            let kept: Option<u64> = std::fs::read_to_string(&path)
                .ok()
                .and_then(|t| value_in(&t, KEY))
                .and_then(|v| v.parse().ok());
            let now: u64 =
                std::fs::read_to_string(root.join("proc/sys/fs/inotify/max_user_watches"))
                    .ok()
                    .and_then(|v| v.trim().parse().ok())
                    .ok_or("the file-watch limit is not readable")?;
            Ok(Planned {
                text: keep.then(|| {
                    format!("# Written by DeeCure (Optimize). Delete this file to remove it.\n{KEY} = {value}\n")
                }),
                commands: vec![Command {
                    program: "/usr/sbin/sysctl",
                    args: vec!["-w".into(), format!("{KEY}={value}")],
                }],
                undo: match kept {
                    Some(k) => Call::SetFileWatchLimit {
                        value: k,
                        keep: true,
                    },
                    None => Call::SetFileWatchLimit {
                        value: now,
                        keep: false,
                    },
                },
                note: "",
                path,
            })
        })(),
        _ => return None,
    })
}

/// Writes the file in one step, or removes it.
pub fn write(path: &Path, text: Option<&str>) -> Result<(), String> {
    match text {
        Some(t) => {
            let dir = path.parent().ok_or("no folder")?;
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("could not create {}: {e}", dir.display()))?;
            let tmp = path.with_extension("deecure-tmp");
            std::fs::write(&tmp, t)
                .and_then(|()| std::fs::rename(&tmp, path))
                .map_err(|e| format!("could not write {}: {e}", path.display()))
        }
        None => match std::fs::remove_file(path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                Err(format!("could not remove {}: {e}", path.display()))
            }
            _ => Ok(()),
        },
    }
}

/// The snap version count to put back: the value set before, or the most versions any snap
/// keeps now (`snap list --all`), which is what snapd keeps when nothing is set.
pub fn snap_retain_before(set: Option<&str>, list_all: &str) -> u32 {
    if let Some(n) = set.and_then(|s| s.trim().parse().ok()) {
        return n;
    }
    let mut count = std::collections::HashMap::new();
    for l in list_all.lines().skip(1) {
        if let Some(name) = l.split_whitespace().next() {
            *count.entry(name).or_insert(0u32) += 1;
        }
    }
    count.into_values().max().unwrap_or(2).clamp(2, 20)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "deecure-helper-tuning-{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("proc/sys/fs/inotify")).unwrap();
        std::fs::write(dir.join("proc/sys/fs/inotify/max_user_watches"), "65536\n").unwrap();
        dir
    }

    #[test]
    fn the_journal_cap_is_one_file_and_undo_puts_back_what_was_there() {
        let r = root("journal");
        let p = plan(&Call::SetJournalCap { mb: Some(500) }, &r)
            .unwrap()
            .unwrap();
        assert!(
            p.text
                .as_deref()
                .unwrap()
                .contains("[Journal]\nSystemMaxUse=500M\n")
        );
        assert_eq!(p.commands[0].args, ["--vacuum-size=500M"]);
        assert_eq!(p.undo, Call::SetJournalCap { mb: None });
        write(&p.path, p.text.as_deref()).unwrap();
        let again = plan(&Call::SetJournalCap { mb: Some(800) }, &r)
            .unwrap()
            .unwrap();
        assert_eq!(again.undo, Call::SetJournalCap { mb: Some(500) });
        let off = plan(&Call::SetJournalCap { mb: None }, &r)
            .unwrap()
            .unwrap();
        assert!(off.text.is_none() && off.commands.is_empty());
        write(&off.path, None).unwrap();
        assert!(!r.join(JOURNAL).exists());
        assert!(
            plan(&Call::SetJournalCap { mb: Some(10) }, &r)
                .unwrap()
                .is_err()
        );
        assert!(plan(&Call::CleanAptCache, &r).is_none());
    }

    #[test]
    fn the_watch_limit_is_set_now_and_undo_returns_the_old_value() {
        let r = root("watches");
        let call = Call::SetFileWatchLimit {
            value: 524_288,
            keep: true,
        };
        let p = plan(&call, &r).unwrap().unwrap();
        assert_eq!(
            p.commands[0].args,
            ["-w", "fs.inotify.max_user_watches=524288"]
        );
        assert!(
            p.text
                .as_deref()
                .unwrap()
                .contains("fs.inotify.max_user_watches = 524288\n")
        );
        assert_eq!(
            p.undo,
            Call::SetFileWatchLimit {
                value: 65_536,
                keep: false
            }
        );
        write(&p.path, p.text.as_deref()).unwrap();
        let again = plan(
            &Call::SetFileWatchLimit {
                value: 1_048_576,
                keep: true,
            },
            &r,
        )
        .unwrap()
        .unwrap();
        assert_eq!(again.undo, call);
        for bad in [1, 5_000_000] {
            assert!(
                plan(
                    &Call::SetFileWatchLimit {
                        value: bad,
                        keep: true
                    },
                    &r
                )
                .unwrap()
                .is_err()
            );
        }
    }

    #[test]
    fn snap_undo_uses_the_set_value_or_what_snapd_keeps_now() {
        let list = "Name Version Rev Tracking Publisher Notes\nfirefox 1 10 latest m -\nfirefox 1 11 latest m disabled\nfirefox 1 12 latest m disabled\ncore22 1 5 latest c base\n";
        assert_eq!(snap_retain_before(Some("4\n"), list), 4);
        assert_eq!(snap_retain_before(None, list), 3);
        assert_eq!(snap_retain_before(None, ""), 2);
    }
}
