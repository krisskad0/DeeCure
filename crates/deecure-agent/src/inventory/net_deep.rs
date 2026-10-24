// SPDX-License-Identifier: Apache-2.0

//! Network in depth for the Expert level (docs/01-product/expert-coverage.md, section 6).
//!
//! `net_deep`: the Wi-Fi link (channel, band, rate, signal in dBm, power saving), errors and
//! drops per interface, TCP and UDP counters (resends, failed connections), DNS per link
//! (server, DNSSEC, DNS over TLS), and routes. `tcp_traffic`: bytes of every open TCP
//! connection with its program, from `ss`. TCP only, and only connections open now: QUIC and
//! other UDP traffic are not counted, which the app says. No admin rights.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use super::command;
use crate::collect::SystemRoot;

const LIMIT: Duration = Duration::from_secs(3);

/// The Wi-Fi network in use on one interface.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct WifiLink {
    pub interface: String,
    pub ssid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_mhz: Option<u32>,
    /// `2.4 GHz`, `5 GHz`, or `6 GHz`, from the frequency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub band: Option<&'static str>,
    /// The network's top rate from the last scan (`nmcli` RATE), not the link speed now.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_mbps: Option<u32>,
    /// Signal as NetworkManager shows it (0-100) and in dBm from the kernel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_percent: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_dbm: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noise_dbm: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<String>,
    /// The NetworkManager connection's UUID, for the power saving control.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    /// Power saving, from NetworkManager's setting for this network or its global default.
    /// Missing when nothing sets it (the driver decides).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_save: Option<bool>,
}

/// Error and drop counters of one interface since start.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct IfaceErrors {
    pub name: String,
    pub rx_errors: u64,
    pub tx_errors: u64,
    pub rx_dropped: u64,
    pub tx_dropped: u64,
    pub collisions: u64,
    /// Times the link went down and up.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carrier_changes: Option<u64>,
}

/// DNS of one network link.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct DnsLink {
    pub link: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
    pub servers: Vec<String>,
    /// Used for names that no other link claims.
    pub default_route: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dnssec: Option<String>,
    pub dns_over_tls: bool,
}

/// One route.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Route {
    pub destination: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    /// The link is down, so the route is not used now.
    pub down: bool,
    /// `4` or `6`.
    pub family: u8,
}

/// What `net_deep` answers.
#[derive(Debug, Default, Serialize)]
pub struct NetDeep {
    /// When this was read, in ms since 1970.
    pub at: u64,
    pub wifi: Vec<WifiLink>,
    pub interfaces: Vec<IfaceErrors>,
    /// `/proc/net/snmp` TCP and UDP counters since start, such as `Tcp.RetransSegs`.
    pub counters: BTreeMap<String, u64>,
    /// Missing when systemd-resolved does not run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns: Option<Vec<DnsLink>>,
    pub routes: Vec<Route>,
    /// Routing rules beyond the three standard ones: VPNs such as WireGuard add them.
    pub extra_rules: usize,
}

/// One open TCP connection.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct TcpConnection {
    pub local: String,
    pub remote: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    /// Bytes received and bytes the other side confirmed, since it opened.
    pub received: u64,
    pub sent: u64,
    /// Bytes sent again because they were lost.
    pub resent: u64,
}

/// What `tcp_traffic` answers.
#[derive(Debug, Default, Serialize)]
pub struct TcpTraffic {
    pub at: u64,
    pub connections: Vec<TcpConnection>,
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// The band of a Wi-Fi frequency.
pub fn band(mhz: u32) -> Option<&'static str> {
    match mhz {
        2400..=2500 => Some("2.4 GHz"),
        5150..=5925 => Some("5 GHz"),
        5926..=7125 => Some("6 GHz"),
        _ => None,
    }
}

/// Splits one line of `nmcli -t` output: `:` separates fields, `\:` is a colon inside one.
pub fn nmcli_fields(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut field = String::new();
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => field.extend(chars.next()),
            ':' => out.push(std::mem::take(&mut field)),
            _ => field.push(c),
        }
    }
    out.push(field);
    out
}

/// Signal and noise in dBm from `/proc/net/wireless` (`-256` means not reported).
pub fn wireless_levels(text: &str) -> BTreeMap<String, (Option<i32>, Option<i32>)> {
    text.lines()
        .skip(2)
        .filter_map(|line| {
            let (name, rest) = line.split_once(':')?;
            let f: Vec<&str> = rest.split_whitespace().collect();
            let dbm = |s: Option<&&str>| {
                s.and_then(|v| v.trim_end_matches('.').parse::<i32>().ok())
                    .filter(|v| *v > -256 && *v < 0)
            };
            Some((name.trim().to_owned(), (dbm(f.get(2)), dbm(f.get(3)))))
        })
        .collect()
}

/// NetworkManager's global `wifi.powersave`: files in `/usr/lib` first, then `/etc`, by name;
/// a file in `/etc` replaces one with the same name in `/usr/lib`, and later files win.
fn global_powersave(root: &SystemRoot) -> Option<u32> {
    let mut files: BTreeMap<String, std::path::PathBuf> = BTreeMap::new();
    for dir in [
        "/usr/lib/NetworkManager/conf.d",
        "/etc/NetworkManager/conf.d",
    ] {
        for e in std::fs::read_dir(root.path(dir))
            .into_iter()
            .flatten()
            .flatten()
        {
            if let Some(n) = e.file_name().to_str().filter(|n| n.ends_with(".conf")) {
                files.insert(n.to_owned(), e.path());
            }
        }
    }
    let mut value = None;
    let mut main = vec![root.path("/etc/NetworkManager/NetworkManager.conf")];
    main.extend(files.into_values());
    for path in main {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let mut in_connection = false;
        for line in text.lines().map(str::trim) {
            if line.starts_with('[') {
                in_connection = line.starts_with("[connection");
            } else if in_connection
                && let Some(v) = line
                    .strip_prefix("wifi.powersave")
                    .and_then(|r| r.trim().strip_prefix('='))
                    .and_then(|v| v.trim().parse().ok())
            {
                value = Some(v);
            }
        }
    }
    value
}

/// `0` (use the default), `1` (ignore), `2` (off), `3` (on).
fn powersave_on(value: u32) -> Option<bool> {
    match value {
        2 => Some(false),
        3 => Some(true),
        _ => None,
    }
}

async fn wifi(root: &SystemRoot) -> Vec<WifiLink> {
    let Some(list) = command::output(
        "nmcli",
        &[
            "-t",
            "-f",
            "IN-USE,SSID,CHAN,FREQ,RATE,SIGNAL,SECURITY,DEVICE",
            "dev",
            "wifi",
            "list",
            "--rescan",
            "no",
        ],
        LIMIT,
    )
    .await
    else {
        return Vec::new();
    };
    let levels = root
        .read("/proc/net/wireless")
        .map(|t| wireless_levels(&t))
        .unwrap_or_default();
    let global = global_powersave(root);
    let mut out = Vec::new();
    for line in list.lines().filter(|l| l.starts_with('*')) {
        let f = nmcli_fields(line);
        let num = |i: usize| {
            f.get(i)
                .and_then(|v| v.split_whitespace().next())
                .and_then(|v| v.parse::<u32>().ok())
        };
        let interface = f.get(7).cloned().unwrap_or_default();
        let connection = command::output(
            "nmcli",
            &["-t", "-g", "GENERAL.CONNECTION", "dev", "show", &interface],
            LIMIT,
        )
        .await
        .map(|c| c.trim().to_owned());
        let uuid = command::output(
            "nmcli",
            &["-t", "-g", "GENERAL.CON-UUID", "dev", "show", &interface],
            LIMIT,
        )
        .await
        .map(|c| c.trim().to_owned())
        .filter(|u| u.len() == 36);
        let own = match &connection {
            Some(c) if !c.is_empty() => command::output(
                "nmcli",
                &[
                    "-t",
                    "-g",
                    "802-11-wireless.powersave",
                    "connection",
                    "show",
                    c,
                ],
                LIMIT,
            )
            .await
            .and_then(|v| {
                v.trim()
                    .split(' ')
                    .next()
                    .and_then(|n| n.parse::<u32>().ok())
            }),
            _ => None,
        };
        let setting = own.filter(|v| *v != 0).or(global);
        let (signal_dbm, noise_dbm) = levels.get(&interface).copied().unwrap_or((None, None));
        out.push(WifiLink {
            ssid: f.get(1).cloned().unwrap_or_default(),
            channel: num(2),
            frequency_mhz: num(3),
            band: num(3).and_then(band),
            rate_mbps: num(4),
            signal_percent: num(5),
            signal_dbm,
            noise_dbm,
            security: f.get(6).cloned().filter(|s| !s.is_empty()),
            power_save: setting.and_then(powersave_on),
            uuid,
            interface,
        });
    }
    out
}

fn interfaces(root: &SystemRoot) -> Vec<IfaceErrors> {
    let mut names: Vec<String> = std::fs::read_dir(root.path("/sys/class/net"))
        .map(|d| {
            d.flatten()
                .filter_map(|e| e.file_name().to_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    names.retain(|n| n != "lo");
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let stat = |f: &str| {
                root.read(&format!("/sys/class/net/{name}/statistics/{f}"))
                    .ok()
                    .and_then(|v| v.trim().parse().ok())
                    .unwrap_or(0)
            };
            IfaceErrors {
                rx_errors: stat("rx_errors"),
                tx_errors: stat("tx_errors"),
                rx_dropped: stat("rx_dropped"),
                tx_dropped: stat("tx_dropped"),
                collisions: stat("collisions"),
                carrier_changes: root
                    .read(&format!("/sys/class/net/{name}/carrier_changes"))
                    .ok()
                    .and_then(|v| v.trim().parse().ok()),
                name,
            }
        })
        .collect()
}

/// `/proc/net/snmp` as `Group.Name` → value, for the TCP and UDP groups.
pub fn snmp(text: &str) -> BTreeMap<String, u64> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = BTreeMap::new();
    for pair in lines.chunks(2) {
        let [names, values] = pair else { continue };
        let (Some((group, names)), Some((_, values))) =
            (names.split_once(':'), values.split_once(':'))
        else {
            continue;
        };
        if group != "Tcp" && group != "Udp" {
            continue;
        }
        for (n, v) in names.split_whitespace().zip(values.split_whitespace()) {
            if let Ok(v) = v.parse::<u64>() {
                out.insert(format!("{group}.{n}"), v);
            }
        }
    }
    out
}

/// The links of `resolvectl status`, with their DNS servers and protocols.
pub fn parse_resolvectl(text: &str) -> Vec<DnsLink> {
    let mut out: Vec<DnsLink> = Vec::new();
    let mut global_dot = false;
    let mut in_global = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line == "Global" {
            in_global = true;
            continue;
        }
        if let Some(rest) = line.strip_prefix("Link ") {
            in_global = false;
            let name = rest
                .split_once('(')
                .map(|(_, n)| n.trim_end_matches(')').to_owned())
                .unwrap_or_else(|| rest.to_owned());
            out.push(DnsLink {
                link: name,
                dns_over_tls: global_dot,
                ..Default::default()
            });
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        if key == "Protocols" {
            let dot = value.split_whitespace().any(|p| {
                p == "+DNSOverTLS"
                    || p.starts_with("DNSOverTLS=yes")
                    || p.starts_with("DNSOverTLS=opportunistic")
            });
            let dnssec = value
                .split_whitespace()
                .find_map(|p| p.strip_prefix("DNSSEC="))
                .map(str::to_owned);
            if in_global {
                global_dot = dot;
            } else if let Some(link) = out.last_mut() {
                link.dns_over_tls = dot;
                link.dnssec = dnssec;
                link.default_route = value.split_whitespace().any(|p| p == "+DefaultRoute");
            }
        } else if let Some(link) = out.last_mut().filter(|_| !in_global) {
            match key {
                "Current DNS Server" => link.current = Some(value.to_owned()),
                "DNS Servers" => {
                    link.servers = value.split_whitespace().map(str::to_owned).collect()
                }
                _ => {}
            }
        }
    }
    out
}

fn routes(json: &str, family: u8) -> Vec<Route> {
    let Ok(Value::Array(items)) = serde_json::from_str::<Value>(json) else {
        return Vec::new();
    };
    items
        .iter()
        .map(|r| Route {
            destination: r["dst"].as_str().unwrap_or("?").to_owned(),
            gateway: r["gateway"].as_str().map(str::to_owned),
            device: r["dev"].as_str().map(str::to_owned),
            metric: r["metric"].as_u64(),
            protocol: r["protocol"].as_str().map(str::to_owned),
            down: r["flags"]
                .as_array()
                .is_some_and(|f| f.iter().any(|x| x == "linkdown")),
            family,
        })
        .collect()
}

/// Reads `net_deep`.
pub async fn read(root: &SystemRoot) -> NetDeep {
    let rules = command::output("ip", &["-j", "rule"], LIMIT)
        .await
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| v.as_array().map(Vec::len))
        .unwrap_or(0);
    let mut all_routes = routes(
        &command::output("ip", &["-j", "route"], LIMIT)
            .await
            .unwrap_or_default(),
        4,
    );
    all_routes.extend(routes(
        &command::output("ip", &["-j", "-6", "route"], LIMIT)
            .await
            .unwrap_or_default(),
        6,
    ));
    NetDeep {
        at: now_ms(),
        wifi: wifi(root).await,
        interfaces: interfaces(root),
        counters: root
            .read("/proc/net/snmp")
            .map(|t| snmp(&t))
            .unwrap_or_default(),
        dns: command::output("resolvectl", &["status", "--no-pager"], LIMIT)
            .await
            .map(|t| parse_resolvectl(&t)),
        routes: all_routes,
        // The three standard rules are local, main, and default.
        extra_rules: rules.saturating_sub(3),
    }
}

/// Parses `ss -tinpH state established`: a connection line, then its info line.
pub fn parse_ss(text: &str) -> Vec<TcpConnection> {
    let mut out: Vec<TcpConnection> = Vec::new();
    for line in text.lines() {
        if line.starts_with(char::is_whitespace) {
            let Some(c) = out.last_mut() else { continue };
            for field in line.split_whitespace() {
                let Some((k, v)) = field.split_once(':') else {
                    continue;
                };
                let Ok(v) = v.parse::<u64>() else { continue };
                match k {
                    "bytes_received" => c.received = v,
                    "bytes_acked" => c.sent = v,
                    "bytes_retrans" => c.resent = v,
                    _ => {}
                }
            }
            continue;
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        let (Some(local), Some(remote)) = (f.get(2), f.get(3)) else {
            continue;
        };
        let users = line.find("users:((").map(|i| &line[i + 8..]);
        let program = users.and_then(|u| u.split('"').nth(1)).map(str::to_owned);
        let pid = users
            .and_then(|u| u.split("pid=").nth(1))
            .and_then(|p| p.split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|p| p.parse().ok());
        out.push(TcpConnection {
            local: local.to_string(),
            remote: remote.to_string(),
            program,
            pid,
            ..Default::default()
        });
    }
    out
}

/// Reads `tcp_traffic`.
pub async fn read_tcp() -> TcpTraffic {
    let text = command::output("ss", &["-tinpH", "state", "established"], LIMIT)
        .await
        .unwrap_or_default();
    TcpTraffic {
        at: now_ms(),
        connections: parse_ss(&text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn nmcli_colons_and_bands() {
        assert_eq!(
            nmcli_fields("*:Home\\:Net:36:5180 MHz"),
            ["*", "Home:Net", "36", "5180 MHz"]
        );
        assert_eq!(band(2437), Some("2.4 GHz"));
        assert_eq!(band(5180), Some("5 GHz"));
        assert_eq!(band(6115), Some("6 GHz"));
        assert_eq!(band(60000), None);
    }

    #[test]
    fn wireless_levels_leave_out_unreported_noise() {
        let t = "Inter-| sta-|   Quality        |   Discarded\n face | tus | link level noise |  nwid\nwlp45s0: 0000   52.  -58.  -256        0      0\n";
        assert_eq!(wireless_levels(t)["wlp45s0"], (Some(-58), None));
    }

    #[test]
    fn snmp_counters_by_group() {
        let t = "Ip: Forwarding\nIp: 1\nTcp: ActiveOpens RetransSegs OutSegs\nTcp: 15993 8523 978611\nUdp: InDatagrams InErrors\nUdp: 10 2\n";
        let s = snmp(t);
        assert_eq!(s["Tcp.RetransSegs"], 8523);
        assert_eq!(s["Udp.InErrors"], 2);
        assert!(!s.contains_key("Ip.Forwarding"));
    }

    #[test]
    fn resolvectl_links() {
        let t = "Global\n         Protocols: -LLMNR -mDNS +DNSOverTLS DNSSEC=no/unsupported\n  resolv.conf mode: stub\n\nLink 3 (wlp45s0)\n    Current Scopes: DNS\n         Protocols: +DefaultRoute -LLMNR -mDNS -DNSOverTLS DNSSEC=yes/supported\nCurrent DNS Server: 192.168.1.1\n       DNS Servers: 192.168.1.1 1.1.1.1\n\nLink 4 (docker0)\n    Current Scopes: none\n";
        let l = parse_resolvectl(t);
        assert_eq!(l.len(), 2);
        assert_eq!(l[0].link, "wlp45s0");
        assert_eq!(l[0].current.as_deref(), Some("192.168.1.1"));
        assert_eq!(l[0].servers, ["192.168.1.1", "1.1.1.1"]);
        assert!(l[0].default_route && !l[0].dns_over_tls);
        assert_eq!(l[0].dnssec.as_deref(), Some("yes/supported"));
        assert!(
            l[1].dns_over_tls,
            "a link without its own protocols line keeps the global DNS over TLS"
        );
    }

    #[test]
    fn ss_connections_with_their_bytes() {
        let t = "0      0      192.168.1.13:36524    57.144.43.32:5222  users:((\"chrome\",pid=5337,fd=35))\n\t cubic wscale:8,10 rto:223 bytes_sent:60261 bytes_retrans:83 bytes_acked:60179 bytes_received:186896 segs_out:1216\n0 0 [::1]:631 [::1]:40000\n\t cubic bytes_acked:1 bytes_received:2\n";
        let c = parse_ss(t);
        assert_eq!(c.len(), 2);
        assert_eq!(
            (c[0].program.as_deref(), c[0].pid),
            (Some("chrome"), Some(5337))
        );
        assert_eq!(
            (c[0].received, c[0].sent, c[0].resent),
            (186_896, 60_179, 83)
        );
        assert_eq!(
            (c[1].program.as_deref(), c[1].received),
            (None, 2),
            "other users' connections have no program"
        );
    }

    #[test]
    fn routes_and_interface_errors() {
        let r = routes(
            r#"[{"dst":"default","gateway":"192.168.1.1","dev":"wlp45s0","protocol":"dhcp","metric":600,"flags":[]},{"dst":"172.17.0.0/16","dev":"docker0","flags":["linkdown"]}]"#,
            4,
        );
        assert_eq!(r[0].gateway.as_deref(), Some("192.168.1.1"));
        assert!(!r[0].down && r[1].down);
        let root = fixture_root(
            "net-deep",
            &[
                ("/sys/class/net/wlp45s0/statistics/rx_errors", "3\n"),
                ("/sys/class/net/wlp45s0/statistics/rx_dropped", "7\n"),
                ("/sys/class/net/wlp45s0/carrier_changes", "4\n"),
                ("/sys/class/net/lo/statistics/rx_errors", "0\n"),
                (
                    "/usr/lib/NetworkManager/conf.d/default-wifi-powersave-on.conf",
                    "[connection]\nwifi.powersave = 3\n",
                ),
                (
                    "/etc/NetworkManager/conf.d/default-wifi-powersave-on.conf",
                    "[connection]\nwifi.powersave = 2\n",
                ),
            ],
        );
        let i = interfaces(&root);
        assert_eq!(i.len(), 1, "loopback is left out");
        assert_eq!(
            (i[0].rx_errors, i[0].rx_dropped, i[0].carrier_changes),
            (3, 7, Some(4))
        );
        assert_eq!(
            global_powersave(&root),
            Some(2),
            "a file in /etc replaces the one in /usr/lib"
        );
        assert_eq!(powersave_on(3), Some(true));
        assert_eq!(powersave_on(0), None);
    }
}
