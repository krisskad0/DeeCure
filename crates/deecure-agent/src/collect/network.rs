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

/// True when a routing table has a usable default route: the computer is on a network
/// that leads somewhere. `ipv4` is `/proc/net/route`, `ipv6` is `/proc/net/ipv6_route`.
/// The loopback's "unreachable" entries and rejected routes do not count.
pub fn has_default_route(ipv4: &str, ipv6: &str) -> bool {
    const UP: u32 = 0x0001;
    const REJECT: u32 = 0x0200;
    let usable =
        |flags: &str| u32::from_str_radix(flags, 16).is_ok_and(|f| f & UP != 0 && f & REJECT == 0);
    let v4 = ipv4.lines().skip(1).any(|l| {
        let f: Vec<&str> = l.split_whitespace().collect();
        f.len() > 3 && f[0] != "lo" && f[1] == "00000000" && usable(f[3])
    });
    let v6 = ipv6.lines().any(|l| {
        let f: Vec<&str> = l.split_whitespace().collect();
        f.len() >= 10
            && f[0].chars().all(|c| c == '0')
            && f[1] == "00"
            && f[9] != "lo"
            && usable(f[8])
    });
    v4 || v6
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
        // Without the IPv4 table (an odd container) the state is unknown, not "offline".
        if let Ok(v4) = root.read("/proc/net/route") {
            let v6 = root.read("/proc/net/ipv6_route").unwrap_or_default();
            sample.metrics.push(metric(
                "net.default_route",
                None,
                f64::from(u8::from(has_default_route(&v4, &v6))),
                Unit::Count,
                now,
            ));
        }
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
    fn finds_a_default_route_but_not_rejects() {
        let v4 = "Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\nwlp0s20f3\t00000000\t0101A8C0\t0003\t0\t0\t600\t00000000\nwlp0s20f3\t0001A8C0\t00000000\t0001\t0\t0\t600\t00FFFFFF\n";
        let no_default =
            "Iface\tDestination\tGateway \tFlags\nwlp0s20f3\t0001A8C0\t00000000\t0001\n";
        let v6_lo = "00000000000000000000000000000000 00 00000000000000000000000000000000 00 00000000000000000000000000000000 ffffffff 00000001 00000000 00200200       lo\n";
        let v6_ok = "00000000000000000000000000000000 00 00000000000000000000000000000000 00 fe800000000000000000000000000001 00000400 00000001 00000000 00450003 wlp0s20f3\n";
        assert!(has_default_route(v4, ""));
        assert!(
            !has_default_route(no_default, v6_lo),
            "a reject route on lo is not a connection"
        );
        assert!(
            has_default_route(no_default, v6_ok),
            "IPv6 only still counts"
        );
        assert!(!has_default_route("Iface\n", ""));
    }

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
