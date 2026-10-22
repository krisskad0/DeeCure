// SPDX-License-Identifier: Apache-2.0

//! Folder sizes, like `du`: the biggest folders in home and on the system disk, and the
//! biggest files in home. A scan reads many files, so it runs only when the app asks,
//! in the background, and its result is kept for an hour.

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, UNIX_EPOCH};

use serde::Serialize;

/// How long a scan result stays fresh.
pub const FRESH: Duration = Duration::from_secs(3600);
/// Files smaller than this are not listed as "largest files".
const BIG_FILE: u64 = 50 * 1024 * 1024;
const TOP_FILES: usize = 25;
/// A scan stops after this many entries, so a huge disk cannot keep it busy for long.
const MAX_ENTRIES: u64 = 3_000_000;

/// One folder or file and its size.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Sized {
    pub path: String,
    /// Allocated bytes, like `du`.
    pub size: u64,
    /// Last change, in milliseconds since 1970 (files only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<i64>,
}

/// A finished scan.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct UsageScan {
    /// When it finished, in milliseconds since 1970.
    pub scanned_at: i64,
    /// How long it took.
    pub seconds: f64,
    /// Top-level folders of home, biggest first.
    pub home: Vec<Sized>,
    /// Top-level folders of the system disk (`/`, same filesystem only), biggest first.
    pub system: Vec<Sized>,
    /// The biggest files in home.
    pub files: Vec<Sized>,
    /// Folders that could not be read (other users' or root's).
    pub unreadable: u64,
    /// True when the scan stopped early at its entry limit.
    pub partial: bool,
}

struct Walk {
    entries: u64,
    unreadable: u64,
    files: BinaryHeap<Reverse<(u64, PathBuf, i64)>>,
    track_files: bool,
    device: Option<u64>,
}

impl Walk {
    fn size(&mut self, path: &Path) -> u64 {
        let mut total = 0;
        let mut stack = vec![path.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                self.unreadable += 1;
                continue;
            };
            for e in entries.flatten() {
                self.entries += 1;
                if self.entries > MAX_ENTRIES {
                    return total;
                }
                let Ok(meta) = e.path().symlink_metadata() else {
                    continue;
                };
                // Stay on one filesystem, like `du -x`.
                if self.device.is_some_and(|d| d != meta.dev()) {
                    continue;
                }
                total += meta.blocks() * 512;
                if meta.is_dir() {
                    stack.push(e.path());
                } else if self.track_files && meta.is_file() && meta.len() >= BIG_FILE {
                    let modified = meta
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map_or(0, |d| d.as_millis() as i64);
                    self.files.push(Reverse((meta.len(), e.path(), modified)));
                    if self.files.len() > TOP_FILES {
                        self.files.pop();
                    }
                }
            }
        }
        total
    }

    fn children(&mut self, dir: &Path, skip: &[&str]) -> Vec<Sized> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        let mut out: Vec<Sized> = entries
            .flatten()
            .filter(|e| !skip.contains(&e.file_name().to_string_lossy().as_ref()))
            .filter_map(|e| {
                let meta = e.path().symlink_metadata().ok()?;
                if self.device.is_some_and(|d| d != meta.dev()) {
                    return None;
                }
                let size = if meta.is_dir() {
                    self.size(&e.path())
                } else {
                    meta.blocks() * 512
                };
                Some(Sized {
                    path: e.path().to_string_lossy().into_owned(),
                    size,
                    modified: None,
                })
            })
            .filter(|s| s.size > 0)
            .collect();
        out.sort_by_key(|a| std::cmp::Reverse(a.size));
        out.truncate(15);
        out
    }
}

/// Scans home and the system disk. Blocking: run it on a blocking thread.
pub fn scan(home: Option<&Path>) -> UsageScan {
    let started = Instant::now();
    let mut walk = Walk {
        entries: 0,
        unreadable: 0,
        files: BinaryHeap::new(),
        track_files: true,
        device: None,
    };
    let home_list = home.map(|h| walk.children(h, &[])).unwrap_or_default();
    walk.track_files = false;
    walk.device = std::fs::metadata("/").ok().map(|m| m.dev());
    // Home is shown on its own; virtual filesystems are skipped by the device check.
    let system = walk.children(
        Path::new("/"),
        &["home", "proc", "sys", "dev", "run", "tmp", "lost+found"],
    );
    let mut files: Vec<Sized> = walk
        .files
        .into_sorted_vec()
        .into_iter()
        .map(|Reverse((size, path, modified))| Sized {
            path: path.to_string_lossy().into_owned(),
            size,
            modified: Some(modified),
        })
        .collect();
    files.sort_by_key(|a| std::cmp::Reverse(a.size));
    UsageScan {
        scanned_at: std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_millis() as i64),
        seconds: started.elapsed().as_secs_f64(),
        home: home_list,
        system,
        files,
        unreadable: walk.unreadable,
        partial: walk.entries > MAX_ENTRIES,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_folders_and_finds_big_files() {
        let dir = std::env::temp_dir().join(format!("deecure-usage-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Videos/2026")).unwrap();
        std::fs::create_dir_all(dir.join("Documents")).unwrap();
        std::fs::write(dir.join("Documents/a.txt"), vec![1u8; 10_000]).unwrap();
        // A sparse 60 MB file: big by length, so it is listed, but it uses little disk.
        let big = std::fs::File::create(dir.join("Videos/2026/film.mkv")).unwrap();
        big.set_len(60 * 1024 * 1024).unwrap();
        let mut walk = Walk {
            entries: 0,
            unreadable: 0,
            files: BinaryHeap::new(),
            track_files: true,
            device: None,
        };
        let list = walk.children(&dir, &[]);
        assert!(
            list.iter()
                .any(|s| s.path.ends_with("Documents") && s.size >= 10_000)
        );
        assert_eq!(walk.files.len(), 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
