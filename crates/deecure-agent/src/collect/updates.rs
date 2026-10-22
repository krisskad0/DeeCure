// SPDX-License-Identifier: Apache-2.0

//! Pending package updates, from `apt list --upgradable`.
//!
//! The command only reads apt's package lists: it needs no admin rights, takes no lock,
//! and changes nothing. It runs at most once an hour; between runs the last answer is
//! repeated, so rules always have fresh facts.

use std::io;
use std::process::Stdio;
use std::time::{Duration, Instant, UNIX_EPOCH};

use deecure_core::{Snapshot, SnapshotKind, Timestamp, Unit};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::{Sample, SystemRoot, metric};

/// How often apt is asked.
const EVERY: Duration = Duration::from_secs(3600);
/// apt should answer in a second or two; give up after this.
const TIMEOUT: Duration = Duration::from_secs(120);

/// One package with a newer version available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Upgradable {
    /// Package name, such as `openssl`.
    pub name: String,
    /// The version installed now.
    pub installed: String,
    /// The version an upgrade installs.
    pub candidate: String,
    /// Where the new version comes from, such as `noble-updates,noble-security`.
    pub origin: String,
    /// True when the new version comes from a security archive.
    pub security: bool,
}

/// The `updates` inventory.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdatesInfo {
    /// Packages with a newer version, security first, then by name.
    pub pending: Vec<Upgradable>,
    /// When apt last refreshed its package lists successfully.
    pub lists_updated_at: Option<Timestamp>,
    /// True when unattended-upgrades installs security updates on its own.
    pub automatic_security: bool,
}

/// Reads `apt list --upgradable` output. Lines look like
/// `openssl/noble-updates,noble-security 3.0.13-0ubuntu3.5 amd64 [upgradable from: 3.0.13-0ubuntu3.4]`.
pub fn parse_upgradable(text: &str) -> Vec<Upgradable> {
    let mut out: Vec<Upgradable> = text
        .lines()
        .filter_map(|line| {
            let (name, rest) = line.split_once('/')?;
            let mut words = rest.split_whitespace();
            let origin = words.next()?.to_owned();
            let candidate = words.next()?.to_owned();
            let installed = rest
                .split_once("[upgradable from: ")?
                .1
                .trim_end_matches(']')
                .trim()
                .to_owned();
            Some(Upgradable {
                name: name.to_owned(),
                installed,
                candidate,
                security: origin.split(',').any(|o| o.ends_with("-security")),
                origin,
            })
        })
        .collect();
    out.sort_by(|a, b| {
        b.security
            .cmp(&a.security)
            .then_with(|| a.name.cmp(&b.name))
    });
    out
}

/// Reads `/etc/apt/apt.conf.d/20auto-upgrades`: `APT::Periodic::Unattended-Upgrade "1";`
pub fn automatic_security(text: &str) -> bool {
    text.lines().any(|l| {
        let l = l.trim();
        l.starts_with("APT::Periodic::Unattended-Upgrade") && l.contains("\"1\"")
    })
}

/// Asks apt once an hour and repeats the last answer in between.
#[derive(Debug, Default)]
pub struct Updates {
    last: Option<(Instant, UpdatesInfo)>,
}

impl Updates {
    /// The name used in logs.
    pub const NAME: &'static str = "updates";

    /// Pending updates and whether a restart is needed.
    pub async fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let stale = self
            .last
            .as_ref()
            .is_none_or(|(at, _)| at.elapsed() >= EVERY);
        if stale && root.path("/usr/bin/apt").exists() {
            let pending = parse_upgradable(&apt_list().await?);
            let info = UpdatesInfo {
                pending,
                lists_updated_at: modified(
                    &root.path("/var/lib/apt/periodic/update-success-stamp"),
                ),
                automatic_security: automatic_security(
                    &root
                        .read("/etc/apt/apt.conf.d/20auto-upgrades")
                        .unwrap_or_default(),
                ),
            };
            self.last = Some((Instant::now(), info));
        }
        let mut sample = Sample::default();
        let restart = root.path("/run/reboot-required").exists();
        sample.metrics.push(metric(
            "system.reboot_required",
            None,
            f64::from(u8::from(restart)),
            Unit::Count,
            now,
        ));
        if let Some((_, info)) = &self.last {
            let security = info.pending.iter().filter(|p| p.security).count();
            sample.metrics.push(metric(
                "updates.pending",
                None,
                info.pending.len() as f64,
                Unit::Count,
                now,
            ));
            sample.metrics.push(metric(
                "updates.security",
                None,
                security as f64,
                Unit::Count,
                now,
            ));
            sample.snapshots.push(Snapshot {
                kind: SnapshotKind::Updates,
                taken_at: now,
                data: serde_json::to_value(info).map_err(io::Error::other)?,
            });
        }
        Ok(sample)
    }
}

async fn apt_list() -> io::Result<String> {
    let child = Command::new("/usr/bin/apt")
        .args(["list", "--upgradable"])
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()?;
    let output = tokio::time::timeout(TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "apt did not answer in time"))??;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "apt list failed ({})",
            output.status
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn modified(path: &std::path::Path) -> Option<Timestamp> {
    let at = std::fs::metadata(path).ok()?.modified().ok()?;
    let ms = at.duration_since(UNIX_EPOCH).ok()?.as_millis();
    Some(Timestamp(i64::try_from(ms).ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = "Listing...
git/noble-updates 1:2.43.0-1ubuntu7.2 amd64 [upgradable from: 1:2.43.0-1ubuntu7.1]
openssl/noble-updates,noble-security 3.0.13-0ubuntu3.5 amd64 [upgradable from: 3.0.13-0ubuntu3.4]
code/stable 1.95.0-1730 amd64 [upgradable from: 1.94.2-1728]
";

    #[test]
    fn parses_upgradable_security_first() {
        let list = parse_upgradable(LIST);
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "openssl");
        assert!(list[0].security);
        assert_eq!(list[0].installed, "3.0.13-0ubuntu3.4");
        assert_eq!(list[0].candidate, "3.0.13-0ubuntu3.5");
        assert_eq!(list[1].name, "code");
        assert!(!list[2].security);
    }

    #[test]
    fn reads_the_automatic_updates_setting() {
        assert!(automatic_security(
            "APT::Periodic::Update-Package-Lists \"1\";\nAPT::Periodic::Unattended-Upgrade \"1\";\n"
        ));
        assert!(!automatic_security(
            "APT::Periodic::Unattended-Upgrade \"0\";\n"
        ));
        assert!(!automatic_security(""));
    }
}
