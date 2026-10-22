// SPDX-License-Identifier: Apache-2.0

//! Logs from the systemd journal (`journalctl`, read-only): recent entries, grouped
//! errors, the kernel log, crashes, boots, and USB device events. Also apport crash
//! reports in `/var/crash`, and recent changes to files in `/etc`.

use std::collections::BTreeMap;
use std::time::{Duration, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;

use super::command;
use crate::collect::SystemRoot;

const LIMIT: Duration = Duration::from_secs(30);
/// How many entries a list returns at most.
const KEEP: usize = 500;

/// One journal entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    /// Milliseconds since 1970.
    pub at: i64,
    /// 0 emergency … 3 error, 4 warning, 5 notice, 6 info, 7 debug.
    pub priority: u8,
    /// The unit or program that wrote it.
    pub unit: String,
    pub message: String,
}

/// Reads `journalctl -o json` lines. Binary messages (byte arrays) are skipped.
pub fn parse_entries(lines: &str) -> Vec<Entry> {
    lines
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter_map(|e| {
            let text = |k: &str| e.get(k).and_then(Value::as_str);
            Some(Entry {
                at: text("__REALTIME_TIMESTAMP")?.parse::<i64>().ok()? / 1000,
                priority: text("PRIORITY").and_then(|p| p.parse().ok()).unwrap_or(6),
                unit: text("_SYSTEMD_UNIT")
                    .map(|u| u.trim_end_matches(".service").to_owned())
                    .filter(|u| !u.starts_with("session-") && !u.starts_with("user@"))
                    .or_else(|| text("SYSLOG_IDENTIFIER").map(str::to_owned))
                    .or_else(|| text("_COMM").map(str::to_owned))
                    .unwrap_or_else(|| "kernel".into()),
                message: text("MESSAGE")?.to_owned(),
            })
        })
        .collect()
}

async fn journal(args: &[&str]) -> Vec<Entry> {
    let mut all = vec![
        "-o",
        "json",
        "--no-pager",
        "-q",
        "--output-fields=PRIORITY,SYSLOG_IDENTIFIER,_SYSTEMD_UNIT,_COMM,MESSAGE",
    ];
    all.extend_from_slice(args);
    command::output("journalctl", &all, LIMIT)
        .await
        .map(|o| parse_entries(&o))
        .unwrap_or_default()
}

fn newest(mut entries: Vec<Entry>) -> Vec<Entry> {
    entries.reverse();
    entries.truncate(KEEP);
    entries
}

/// The `journal` inventory: entries of the last 24 hours, and how many came each hour.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct JournalInfo {
    /// 24 counts, oldest hour first; the last is the current hour.
    pub volume: Vec<u32>,
    /// Newest first.
    pub entries: Vec<Entry>,
    pub total: usize,
}

/// Counts entries per hour for the 24 hours up to `now`.
pub fn hourly(entries: &[Entry], now: i64) -> Vec<u32> {
    let mut v = vec![0; 24];
    for e in entries {
        let ago = (now - e.at) / 3_600_000;
        if (0..24).contains(&ago) {
            v[23 - ago as usize] += 1;
        }
    }
    v
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

pub async fn read_journal() -> JournalInfo {
    let entries = journal(&["--since", "-24h"]).await;
    JournalInfo {
        volume: hourly(&entries, now_ms()),
        total: entries.len(),
        entries: newest(entries),
    }
}

/// A group of the same error, with numbers and IDs taken out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ErrorGroup {
    /// The newest message of the group, as written.
    pub message: String,
    pub unit: String,
    pub count: usize,
    pub last_seen: i64,
    /// `attention` (worth a look), `harmless` (known noise), or `review`.
    pub class: String,
}

/// The `errors` inventory: errors (priority 3 or worse) of the last 7 days.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ErrorsInfo {
    pub total: usize,
    pub groups: Vec<ErrorGroup>,
    /// When each error happened, for the day × hour heatmap.
    pub times: Vec<i64>,
}

/// "usb 3-1: device 12 not accepting address 0x1f" → "usb #-#: device # not accepting address #".
pub fn pattern(message: &str) -> String {
    let mut out = String::with_capacity(message.len());
    let mut in_number = false;
    for c in message.chars() {
        if c.is_ascii_digit() || (in_number && (c.is_ascii_hexdigit() || c == 'x')) {
            if !in_number {
                out.push('#');
                in_number = true;
            }
        } else {
            in_number = false;
            out.push(c);
        }
    }
    out
}

/// Noise that would otherwise match an "attention" pattern, such as GNOME starting
/// helper apps that are not installed.
const NOISE: &[&str] = &["Application launched by gnome-session-binary"];
/// Known messages that look alarming but are normal on many computers.
const HARMLESS: &[&str] = &[
    "ACPI BIOS Error",
    "ACPI Error",
    "firmware bug",
    "Firmware Bug",
    "Direct firmware load",
    "bluetoothd",
    "gkr-pam",
    "Failed to load module",
    "tpm_crb",
    "spd5118",
    "ucsi_acpi",
    "pcieport",
    "AER: Corrected",
];
/// Messages that usually mean something needs fixing.
const ATTENTION: &[&str] = &[
    "segfault",
    "Out of memory",
    "oom-kill",
    "I/O error",
    "EXT4-fs error",
    "BTRFS error",
    "Failed to start",
    "code=dumped",
    "Hardware Error",
    "critical temperature",
    "Read-only file system",
];

pub fn classify(message: &str) -> &'static str {
    if NOISE.iter().any(|p| message.contains(p)) {
        "harmless"
    } else if ATTENTION.iter().any(|p| message.contains(p)) {
        "attention"
    } else if HARMLESS.iter().any(|p| message.contains(p)) {
        "harmless"
    } else {
        "review"
    }
}

pub fn group_errors(entries: &[Entry]) -> Vec<ErrorGroup> {
    let mut groups: BTreeMap<(String, String), ErrorGroup> = BTreeMap::new();
    for e in entries {
        let g = groups
            .entry((e.unit.clone(), pattern(&e.message)))
            .or_insert_with(|| ErrorGroup {
                message: e.message.clone(),
                unit: e.unit.clone(),
                count: 0,
                last_seen: e.at,
                class: classify(&e.message).into(),
            });
        g.count += 1;
        if e.at >= g.last_seen {
            g.last_seen = e.at;
            g.message = e.message.clone();
        }
    }
    let rank = |c: &str| match c {
        "attention" => 0,
        "review" => 1,
        _ => 2,
    };
    let mut out: Vec<ErrorGroup> = groups.into_values().collect();
    out.sort_by(|a, b| {
        rank(&a.class)
            .cmp(&rank(&b.class))
            .then(b.count.cmp(&a.count))
    });
    out
}

pub async fn read_errors() -> ErrorsInfo {
    let entries = journal(&["--since", "-7d", "-p", "err"]).await;
    ErrorsInfo {
        total: entries.len(),
        groups: group_errors(&entries),
        times: entries.iter().map(|e| e.at).collect(),
    }
}

/// One kernel message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KernelLine {
    pub at: i64,
    pub priority: u8,
    /// The part before the first colon, such as `ACPI`, `usb`, or `nvme`.
    pub subsystem: String,
    pub message: String,
}

pub fn subsystem(message: &str) -> String {
    let head = message.split(':').next().unwrap_or("").trim();
    let word = head.split_whitespace().next().unwrap_or("");
    if head.len() > 30 || word.is_empty() || head == message.trim() {
        "kernel".into()
    } else {
        word.trim_end_matches(|c: char| c == '[' || c.is_ascii_digit())
            .to_owned()
    }
}

pub async fn read_kernel_log() -> Vec<KernelLine> {
    let entries = journal(&["-k", "--since", "-7d", "-n", "2000"]).await;
    newest(entries)
        .into_iter()
        .map(|e| KernelLine {
            subsystem: subsystem(&e.message),
            at: e.at,
            priority: e.priority,
            message: e.message,
        })
        .collect()
}

/// One crash, or several of the same program with the same signal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Crash {
    /// The newest one.
    pub at: i64,
    pub app: String,
    /// Such as `SIGSEGV` or `SIGKILL (out of memory)`.
    pub signal: String,
    /// `kernel`, `systemd`, or `apport`.
    pub source: String,
    pub count: usize,
}

const SIGNALS: &[(u32, &str)] = &[
    (4, "SIGILL"),
    (6, "SIGABRT"),
    (7, "SIGBUS"),
    (8, "SIGFPE"),
    (9, "SIGKILL"),
    (11, "SIGSEGV"),
    (31, "SIGSYS"),
];

/// Finds crashes in journal messages: kernel segfaults, OOM kills, and services that dumped core.
pub fn crashes_in(entries: &[Entry]) -> Vec<(i64, String, String, String)> {
    let mut out = Vec::new();
    for e in entries {
        let m = &e.message;
        if let Some(i) = m.find(" segfault at ") {
            // "bluetoothd[7702]: segfault at 0 ip …"
            let app = m[..i].split('[').next().unwrap_or("?").trim().to_owned();
            out.push((e.at, app, "SIGSEGV".into(), "kernel".into()));
        } else if let Some(rest) = m.strip_prefix("Out of memory: Killed process ") {
            let app = rest
                .split('(')
                .nth(1)
                .and_then(|s| s.split(')').next())
                .unwrap_or("?")
                .to_owned();
            out.push((e.at, app, "SIGKILL (out of memory)".into(), "kernel".into()));
        } else if m.contains("Main process exited, code=dumped") {
            let app = m
                .split(':')
                .next()
                .unwrap_or("?")
                .trim_end_matches(".service")
                .to_owned();
            let status = m
                .split("status=")
                .nth(1)
                .and_then(|s| s.split('/').next())
                .and_then(|s| s.trim().parse::<u32>().ok());
            let signal = status
                .and_then(|s| {
                    SIGNALS
                        .iter()
                        .find(|(n, _)| *n == s)
                        .map(|(_, n)| (*n).to_owned())
                })
                .unwrap_or_else(|| "core dump".into());
            out.push((e.at, app, signal, "systemd".into()));
        }
    }
    out
}

/// Reads one apport report header: `ExecutablePath`, `Date`, `Signal`.
pub fn apport(text: &str, modified: i64) -> Option<(i64, String, String, String)> {
    let field = |k: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(&format!("{k}: ")))
            .map(str::to_owned)
    };
    let app = field("ExecutablePath")
        .or_else(|| field("Package"))?
        .rsplit('/')
        .next()?
        .to_owned();
    let signal = field("Signal")
        .and_then(|s| s.trim().parse::<u32>().ok())
        .and_then(|s| {
            SIGNALS
                .iter()
                .find(|(n, _)| *n == s)
                .map(|(_, n)| (*n).to_owned())
        })
        .unwrap_or_else(|| "crash".into());
    Some((modified, app, signal, "apport".into()))
}

pub fn group_crashes(list: Vec<(i64, String, String, String)>) -> Vec<Crash> {
    let mut groups: BTreeMap<(String, String), Crash> = BTreeMap::new();
    for (at, app, signal, source) in list {
        let c = groups
            .entry((app.clone(), signal.clone()))
            .or_insert(Crash {
                at,
                app,
                signal,
                source,
                count: 0,
            });
        c.count += 1;
        c.at = c.at.max(at);
    }
    let mut out: Vec<Crash> = groups.into_values().collect();
    out.sort_by_key(|c| std::cmp::Reverse(c.at));
    out
}

pub async fn read_crashes(root: &SystemRoot) -> Vec<Crash> {
    let mut list = crashes_in(
        &journal(&[
            "--since",
            "-7d",
            "--grep",
            "segfault at|Out of memory: Killed|code=dumped",
        ])
        .await,
    );
    let week = now_ms() - 7 * 86_400_000;
    if let Ok(dir) = std::fs::read_dir(root.path("/var/crash")) {
        for e in dir
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "crash"))
        {
            let modified = e
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_millis() as i64);
            // Reports belong to their owner; others' are unreadable and skipped. Only the header is needed.
            let head: String = std::fs::read(e.path())
                .map(|b| String::from_utf8_lossy(&b[..b.len().min(8192)]).into_owned())
                .unwrap_or_default();
            if modified >= week
                && let Some(c) = apport(&head, modified)
            {
                list.push(c);
            }
        }
    }
    group_crashes(list)
}

/// One boot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Boot {
    pub started: i64,
    /// Last journal entry of that boot.
    pub ended: i64,
    /// `running`, `clean`, or `unclean` (no orderly shutdown was logged).
    pub shutdown: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel: Option<String>,
}

/// True when a boot's last messages show an orderly shutdown or reboot.
pub fn clean_end(last_lines: &str) -> bool {
    [
        "Journal stopped",
        "Reached target System Power Off",
        "Reached target System Reboot",
        "Reached target shutdown.target",
        "systemd-shutdown",
        "System is powering down",
        "System is rebooting",
    ]
    .iter()
    .any(|p| last_lines.contains(p))
}

pub async fn read_boots() -> Vec<Boot> {
    let Some(text) = command::output(
        "journalctl",
        &["--list-boots", "-o", "json", "--no-pager", "-q"],
        LIMIT,
    )
    .await
    else {
        return Vec::new();
    };
    let Ok(Value::Array(list)) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let n = list.len();
    for (i, b) in list.iter().enumerate().skip(n.saturating_sub(10)) {
        let (Some(id), Some(first), Some(last)) = (
            b.get("boot_id").and_then(Value::as_str),
            b.get("first_entry").and_then(Value::as_i64),
            b.get("last_entry").and_then(Value::as_i64),
        ) else {
            continue;
        };
        let kernel = command::output(
            "journalctl",
            &[
                "-b",
                id,
                "-k",
                "-o",
                "cat",
                "--no-pager",
                "-q",
                "--grep",
                "^Linux version",
                "-n",
                "1",
            ],
            LIMIT,
        )
        .await
        .and_then(|l| l.split_whitespace().nth(2).map(str::to_owned));
        let shutdown = if i + 1 == n {
            "running".into()
        } else {
            let tail = command::output(
                "journalctl",
                &["-b", id, "-o", "cat", "--no-pager", "-q", "-n", "40"],
                LIMIT,
            )
            .await
            .unwrap_or_default();
            if clean_end(&tail) {
                "clean".into()
            } else {
                "unclean".into()
            }
        };
        out.push(Boot {
            started: first / 1000,
            ended: last / 1000,
            shutdown,
            kernel,
        });
    }
    out.reverse();
    out
}

/// One USB plug or unplug.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeviceEvent {
    pub at: i64,
    /// `connected` or `disconnected`.
    pub event: String,
    pub device: String,
    /// `vendor:product`, when logged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Pairs kernel lines: "usb 3-1: New USB device found, idVendor=046d, idProduct=c52b",
/// "usb 3-1: Product: USB Receiver", "usb 3-1: USB disconnect, device number 5".
/// Root hubs (`usb1`) are left out.
pub fn device_events(entries: &[Entry]) -> Vec<DeviceEvent> {
    let mut names: BTreeMap<String, (String, Option<String>)> = BTreeMap::new();
    let mut out: Vec<DeviceEvent> = Vec::new();
    for e in entries {
        let Some(rest) = e.message.strip_prefix("usb ") else {
            continue;
        };
        let Some((port, text)) = rest.split_once(": ") else {
            continue;
        };
        if port.starts_with("usb") {
            continue;
        }
        if text.starts_with("New USB device found") {
            let id = |k: &str| {
                text.split(k)
                    .nth(1)
                    .map(|v| v.chars().take(4).collect::<String>())
            };
            let ids = id("idVendor=")
                .zip(id("idProduct="))
                .map(|(v, p)| format!("{v}:{p}"));
            names.insert(
                port.to_owned(),
                (format!("USB device on port {port}"), ids.clone()),
            );
            out.push(DeviceEvent {
                at: e.at,
                event: "connected".into(),
                device: format!("USB device on port {port}"),
                id: ids,
            });
        } else if let Some(product) = text.strip_prefix("Product: ") {
            if let Some(entry) = names.get_mut(port) {
                entry.0 = product.to_owned();
            }
            if let Some(last) = out.iter_mut().rev().find(|d| {
                d.event == "connected" && d.device == format!("USB device on port {port}")
            }) {
                last.device = product.to_owned();
            }
        } else if text.starts_with("USB disconnect") {
            let (device, id) = names
                .get(port)
                .cloned()
                .unwrap_or((format!("USB device on port {port}"), None));
            out.push(DeviceEvent {
                at: e.at,
                event: "disconnected".into(),
                device,
                id,
            });
        }
    }
    out.reverse();
    out.truncate(KEEP);
    out
}

pub async fn read_device_events() -> Vec<DeviceEvent> {
    device_events(
        &journal(&[
            "-k",
            "--since",
            "-30d",
            "--grep",
            "^usb .*(New USB device found|Product:|USB disconnect)",
        ])
        .await,
    )
}

/// A file in `/etc` changed recently.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConfigChange {
    pub path: String,
    pub modified: i64,
    /// The file's owner.
    pub owner: String,
}

/// Files under `/etc` changed in the last 30 days, newest first.
pub fn config_changes(
    root: &SystemRoot,
    users: &std::collections::HashMap<u32, String>,
) -> Vec<ConfigChange> {
    use std::os::unix::fs::MetadataExt;
    let since = now_ms() - 30 * 86_400_000;
    let mut out = Vec::new();
    let mut stack = vec![root.path("/etc")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let Ok(meta) = e.path().symlink_metadata() else {
                continue;
            };
            if meta.is_dir() {
                stack.push(e.path());
                continue;
            }
            let modified = meta.mtime() * 1000;
            if modified >= since {
                let path = e.path().to_string_lossy().into_owned();
                let shown = root.path("/").to_string_lossy().trim_end_matches('/').len();
                out.push(ConfigChange {
                    path: path[shown..].to_owned(),
                    modified,
                    owner: users
                        .get(&meta.uid())
                        .cloned()
                        .unwrap_or_else(|| meta.uid().to_string()),
                });
            }
        }
    }
    out.sort_by_key(|c| std::cmp::Reverse(c.modified));
    out.truncate(KEEP);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(at: i64, unit: &str, message: &str) -> Entry {
        Entry {
            at,
            priority: 3,
            unit: unit.into(),
            message: message.into(),
        }
    }

    #[test]
    fn reads_entries_and_counts_hours() {
        let lines = r#"{"__REALTIME_TIMESTAMP":"3600000000","PRIORITY":"4","_SYSTEMD_UNIT":"bluetooth.service","MESSAGE":"hello"}
{"__REALTIME_TIMESTAMP":"7200000000","PRIORITY":"6","SYSLOG_IDENTIFIER":"kernel","MESSAGE":[1,2,3]}
{"__REALTIME_TIMESTAMP":"7300000000","SYSLOG_IDENTIFIER":"gnome-shell","_SYSTEMD_UNIT":"session-2.scope","MESSAGE":"x"}"#;
        let e = parse_entries(lines);
        assert_eq!(e.len(), 2, "binary messages are skipped");
        assert_eq!((e[0].unit.as_str(), e[0].priority), ("bluetooth", 4));
        assert_eq!(
            e[1].unit, "gnome-shell",
            "a login session scope is named by its program"
        );
        let v = hourly(
            &[
                entry(10 * 3_600_000, "a", "x"),
                entry(10 * 3_600_000 + 5, "a", "x"),
                entry(0, "a", "x"),
            ],
            10 * 3_600_000 + 60_000,
        );
        assert_eq!((v[23], v[13]), (2, 1));
    }

    #[test]
    fn groups_errors_by_pattern() {
        assert_eq!(
            pattern("usb 3-1: device 12 not accepting address 0x1f"),
            "usb #-#: device # not accepting address #"
        );
        let groups = group_errors(&[
            entry(1, "kernel", "ACPI BIOS Error (bug): AE_NOT_FOUND 12"),
            entry(2, "kernel", "ACPI BIOS Error (bug): AE_NOT_FOUND 13"),
            entry(
                3,
                "bluetooth",
                "bluetooth.service: Main process exited, code=dumped, status=11/SEGV",
            ),
            entry(4, "app", "Something odd 5"),
        ]);
        assert_eq!(
            groups
                .iter()
                .map(|g| (g.class.as_str(), g.count))
                .collect::<Vec<_>>(),
            [("attention", 1), ("review", 1), ("harmless", 2)]
        );
        assert_eq!(groups[2].message, "ACPI BIOS Error (bug): AE_NOT_FOUND 13");
        assert_eq!(
            classify(
                "Failed to start app-gnome-x-1.scope - Application launched by gnome-session-binary."
            ),
            "harmless"
        );
        assert_eq!(
            classify("Failed to start cups.service - CUPS Scheduler."),
            "attention"
        );
    }

    #[test]
    fn finds_crashes() {
        let found = crashes_in(&[
            entry(
                1,
                "kernel",
                "bluetoothd[7702]: segfault at 0 ip 000055d1c2a4 sp 0 error 4",
            ),
            entry(
                2,
                "kernel",
                "Out of memory: Killed process 30412 (chrome) total-vm:1kB",
            ),
            entry(
                3,
                "systemd",
                "bluetooth.service: Main process exited, code=dumped, status=11/SEGV",
            ),
        ]);
        assert_eq!(
            found
                .iter()
                .map(|c| (c.1.as_str(), c.2.as_str()))
                .collect::<Vec<_>>(),
            [
                ("bluetoothd", "SIGSEGV"),
                ("chrome", "SIGKILL (out of memory)"),
                ("bluetooth", "SIGSEGV")
            ]
        );
        let report = apport(
            "ProblemType: Crash\nExecutablePath: /usr/bin/gnome-calculator\nSignal: 6\n",
            99,
        )
        .unwrap();
        assert_eq!(
            (report.1.as_str(), report.2.as_str()),
            ("gnome-calculator", "SIGABRT")
        );
        let grouped = group_crashes(vec![
            (1, "a".into(), "SIGSEGV".into(), "kernel".into()),
            (5, "a".into(), "SIGSEGV".into(), "kernel".into()),
        ]);
        assert_eq!((grouped[0].count, grouped[0].at), (2, 5));
    }

    #[test]
    fn names_kernel_subsystems_and_shutdowns() {
        assert_eq!(subsystem("ACPI BIOS Error (bug): AE_NOT_FOUND"), "ACPI");
        assert_eq!(subsystem("usb 3-1: new high-speed USB device"), "usb");
        assert_eq!(subsystem("Linux version 6.8.0"), "kernel");
        assert!(clean_end("...\nJournal stopped\n"));
        assert!(!clean_end("wlp0s20f3: disconnected\n"));
    }

    #[test]
    fn pairs_usb_events() {
        let events = device_events(&[
            entry(
                1,
                "kernel",
                "usb usb1: New USB device found, idVendor=1d6b, idProduct=0002",
            ),
            entry(
                2,
                "kernel",
                "usb 3-1: New USB device found, idVendor=0781, idProduct=5583, bcdDevice= 1.00",
            ),
            entry(3, "kernel", "usb 3-1: Product: Ultra USB 3.0"),
            entry(4, "kernel", "usb 3-1: USB disconnect, device number 5"),
        ]);
        assert_eq!(
            events
                .iter()
                .map(|d| (d.event.as_str(), d.device.as_str(), d.id.as_deref()))
                .collect::<Vec<_>>(),
            [
                ("disconnected", "Ultra USB 3.0", Some("0781:5583")),
                ("connected", "Ultra USB 3.0", Some("0781:5583"))
            ]
        );
    }
}
