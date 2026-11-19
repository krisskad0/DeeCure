// SPDX-License-Identifier: Apache-2.0

//! Pending package updates, from `apt list --upgradable`.
//!
//! The command only reads apt's package lists: it needs no admin rights, takes no lock,
//! and changes nothing. It runs once an hour, and again as soon as the installed packages
//! (`/var/lib/dpkg/status`) or the package lists change, so a check right after an install
//! sees it. Between runs the last answer is repeated, so rules always have fresh facts.

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
    /// Ubuntu rolls this update out in stages and has not reached this computer yet
    /// (phased updates): apt and Software Updater wait for it, and so does DeeCure.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub phased: bool,
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
                phased: false,
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

/// The packages a dry run of `apt-get upgrade` defers because Ubuntu has not phased them in
/// for this computer yet: the names after "The following upgrades have been deferred due to
/// phasing:", one indented line or more.
pub fn parse_phased(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        if line.starts_with("The following upgrades have been deferred due to phasing") {
            inside = true;
        } else if inside && line.starts_with(char::is_whitespace) {
            out.extend(line.split_whitespace().map(str::to_owned));
        } else {
            inside = false;
        }
    }
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
    /// The apt read in progress. It runs on its own, so the minute's collection (and the
    /// first result after start) never waits for apt; its answer is used once it is ready.
    running: Option<tokio::task::JoinHandle<io::Result<Vec<Upgradable>>>>,
    /// When the installed packages and the package lists last changed, as seen when the newest
    /// read started.
    read_for: Option<Changes>,
}

/// When the installed packages (`dpkg/status`) and the package lists (apt's success stamp)
/// last changed.
type Changes = (Option<Timestamp>, Option<Timestamp>);

/// How long one collection waits for a read started because packages changed. A read takes
/// about a second; a slower one goes on in the background.
const WAIT: Duration = Duration::from_secs(10);

/// True when apt must be read again: no answer yet, an answer older than `EVERY`, or packages
/// or lists changed since the newest read started.
pub fn needs_read(age: Option<Duration>, read_for: Option<Changes>, now: Changes) -> bool {
    age.is_none_or(|a| a >= EVERY) || read_for.is_some_and(|r| r != now)
}

impl Updates {
    /// The name used in logs.
    pub const NAME: &'static str = "updates";

    /// Pending updates and whether a restart is needed.
    pub async fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let changes: Changes = (
            modified(&root.path("/var/lib/dpkg/status")),
            modified(&root.path("/var/lib/apt/periodic/update-success-stamp")),
        );
        let changed = self.read_for.is_some_and(|r| r != changes);
        let age = self.last.as_ref().map(|(at, _)| at.elapsed());
        if needs_read(age, self.read_for, changes)
            && self.running.is_none()
            && root.path("/usr/bin/apt").exists()
        {
            self.running = Some(tokio::spawn(read_pending()));
            self.read_for = Some(changes);
        }
        // After an install or an apt update, wait a little for the new answer, so the check
        // right after a fix sees it. Never at start: the first result does not wait for apt.
        if changed && self.last.is_some() {
            let until = Instant::now() + WAIT;
            while self.running.as_ref().is_some_and(|r| !r.is_finished()) && Instant::now() < until
            {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
        // Use the background read once it has finished.
        if self.running.as_ref().is_some_and(|r| r.is_finished())
            && let Some(running) = self.running.take()
        {
            let pending = running.await.map_err(io::Error::other)??;
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
            // Only updates this computer can install now; phased ones come when Ubuntu says.
            let ready = info.pending.iter().filter(|p| !p.phased);
            let security = ready.clone().filter(|p| p.security).count();
            sample.metrics.push(metric(
                "updates.pending",
                None,
                ready.count() as f64,
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

/// The pending updates, with the ones Ubuntu still phases in marked.
async fn read_pending() -> io::Result<Vec<Upgradable>> {
    let mut pending = parse_upgradable(&run_apt("/usr/bin/apt", &["list", "--upgradable"]).await?);
    // A dry run (no lock, no changes) tells which updates Ubuntu holds back for now.
    let phased = run_apt(
        "/usr/bin/apt-get",
        &["-s", "-o", "Debug::NoLocking=1", "upgrade"],
    )
    .await
    .map(|t| parse_phased(&t))
    .unwrap_or_default();
    for p in &mut pending {
        p.phased = phased.contains(&p.name);
    }
    Ok(pending)
}

/// Runs apt read-only (`list`, or a dry run with `-s`) and returns what it printed.
async fn run_apt(program: &str, args: &[&str]) -> io::Result<String> {
    let child = Command::new(program)
        .args(args)
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
            "{program} failed ({})",
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
    fn apt_is_read_again_after_an_install_or_a_list_update() {
        let t = |ms| Some(Timestamp(ms));
        let before = (t(1), t(10));
        // No answer yet, or an old one.
        assert!(needs_read(None, None, before));
        assert!(needs_read(Some(EVERY), Some(before), before));
        // A fresh answer, nothing changed.
        assert!(!needs_read(
            Some(Duration::from_secs(60)),
            Some(before),
            before
        ));
        // Packages installed (dpkg status newer), or the lists updated.
        assert!(needs_read(
            Some(Duration::from_secs(60)),
            Some(before),
            (t(2), t(10))
        ));
        assert!(needs_read(
            Some(Duration::from_secs(60)),
            Some(before),
            (t(1), t(11))
        ));
    }

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
    fn reads_which_updates_ubuntu_holds_back() {
        let dry_run = "Reading package lists...\nCalculating upgrade...\nThe following upgrades have been deferred due to phasing:\n  distro-info libdistro-info-perl lintian\n  python3-distro-info snapd\n0 upgraded, 0 newly installed, 0 to remove and 5 not upgraded.\n";
        assert_eq!(
            parse_phased(dry_run),
            [
                "distro-info",
                "libdistro-info-perl",
                "lintian",
                "python3-distro-info",
                "snapd"
            ]
        );
        assert!(
            parse_phased("0 upgraded, 0 newly installed, 0 to remove and 0 not upgraded.\n")
                .is_empty()
        );
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
