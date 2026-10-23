// SPDX-License-Identifier: Apache-2.0

//! Network traffic from `/proc/net/dev`.

use std::collections::BTreeMap;
use std::io;
use std::time::Instant;

use deecure_core::{Timestamp, Unit};

use super::{Collector, Sample, SystemRoot, metric};

/// Byte counters per device or interface: (in, out).
type Counters = BTreeMap<String, (u64, u64)>;

/// Bytes received and sent per interface since boot.
pub fn parse_net_dev(text: &str) -> Counters {
    text.lines()
        .skip(2)
        .filter_map(|line| {
            let (name, rest) = line.split_once(':')?;
            let fields: Vec<u64> = rest
                .split_whitespace()
                .filter_map(|f| f.parse().ok())
                .collect();
            Some((name.trim().to_owned(), (*fields.first()?, *fields.get(8)?)))
        })
        .collect()
}

/// TCP ports listening on every network (0.0.0.0 or ::), from `/proc/net/tcp{,6}`.
pub fn listening_on_all(tcp: &str) -> usize {
    tcp.lines()
        .skip(1)
        .filter(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            let any = f.get(1).is_some_and(|a| {
                a.split(':')
                    .next()
                    .is_some_and(|ip| ip.chars().all(|c| c == '0'))
            });
            any && f.get(3) == Some(&"0A")
        })
        .count()
}

/// Receive and send rates, in total and per interface. The loopback interface is left out,
/// and so are container links (`veth…`): each container gets a new one, and their traffic
/// is already counted on the bridge, so keeping them would only fill the history.
/// Also whether the ufw firewall is on, and how many ports listen on every network.
#[derive(Debug, Default)]
pub struct Network {
    previous: Option<(Instant, Counters)>,
}

impl Collector for Network {
    fn name(&self) -> &'static str {
        "network"
    }

    fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let mut counters = parse_net_dev(&root.read("/proc/net/dev")?);
        counters.retain(|name, _| name != "lo" && !name.starts_with("veth"));
        let at = Instant::now();
        let mut sample = Sample::default();
        if let Some((before, old)) = self.previous.replace((at, counters.clone())) {
            let seconds = at.duration_since(before).as_secs_f64().max(0.001);
            let (mut rx_total, mut tx_total) = (0.0, 0.0);
            for (name, (rx, tx)) in &counters {
                let Some((old_rx, old_tx)) = old.get(name) else {
                    continue;
                };
                let rx_rate = rx.saturating_sub(*old_rx) as f64 / seconds;
                let tx_rate = tx.saturating_sub(*old_tx) as f64 / seconds;
                rx_total += rx_rate;
                tx_total += tx_rate;
                sample.metrics.push(metric(
                    "net.rx",
                    Some(name),
                    rx_rate,
                    Unit::BytesPerSecond,
                    now,
                ));
                sample.metrics.push(metric(
                    "net.tx",
                    Some(name),
                    tx_rate,
                    Unit::BytesPerSecond,
                    now,
                ));
            }
            sample
                .metrics
                .push(metric("net.rx", None, rx_total, Unit::BytesPerSecond, now));
            sample
                .metrics
                .push(metric("net.tx", None, tx_total, Unit::BytesPerSecond, now));
        }
        let open = listening_on_all(&root.read("/proc/net/tcp").unwrap_or_default())
            + listening_on_all(&root.read("/proc/net/tcp6").unwrap_or_default());
        sample.metrics.push(metric(
            "net.listening_all",
            None,
            open as f64,
            Unit::Count,
            now,
        ));
        // No ufw config means ufw is not installed: the state is unknown, not "off".
        if let Ok(conf) = root.read("/etc/ufw/ufw.conf") {
            let on = conf.lines().any(|l| l.trim() == "ENABLED=yes");
            sample.metrics.push(metric(
                "firewall.enabled",
                None,
                f64::from(u8::from(on)),
                Unit::Count,
                now,
            ));
        }
        Ok(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NET_DEV: &str = "Inter-|   Receive |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets
    lo: 14514822   51338    0    0    0     0          0         0 14514822   51338    0    0    0     0       0          0
wlp0s20f3: 9000000 7000 0 0 0 0 0 0 3000000 2000 0 0 0 0 0 0
";

    #[test]
    fn counts_ports_open_to_every_network() {
        let tcp = "  sl  local_address rem_address   st\n   0: 0100007F:0277 00000000:0000 0A\n   1: 00000000:1538 00000000:0000 0A\n   2: 00000000:C8E8 0EB7FA8E:01BB 01\n";
        assert_eq!(listening_on_all(tcp), 1);
        let tcp6 = "  sl  local_address rem_address st\n   0: 00000000000000000000000000000000:0016 00000000000000000000000000000000:0000 0A\n";
        assert_eq!(listening_on_all(tcp6), 1);
    }

    #[test]
    fn reads_received_and_sent_bytes() {
        let counters = parse_net_dev(NET_DEV);
        assert_eq!(counters["wlp0s20f3"], (9_000_000, 3_000_000));
        assert_eq!(counters["lo"].0, 14_514_822);
    }
}
