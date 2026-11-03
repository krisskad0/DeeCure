// SPDX-License-Identifier: Apache-2.0

//! Download and upload per app over time, TCP only (docs/02-architecture/collectors.md,
//! "Traffic per app").
//!
//! Every minute, `ss` gives each open TCP connection of the user's apps with the bytes it
//! received and sent since it opened. The growth since the minute before is added to the app
//! the connection's process runs in. Every 5 minutes the totals are saved as `app.net_rx` and
//! `app.net_tx` (bytes in those 5 minutes), like `app.cpu_seconds`.
//!
//! Without admin rights this misses QUIC and other UDP traffic (video sites, Google), the
//! last bytes of a connection that closes between two readings, and connections that open
//! and close within a minute. The app says "TCP only", and the numbers are a lower bound.

use std::collections::HashMap;
use std::io;

use deecure_core::{Timestamp, Unit};

use super::app_battery::unit_of;
use super::app_usage::{biggest, subject};
use super::{Sample, SystemRoot, metric};
use crate::inventory::cgroups;
use crate::inventory::net_deep::{self, TcpConnection};

pub const RX: &str = "app.net_rx";
pub const TX: &str = "app.net_tx";
/// How often the totals are saved, by the clock.
const SAVE_EVERY_MS: i64 = 5 * 60_000 - 5_000;
/// Smaller than this in 5 minutes, an app goes to `other`.
const MIN_BYTES: f64 = 100.0 * 1024.0;

/// Connection counters from the reading before, and the totals not saved yet.
#[derive(Debug, Default)]
pub struct AppTraffic {
    read_once: bool,
    saved: Option<Timestamp>,
    /// (local, remote) → (received, sent) at the reading before.
    connections: HashMap<(String, String), (u64, u64)>,
    /// Per unit, bytes received and sent since the last save.
    pending: HashMap<String, (u64, u64)>,
    /// Unit → subject, kept from the last save, for apps that closed since.
    names: HashMap<String, String>,
}

impl AppTraffic {
    /// A short name for logs.
    pub const NAME: &'static str = "app traffic";

    /// Reads the connections once; every 5 minutes, also returns the totals per app.
    pub async fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let connections = net_deep::read_tcp().await.connections;
        Ok(self.step(root, connections, now))
    }

    /// Adds what the connections moved since the reading before; every 5 minutes, returns
    /// the totals per app.
    fn step(
        &mut self,
        root: &SystemRoot,
        connections: Vec<TcpConnection>,
        now: Timestamp,
    ) -> Sample {
        let mut pids = HashMap::new();
        let mut seen = HashMap::new();
        for c in connections {
            let Some(pid) = c.pid else { continue };
            let key = (c.local, c.remote);
            let (rx, tx) = match self.connections.get(&key) {
                Some(&(rx, tx)) => (c.received.saturating_sub(rx), c.sent.saturating_sub(tx)),
                // New since the reading before: all of it is new, except on the first reading,
                // which only sets the starting point.
                None if self.read_once => (c.received, c.sent),
                None => (0, 0),
            };
            seen.insert(key, (c.received, c.sent));
            if rx + tx == 0 {
                continue;
            }
            let unit = pids.entry(pid).or_insert_with(|| unit_of(root, pid));
            if let Some(unit) = unit {
                let t = self.pending.entry(unit.clone()).or_default();
                t.0 += rx;
                t.1 += tx;
            }
        }
        self.connections = seen;
        self.read_once = true;

        if self
            .saved
            .is_some_and(|s| now.as_millis() - s.as_millis() < SAVE_EVERY_MS)
        {
            return Sample::default();
        }
        self.saved = Some(now);
        self.save(root, now)
    }

    fn save(&mut self, root: &SystemRoot, now: Timestamp) -> Sample {
        let current: HashMap<String, String> = cgroups::read(root)
            .groups
            .into_iter()
            .map(|g| (g.unit, subject(g.kind, &g.name)))
            .collect();
        let (mut rx, mut tx): (HashMap<String, f64>, HashMap<String, f64>) = Default::default();
        for (unit, (r, t)) in self.pending.drain() {
            // An app that closed since keeps the name it had at the last save.
            let subject = current
                .get(&unit)
                .or_else(|| self.names.get(&unit))
                .cloned()
                .unwrap_or_else(|| fallback(&unit));
            *rx.entry(subject.clone()).or_default() += r as f64;
            *tx.entry(subject).or_default() += t as f64;
        }
        self.names = current;
        let mut sample = Sample::default();
        for (name, values) in [(RX, rx), (TX, tx)] {
            for (subject, bytes) in biggest(values, MIN_BYTES) {
                sample
                    .metrics
                    .push(metric(name, Some(&subject), bytes, Unit::Bytes, now));
            }
        }
        sample
    }
}

/// A name for a unit that closed before it was ever named.
fn fallback(unit: &str) -> String {
    match cgroups::app_id(unit) {
        Some(id) => subject("app", &id),
        None => subject(
            "service",
            &cgroups::unescape(unit.trim_end_matches(".service").trim_end_matches(".scope")),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::{fixture_root, value};

    fn conn(remote: &str, pid: u32, received: u64, sent: u64) -> TcpConnection {
        TcpConnection {
            local: "192.168.1.13:40000".into(),
            remote: remote.into(),
            program: None,
            pid: Some(pid),
            received,
            sent,
            resent: 0,
        }
    }

    #[test]
    fn adds_what_each_connection_moved_per_app() {
        let app = "/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice";
        let root = fixture_root(
            "apptraffic",
            &[
                ("/proc/self/status", "Uid:\t1000\t1000\t1000\t1000\n"),
                (
                    &format!("{app}/app-gnome-firefox-9.scope/memory.current"),
                    "1\n",
                ),
                (
                    "/proc/7/cgroup",
                    "0::/user.slice/user-1000.slice/user@1000.service/app.slice/app-gnome-firefox-9.scope\n",
                ),
                // A process of an app that is gone by the save: named from its unit.
                (
                    "/proc/8/cgroup",
                    "0::/user.slice/user-1000.slice/user@1000.service/app.slice/app-gnome-org.gnome.Maps-3.scope\n",
                ),
            ],
        );
        let mb = 1024 * 1024;
        let mut t = AppTraffic::default();
        // The first reading only sets the starting point (and saves nothing yet).
        let s = t.step(
            &root,
            vec![conn("1.1.1.1:443", 7, 50 * mb, mb)],
            Timestamp(0),
        );
        assert!(s.metrics.is_empty());
        // A minute later: 2 MB more on the old one, a new connection with 3 MB, Maps 1 MB.
        let s = t.step(
            &root,
            vec![
                conn("1.1.1.1:443", 7, 52 * mb, mb),
                conn("8.8.8.8:443", 7, 3 * mb, 0),
                conn("9.9.9.9:443", 8, mb, 0),
            ],
            Timestamp(60_000),
        );
        assert!(s.metrics.is_empty(), "not 5 minutes yet");
        let s = t.step(&root, vec![], Timestamp(300_000));
        let firefox = Some("app:firefox");
        assert_eq!(value(&s, RX, firefox), Some((5 * mb) as f64));
        assert_eq!(value(&s, RX, Some("app:org.gnome.Maps")), Some(mb as f64));
        assert_eq!(
            value(&s, TX, firefox),
            None,
            "nothing sent: under the minimum"
        );
        assert_eq!(fallback("cups.service"), "service:cups");
    }
}
