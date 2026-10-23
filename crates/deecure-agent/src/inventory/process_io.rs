// SPDX-License-Identifier: Apache-2.0

//! Disk use per process for the Expert level (docs/01-product/expert-coverage.md, section 3),
//! from `/proc/<pid>/io`. Only the user's own processes can be read without admin rights; the
//! answer says how many others there were. Totals since each process started, with the reading
//! time, so the app works out rates.

use serde::Serialize;

use crate::collect::SystemRoot;

/// One process's disk use since it started.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProcessIo {
    pub pid: u32,
    pub name: String,
    /// Bytes really read from and written to storage (not served from the cache).
    pub read_bytes: u64,
    pub write_bytes: u64,
    /// Written data that was thrown away before it reached the disk.
    pub cancelled_write_bytes: u64,
}

/// What `process_io` answers.
#[derive(Debug, Default, Serialize)]
pub struct ProcessesIo {
    /// When this was read, in ms since 1970.
    pub at: u64,
    /// Processes that did any disk I/O.
    pub processes: Vec<ProcessIo>,
    /// Processes of other users, which need admin rights to read.
    pub unreadable: usize,
}

/// The fields of `/proc/<pid>/io` that count storage, not cache.
pub fn parse_io(text: &str) -> Option<(u64, u64, u64)> {
    let field = |key: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(key)?.trim().parse::<u64>().ok())
    };
    Some((
        field("read_bytes:")?,
        field("write_bytes:")?,
        field("cancelled_write_bytes:").unwrap_or(0),
    ))
}

/// Reads every process the user may read.
pub fn read(root: &SystemRoot) -> ProcessesIo {
    let pids: Vec<u32> = std::fs::read_dir(root.path("/proc"))
        .map(|d| {
            d.flatten()
                .filter_map(|e| e.file_name().to_str()?.parse().ok())
                .collect()
        })
        .unwrap_or_default();
    let mut out = ProcessesIo::default();
    for pid in pids {
        let Ok(stat) = root.read(&format!("/proc/{pid}/stat")) else {
            continue;
        };
        let Some(name) = stat
            .find('(')
            .zip(stat.rfind(')'))
            .and_then(|(a, b)| stat.get(a + 1..b))
        else {
            continue;
        };
        match root.read(&format!("/proc/{pid}/io")) {
            Ok(text) => {
                if let Some((read_bytes, write_bytes, cancelled_write_bytes)) = parse_io(&text)
                    && (read_bytes > 0 || write_bytes > 0)
                {
                    out.processes.push(ProcessIo {
                        pid,
                        name: name.to_owned(),
                        read_bytes,
                        write_bytes,
                        cancelled_write_bytes,
                    });
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => out.unreadable += 1,
            Err(_) => {}
        }
    }
    out.processes
        .sort_by_key(|p| std::cmp::Reverse(p.read_bytes + p.write_bytes));
    out.at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn reads_storage_bytes_not_cache() {
        assert_eq!(
            parse_io(
                "rchar: 99\nwchar: 9\nread_bytes: 4096\nwrite_bytes: 8192\ncancelled_write_bytes: 0\n"
            ),
            Some((4096, 8192, 0))
        );
        assert_eq!(parse_io("rchar: 1\n"), None);
    }

    #[test]
    fn lists_processes_that_used_the_disk() {
        let root = fixture_root(
            "process-io",
            &[
                ("/proc/5/stat", "5 (code) S 1\n"),
                (
                    "/proc/5/io",
                    "read_bytes: 100\nwrite_bytes: 900\ncancelled_write_bytes: 4\n",
                ),
                ("/proc/6/stat", "6 (idle) S 1\n"),
                ("/proc/6/io", "read_bytes: 0\nwrite_bytes: 0\n"),
            ],
        );
        let r = read(&root);
        assert_eq!(r.processes.len(), 1);
        assert_eq!(
            (r.processes[0].name.as_str(), r.processes[0].write_bytes),
            ("code", 900)
        );
        assert_eq!(r.unreadable, 0);
    }
}
