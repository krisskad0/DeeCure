// SPDX-License-Identifier: Apache-2.0

//! Events from the systemd journal: USB devices plugged in and out, crashes, out-of-memory
//! kills, failed services, and boots (with unclean shutdowns and kernel changes).
//!
//! Repeats are grouped per hour: one event says "crashed 14 times" instead of 14 events,
//! so a program stuck in a crash loop cannot fill the database. Each read starts at the
//! beginning of the hour of the last read, so every hour's count is complete; the stored
//! event for that hour is then replaced with the full count.

use std::collections::BTreeMap;

use deecure_core::{Event, EventKind, Timestamp};
use serde_json::json;

use crate::inventory::logs::{self, Boot, Entry};

pub const HOUR_MS: i64 = 3_600_000;
/// The journal pattern for everything this module reads.
pub const GREP: &str = "^usb .*(New USB device found|Product:|USB disconnect)|segfault at|Out of memory: Killed|code=dumped|: Failed with result";

/// One group: the newest time and how many.
#[derive(Debug, Clone, Copy)]
struct Group {
    at: i64,
    count: u64,
}

fn add(groups: &mut BTreeMap<(String, String, i64), Group>, key: (String, String), at: i64) {
    let g = groups
        .entry((key.0, key.1, at.div_euclid(HOUR_MS)))
        .or_insert(Group { at, count: 0 });
    g.at = g.at.max(at);
    g.count += 1;
}

fn times(count: u64) -> String {
    if count > 1 {
        format!(" ({count} times)")
    } else {
        String::new()
    }
}

/// "bluetooth.service: Failed with result 'exit-code'." → ("bluetooth.service", "exit-code")
fn failed_unit(message: &str) -> Option<(String, String)> {
    let (unit, rest) = message.split_once(": Failed with result '")?;
    let result = rest.split('\'').next()?;
    (!unit.contains(' ')).then(|| (unit.to_owned(), result.to_owned()))
}

/// Turns journal entries (oldest first) into grouped events, as `(key, event)`.
pub fn events(entries: &[Entry]) -> Vec<(String, Event)> {
    let mut out = Vec::new();

    let mut devices = BTreeMap::new();
    let mut ids: BTreeMap<String, Option<String>> = BTreeMap::new();
    for d in logs::device_events(entries) {
        ids.entry(d.device.clone()).or_insert(d.id.clone());
        add(&mut devices, (d.event.clone(), d.device.clone()), d.at);
    }
    for ((event, device, hour), g) in devices {
        let connected = event == "connected";
        out.push((
            format!("usb:{event}:{device}:{hour}"),
            Event {
                id: None,
                kind: if connected { EventKind::DeviceAdded } else { EventKind::DeviceRemoved },
                at: Timestamp(g.at),
                title: format!(
                    "{device} {}{}",
                    if connected { "connected" } else { "disconnected" },
                    times(g.count)
                ),
                detail: None,
                subject: Some(device.clone()),
                source: "kernel".into(),
                data: Some(json!({ "bus": "USB", "id": ids.get(&device).cloned().flatten(), "count": g.count })),
            },
        ));
    }

    let mut crashes = BTreeMap::new();
    for (at, app, signal, _) in logs::crashes_in(entries) {
        add(&mut crashes, (app, signal), at);
    }
    for ((app, signal, hour), g) in crashes {
        let oom = signal.contains("out of memory");
        out.push((
            format!("crash:{app}:{signal}:{hour}"),
            Event {
                id: None,
                kind: if oom {
                    EventKind::OutOfMemoryKill
                } else {
                    EventKind::Crash
                },
                at: Timestamp(g.at),
                title: if oom {
                    format!("{app} was stopped: memory ran out{}", times(g.count))
                } else {
                    format!("{app} crashed{}", times(g.count))
                },
                detail: None,
                subject: Some(app.clone()),
                source: "journal".into(),
                data: Some(json!({ "signal": signal, "count": g.count })),
            },
        ));
    }

    let mut failed = BTreeMap::new();
    for e in entries {
        if let Some((unit, result)) = failed_unit(&e.message) {
            add(&mut failed, (unit, result), e.at);
        }
    }
    for ((unit, result, hour), g) in failed {
        out.push((
            format!("failed:{unit}:{hour}"),
            Event {
                id: None,
                kind: EventKind::ServiceFailed,
                at: Timestamp(g.at),
                title: format!("{unit} failed{}", times(g.count)),
                detail: None,
                subject: Some(unit.clone()),
                source: "systemd".into(),
                data: Some(json!({ "result": result, "count": g.count })),
            },
        ));
    }
    out
}

/// Boot events from the boot list (newest first): each start, a note when the kernel
/// changed since the boot before, and each unclean shutdown.
pub fn boot_events(boots: &[Boot]) -> Vec<(String, Event)> {
    let mut out = Vec::new();
    for (i, b) in boots.iter().enumerate() {
        let before = boots.get(i + 1).and_then(|p| p.kernel.clone());
        let changed = matches!((&b.kernel, &before), (Some(k), Some(p)) if k != p);
        let title = match (&b.kernel, &before) {
            (Some(k), Some(p)) if changed => format!("Started with a new kernel {k} (was {p})"),
            _ => "Computer started".to_owned(),
        };
        let mut data = json!({ "boot_id": b.id, "kernel": b.kernel });
        if changed {
            data["previous_kernel"] = json!(before);
            data["kernel_changed"] = json!(true);
        }
        out.push((
            format!("boot:{}", b.id),
            Event {
                id: None,
                kind: EventKind::Boot,
                at: Timestamp(b.started),
                title,
                detail: b.kernel.as_ref().map(|k| format!("Kernel {k}")),
                subject: None,
                source: "journal".into(),
                data: Some(data),
            },
        ));
        if b.shutdown == "unclean" {
            out.push((
                format!("unclean:{}", b.id),
                Event {
                    id: None,
                    kind: EventKind::UncleanShutdown,
                    at: Timestamp(b.ended),
                    title: "Unclean shutdown".into(),
                    detail: Some("Power loss, a freeze, or a forced power-off".into()),
                    subject: None,
                    source: "journal".into(),
                    data: Some(json!({ "boot_id": b.id, "kernel": b.kernel })),
                },
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(at: i64, message: &str) -> Entry {
        Entry {
            at,
            priority: 3,
            unit: "kernel".into(),
            message: message.into(),
        }
    }

    #[test]
    fn repeats_in_one_hour_are_one_event() {
        let h = 10 * HOUR_MS;
        let entries: Vec<Entry> = (0..14)
            .map(|i| {
                entry(
                    h + i * 60_000,
                    "bluetoothd[7702]: segfault at 0 ip 000055 sp 00007ff error 4",
                )
            })
            .chain([
                entry(
                    h + 2 * HOUR_MS,
                    "bluetoothd[7900]: segfault at 0 ip 1 sp 2 error 4",
                ),
                entry(
                    h + 5,
                    "Out of memory: Killed process 30412 (chrome) total-vm:1kB",
                ),
                entry(h + 6, "bluetooth.service: Failed with result 'core-dump'."),
                entry(
                    h + 7,
                    "something: Failed with result 'exit-code' and more words",
                ),
            ])
            .collect();
        let events = events(&entries);
        let titles: Vec<&str> = events.iter().map(|(_, e)| e.title.as_str()).collect();
        assert!(
            titles.contains(&"bluetoothd crashed (14 times)"),
            "{titles:?}"
        );
        assert!(titles.contains(&"bluetoothd crashed"));
        assert!(titles.contains(&"chrome was stopped: memory ran out"));
        assert!(titles.contains(&"bluetooth.service failed"));
        assert!(titles.contains(&"something failed"));
        let (key, crash) = events
            .iter()
            .find(|(_, e)| e.title.ends_with("(14 times)"))
            .unwrap();
        assert_eq!(key, "crash:bluetoothd:SIGSEGV:10");
        assert_eq!(crash.at, Timestamp(h + 13 * 60_000));
        assert_eq!(crash.data.as_ref().unwrap()["count"], 14);
    }

    #[test]
    fn usb_devices_are_named_and_counted() {
        let entries = [
            entry(
                1000,
                "usb 3-2: New USB device found, idVendor=0781, idProduct=5583, bcdDevice= 1.00",
            ),
            entry(1001, "usb 3-2: Product: Ultra Fit"),
            entry(5000, "usb 3-2: USB disconnect, device number 4"),
        ];
        let events = events(&entries);
        assert_eq!(events.len(), 2);
        let connected = events
            .iter()
            .find(|(_, e)| e.kind == EventKind::DeviceAdded)
            .unwrap();
        assert_eq!(connected.1.title, "Ultra Fit connected");
        assert_eq!(connected.1.data.as_ref().unwrap()["id"], "0781:5583");
        let gone = events
            .iter()
            .find(|(_, e)| e.kind == EventKind::DeviceRemoved)
            .unwrap();
        assert_eq!(gone.1.title, "Ultra Fit disconnected");
    }

    #[test]
    fn boots_show_kernel_changes_and_unclean_shutdowns() {
        let boot = |id: &str, kernel: &str, shutdown: &str, started: i64| Boot {
            id: id.into(),
            started,
            ended: started + 100,
            shutdown: shutdown.into(),
            kernel: Some(kernel.into()),
        };
        let boots = [
            boot("c", "6.8.0-47-generic", "running", 3000),
            boot("b", "6.8.0-45-generic", "unclean", 2000),
            boot("a", "6.8.0-45-generic", "clean", 1000),
        ];
        let events = boot_events(&boots);
        assert_eq!(events.len(), 4);
        assert_eq!(events[0].0, "boot:c");
        assert_eq!(
            events[0].1.title,
            "Started with a new kernel 6.8.0-47-generic (was 6.8.0-45-generic)"
        );
        assert_eq!(events[0].1.data.as_ref().unwrap()["kernel_changed"], true);
        assert_eq!(events[1].1.title, "Computer started");
        assert_eq!(events[2].1.kind, EventKind::UncleanShutdown);
        assert_eq!(events[2].1.at, Timestamp(2100));
    }
}
