// SPDX-License-Identifier: Apache-2.0

//! The fix planner: turns live data into a [`Plan`] the person reviews before anything
//! runs. Planning only reads. The desktop app or CLI runs the steps (inside the person's
//! session, so polkit can ask for the password) and sends the result back to be recorded.

use std::time::Duration;

use deecure_core::fix::{Call, Plan, Step, Undo};
use deecure_core::{FixActionId, RuleId, SnapshotKind};

use crate::collect::UpdatesInfo;
use crate::context::Agent;
use crate::inventory::{command, kernel, packages, storage, units};
use crate::store;

pub use deecure_core::fix::ACTIONS;

#[allow(clippy::too_many_arguments)] // one argument per field, in the order a step reads
fn step(
    id: &str,
    title: &str,
    description: String,
    size: Option<u64>,
    undo: Undo,
    checked: bool,
    command: String,
    call: Call,
) -> Step {
    Step {
        id: id.into(),
        title: title.into(),
        description,
        size,
        undo,
        checked,
        command,
        call,
    }
}

fn need<'a>(subject: Option<&'a str>, what: &str) -> Result<&'a str, String> {
    subject
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("this fix needs a {what}"))
}

async fn disabled_snaps(agent: &Agent) -> Vec<(String, String, u64)> {
    let list = command::output("snap", &["list", "--all"], Duration::from_secs(10))
        .await
        .unwrap_or_default();
    storage::disabled_snaps(&list)
        .into_iter()
        .map(|(n, r)| {
            let size = std::fs::metadata(
                agent
                    .root
                    .path(&format!("/var/lib/snapd/snaps/{n}_{r}.snap")),
            )
            .map(|m| m.len())
            .unwrap_or(0);
            (n, r, size)
        })
        .collect()
}

/// Steps that clean one storage candidate.
async fn storage_steps(agent: &Agent, candidate: &storage::Candidate) -> Vec<Step> {
    let c = candidate;
    let one = |id: &str, undo: Undo, command: &str, call: Call| {
        vec![step(
            id,
            &c.title,
            c.details.clone(),
            Some(c.size),
            undo,
            true,
            command.into(),
            call,
        )]
    };
    match c.action.as_str() {
        "storage.empty_trash" => one("trash", Undo::None, "gio trash --empty", Call::EmptyTrash),
        "storage.clear_thumbnails" => one(
            "thumbnails",
            Undo::Regenerates,
            "rm -r ~/.cache/thumbnails/*",
            Call::ClearThumbnails,
        ),
        "storage.clean_apt_cache" => one(
            "apt",
            Undo::Regenerates,
            "sudo apt-get clean",
            Call::CleanAptCache,
        ),
        "storage.vacuum_journal" => one(
            "journal",
            Undo::None,
            "sudo journalctl --vacuum-size=200M",
            Call::VacuumJournal { keep_mb: 200 },
        ),
        "storage.prune_docker_images" => one(
            "docker",
            Undo::Regenerates,
            "docker image prune --all --force",
            Call::DockerPruneImages,
        ),
        "storage.prune_docker_build_cache" => one(
            "docker-build",
            Undo::Regenerates,
            "docker builder prune --force",
            Call::DockerPruneBuildCache,
        ),
        "storage.remove_old_snap_revisions" => disabled_snaps(agent)
            .await
            .into_iter()
            .map(|(name, rev, size)| {
                step(
                    &format!("snap-{name}-{rev}"),
                    &format!("Remove {name} revision {rev}"),
                    "Kept after an update; not used".into(),
                    Some(size),
                    Undo::None,
                    true,
                    format!("sudo snap remove {name} --revision={rev}"),
                    Call::RemoveSnapRevision {
                        name,
                        revision: rev,
                    },
                )
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Builds the plan for one fix. `subject` names the target for fixes that need one: a
/// service unit, a startup app ID, `pid:name` for a process, or a power profile.
pub async fn plan(agent: &Agent, action: &str, subject: Option<&str>) -> Result<Plan, String> {
    let id = FixActionId::new(action).map_err(|e| e.to_string())?;
    let title = ACTIONS
        .iter()
        .find(|(a, _, _)| *a == action)
        .map(|(_, t, _)| (*t).to_owned())
        .ok_or_else(|| format!("unknown fix '{action}'"))?;
    let mut rule = None;
    let mut subtitle = String::new();
    let steps: Vec<Step> = match action {
        "storage.free_space" => {
            rule = RuleId::new("storage.root_almost_full").ok();
            subtitle = "Main disk ( / )".into();
            let info = storage::read(&agent.root).await;
            let mut steps = Vec::new();
            for c in &info.candidates {
                steps.extend(storage_steps(agent, c).await);
            }
            steps
        }
        a if a.starts_with("storage.") => {
            let info = storage::read(&agent.root).await;
            match info.candidates.iter().find(|c| c.action == a) {
                Some(c) => storage_steps(agent, c).await,
                None => Vec::new(),
            }
        }
        "packages.autoremove" | "packages.purge_residual_config" => {
            let info = packages::read(&agent.root).await;
            let found = info.removable.into_iter().find(|r| r.action == action);
            match (action, found) {
                ("packages.autoremove", Some(r)) => vec![step(
                    "autoremove",
                    &r.title,
                    r.names.join(", "),
                    Some(r.size),
                    Undo::Regenerates,
                    true,
                    "sudo apt-get autoremove".into(),
                    Call::Autoremove,
                )],
                ("packages.purge_residual_config", Some(r)) => vec![step(
                    "purge",
                    &r.title,
                    r.names.join(", "),
                    None,
                    Undo::None,
                    true,
                    format!("sudo apt-get purge {}", r.names.join(" ")),
                    Call::PurgeResidual { packages: r.names },
                )],
                _ => Vec::new(),
            }
        }
        "security.install_security_updates" | "system.install_updates" => {
            let security_only = action == "security.install_security_updates";
            if security_only {
                rule = RuleId::new("security.updates_pending").ok();
            }
            let snapshot = store::lock(&agent.store)
                .latest_snapshot(SnapshotKind::Updates)
                .map_err(|e| e.to_string())?;
            let info: UpdatesInfo = snapshot
                .and_then(|s| serde_json::from_value(s.data).ok())
                .unwrap_or_default();
            let names: Vec<String> = info
                .pending
                .iter()
                .filter(|p| p.security || !security_only)
                .map(|p| p.name.clone())
                .collect();
            subtitle = if security_only {
                "Security archive".into()
            } else {
                "All pending updates".into()
            };
            if names.is_empty() {
                Vec::new()
            } else {
                let shown = names.iter().take(4).cloned().collect::<Vec<_>>().join(", ")
                    + if names.len() > 4 { " and more" } else { "" };
                vec![step(
                    "updates",
                    &format!(
                        "Install {} update{}",
                        names.len(),
                        if names.len() == 1 { "" } else { "s" }
                    ),
                    shown,
                    None,
                    Undo::None,
                    true,
                    format!("sudo apt-get install --only-upgrade {}", names.join(" ")),
                    Call::UpgradePackages { packages: names },
                )]
            }
        }
        "security.enable_firewall" => {
            rule = RuleId::new("security.firewall_off").ok();
            subtitle = "ufw · incoming connections blocked, outgoing allowed".into();
            vec![step("ufw", "Turn on the ufw firewall", "Your own apps keep working; other computers can no longer reach programs listening on this one".into(), None, Undo::Full, true, "sudo ufw enable".into(), Call::SetFirewall { on: true })]
        }
        "system.remove_old_kernels" => {
            let k = kernel::read(&agent.root);
            subtitle = "The running and newest kernels are kept".into();
            let old: Vec<&kernel::InstalledKernel> =
                k.installed.iter().filter(|i| i.status == "old").collect();
            if old.is_empty() {
                Vec::new()
            } else {
                vec![step(
                    "kernels",
                    &format!(
                        "Remove {} old kernel{}",
                        old.len(),
                        if old.len() == 1 { "" } else { "s" }
                    ),
                    old.iter()
                        .map(|i| i.version.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    Some(old.iter().map(|i| i.size).sum()),
                    Undo::None,
                    true,
                    format!("sudo apt-get purge linux-image-{}", old[0].version),
                    Call::RemoveKernels {
                        versions: old.iter().map(|i| i.version.clone()).collect(),
                    },
                )]
            }
        }
        "services.restart" | "services.disable" | "services.enable" => {
            let unit = need(subject, "service")?.to_owned();
            if action == "services.restart" {
                rule = RuleId::new("services.unit_failed").ok();
            }
            subtitle = unit.clone();
            let (title, cmd, call, undo) = match action {
                "services.restart" => (
                    "Restart the service",
                    format!("sudo systemctl restart {unit}"),
                    Call::RestartService { unit: unit.clone() },
                    Undo::None,
                ),
                "services.disable" => (
                    "Stop it and do not start it at boot",
                    format!("sudo systemctl disable --now {unit}"),
                    Call::SetServiceEnabled {
                        unit: unit.clone(),
                        enabled: false,
                    },
                    Undo::Full,
                ),
                _ => (
                    "Start it now and at every boot",
                    format!("sudo systemctl enable --now {unit}"),
                    Call::SetServiceEnabled {
                        unit: unit.clone(),
                        enabled: true,
                    },
                    Undo::Full,
                ),
            };
            vec![step(
                "service",
                title,
                unit.clone(),
                None,
                undo,
                true,
                cmd,
                call,
            )]
        }
        "services.disable_startup_app" | "services.enable_startup_app" => {
            let id = need(subject, "startup app")?;
            let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
            let app = units::startup(&agent.root, home.as_deref())
                .into_iter()
                .find(|a| a.id == id)
                .ok_or_else(|| format!("no startup app '{id}'"))?;
            let enabled = action.ends_with("enable_startup_app");
            subtitle = app.name.clone();
            vec![step(
                "autostart",
                if enabled {
                    "Start it at login"
                } else {
                    "Do not start it at login"
                },
                app.command.clone(),
                None,
                Undo::Full,
                true,
                format!("~/.config/autostart/{id}: X-GNOME-Autostart-enabled={enabled}"),
                Call::SetStartupApp {
                    id: id.to_owned(),
                    enabled,
                },
            )]
        }
        "processes.end" => {
            let s = need(subject, "process (pid:name)")?;
            let (pid, name) = s
                .split_once(':')
                .ok_or("the process is given as pid:name")?;
            let pid: u32 = pid.parse().map_err(|_| "not a process ID")?;
            subtitle = format!("{name} (PID {pid})");
            vec![step(
                "end",
                "Ask it to quit",
                "Unsaved work in it may be lost".into(),
                None,
                Undo::None,
                true,
                format!("kill {pid}"),
                Call::EndProcess {
                    pid,
                    name: name.to_owned(),
                },
            )]
        }
        "power.set_profile" => {
            let profile = need(subject, "power mode")?;
            if !["power-saver", "balanced", "performance"].contains(&profile) {
                return Err(format!("unknown power mode '{profile}'"));
            }
            subtitle = profile.to_owned();
            vec![step(
                "profile",
                &format!("Switch to {profile}"),
                "power-profiles-daemon".into(),
                None,
                Undo::Full,
                true,
                format!("powerprofilesctl set {profile}"),
                Call::SetPowerProfile {
                    profile: profile.to_owned(),
                },
            )]
        }
        "power.set_battery_action" => {
            subtitle = "UPower · /etc/UPower/UPower.conf".into();
            vec![step(
                "upower",
                "Power off cleanly at 5%",
                "Prevents sudden shutdowns that can lose work".into(),
                None,
                Undo::Full,
                true,
                "PercentageAction=5, CriticalPowerAction=PowerOff".into(),
                Call::SetBatteryAction {
                    action: "PowerOff".into(),
                    percent: 5,
                },
            )]
        }
        other => return Err(format!("no planner for '{other}'")),
    };
    Ok(Plan {
        action: id,
        title,
        subtitle,
        rule,
        subject: subject.map(str::to_owned),
        steps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::SystemRoot;
    use crate::store::Store;
    use std::sync::{Arc, Mutex};

    fn agent() -> Arc<Agent> {
        Agent::new(
            Arc::new(Mutex::new(Store::open_in_memory().unwrap())),
            SystemRoot::real(),
        )
    }

    #[test]
    fn every_action_is_a_valid_id() {
        for (a, _, _) in ACTIONS {
            assert!(FixActionId::new(*a).is_ok(), "{a}");
        }
    }

    #[tokio::test]
    async fn plans_need_their_subject_and_reject_unknown_ones() {
        let a = agent();
        assert!(
            plan(&a, "services.restart", None)
                .await
                .unwrap_err()
                .contains("needs a service")
        );
        assert!(plan(&a, "storage.format_disk", None).await.is_err());
        assert!(plan(&a, "power.set_profile", Some("turbo")).await.is_err());
        let p = plan(&a, "services.disable", Some("cups.service"))
            .await
            .unwrap();
        assert_eq!(
            p.steps[0].call,
            Call::SetServiceEnabled {
                unit: "cups.service".into(),
                enabled: false
            }
        );
        assert_eq!(p.steps[0].undo, Undo::Full);
        let p = plan(&a, "processes.end", Some("4121:chrome"))
            .await
            .unwrap();
        assert_eq!(
            p.steps[0].call,
            Call::EndProcess {
                pid: 4121,
                name: "chrome".into()
            }
        );
        let p = plan(&a, "security.enable_firewall", None).await.unwrap();
        assert_eq!(
            p.rule.as_ref().map(|r| r.as_str()),
            Some("security.firewall_off")
        );
    }

    #[tokio::test]
    async fn update_plans_come_from_the_updates_inventory() {
        let a = agent();
        let info = UpdatesInfo {
            pending: vec![
                crate::collect::Upgradable {
                    name: "openssl".into(),
                    installed: "1".into(),
                    candidate: "2".into(),
                    origin: "noble-security".into(),
                    security: true,
                },
                crate::collect::Upgradable {
                    name: "git".into(),
                    installed: "1".into(),
                    candidate: "2".into(),
                    origin: "noble-updates".into(),
                    security: false,
                },
            ],
            ..UpdatesInfo::default()
        };
        store::lock(&a.store)
            .add_snapshot_if_changed(&deecure_core::Snapshot {
                kind: SnapshotKind::Updates,
                taken_at: deecure_core::Timestamp(1),
                data: serde_json::to_value(&info).unwrap(),
            })
            .unwrap();
        let p = plan(&a, "security.install_security_updates", None)
            .await
            .unwrap();
        assert_eq!(
            p.steps[0].call,
            Call::UpgradePackages {
                packages: vec!["openssl".into()]
            }
        );
        let p = plan(&a, "system.install_updates", None).await.unwrap();
        assert_eq!(
            p.steps[0].call,
            Call::UpgradePackages {
                packages: vec!["openssl".into(), "git".into()]
            }
        );
    }
}
