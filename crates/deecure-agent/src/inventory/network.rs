// SPDX-License-Identifier: Apache-2.0

//! Network interfaces from `/sys/class/net` and open connections from `/proc/net`.

use serde::Serialize;

use crate::collect::SystemRoot;

/// One network interface.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Interface {
    /// Name, such as `wlp45s0`.
    pub name: String,
    /// Plain-English kind: Wi-Fi, Ethernet, Ethernet (USB), Bridge, Loopback, Mobile, VPN or tunnel, Virtual.
    pub kind: String,
    /// `up`, `down`, `dormant`, or `unknown`, as the kernel says.
    pub state: String,
    /// Hardware address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac: Option<String>,
    /// Link speed in megabits per second, when the driver reports one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_mbps: Option<u32>,
    /// Largest packet, in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
}

/// Counts of open sockets.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Connections {
    /// TCP connections that are open (established).
    pub tcp_established: usize,
    /// TCP ports waiting for connections.
    pub tcp_listening: usize,
    /// UDP sockets.
    pub udp: usize,
}

/// Interfaces and connections.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct NetworkInfo {
    /// Every interface, sorted by name.
    pub interfaces: Vec<Interface>,
    /// Open sockets, IPv4 and IPv6 together.
    pub connections: Connections,
}

/// Reads interfaces and connections.
pub fn read(root: &SystemRoot) -> NetworkInfo {
    let mut interfaces: Vec<Interface> = std::fs::read_dir(root.path("/sys/class/net"))
        .map(|dir| {
            dir.flatten()
                .map(|e| interface(root, &e.file_name().to_string_lossy()))
                .collect()
        })
        .unwrap_or_default();
    interfaces.sort_by(|a, b| a.name.cmp(&b.name));
    NetworkInfo {
        interfaces,
        connections: connections(root),
    }
}

fn interface(root: &SystemRoot, name: &str) -> Interface {
    let base = format!("/sys/class/net/{name}");
    let text = |file: &str| {
        root.read(&format!("{base}/{file}"))
            .ok()
            .map(|s| s.trim().to_owned())
    };
    let devtype = text("uevent").and_then(|u| {
        u.lines()
            .find_map(|l| l.strip_prefix("DEVTYPE="))
            .map(str::to_owned)
    });
    let device = std::fs::canonicalize(root.path(&format!("{base}/device"))).ok();
    let kind = match (devtype.as_deref(), text("type").as_deref()) {
        (Some("wlan"), _) => "Wi-Fi",
        (Some("bridge"), _) => "Bridge",
        (Some("wwan"), _) => "Mobile",
        (_, Some("772")) => "Loopback",
        (_, Some("65534")) => "VPN or tunnel",
        _ => match &device {
            None => "Virtual",
            Some(path) if path.to_string_lossy().contains("/usb") => "Ethernet (USB)",
            Some(_) => "Ethernet",
        },
    };
    Interface {
        name: name.to_owned(),
        kind: kind.to_owned(),
        state: text("operstate").unwrap_or_else(|| "unknown".into()),
        mac: text("address").filter(|m| !m.is_empty() && m != "00:00:00:00:00:00"),
        // Down links report -1 or fail to read.
        speed_mbps: text("speed")
            .and_then(|s| s.parse::<i64>().ok())
            .and_then(|s| u32::try_from(s).ok())
            .filter(|s| *s > 0),
        mtu: text("mtu").and_then(|m| m.parse().ok()),
    }
}

/// Socket states in `/proc/net/tcp`: 01 established, 0A listening.
fn connections(root: &SystemRoot) -> Connections {
    let states = |file: &str| -> Vec<String> {
        root.read(file)
            .unwrap_or_default()
            .lines()
            .skip(1)
            .filter_map(|l| l.split_whitespace().nth(3).map(str::to_owned))
            .collect()
    };
    let tcp: Vec<String> = [states("/proc/net/tcp"), states("/proc/net/tcp6")].concat();
    Connections {
        tcp_established: tcp.iter().filter(|s| *s == "01").count(),
        tcp_listening: tcp.iter().filter(|s| *s == "0A").count(),
        udp: states("/proc/net/udp").len() + states("/proc/net/udp6").len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    const TCP: &str = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 0100007F:0277 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 1 1
   1: 0F02000A:A1B2 22B8D58E:01BB 01 00000000:00000000 00:00000000 00000000  1000        0 2 1
   2: 0F02000A:A1B3 22B8D58E:01BB 06 00000000:00000000 00:00000000 00000000  1000        0 3 1
";

    #[test]
    fn reads_interfaces_and_counts_sockets() {
        let root = fixture_root(
            "network",
            &[
                (
                    "/sys/class/net/wlp45s0/uevent",
                    "DEVTYPE=wlan\nINTERFACE=wlp45s0\n",
                ),
                ("/sys/class/net/wlp45s0/type", "1\n"),
                ("/sys/class/net/wlp45s0/operstate", "up\n"),
                ("/sys/class/net/wlp45s0/address", "a4:c3:f0:11:22:33\n"),
                ("/sys/class/net/wlp45s0/mtu", "1500\n"),
                ("/sys/class/net/lo/type", "772\n"),
                ("/sys/class/net/lo/operstate", "unknown\n"),
                ("/sys/class/net/lo/address", "00:00:00:00:00:00\n"),
                ("/sys/class/net/veth1/type", "1\n"),
                ("/sys/class/net/veth1/operstate", "down\n"),
                ("/sys/class/net/veth1/speed", "-1\n"),
                ("/proc/net/tcp", TCP),
                ("/proc/net/udp", "header\n 1: x y 07\n"),
            ],
        );
        let info = read(&root);
        let kinds: Vec<(&str, &str)> = info
            .interfaces
            .iter()
            .map(|i| (i.name.as_str(), i.kind.as_str()))
            .collect();
        assert_eq!(
            kinds,
            [
                ("lo", "Loopback"),
                ("veth1", "Virtual"),
                ("wlp45s0", "Wi-Fi")
            ]
        );
        assert_eq!(info.interfaces[0].mac, None);
        assert_eq!(info.interfaces[1].speed_mbps, None);
        assert_eq!(info.interfaces[2].mtu, Some(1500));
        assert_eq!(
            info.connections,
            Connections {
                tcp_established: 1,
                tcp_listening: 1,
                udp: 1
            }
        );
    }
}
