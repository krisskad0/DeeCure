// SPDX-License-Identifier: Apache-2.0

//! Storage details: disk encryption, `/etc/fstab` checks, the root reserve, and what
//! could be cleaned. Nothing here deletes anything; cleaning comes with the fix engine.

use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::Duration;

use serde::Serialize;

use super::command;
use crate::collect::SystemRoot;

/// One thing that could be cleaned, with what cleaning it means.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Candidate {
    /// The fix action that would clean it, such as `storage.empty_trash`.
    pub action: String,
    pub title: String,
    pub details: String,
    /// Bytes that cleaning would free.
    pub size: u64,
    /// `irreversible` (gone for good) or `regenerates` (comes back when needed).
    pub undo: String,
    /// True when cleaning needs the admin password.
    pub needs_password: bool,
}

/// One `/etc/fstab` problem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FstabProblem {
    pub line: usize,
    pub problem: String,
}

/// The `storage` inventory.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct StorageInfo {
    /// True when the root filesystem sits on LUKS (dm-crypt).
    pub encrypted: bool,
    /// The device chain under `/`, such as `["dm-1", "dm-0", "nvme0n1p3"]`.
    pub root_chain: Vec<String>,
    /// Bytes of `/` that only root may use.
    pub root_reserved: u64,
    /// `/etc/fstab` lines checked, and problems found.
    pub fstab_entries: usize,
    pub fstab_problems: Vec<FstabProblem>,
    pub candidates: Vec<Candidate>,
}

/// Follows a block device down through device-mapper layers, noting each name.
fn chain(root: &SystemRoot, name: &str, out: &mut Vec<String>, encrypted: &mut bool, depth: u8) {
    if depth > 8 || out.contains(&name.to_owned()) {
        return;
    }
    out.push(name.to_owned());
    let base = format!("/sys/class/block/{name}");
    if root
        .read(&format!("{base}/dm/uuid"))
        .is_ok_and(|u| u.starts_with("CRYPT-"))
    {
        *encrypted = true;
    }
    // A partition's parent disk is not a lower layer; only `slaves` are.
    if let Ok(dir) = std::fs::read_dir(root.path(&format!("{base}/slaves"))) {
        for e in dir.flatten() {
            chain(
                root,
                &e.file_name().to_string_lossy(),
                out,
                encrypted,
                depth + 1,
            );
        }
    }
}

fn root_device(root: &SystemRoot) -> Option<String> {
    let mounts = root.read("/proc/mounts").ok()?;
    let source = mounts.lines().find_map(|l| {
        let mut f = l.split_whitespace();
        let (src, at) = (f.next()?, f.next()?);
        (at == "/" && src.starts_with("/dev/")).then(|| src.to_owned())
    })?;
    // /dev/mapper/name is a link to /dev/dm-N; the kernel's name is the last part.
    let real = std::fs::canonicalize(root.path(&source)).ok()?;
    real.file_name().map(|n| n.to_string_lossy().into_owned())
}

/// Checks `/etc/fstab`: every line has its fields, and every device it names exists.
pub fn check_fstab(root: &SystemRoot, text: &str) -> (usize, Vec<FstabProblem>) {
    let mut entries = 0;
    let mut problems = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        entries += 1;
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 4 {
            problems.push(FstabProblem {
                line: i + 1,
                problem: "has fewer than 4 fields".into(),
            });
            continue;
        }
        let optional = f[3].split(',').any(|o| o == "nofail" || o == "noauto");
        let device = match f[0].split_once('=') {
            Some(("UUID", v)) => Some(format!("/dev/disk/by-uuid/{v}")),
            Some(("LABEL", v)) => Some(format!("/dev/disk/by-label/{v}")),
            Some(("PARTUUID", v)) => Some(format!("/dev/disk/by-partuuid/{v}")),
            None if f[0].starts_with("/dev/") => Some(f[0].to_owned()),
            _ => None, // tmpfs, network shares, swap files: nothing to look up
        };
        if let Some(path) = device
            && !optional
            && !root.path(&path).exists()
        {
            problems.push(FstabProblem {
                line: i + 1,
                problem: format!("{} is not connected", f[0]),
            });
        }
    }
    (entries, problems)
}

/// Allocated size of everything under `path` (like `du`), and the number of files.
pub fn tree_size(path: &Path) -> (u64, u64) {
    let mut stack = vec![path.to_path_buf()];
    let (mut bytes, mut files) = (0, 0);
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let Ok(meta) = e.path().symlink_metadata() else {
                continue;
            };
            bytes += meta.blocks() * 512;
            if meta.is_dir() {
                stack.push(e.path());
            } else {
                files += 1;
            }
        }
    }
    (bytes, files)
}

/// Reads `snap list --all` and returns `(name, revision)` of disabled revisions.
pub fn disabled_snaps(list: &str) -> Vec<(String, String)> {
    list.lines()
        .skip(1)
        .filter(|l| {
            l.split_whitespace()
                .last()
                .is_some_and(|notes| notes.split(',').any(|n| n == "disabled"))
        })
        .filter_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            Some(((*f.first()?).to_owned(), (*f.get(2)?).to_owned()))
        })
        .collect()
}

/// "4.0GB (79%)" or "157.2MB" (docker's decimal units) → bytes.
pub fn docker_size(text: &str) -> u64 {
    let number = text.split_whitespace().next().unwrap_or("");
    let split = number
        .find(|c: char| c.is_ascii_alphabetic())
        .unwrap_or(number.len());
    let value: f64 = number[..split].parse().unwrap_or(0.0);
    let unit = match &number[split..] {
        "kB" | "KB" => 1e3,
        "MB" => 1e6,
        "GB" => 1e9,
        "TB" => 1e12,
        _ => 1.0,
    };
    (value * unit) as u64
}

/// "Archived and active journals take up 525.7M in the file system." → bytes.
pub fn journal_size(text: &str) -> Option<u64> {
    let word = text
        .split_whitespace()
        .find(|w| w.ends_with(['K', 'M', 'G', 'T']) && w[..w.len() - 1].parse::<f64>().is_ok())?;
    let (num, unit) = word.split_at(word.len() - 1);
    let mult = match unit {
        "K" => 1024f64,
        "M" => 1024f64.powi(2),
        "G" => 1024f64.powi(3),
        _ => 1024f64.powi(4),
    };
    Some((num.parse::<f64>().ok()? * mult) as u64)
}

/// The journal size DeeCure suggests keeping.
const JOURNAL_KEEP: u64 = 200 * 1024 * 1024;

async fn candidates(root: &SystemRoot) -> Vec<Candidate> {
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    let mut out = Vec::new();
    let mut add = |action: &str,
                   title: &str,
                   details: String,
                   size: u64,
                   undo: &str,
                   needs_password: bool| {
        if size > 0 {
            out.push(Candidate {
                action: action.into(),
                title: title.into(),
                details,
                size,
                undo: undo.into(),
                needs_password,
            });
        }
    };
    if let Some(home) = &home {
        let (size, files) = tree_size(&home.join(".local/share/Trash/files"));
        add(
            "storage.empty_trash",
            "Empty Trash",
            format!("{files} files"),
            size,
            "irreversible",
            false,
        );
        let (size, files) = tree_size(&home.join(".cache/thumbnails"));
        add(
            "storage.clear_thumbnails",
            "Clear thumbnail cache",
            format!("{files} picture previews"),
            size,
            "regenerates",
            false,
        );
    }
    let debs: Vec<u64> = std::fs::read_dir(root.path("/var/cache/apt/archives"))
        .map(|d| {
            d.flatten()
                .filter(|e| e.path().extension().is_some_and(|x| x == "deb"))
                .filter_map(|e| e.metadata().ok())
                .map(|m| m.len())
                .collect()
        })
        .unwrap_or_default();
    add(
        "storage.clean_apt_cache",
        "Clean APT package cache",
        format!("{} downloaded .deb files", debs.len()),
        debs.iter().sum(),
        "regenerates",
        true,
    );
    if let Some(list) = command::output("snap", &["list", "--all"], Duration::from_secs(10)).await {
        let old = disabled_snaps(&list);
        let size: u64 = old
            .iter()
            .filter_map(|(n, r)| {
                std::fs::metadata(root.path(&format!("/var/lib/snapd/snaps/{n}_{r}.snap"))).ok()
            })
            .map(|m| m.len())
            .sum();
        add(
            "storage.remove_old_snap_revisions",
            "Remove disabled snap revisions",
            format!("{} old revisions kept after updates", old.len()),
            size,
            "irreversible",
            true,
        );
    }
    if let Some(text) =
        command::output("journalctl", &["--disk-usage"], Duration::from_secs(10)).await
        && let Some(size) = journal_size(&text)
    {
        add(
            "storage.vacuum_journal",
            "Shrink the system log",
            format!("Keep the newest {} MB of logs", JOURNAL_KEEP / 1024 / 1024),
            size.saturating_sub(JOURNAL_KEEP),
            "irreversible",
            true,
        );
    }
    // Docker is asked only when its socket exists (the daemon runs). Volumes are never
    // offered: they hold data. Unused images and build cache come back when needed.
    if root.path("/var/run/docker.sock").exists()
        && let Some(df) = command::output(
            "docker",
            &["system", "df", "--format", "{{json .}}"],
            Duration::from_secs(10),
        )
        .await
    {
        for line in df.lines() {
            let Ok(row) = serde_json::from_str::<serde_json::Value>(line) else {
                continue;
            };
            let reclaim = docker_size(
                row.get("Reclaimable")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
            );
            match row.get("Type").and_then(|v| v.as_str()) {
                Some("Images") => add(
                    "storage.prune_docker_images",
                    "Prune unused Docker images",
                    "Images no container uses".into(),
                    reclaim,
                    "regenerates",
                    false,
                ),
                Some("Build Cache") => add(
                    "storage.prune_docker_build_cache",
                    "Clear Docker build cache",
                    "Layers from earlier builds".into(),
                    reclaim,
                    "regenerates",
                    false,
                ),
                _ => {}
            }
        }
    }
    out.sort_by_key(|a| std::cmp::Reverse(a.size));
    out
}

/// Reads the storage inventory.
pub async fn read(root: &SystemRoot) -> StorageInfo {
    let mut chain_names = Vec::new();
    let mut encrypted = false;
    if let Some(dev) = root_device(root) {
        chain(root, &dev, &mut chain_names, &mut encrypted, 0);
    }
    let root_reserved = rustix::fs::statvfs(root.path("/"))
        .map(|s| {
            s.f_bfree
                .saturating_sub(s.f_bavail)
                .saturating_mul(s.f_frsize)
        })
        .unwrap_or(0);
    let (fstab_entries, fstab_problems) =
        check_fstab(root, &root.read("/etc/fstab").unwrap_or_default());
    StorageInfo {
        encrypted,
        root_chain: chain_names,
        root_reserved,
        fstab_entries,
        fstab_problems,
        candidates: candidates(root).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn parses_tool_outputs() {
        let snaps = "Name Version Rev Tracking Publisher Notes\ncore18 20260204 2999 latest/stable canonical** base,disabled\ncore18 20260901 3084 latest/stable canonical** base\nfirefox 1 10 latest/stable mozilla** disabled\n";
        assert_eq!(
            disabled_snaps(snaps),
            [
                ("core18".to_owned(), "2999".to_owned()),
                ("firefox".to_owned(), "10".to_owned())
            ]
        );
        assert_eq!(docker_size("4.0GB (79%)"), 4_000_000_000);
        assert_eq!(docker_size("157.2MB"), 157_200_000);
        assert_eq!(docker_size("0B"), 0);
        assert_eq!(
            journal_size("Archived and active journals take up 525.7M in the file system."),
            Some((525.7 * 1024.0 * 1024.0) as u64)
        );
        assert_eq!(journal_size("No journal files were found."), None);
    }

    #[test]
    fn checks_fstab_devices() {
        let root = fixture_root("fstab", &[("/dev/disk/by-uuid/1111", "")]);
        let fstab = "# comment\nUUID=1111 / ext4 defaults 0 1\nUUID=2222 /data ext4 defaults 0 2\nUUID=3333 /usb ext4 defaults,nofail 0 2\ntmpfs /tmp tmpfs defaults 0 0\nbroken\n";
        let (entries, problems) = check_fstab(&root, fstab);
        assert_eq!(entries, 5);
        assert_eq!(
            problems,
            [
                FstabProblem {
                    line: 3,
                    problem: "UUID=2222 is not connected".into()
                },
                FstabProblem {
                    line: 6,
                    problem: "has fewer than 4 fields".into()
                }
            ]
        );
    }

    #[test]
    fn finds_encryption_below_lvm() {
        let root = fixture_root(
            "luks",
            &[
                ("/sys/class/block/dm-1/dm/uuid", "LVM-abc\n"),
                ("/sys/class/block/dm-1/slaves/dm-0/x", ""),
                (
                    "/sys/class/block/dm-0/dm/uuid",
                    "CRYPT-LUKS2-abc-dm_crypt-0\n",
                ),
                ("/sys/class/block/dm-0/slaves/nvme0n1p3/x", ""),
            ],
        );
        let (mut names, mut enc) = (Vec::new(), false);
        chain(&root, "dm-1", &mut names, &mut enc, 0);
        assert!(enc);
        assert_eq!(names, ["dm-1", "dm-0", "nvme0n1p3"]);
    }
}
