// SPDX-License-Identifier: Apache-2.0

//! Open sockets from `/proc/net/{tcp,tcp6,udp,udp6}`, with the program that owns each one
//! when it is the user's own (other users' processes cannot be looked into).

use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

use serde::Serialize;

use crate::collect::SystemRoot;

/// One socket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Socket {
    /// `tcp` or `udp`.
    pub proto: String,
    pub local: String,
    pub local_port: u16,
    pub remote: String,
    /// ESTAB, LISTEN, UNCONN, TIME-WAIT, …
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process: Option<String>,
    /// Who can reach a listening socket: `local` (this computer), `all` (every network), or `network` (one address).
    pub exposure: String,
}

const STATES: &[(&str, &str)] = &[
    ("01", "ESTAB"),
    ("02", "SYN-SENT"),
    ("03", "SYN-RECV"),
    ("04", "FIN-WAIT-1"),
    ("05", "FIN-WAIT-2"),
    ("06", "TIME-WAIT"),
    ("07", "UNCONN"),
    ("08", "CLOSE-WAIT"),
    ("09", "LAST-ACK"),
    ("0A", "LISTEN"),
    ("0B", "CLOSING"),
];

/// "0100007F:0277" (IPv4) or a 32-hex-digit IPv6 address, as the kernel writes them.
pub fn address(text: &str) -> Option<(String, u16)> {
    let (ip, port) = text.split_once(':')?;
    let port = u16::from_str_radix(port, 16).ok()?;
    let ip = if ip.len() == 8 {
        Ipv4Addr::from(u32::from_str_radix(ip, 16).ok()?.to_le_bytes()).to_string()
    } else {
        // Four 32-bit words, each in little-endian order.
        let mut bytes = [0u8; 16];
        for (i, chunk) in ip.as_bytes().chunks(8).enumerate().take(4) {
            let word = u32::from_str_radix(std::str::from_utf8(chunk).ok()?, 16).ok()?;
            bytes[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        let v6 = Ipv6Addr::from(bytes);
        v6.to_ipv4_mapped()
            .map_or_else(|| v6.to_string(), |v4| v4.to_string())
    };
    Some((ip, port))
}

fn exposure(ip: &str) -> &'static str {
    if ip.starts_with("127.") || ip == "::1" {
        "local"
    } else if ip == "0.0.0.0" || ip == "::" {
        "all"
    } else {
        "network"
    }
}

/// Socket inode → program name, for processes this user may look into.
fn owners(root: &SystemRoot) -> HashMap<u64, String> {
    let mut out = HashMap::new();
    let Ok(procs) = std::fs::read_dir(root.path("/proc")) else {
        return out;
    };
    for p in procs.flatten() {
        let name = p.file_name();
        if !name.to_string_lossy().chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let Ok(fds) = std::fs::read_dir(p.path().join("fd")) else {
            continue;
        };
        let comm = std::fs::read_to_string(p.path().join("comm"))
            .map(|c| c.trim().to_owned())
            .unwrap_or_default();
        for fd in fds.flatten() {
            if let Ok(target) = std::fs::read_link(fd.path())
                && let Some(inode) = target
                    .to_str()
                    .and_then(|t| t.strip_prefix("socket:["))
                    .and_then(|t| t.strip_suffix(']'))
                    .and_then(|t| t.parse().ok())
            {
                out.insert(inode, comm.clone());
            }
        }
    }
    out
}

/// Reads one `/proc/net` table.
pub fn parse(text: &str, proto: &str, names: &HashMap<u64, String>) -> Vec<Socket> {
    text.lines()
        .skip(1)
        .filter_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            let (local, local_port) = address(f.get(1)?)?;
            let (remote, remote_port) = address(f.get(2)?)?;
            let code = *f.get(3)?;
            let state = STATES
                .iter()
                .find(|(c, _)| *c == code)
                .map_or("?", |(_, s)| s)
                .to_owned();
            let inode: u64 = f.get(9)?.parse().ok()?;
            Some(Socket {
                proto: proto.to_owned(),
                exposure: exposure(&local).to_owned(),
                local: format!("{local}:{local_port}"),
                local_port,
                remote: if remote_port == 0 {
                    "*".into()
                } else {
                    format!("{remote}:{remote_port}")
                },
                state,
                process: names.get(&inode).cloned(),
            })
        })
        .collect()
}

/// All sockets, listening ones first.
pub fn read(root: &SystemRoot) -> Vec<Socket> {
    let names = owners(root);
    let mut all = Vec::new();
    for (file, proto) in [
        ("tcp", "tcp"),
        ("tcp6", "tcp"),
        ("udp", "udp"),
        ("udp6", "udp"),
    ] {
        all.extend(parse(
            &root.read(&format!("/proc/net/{file}")).unwrap_or_default(),
            proto,
            &names,
        ));
    }
    all.sort_by(|a, b| {
        (a.state != "LISTEN")
            .cmp(&(b.state != "LISTEN"))
            .then(a.local_port.cmp(&b.local_port))
    });
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_kernel_addresses() {
        assert_eq!(address("0100007F:0277"), Some(("127.0.0.1".into(), 631)));
        assert_eq!(
            address("00000000000000000000000000000000:1F90"),
            Some(("::".into(), 8080))
        );
        assert_eq!(
            address("0000000000000000FFFF00000100007F:0035"),
            Some(("127.0.0.1".into(), 53))
        );
        assert_eq!(
            address("00000000000000000000000001000000:0016"),
            Some(("::1".into(), 22))
        );
    }

    #[test]
    fn parses_sockets_with_owners() {
        let text = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 0100007F:0277 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 111 1
   1: 1801A8C0:C8E8 0EB7FA8E:01BB 01 00000000:00000000 00:00000000 00000000  1000        0 222 1
";
        let names: HashMap<u64, String> = [(222, "chrome".to_owned())].into();
        let s = parse(text, "tcp", &names);
        assert_eq!(
            (
                s[0].state.as_str(),
                s[0].exposure.as_str(),
                s[0].process.as_deref()
            ),
            ("LISTEN", "local", None)
        );
        assert_eq!(
            (
                s[1].local.as_str(),
                s[1].remote.as_str(),
                s[1].process.as_deref()
            ),
            ("192.168.1.24:51432", "142.250.183.14:443", Some("chrome"))
        );
    }
}
