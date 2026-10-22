// SPDX-License-Identifier: Apache-2.0

//! Network details: addresses, traffic totals, gateway, Wi-Fi, connectivity, DNS, VPNs,
//! and the firewall. From `ip -j addr`, `/proc/net`, `/sys/class/net`, NetworkManager
//! (read-only D-Bus, only when it runs), systemd-resolved files, and ufw's config files.

use std::collections::HashMap;
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

use super::command;
use crate::collect::SystemRoot;
use crate::context::{Agent, Bus};

const NM: &str = "org.freedesktop.NetworkManager";

/// Addresses and traffic of one interface.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Link {
    pub name: String,
    /// Such as `192.168.1.24/24`.
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
    /// Bytes since boot.
    pub rx: u64,
    pub tx: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Wifi {
    pub interface: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssid: Option<String>,
    /// Signal in percent (NetworkManager) and dBm (kernel).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strength: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dbm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_mhz: Option<u32>,
    /// Link speed in megabits per second.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate_mbps: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Vpn {
    pub name: String,
    /// `vpn`, `wireguard`, `tun`, or `tailscale`.
    pub kind: String,
    pub connected: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Firewall {
    /// True when ufw is installed.
    pub installed: bool,
    pub enabled: bool,
    /// Default policy for incoming traffic, such as `DROP`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incoming: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outgoing: Option<String>,
    /// Rules from `/etc/ufw/user.rules`, when readable (it needs root on Ubuntu).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<String>>,
}

/// The `net_details` inventory.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct NetDetails {
    pub links: Vec<Link>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wifi: Option<Wifi>,
    /// From NetworkManager: `full`, `limited`, `portal`, `none`. Missing when NM is not running.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connectivity: Option<String>,
    /// `systemd-resolved` or `resolv.conf`.
    pub resolver: String,
    pub dns_servers: Vec<String>,
    pub vpns: Vec<Vpn>,
    pub firewall: Firewall,
}

/// Reads `ip -j addr`.
pub fn parse_addresses(json: &str) -> Vec<Link> {
    let Ok(Value::Array(list)) = serde_json::from_str::<Value>(json) else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|i| {
            let name = i.get("ifname")?.as_str()?.to_owned();
            let mut link = Link {
                name,
                ..Link::default()
            };
            for a in i
                .get("addr_info")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let (Some(family), Some(local), Some(prefix)) = (
                    a.get("family").and_then(Value::as_str),
                    a.get("local").and_then(Value::as_str),
                    a.get("prefixlen").and_then(Value::as_u64),
                ) else {
                    continue;
                };
                match family {
                    "inet" => link.ipv4.push(format!("{local}/{prefix}")),
                    "inet6" => link.ipv6.push(format!("{local}/{prefix}")),
                    _ => {}
                }
            }
            Some(link)
        })
        .collect()
}

/// The default gateway from `/proc/net/route`; addresses there are little-endian hex.
pub fn default_gateway(route: &str) -> Option<String> {
    route.lines().skip(1).find_map(|l| {
        let f: Vec<&str> = l.split_whitespace().collect();
        if f.get(1) != Some(&"00000000") {
            return None;
        }
        let g = u32::from_str_radix(f.get(2)?, 16).ok()?;
        (g != 0).then(|| std::net::Ipv4Addr::from(g.to_le_bytes()).to_string())
    })
}

/// Signal level in dBm per interface, from `/proc/net/wireless`.
pub fn wireless_dbm(text: &str) -> HashMap<String, f64> {
    text.lines()
        .skip(2)
        .filter_map(|l| {
            let (name, rest) = l.split_once(':')?;
            let level: f64 = rest
                .split_whitespace()
                .nth(2)?
                .trim_end_matches('.')
                .parse()
                .ok()?;
            Some((name.trim().to_owned(), level))
        })
        .collect()
}

fn resolv_servers(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| l.trim().strip_prefix("nameserver"))
        .map(|s| s.trim().to_owned())
        .collect()
}

fn ufw(root: &SystemRoot) -> Firewall {
    let conf = root.read("/etc/ufw/ufw.conf");
    let defaults = root.read("/etc/default/ufw").unwrap_or_default();
    let value = |text: &str, key: &str| {
        text.lines()
            .find_map(|l| l.trim().strip_prefix(&format!("{key}=")))
            .map(|v| v.trim_matches('"').to_owned())
    };
    Firewall {
        installed: conf.is_ok(),
        enabled: conf
            .as_deref()
            .is_ok_and(|c| value(c, "ENABLED").as_deref() == Some("yes")),
        incoming: value(&defaults, "DEFAULT_INPUT_POLICY"),
        outgoing: value(&defaults, "DEFAULT_OUTPUT_POLICY"),
        rules: root.read("/etc/ufw/user.rules").ok().map(|r| {
            r.lines()
                .filter_map(|l| l.strip_prefix("### tuple ### "))
                .map(str::to_owned)
                .collect()
        }),
    }
}

type Props = HashMap<String, OwnedValue>;

fn get<T: TryFrom<OwnedValue>>(p: &Props, key: &str) -> Option<T> {
    T::try_from(p.get(key)?.try_clone().ok()?).ok()
}

async fn props(bus: &zbus::Connection, path: &str, interface: &str) -> Option<Props> {
    let reply = bus
        .call_method(
            Some(NM),
            path,
            Some("org.freedesktop.DBus.Properties"),
            "GetAll",
            &(interface,),
        )
        .await
        .ok()?;
    reply.body().deserialize().ok()
}

/// Wi-Fi, connectivity, and VPNs from NetworkManager.
async fn network_manager(
    agent: &Agent,
    dbm: &HashMap<String, f64>,
) -> (Option<Wifi>, Option<String>, Vec<Vpn>) {
    if !agent.is_running(Bus::System, NM).await {
        return (None, None, Vec::new());
    }
    let Ok(bus) = agent.bus(Bus::System).await else {
        return (None, None, Vec::new());
    };
    let Some(nm) = props(&bus, "/org/freedesktop/NetworkManager", NM).await else {
        return (None, None, Vec::new());
    };
    let connectivity = get::<u32>(&nm, "Connectivity").map(|c| {
        match c {
            4 => "full",
            3 => "limited",
            2 => "portal",
            1 => "none",
            _ => "unknown",
        }
        .to_owned()
    });
    let mut wifi = None;
    for dev in get::<Vec<OwnedObjectPath>>(&nm, "Devices").unwrap_or_default() {
        let Some(d) = props(&bus, dev.as_str(), "org.freedesktop.NetworkManager.Device").await
        else {
            continue;
        };
        // Device type 2 is Wi-Fi; state 100 is "activated" (connected).
        if get::<u32>(&d, "DeviceType") != Some(2) || get::<u32>(&d, "State") != Some(100) {
            continue;
        }
        let interface: String = get(&d, "Interface").unwrap_or_default();
        let w = props(
            &bus,
            dev.as_str(),
            "org.freedesktop.NetworkManager.Device.Wireless",
        )
        .await
        .unwrap_or_default();
        let ap = match get::<OwnedObjectPath>(&w, "ActiveAccessPoint") {
            Some(p) if p.as_str() != "/" => {
                props(
                    &bus,
                    p.as_str(),
                    "org.freedesktop.NetworkManager.AccessPoint",
                )
                .await
            }
            _ => None,
        }
        .unwrap_or_default();
        wifi = Some(Wifi {
            ssid: get::<Vec<u8>>(&ap, "Ssid").map(|s| String::from_utf8_lossy(&s).into_owned()),
            strength: get(&ap, "Strength"),
            frequency_mhz: get(&ap, "Frequency"),
            bitrate_mbps: get::<u32>(&w, "Bitrate").map(|kbps| kbps / 1000),
            dbm: dbm.get(&interface).copied(),
            interface,
        });
        break;
    }
    // Saved VPN connections, and which are up.
    let mut active = Vec::new();
    for a in get::<Vec<OwnedObjectPath>>(&nm, "ActiveConnections").unwrap_or_default() {
        if let Some(p) = props(
            &bus,
            a.as_str(),
            "org.freedesktop.NetworkManager.Connection.Active",
        )
        .await
        {
            active.push(get::<String>(&p, "Id").unwrap_or_default());
        }
    }
    let mut vpns = Vec::new();
    if let Ok(reply) = bus
        .call_method(
            Some(NM),
            "/org/freedesktop/NetworkManager/Settings",
            Some("org.freedesktop.NetworkManager.Settings"),
            "ListConnections",
            &(),
        )
        .await
        && let Ok(paths) = reply.body().deserialize::<Vec<OwnedObjectPath>>()
    {
        for p in paths {
            let Ok(reply) = bus
                .call_method(
                    Some(NM),
                    p.as_str(),
                    Some("org.freedesktop.NetworkManager.Settings.Connection"),
                    "GetSettings",
                    &(),
                )
                .await
            else {
                continue;
            };
            let Ok(settings) = reply.body().deserialize::<HashMap<String, Props>>() else {
                continue;
            };
            let Some(c) = settings.get("connection") else {
                continue;
            };
            let kind: String = get(c, "type").unwrap_or_default();
            if matches!(kind.as_str(), "vpn" | "wireguard" | "tun") {
                let name: String = get(c, "id").unwrap_or_default();
                vpns.push(Vpn {
                    connected: active.contains(&name),
                    name,
                    kind,
                });
            }
        }
    }
    (wifi, connectivity, vpns)
}

/// Reads the details.
pub async fn read(agent: &Agent) -> NetDetails {
    let root = &agent.root;
    let mut links = parse_addresses(
        &command::output("ip", &["-j", "addr"], Duration::from_secs(5))
            .await
            .unwrap_or_default(),
    );
    for link in &mut links {
        let stat = |f: &str| {
            root.read(&format!("/sys/class/net/{}/statistics/{f}", link.name))
                .ok()
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(0)
        };
        link.rx = stat("rx_bytes");
        link.tx = stat("tx_bytes");
    }
    let dbm = wireless_dbm(&root.read("/proc/net/wireless").unwrap_or_default());
    let (wifi, connectivity, mut vpns) = network_manager(agent, &dbm).await;
    if links.iter().any(|l| l.name == "tailscale0") && !vpns.iter().any(|v| v.kind == "tailscale") {
        vpns.push(Vpn {
            name: "Tailscale".into(),
            kind: "tailscale".into(),
            connected: true,
        });
    }
    // With systemd-resolved, /etc/resolv.conf points at its stub; the real servers are elsewhere.
    let stub = resolv_servers(&root.read("/etc/resolv.conf").unwrap_or_default());
    let (resolver, dns_servers) = if stub.iter().any(|s| s == "127.0.0.53") {
        let upstream = root
            .read("/run/systemd/resolve/resolv.conf")
            .map(|t| resolv_servers(&t))
            .unwrap_or_default();
        ("systemd-resolved".to_owned(), upstream)
    } else {
        ("resolv.conf".to_owned(), stub)
    };
    NetDetails {
        gateway: default_gateway(&root.read("/proc/net/route").unwrap_or_default()),
        links,
        wifi,
        connectivity,
        resolver,
        dns_servers,
        vpns,
        firewall: ufw(root),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_addresses_gateway_and_signal() {
        let json = r#"[{"ifname":"lo","addr_info":[{"family":"inet","local":"127.0.0.1","prefixlen":8}]},{"ifname":"wlp0s20f3","addr_info":[{"family":"inet","local":"192.168.1.24","prefixlen":24},{"family":"inet6","local":"fe80::1","prefixlen":64}]}]"#;
        let links = parse_addresses(json);
        assert_eq!(links[1].ipv4, ["192.168.1.24/24"]);
        assert_eq!(links[1].ipv6, ["fe80::1/64"]);
        let route = "Iface\tDestination\tGateway\nwlp0s20f3\t00000000\t0101A8C0\t0003\ndocker0\t000011AC\t00000000\t0001\n";
        assert_eq!(default_gateway(route).as_deref(), Some("192.168.1.1"));
        let wireless = "Inter-| sta-|   Quality\n face | tus | link level noise\nwlp45s0: 0000   49.  -61.  -256        0\n";
        assert_eq!(wireless_dbm(wireless).get("wlp45s0"), Some(&-61.0));
    }
}
