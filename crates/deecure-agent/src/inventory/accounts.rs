// SPDX-License-Identifier: Apache-2.0

//! People and access: users, groups, sessions, sign-ins, sudo use, and SSH keys.
//! From `/etc/passwd`, `/etc/group`, logind (read-only D-Bus), `last`, the journal, and `~/.ssh`.
//! Password hashes (`/etc/shadow`) are root-only and are never read.

use std::collections::HashMap;
use std::time::{Duration, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

use super::command;
use crate::collect::SystemRoot;
use crate::context::{Agent, Bus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct User {
    pub name: String,
    pub full_name: String,
    pub uid: u32,
    pub groups: Vec<String>,
    /// In `sudo` or `admin`: can run commands as root.
    pub admin: bool,
    pub shell: String,
    pub home: String,
    /// Newest sign-in from `last`, in milliseconds since 1970.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sign_in: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Group {
    pub name: String,
    pub gid: u32,
    pub members: Vec<String>,
    /// What membership allows, in plain words, when it matters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privilege: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Session {
    pub id: String,
    pub user: String,
    pub seat: String,
    /// `wayland`, `x11`, `tty`, or `unspecified`.
    pub kind: String,
    /// `user`, `greeter`, `background`, …
    pub class: String,
    pub state: String,
    pub remote: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<i64>,
}

/// A sign-in or a sudo use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoginEvent {
    pub at: i64,
    pub user: String,
    /// `sign-in` or `sudo`.
    pub event: String,
    /// The terminal or the command.
    pub source: String,
    pub success: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SshKey {
    pub file: String,
    /// ED25519, RSA, ECDSA, …
    pub kind: String,
    pub bits: u32,
    pub fingerprint: String,
    pub created: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Ssh {
    pub server_installed: bool,
    /// From `/etc/ssh/sshd_config` when the server is installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_login: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_login: Option<String>,
    pub authorized_keys: usize,
    pub client_keys: Vec<SshKey>,
}

/// The `accounts` inventory.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AccountsInfo {
    pub users: Vec<User>,
    pub groups: Vec<Group>,
    pub sessions: Vec<Session>,
    /// Last 30 days, newest first.
    pub events: Vec<LoginEvent>,
    pub ssh: Ssh,
}

/// What a group lets its members do.
fn privilege(group: &str) -> Option<&'static str> {
    Some(match group {
        "sudo" | "admin" | "wheel" => "Admin (sudo)",
        "docker" => "Root-equivalent (Docker)",
        "lxd" | "incus-admin" => "Root-equivalent (containers)",
        "libvirt" | "kvm" => "Run virtual machines",
        "adm" | "systemd-journal" => "Read system logs",
        "lpadmin" => "Manage printers",
        "dialout" => "Serial ports",
        "plugdev" => "Mount removable devices",
        "netdev" => "Manage network devices",
        "wireshark" => "Capture network traffic",
        _ => return None,
    })
}

/// Users who can sign in: root and human accounts (UID 1000–59999) with a real shell.
pub fn users(passwd: &str, group: &str) -> (Vec<User>, Vec<Group>) {
    let mut groups: Vec<Group> = group
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split(':').collect();
            Some(Group {
                name: (*f.first()?).to_owned(),
                gid: f.get(2)?.parse().ok()?,
                members: f
                    .get(3)
                    .map(|m| {
                        m.split(',')
                            .filter(|x| !x.is_empty())
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default(),
                privilege: None,
            })
        })
        .collect();
    let by_gid: HashMap<u32, String> = groups.iter().map(|g| (g.gid, g.name.clone())).collect();
    let users: Vec<User> = passwd
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split(':').collect();
            let uid: u32 = f.get(2)?.parse().ok()?;
            let shell = (*f.get(6)?).to_owned();
            let human = uid == 0 || (1000..60000).contains(&uid);
            if !human || shell.ends_with("nologin") || shell.ends_with("false") {
                return None;
            }
            let name = (*f.first()?).to_owned();
            let gid: u32 = f.get(3)?.parse().ok()?;
            let mut member_of: Vec<String> = by_gid.get(&gid).cloned().into_iter().collect();
            member_of.extend(
                groups
                    .iter()
                    .filter(|g| g.members.contains(&name))
                    .map(|g| g.name.clone()),
            );
            member_of.dedup();
            Some(User {
                admin: uid == 0
                    || member_of
                        .iter()
                        .any(|g| matches!(g.as_str(), "sudo" | "admin" | "wheel")),
                full_name: f
                    .get(4)
                    .map(|g| g.split(',').next().unwrap_or("").to_owned())
                    .unwrap_or_default(),
                home: (*f.get(5)?).to_owned(),
                groups: member_of,
                last_sign_in: None,
                name,
                uid,
                shell,
            })
        })
        .collect();
    groups.retain(|g| !g.members.is_empty());
    for g in &mut groups {
        g.privilege = privilege(&g.name).map(str::to_owned);
    }
    groups.sort_by(|a, b| {
        b.privilege
            .is_some()
            .cmp(&a.privilege.is_some())
            .then_with(|| a.name.cmp(&b.name))
    });
    (users, groups)
}

/// Reads `last -w -F --time-format iso` sign-ins: `riya tty2 tty2 2026-10-22T10:06:21+05:30 …`.
pub fn parse_last(text: &str) -> Vec<LoginEvent> {
    text.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            let user = *f.first()?;
            if matches!(user, "reboot" | "shutdown" | "wtmp" | "") {
                return None;
            }
            let at = f.iter().find_map(|w| parse_iso(w))?;
            Some(LoginEvent {
                at,
                user: user.to_owned(),
                event: "sign-in".into(),
                source: (*f.get(1)?).to_owned(),
                success: true,
            })
        })
        .collect()
}

/// "2026-10-22T10:06:21+05:30" → milliseconds since 1970.
pub fn parse_iso(text: &str) -> Option<i64> {
    if text.len() < 25 || text.as_bytes().get(10) != Some(&b'T') {
        return None;
    }
    let n = |r: std::ops::Range<usize>| text.get(r)?.parse::<i64>().ok();
    let (y, mo, d, h, mi, s) = (
        n(0..4)?,
        n(5..7)?,
        n(8..10)?,
        n(11..13)?,
        n(14..16)?,
        n(17..19)?,
    );
    let sign = if text.as_bytes()[19] == b'-' { -1 } else { 1 };
    let offset = sign * (n(20..22)? * 3600 + n(23..25)? * 60);
    // Days from civil date (Howard Hinnant's algorithm).
    let (y2, m2) = if mo <= 2 {
        (y - 1, mo + 9)
    } else {
        (y, mo - 3)
    };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let doy = (153 * m2 + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some((days * 86_400 + h * 3600 + mi * 60 + s - offset) * 1000)
}

/// Sudo uses from journal lines (`SYSLOG_IDENTIFIER=sudo`).
pub fn parse_sudo(lines: &str) -> Vec<LoginEvent> {
    lines
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter_map(|e| {
            let at = e
                .get("__REALTIME_TIMESTAMP")?
                .as_str()?
                .parse::<i64>()
                .ok()?
                / 1000;
            let m = e.get("MESSAGE")?.as_str()?;
            let (user, rest) = m.split_once(" : ")?;
            let user = user.trim().to_owned();
            if rest.contains("incorrect password attempt") {
                let command = rest.split("COMMAND=").nth(1).unwrap_or("").to_owned();
                return Some(LoginEvent {
                    at,
                    user,
                    event: "sudo".into(),
                    source: command,
                    success: false,
                });
            }
            let command = rest.split("COMMAND=").nth(1)?.to_owned();
            Some(LoginEvent {
                at,
                user,
                event: "sudo".into(),
                source: command,
                success: true,
            })
        })
        .collect()
}

/// Reads `ssh-keygen -lf` output: `256 SHA256:k3F…Qw8 riya@laptop (ED25519)`.
pub fn parse_fingerprint(text: &str) -> Option<(u32, String, String)> {
    let f: Vec<&str> = text.split_whitespace().collect();
    let kind = f.last()?.trim_matches(|c| c == '(' || c == ')').to_owned();
    Some((f.first()?.parse().ok()?, (*f.get(1)?).to_owned(), kind))
}

type Props = HashMap<String, OwnedValue>;

async fn sessions(agent: &Agent) -> Vec<Session> {
    const LOGIN: &str = "org.freedesktop.login1";
    if !agent.is_running(Bus::System, LOGIN).await {
        return Vec::new();
    }
    let Ok(bus) = agent.bus(Bus::System).await else {
        return Vec::new();
    };
    let Ok(reply) = bus
        .call_method(
            Some(LOGIN),
            "/org/freedesktop/login1",
            Some("org.freedesktop.login1.Manager"),
            "ListSessions",
            &(),
        )
        .await
    else {
        return Vec::new();
    };
    let Ok(list) = reply
        .body()
        .deserialize::<Vec<(String, u32, String, String, OwnedObjectPath)>>()
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (id, _, user, seat, path) in list {
        let props: Props = match bus
            .call_method(
                Some(LOGIN),
                path.as_str(),
                Some("org.freedesktop.DBus.Properties"),
                "GetAll",
                &("org.freedesktop.login1.Session",),
            )
            .await
        {
            Ok(r) => r.body().deserialize().unwrap_or_default(),
            Err(_) => Props::new(),
        };
        let text = |k: &str| {
            props
                .get(k)
                .and_then(|v| v.try_clone().ok())
                .and_then(|v| String::try_from(v).ok())
                .unwrap_or_default()
        };
        out.push(Session {
            kind: text("Type"),
            class: text("Class"),
            state: text("State"),
            remote: props
                .get("Remote")
                .and_then(|v| bool::try_from(v).ok())
                .unwrap_or(false),
            since: props
                .get("Timestamp")
                .and_then(|v| u64::try_from(v).ok())
                .filter(|t| *t > 0)
                .and_then(|t| i64::try_from(t / 1000).ok()),
            id,
            user,
            seat,
        });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

async fn ssh(root: &SystemRoot, home: Option<&std::path::Path>) -> Ssh {
    let server_installed = root.path("/usr/sbin/sshd").exists();
    let config = root.read("/etc/ssh/sshd_config").unwrap_or_default();
    let setting = |key: &str| {
        config
            .lines()
            .map(str::trim)
            .filter(|l| !l.starts_with('#'))
            .find_map(|l| l.strip_prefix(key))
            .map(|v| v.trim().to_owned())
    };
    let mut ssh = Ssh {
        server_installed,
        // OpenSSH's defaults when a setting is not written: passwords allowed, root by key only.
        password_login: server_installed
            .then(|| setting("PasswordAuthentication").is_none_or(|v| v == "yes")),
        root_login: server_installed
            .then(|| setting("PermitRootLogin").unwrap_or_else(|| "prohibit-password".into())),
        ..Ssh::default()
    };
    let Some(home) = home else { return ssh };
    let dir = home.join(".ssh");
    ssh.authorized_keys = std::fs::read_to_string(dir.join("authorized_keys"))
        .map(|t| {
            t.lines()
                .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                .count()
        })
        .unwrap_or(0);
    let mut pubs: Vec<_> = std::fs::read_dir(&dir)
        .map(|d| {
            d.flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "pub"))
                .collect()
        })
        .unwrap_or_default();
    pubs.sort();
    for p in pubs {
        let Some(path) = p.to_str() else { continue };
        let Some((bits, fingerprint, kind)) =
            command::output("ssh-keygen", &["-lf", path], Duration::from_secs(5))
                .await
                .as_deref()
                .and_then(parse_fingerprint)
        else {
            continue;
        };
        let created = std::fs::metadata(&p)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_millis() as i64);
        ssh.client_keys.push(SshKey {
            file: p
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default(),
            kind,
            bits,
            fingerprint,
            created,
        });
    }
    ssh
}

/// Reads the accounts inventory.
pub async fn read(agent: &Agent) -> AccountsInfo {
    let root = &agent.root;
    let (mut users, groups) = users(
        &root.read("/etc/passwd").unwrap_or_default(),
        &root.read("/etc/group").unwrap_or_default(),
    );
    let mut events = parse_last(
        &command::output(
            "last",
            &["-w", "-F", "--time-format", "iso", "-n", "300"],
            Duration::from_secs(10),
        )
        .await
        .unwrap_or_default(),
    );
    for u in &mut users {
        u.last_sign_in = events
            .iter()
            .filter(|e| e.user == u.name)
            .map(|e| e.at)
            .max();
    }
    let sudo = command::output(
        "journalctl",
        &[
            "-o",
            "json",
            "--no-pager",
            "-q",
            "--since",
            "-30d",
            "--output-fields=MESSAGE",
            "SYSLOG_IDENTIFIER=sudo",
        ],
        Duration::from_secs(15),
    )
    .await
    .unwrap_or_default();
    events.extend(parse_sudo(&sudo));
    let month = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
        - 30 * 86_400_000;
    events.retain(|e| e.at >= month);
    events.sort_by_key(|e| std::cmp::Reverse(e.at));
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    AccountsInfo {
        users,
        groups,
        sessions: sessions(agent).await,
        events,
        ssh: ssh(root, home.as_deref()).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_people_and_their_privileges() {
        let passwd = "root:x:0:0:root:/root:/bin/bash\ndaemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin\nriya:x:1000:1000:Riya,,,:/home/riya:/bin/bash\nguest:x:1001:1001:Guest:/home/guest:/bin/bash\nnobody:x:65534:65534:nobody:/:/usr/sbin/nologin\n";
        let group = "root:x:0:\nsudo:x:27:riya\ndocker:x:999:riya\nriya:x:1000:\nguest:x:1001:\nplugdev:x:46:riya,guest\n";
        let (users, groups) = users(passwd, group);
        assert_eq!(
            users
                .iter()
                .map(|u| (u.name.as_str(), u.admin))
                .collect::<Vec<_>>(),
            [("root", true), ("riya", true), ("guest", false)]
        );
        assert_eq!(users[1].groups, ["riya", "sudo", "docker", "plugdev"]);
        assert_eq!(users[1].full_name, "Riya");
        assert_eq!(groups[0].name, "docker");
        assert_eq!(
            groups
                .iter()
                .find(|g| g.name == "sudo")
                .and_then(|g| g.privilege.as_deref()),
            Some("Admin (sudo)")
        );
    }

    #[test]
    fn parses_times_and_tools() {
        assert_eq!(parse_iso("1970-01-01T05:30:00+05:30"), Some(0));
        assert_eq!(
            parse_iso("2026-10-22T10:06:21+05:30"),
            Some(1_792_643_781_000)
        );
        let last = "krishnak tty2         tty2             2026-10-22T10:06:21+05:30   still logged in\nreboot   system boot  7.0.0-38-generic 2026-10-22T10:06:06+05:30   still running\n\nwtmp begins 2026-08-01T09:00:00+05:30\n";
        let e = parse_last(last);
        assert_eq!(e.len(), 1);
        assert_eq!(
            (e[0].user.as_str(), e[0].source.as_str()),
            ("krishnak", "tty2")
        );
        let sudo = r#"{"__REALTIME_TIMESTAMP":"2000","MESSAGE":"riya : TTY=pts/0 ; PWD=/home/riya ; USER=root ; COMMAND=/usr/bin/apt update"}
{"__REALTIME_TIMESTAMP":"3000","MESSAGE":"riya : 3 incorrect password attempts ; TTY=pts/0 ; PWD=/home/riya ; USER=root ; COMMAND=/usr/bin/su"}
{"__REALTIME_TIMESTAMP":"4000","MESSAGE":"pam_unix(sudo:session): session opened for user root"}"#;
        let s = parse_sudo(sudo);
        assert_eq!(
            s.iter()
                .map(|e| (e.source.as_str(), e.success))
                .collect::<Vec<_>>(),
            [("/usr/bin/apt update", true), ("/usr/bin/su", false)]
        );
        assert_eq!(
            parse_fingerprint("256 SHA256:k3FQw8 riya@laptop (ED25519)\n"),
            Some((256, "SHA256:k3FQw8".into(), "ED25519".into()))
        );
    }
}
