// SPDX-License-Identifier: Apache-2.0

//! Running processes from `/proc`.

use std::collections::HashMap;
use std::io;
use std::time::Instant;

use serde::Serialize;

use crate::collect::SystemRoot;

/// One running process.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Process {
    /// Process ID.
    pub pid: u32,
    /// Short name, such as `firefox`.
    pub name: String,
    /// The user it runs as.
    pub user: String,
    /// State letter: R running, S sleeping, D waiting for disk, Z zombie, …
    pub state: String,
    /// CPU use, in percent of one CPU, since the last reading.
    pub cpu_percent: f64,
    /// Memory in use (resident set), in bytes.
    pub memory_bytes: u64,
    /// Threads.
    pub threads: u32,
    /// The full command line, shortened to 200 characters.
    pub command: String,
}

/// Fields read from `/proc/<pid>/stat`.
struct Stat {
    state: String,
    ticks: u64,
    start_ticks: u64,
}

fn parse_stat(text: &str) -> Option<Stat> {
    // The name is in brackets and may hold spaces, so split after the last ')'.
    let rest = text.get(text.rfind(')')? + 1..)?;
    let fields: Vec<&str> = rest.split_whitespace().collect();
    let num = |i: usize| fields.get(i).and_then(|f| f.parse::<u64>().ok());
    Some(Stat {
        state: (*fields.first()?).to_owned(),
        ticks: num(11)? + num(12)?,
        start_ticks: num(19)?,
    })
}

/// Reads processes and works out CPU use since the previous reading.
#[derive(Debug, Default)]
pub struct ProcessSampler {
    previous: Option<(Instant, HashMap<u32, u64>)>,
}

impl ProcessSampler {
    /// All processes, busiest first.
    pub fn read(&mut self, root: &SystemRoot) -> io::Result<Vec<Process>> {
        let hz = rustix::param::clock_ticks_per_second() as f64;
        let uptime: f64 = root
            .read("/proc/uptime")?
            .split_whitespace()
            .next()
            .and_then(|u| u.parse().ok())
            .unwrap_or(0.0);
        let users = user_names(root);
        let now = Instant::now();
        let mut ticks_now = HashMap::new();
        let mut processes = Vec::new();

        for entry in std::fs::read_dir(root.path("/proc"))?.flatten() {
            let Some(pid) = entry
                .file_name()
                .to_str()
                .and_then(|n| n.parse::<u32>().ok())
            else {
                continue;
            };
            let dir = entry.path();
            // A process can end while we read it; skip it then.
            let Some(stat) = std::fs::read_to_string(dir.join("stat"))
                .ok()
                .as_deref()
                .and_then(parse_stat)
            else {
                continue;
            };
            let status = std::fs::read_to_string(dir.join("status")).unwrap_or_default();
            let field = |key: &str| {
                status
                    .lines()
                    .find_map(|l| l.strip_prefix(key))
                    .map(str::trim)
            };
            let uid = field("Uid:")
                .and_then(|v| v.split_whitespace().next())
                .and_then(|v| v.parse::<u32>().ok());
            let command = std::fs::read(dir.join("cmdline"))
                .map(|raw| {
                    raw.split(|b| *b == 0)
                        .filter(|p| !p.is_empty())
                        .map(String::from_utf8_lossy)
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default();

            let cpu_percent = match &self.previous {
                Some((before, old)) => {
                    let seconds = now.duration_since(*before).as_secs_f64().max(0.001);
                    let old_ticks = old.get(&pid).copied().unwrap_or(stat.ticks);
                    stat.ticks.saturating_sub(old_ticks) as f64 / hz / seconds * 100.0
                }
                None => {
                    // First reading: the average since the process started.
                    let age = (uptime - stat.start_ticks as f64 / hz).max(1.0);
                    stat.ticks as f64 / hz / age * 100.0
                }
            };
            ticks_now.insert(pid, stat.ticks);
            processes.push(Process {
                pid,
                name: full_name(field("Name:").unwrap_or("?"), &command),
                user: uid
                    .map(|u| users.get(&u).cloned().unwrap_or_else(|| u.to_string()))
                    .unwrap_or_default(),
                state: stat.state,
                cpu_percent: (cpu_percent * 10.0).round() / 10.0,
                memory_bytes: field("VmRSS:")
                    .and_then(|v| v.trim_end_matches("kB").trim().parse::<u64>().ok())
                    .map_or(0, |kb| kb * 1024),
                threads: field("Threads:").and_then(|v| v.parse().ok()).unwrap_or(1),
                command: command.chars().take(200).collect(),
            });
        }
        self.previous = Some((now, ticks_now));
        processes.sort_by(|a, b| {
            b.cpu_percent
                .total_cmp(&a.cpu_percent)
                .then(b.memory_bytes.cmp(&a.memory_bytes))
        });
        Ok(processes)
    }
}

/// Linux cuts names at 15 characters. Use the program file name when it continues the cut name.
fn full_name(short: &str, command: &str) -> String {
    let program = command.split(' ').next().unwrap_or("");
    let program = program.rsplit('/').next().unwrap_or(program);
    if short.len() == 15 && program.len() > 15 && program.starts_with(short) {
        program.to_owned()
    } else {
        short.to_owned()
    }
}

/// User IDs to names, from `/etc/passwd`.
fn user_names(root: &SystemRoot) -> HashMap<u32, String> {
    root.read("/etc/passwd")
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let mut parts = line.split(':');
            let name = parts.next()?;
            let uid = parts.nth(1)?.parse().ok()?;
            Some((uid, name.to_owned()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    const STAT: &str = "4242 (Web Content (x)) S 1 4242 4242 0 -1 4194560 100 0 0 0 600 150 0 0 20 0 42 0 1000 0 0";

    #[test]
    fn parses_names_with_spaces_and_brackets() {
        let stat = parse_stat(STAT).unwrap();
        assert_eq!(stat.state, "S");
        assert_eq!(stat.ticks, 750);
        assert_eq!(stat.start_ticks, 1000);
    }

    #[test]
    fn long_names_come_from_the_command_line() {
        let cut = "WebKitWebProces";
        assert_eq!(
            full_name(cut, "/usr/lib/WebKitWebProcess 4 24"),
            "WebKitWebProcess"
        );
        assert_eq!(full_name(cut, ""), cut);
        assert_eq!(full_name("bash", "/bin/bash --login"), "bash");
    }

    #[test]
    fn reads_processes_from_proc() {
        let root = fixture_root(
            "processes",
            &[
                ("/proc/uptime", "110.00 50.00\n"),
                (
                    "/etc/passwd",
                    "root:x:0:0:root:/root:/bin/bash\nriya:x:1000:1000:Riya:/home/riya:/bin/bash\n",
                ),
                ("/proc/4242/stat", STAT),
                (
                    "/proc/4242/status",
                    "Name:\tWeb Content\nUid:\t1000\t1000\t1000\t1000\nVmRSS:\t  204800 kB\nThreads:\t42\n",
                ),
                (
                    "/proc/4242/cmdline",
                    "/usr/lib/firefox/firefox\0-contentproc\0",
                ),
                ("/proc/self/stat", "not a process"),
            ],
        );
        let processes = ProcessSampler::default().read(&root).unwrap();
        assert_eq!(processes.len(), 1);
        let p = &processes[0];
        assert_eq!(
            (p.pid, p.name.as_str(), p.user.as_str()),
            (4242, "Web Content", "riya")
        );
        assert_eq!(p.memory_bytes, 204_800 * 1024);
        assert_eq!(p.threads, 42);
        assert_eq!(p.command, "/usr/lib/firefox/firefox -contentproc");
        assert!(p.cpu_percent > 0.0);
    }
}
