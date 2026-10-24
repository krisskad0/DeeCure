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

/// `on` or `off`.
fn on_off(state: &str) -> Result<bool, String> {
    match state {
        "on" => Ok(true),
        "off" => Ok(false),
        other => Err(format!("'{other}' is not on or off")),
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
        // Cleanup parts; the other storage fixes (log cap, snap versions, scheduler) are below.
        a if a.starts_with("storage.")
            && !matches!(
                a,
                "storage.cap_journal" | "storage.keep_fewer_snaps" | "storage.set_disk_scheduler"
            ) =>
        {
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
            // A package name as subject installs only that update (such as linux-firmware).
            let names: Vec<String> = info
                .pending
                .iter()
                .filter(|p| p.security || !security_only)
                .filter(|p| subject.is_none_or(|s| s == p.name))
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
        "power.set_charge_limits" => {
            // `BAT0:75:80`, or `BAT0:-:80` for a battery with only a stop limit.
            let raw = need(subject, "battery:start:stop")?;
            let parts: Vec<&str> = raw.split(':').collect();
            let [battery, start, stop] = parts.as_slice() else {
                return Err("the subject is battery:start:stop".into());
            };
            let stop: u32 = stop.parse().map_err(|_| "the stop limit is not a number")?;
            let start: Option<u32> = if *start == "-" {
                None
            } else {
                Some(
                    start
                        .parse()
                        .map_err(|_| "the start limit is not a number")?,
                )
            };
            let dir = format!("/sys/class/power_supply/{battery}");
            subtitle = format!("{battery} · {dir}");
            let text = match start {
                Some(s) => format!("Charge from below {s}% up to {stop}%"),
                None => format!("Stop charging at {stop}%"),
            };
            let command = match start {
                Some(s) => format!(
                    "echo {s} | sudo tee {dir}/charge_control_start_threshold; echo {stop} | sudo tee {dir}/charge_control_end_threshold"
                ),
                None => format!("echo {stop} | sudo tee {dir}/charge_control_end_threshold"),
            };
            vec![step(
                "charge-limits",
                &text,
                "Keeping a battery below full slows its wear. DeeCure sets it again at each start."
                    .into(),
                None,
                Undo::Full,
                true,
                command,
                Call::SetChargeLimits {
                    battery: (*battery).to_owned(),
                    start,
                    stop,
                },
            )]
        }
        "power.set_charge_mode" => {
            let raw = need(subject, "battery:mode")?;
            let (battery, mode) = raw.split_once(':').ok_or("the subject is battery:mode")?;
            subtitle = format!("{battery} · charge_types");
            vec![step(
                "charge-mode",
                &format!("Use the {mode} charge mode"),
                "The firmware's own charging behaviour".into(),
                None,
                Undo::Full,
                true,
                format!("echo {mode} | sudo tee /sys/class/power_supply/{battery}/charge_types"),
                Call::SetChargeMode {
                    battery: battery.to_owned(),
                    mode: mode.to_owned(),
                },
            )]
        }
        "power.set_turbo" => {
            let on = on_off(need(subject, "on|off")?)?;
            let intel = agent
                .root
                .path("/sys/devices/system/cpu/intel_pstate/no_turbo")
                .exists();
            subtitle = "Kept after restart".into();
            let command = if intel {
                format!(
                    "echo {} | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo",
                    if on { 0 } else { 1 }
                )
            } else {
                format!(
                    "echo {} | sudo tee /sys/devices/system/cpu/cpufreq/boost",
                    if on { 1 } else { 0 }
                )
            };
            vec![step(
                "turbo",
                if on {
                    "Turn turbo on"
                } else {
                    "Turn turbo off"
                },
                if on {
                    "Full speed when busy, more heat and power".into()
                } else {
                    "Lower top speed, less heat, longer battery".into()
                },
                None,
                Undo::Full,
                true,
                command,
                Call::SetTurbo { on },
            )]
        }
        "power.set_device_wakeup" | "power.set_device_power_saving" => {
            let raw = need(subject, "device|on|off")?;
            let (device, state) = raw.rsplit_once('|').ok_or("the subject is device|on|off")?;
            let on = on_off(state)?;
            subtitle = format!("/sys/devices/{device} · kept after restart");
            if action == "power.set_device_wakeup" {
                vec![step(
                    "wakeup",
                    if on {
                        "Allow it to wake the computer"
                    } else {
                        "Stop it waking the computer"
                    },
                    "From suspend".into(),
                    None,
                    Undo::Full,
                    true,
                    format!(
                        "echo {} | sudo tee /sys/devices/{device}/power/wakeup",
                        if on { "enabled" } else { "disabled" }
                    ),
                    Call::SetDeviceWakeup {
                        device: device.to_owned(),
                        enabled: on,
                    },
                )]
            } else {
                vec![step(
                    "power-saving",
                    if on {
                        "Let it save power when idle"
                    } else {
                        "Keep it always awake"
                    },
                    "Runtime power management".into(),
                    None,
                    Undo::Full,
                    true,
                    format!(
                        "echo {} | sudo tee /sys/devices/{device}/power/control",
                        if on { "auto" } else { "on" }
                    ),
                    Call::SetDevicePowerSaving {
                        device: device.to_owned(),
                        on,
                    },
                )]
            }
        }
        "power.set_keyboard_backlight" => {
            let level: u32 = need(subject, "level")?
                .parse()
                .map_err(|_| "the level is not a number")?;
            subtitle = "UPower".into();
            vec![step(
                "kbd",
                &format!("Set the keyboard backlight to level {level}"),
                String::new(),
                None,
                Undo::Full,
                true,
                format!(
                    "busctl call org.freedesktop.UPower /org/freedesktop/UPower/KbdBacklight org.freedesktop.UPower.KbdBacklight SetBrightness i {level}"
                ),
                Call::SetKeyboardBacklight { level },
            )]
        }
        "power.set_battery_cpu_limit" => {
            let raw = need(subject, "percent|off")?;
            let percent = if raw == "off" {
                None
            } else {
                let p: u32 = raw.parse().map_err(|_| "the limit is not a number")?;
                if !(30..=99).contains(&p) {
                    return Err("the limit must be between 30 and 99%".into());
                }
                Some(p)
            };
            subtitle = "Set again at each plug in, unplug, and start".into();
            vec![step(
                "cpu-limit",
                &match percent {
                    Some(p) => format!("On battery, run the CPU at most at {p}% of its top speed"),
                    None => "Remove the CPU speed limit on battery".into(),
                },
                "On mains power the CPU keeps its full speed. A lower top speed means less heat \
                 and longer battery, and heavy work takes longer."
                    .into(),
                None,
                Undo::Full,
                true,
                match percent {
                    Some(p) => format!(
                        "for p in /sys/devices/system/cpu/cpufreq/policy*; do echo $(( $(cat $p/cpuinfo_max_freq) * {p} / 100 )) | sudo tee $p/scaling_max_freq; done   # on battery only"
                    ),
                    None => "for p in /sys/devices/system/cpu/cpufreq/policy*; do cat $p/cpuinfo_max_freq | sudo tee $p/scaling_max_freq; done".into(),
                },
                Call::SetBatteryCpuLimit { percent },
            )]
        }
        "power.set_aspm_policy" | "power.set_suspend_mode" => {
            let raw = need(subject, "value|try|keep")?;
            let (value, how) = raw.split_once('|').ok_or("the subject is value|try|keep")?;
            let keep = match how {
                "try" => false,
                "keep" => true,
                _ => return Err(format!("'{how}' is not try or keep")),
            };
            let aspm = action == "power.set_aspm_policy";
            let allowed: &[&str] = if aspm {
                &["default", "performance", "powersave", "powersupersave"]
            } else {
                &["s2idle", "shallow", "deep"]
            };
            if !allowed.contains(&value) {
                return Err(format!("unknown value '{value}'"));
            }
            subtitle = if keep {
                "Kept after restart".into()
            } else {
                "For this session only: try it, then keep it if all works".into()
            };
            let (file, title, about) = if aspm {
                (
                    "/sys/module/pcie_aspm/parameters/policy",
                    format!("Use the {value} PCIe power saving policy"),
                    "Lets PCIe links (Wi-Fi, drives, graphics) sleep when idle. Some devices \
                     misbehave with more saving; a restart puts it back while it is not kept.",
                )
            } else {
                (
                    "/sys/power/mem_sleep",
                    format!("Use {value} suspend"),
                    "deep turns off more and uses less battery while suspended, but some laptops \
                     do not wake from it. Suspend and wake once to test; if it does not wake, \
                     hold the power button, and after the restart it is back.",
                )
            };
            vec![step(
                if aspm { "aspm" } else { "suspend" },
                &title,
                about.into(),
                None,
                Undo::Full,
                true,
                format!("echo {value} | sudo tee {file}"),
                if aspm {
                    Call::SetAspmPolicy {
                        policy: value.to_owned(),
                        keep,
                    }
                } else {
                    Call::SetSuspendMode {
                        mode: value.to_owned(),
                        keep,
                    }
                },
            )]
        }
        "system.update_deecure" => {
            let version = need(subject, "version")?.to_owned();
            if !deecure_core::update::valid_version(&version) {
                return Err(format!("'{version}' is not a DeeCure version"));
            }
            subtitle = format!("DeeCure {} to {version}", deecure_core::VERSION);
            vec![step(
                "update",
                &format!("Install DeeCure {version}"),
                "Downloads it from DeeCure's GitHub release page, checks each package against the release's checksums and its own name and version, then installs it with apt. Afterwards, open DeeCure again.".into(),
                None,
                Undo::None,
                true,
                format!(
                    "sudo apt-get install ./deecure_{}_amd64.deb   # after the checksum check",
                    deecure_core::update::deb_version(&version)
                ),
                Call::InstallUpdate { version },
            )]
        }
        "system.set_auto_update" => {
            let on = on_off(need(subject, "on|off")?)?;
            let settings =
                crate::updates::settings(&store::lock(&agent.store)).map_err(|e| e.to_string())?;
            subtitle = "deecure-update.timer".into();
            vec![step(
                "auto-update",
                if on {
                    "Install DeeCure updates by themselves"
                } else {
                    "Ask before installing DeeCure updates"
                },
                if on {
                    "A small timer looks for a new release and installs it with the same checks, not on a metered connection. You get a notification afterwards.".into()
                } else {
                    "The timer stops installing; DeeCure tells you about new releases instead."
                        .into()
                },
                None,
                Undo::Full,
                true,
                if on {
                    "echo '{\"auto\": true}' | sudo tee /etc/deecure/update.json".into()
                } else {
                    "sudo rm /etc/deecure/update.json".into()
                },
                Call::SetAutoUpdate {
                    on,
                    check: settings.check,
                    channel: settings.channel,
                },
            )]
        }
        "power.set_battery_turbo" => {
            let on = on_off(need(subject, "on|off")?)?;
            subtitle = "Set again at each plug in, unplug, and start".into();
            vec![step(
                "battery-turbo",
                if on {
                    "Allow turbo on battery again"
                } else {
                    "Turn turbo off while on battery"
                },
                "On mains, turbo follows your turbo setting (on unless you turned it off). Off on battery: lower top speed, less heat, longer battery.".into(),
                None,
                Undo::Full,
                true,
                format!(
                    "on battery: echo {} | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo",
                    if on { 0 } else { 1 }
                ),
                Call::SetBatteryTurbo { on },
            )]
        }
        "storage.cap_journal" => {
            let raw = need(subject, "megabytes|off")?;
            let mb = if raw == "off" {
                None
            } else {
                let m: u32 = raw.parse().map_err(|_| "the cap is not a number")?;
                if !(100..=10_000).contains(&m) {
                    return Err("the cap must be 100-10000 MB".into());
                }
                Some(m)
            };
            subtitle = "Applies to new logs after the next restart".into();
            let file = "/etc/systemd/journald.conf.d/50-deecure-size.conf";
            vec![step(
                "journal-cap",
                &match mb {
                    Some(m) => format!("Keep the system log under {m} MB"),
                    None => "Remove DeeCure's log size cap".into(),
                },
                "journald reads the cap when it starts; the log is shrunk to the cap now.".into(),
                None,
                Undo::Full,
                true,
                match mb {
                    Some(m) => format!(
                        "printf '[Journal]\\nSystemMaxUse={m}M\\n' | sudo tee {file} && sudo journalctl --vacuum-size={m}M"
                    ),
                    None => format!("sudo rm {file}"),
                },
                Call::SetJournalCap { mb },
            )]
        }
        "storage.keep_fewer_snaps" => {
            let keep: u32 = need(subject, "count")?
                .parse()
                .map_err(|_| "the count is not a number")?;
            if !(2..=20).contains(&keep) {
                return Err("snap keeps 2 to 20 versions".into());
            }
            subtitle = "snapd".into();
            vec![step(
                "snap-retain",
                &format!("Keep {keep} versions of each snap"),
                "The running version and one to go back to. Old versions are removed at the next refresh.".into(),
                None,
                Undo::Full,
                true,
                format!("sudo snap set system refresh.retain={keep}"),
                Call::SetSnapRetain { keep },
            )]
        }
        "packages.remove_unused_flatpaks" => {
            let system = match need(subject, "user|system")? {
                "system" => true,
                "user" => false,
                other => return Err(format!("'{other}' is not user or system")),
            };
            let flag = if system { "--system" } else { "--user" };
            subtitle = format!("Flatpak ({})", if system { "system" } else { "your own" });
            vec![step(
                "flatpak-unused",
                "Remove runtimes no installed app uses",
                "An app that needs one later installs it again.".into(),
                None,
                Undo::Regenerates,
                true,
                format!(
                    "{}flatpak uninstall --unused {flag} -y --noninteractive",
                    if system { "sudo " } else { "" }
                ),
                Call::RemoveUnusedFlatpaks { system },
            )]
        }
        "system.raise_file_watches" => {
            let value: u64 = need(subject, "value")?
                .parse()
                .map_err(|_| "the limit is not a number")?;
            if !(8192..=4_194_304).contains(&value) {
                return Err("the limit must be 8192-4194304".into());
            }
            subtitle = "Now and at every start".into();
            vec![step(
                "file-watches",
                &format!("Raise the file-watch limit to {value}"),
                "Each watch uses about 1 KB of kernel memory only when an app uses it.".into(),
                None,
                Undo::Full,
                true,
                format!(
                    "echo 'fs.inotify.max_user_watches = {value}' | sudo tee /etc/sysctl.d/60-deecure-inotify.conf && sudo sysctl -w fs.inotify.max_user_watches={value}"
                ),
                Call::SetFileWatchLimit { value, keep: true },
            )]
        }
        "system.enable_zswap" => {
            let on = on_off(need(subject, "on|off")?)?;
            subtitle = "Kept after restart".into();
            vec![step(
                "zswap",
                if on {
                    "Turn on compressed swap (zswap)"
                } else {
                    "Turn off compressed swap (zswap)"
                },
                "Memory that would go to the swap file is first kept compressed in RAM.".into(),
                None,
                Undo::Full,
                true,
                format!(
                    "echo {} | sudo tee /sys/module/zswap/parameters/enabled",
                    if on { "Y" } else { "N" }
                ),
                Call::SetZswap { on },
            )]
        }
        "storage.set_disk_scheduler" => {
            let raw = need(subject, "disk|scheduler")?;
            let (disk, scheduler) = raw.split_once('|').ok_or("the subject is disk|scheduler")?;
            if !["mq-deadline", "bfq", "none"].contains(&scheduler) {
                return Err(format!("unknown scheduler '{scheduler}'"));
            }
            subtitle = format!("/dev/{disk} · kept after restart");
            vec![step(
                "scheduler",
                &format!("Use the {scheduler} scheduler for {disk}"),
                "A hard disk is faster when requests go in disk order.".into(),
                None,
                Undo::Full,
                true,
                format!("echo {scheduler} | sudo tee /sys/block/{disk}/queue/scheduler"),
                Call::SetDiskScheduler {
                    disk: disk.to_owned(),
                    scheduler: scheduler.to_owned(),
                },
            )]
        }
        "packages.finish_setup" => {
            subtitle = "dpkg".into();
            vec![step(
                "dpkg-configure",
                "Finish package installs that stopped half way",
                "Keeps your own settings files when a package asks.".into(),
                None,
                Undo::None,
                true,
                "sudo dpkg --force-confdef --force-confold --configure -a".into(),
                Call::FinishPackageSetup,
            )]
        }
        "packages.install_driver" => {
            let package = need(subject, "package")?.to_owned();
            subtitle = "From Ubuntu's archive".into();
            vec![step(
                "install",
                &format!("Install {package}"),
                format!("To take it back, remove {package} in Packages."),
                None,
                Undo::None,
                true,
                format!("sudo apt-get install {package}"),
                Call::InstallPackages {
                    packages: vec![package.clone()],
                },
            )]
        }
        "system.rebuild_dkms" => {
            let kernel = need(subject, "kernel")?.to_owned();
            subtitle = format!("Kernel {kernel}");
            vec![step(
                "dkms",
                &format!("Build the extra drivers for kernel {kernel}"),
                "Only adds the missing builds.".into(),
                None,
                Undo::Regenerates,
                true,
                format!("sudo dkms autoinstall -k {kernel}"),
                Call::RebuildDkms { kernel },
            )]
        }
        "system.update_firmware" => {
            let device = need(subject, "device id")?.to_owned();
            subtitle = "fwupd (asks for your password itself)".into();
            vec![step(
                "firmware",
                "Install the firmware update",
                "Some firmware cannot go back to an older version, and some updates finish during a restart.".into(),
                None,
                Undo::None,
                true,
                format!("fwupdmgr update {device}"),
                Call::UpdateFirmware { device },
            )]
        }
        "power.set_lid_action" => {
            let raw = need(subject, "battery|power")?;
            let (battery, power) = raw.split_once('|').ok_or("the subject is battery|power")?;
            let name = |v: &str| match v {
                "suspend" => Ok("suspend"),
                "lock" => Ok("lock the screen"),
                "ignore" => Ok("do nothing"),
                "poweroff" => Ok("power off"),
                "default" => Ok("use the system's own setting"),
                _ => Err(format!("unknown lid action '{v}'")),
            };
            let (b, p) = (name(battery)?, name(power)?);
            subtitle = "Applies after the next restart".into();
            let file = "/etc/systemd/logind.conf.d/50-deecure-lid.conf";
            let command = if battery == "default" && power == "default" {
                format!("sudo rm {file}")
            } else {
                let mut lines = String::from("[Login]\\n");
                if battery != "default" {
                    lines += &format!("HandleLidSwitch={battery}\\n");
                }
                if power != "default" {
                    lines += &format!("HandleLidSwitchExternalPower={power}\\n");
                }
                format!(
                    "sudo mkdir -p /etc/systemd/logind.conf.d && printf '{lines}' | sudo tee {file}"
                )
            };
            vec![step(
                "lid",
                &format!("Closing the lid: on battery {b}, plugged in {p}"),
                "logind reads this when it starts, so it applies after the next restart. \
                 With an external screen connected, the desktop decides instead."
                    .into(),
                None,
                Undo::Full,
                true,
                command,
                Call::SetLidAction {
                    on_battery: battery.to_owned(),
                    on_power: power.to_owned(),
                },
            )]
        }
        "power.set_radio" => {
            let raw = need(subject, "kind|on|off")?;
            let (kind, state) = raw.split_once('|').ok_or("the subject is kind|on|off")?;
            if !["wlan", "bluetooth", "wwan"].contains(&kind) {
                return Err(format!("unknown radio '{kind}'"));
            }
            let on = on_off(state)?;
            let name = match kind {
                "wlan" => "Wi-Fi",
                "bluetooth" => "Bluetooth",
                _ => "Mobile broadband",
            };
            subtitle = "rfkill".into();
            vec![step(
                "radio",
                &format!("Turn {name} {}", if on { "on" } else { "off" }),
                String::new(),
                None,
                Undo::Full,
                true,
                format!("rfkill {} {kind}", if on { "unblock" } else { "block" }),
                Call::SetRadio {
                    kind: kind.to_owned(),
                    on,
                },
            )]
        }
        "power.set_wifi_power_save" => {
            let raw = need(subject, "uuid|on|off|default")?;
            let (uuid, state) = raw
                .rsplit_once('|')
                .ok_or("the subject is uuid|on|off|default")?;
            let value = match state {
                "on" => 3,
                "off" => 2,
                "default" => 0,
                other => return Err(format!("unknown state '{other}'")),
            };
            subtitle = "NetworkManager · applies the next time Wi-Fi connects".into();
            vec![step(
                "wifi-powersave",
                match value {
                    3 => "Turn Wi-Fi power saving on",
                    2 => "Turn Wi-Fi power saving off",
                    _ => "Use the default Wi-Fi power saving",
                },
                "Saves battery, but can add a little delay".into(),
                None,
                Undo::Full,
                true,
                format!("nmcli connection modify {uuid} 802-11-wireless.powersave {value}"),
                Call::SetWifiPowerSave {
                    uuid: uuid.to_owned(),
                    value,
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
    async fn power_controls_read_their_subjects() {
        let a = agent();
        let call = |p: Plan| p.steps[0].call.clone();
        assert_eq!(
            call(
                plan(&a, "power.set_charge_limits", Some("BAT0:75:80"))
                    .await
                    .unwrap()
            ),
            Call::SetChargeLimits {
                battery: "BAT0".into(),
                start: Some(75),
                stop: 80
            }
        );
        assert_eq!(
            call(
                plan(&a, "power.set_charge_limits", Some("BAT0:-:80"))
                    .await
                    .unwrap()
            ),
            Call::SetChargeLimits {
                battery: "BAT0".into(),
                start: None,
                stop: 80
            }
        );
        assert_eq!(
            call(
                plan(
                    &a,
                    "power.set_device_wakeup",
                    Some("pci0000:00/0000:00:14.0|off")
                )
                .await
                .unwrap()
            ),
            Call::SetDeviceWakeup {
                device: "pci0000:00/0000:00:14.0".into(),
                enabled: false
            },
            "device paths keep their colons"
        );
        assert_eq!(
            call(
                plan(
                    &a,
                    "power.set_wifi_power_save",
                    Some("0a1b2c3d-4e5f-6a7b-8c9d-0e1f2a3b4c5d|off")
                )
                .await
                .unwrap()
            ),
            Call::SetWifiPowerSave {
                uuid: "0a1b2c3d-4e5f-6a7b-8c9d-0e1f2a3b4c5d".into(),
                value: 2
            }
        );
        assert_eq!(
            call(plan(&a, "power.set_turbo", Some("off")).await.unwrap()),
            Call::SetTurbo { on: false }
        );
        assert_eq!(
            call(
                plan(&a, "power.set_battery_cpu_limit", Some("60"))
                    .await
                    .unwrap()
            ),
            Call::SetBatteryCpuLimit { percent: Some(60) }
        );
        assert_eq!(
            call(
                plan(&a, "power.set_battery_cpu_limit", Some("off"))
                    .await
                    .unwrap()
            ),
            Call::SetBatteryCpuLimit { percent: None }
        );
        assert_eq!(
            call(
                plan(&a, "power.set_suspend_mode", Some("deep|try"))
                    .await
                    .unwrap()
            ),
            Call::SetSuspendMode {
                mode: "deep".into(),
                keep: false
            }
        );
        assert_eq!(
            call(
                plan(&a, "power.set_aspm_policy", Some("powersave|keep"))
                    .await
                    .unwrap()
            ),
            Call::SetAspmPolicy {
                policy: "powersave".into(),
                keep: true
            }
        );
        let lid = plan(&a, "power.set_lid_action", Some("suspend|lock"))
            .await
            .unwrap();
        assert!(
            lid.steps[0]
                .command
                .contains("HandleLidSwitch=suspend\\nHandleLidSwitchExternalPower=lock")
        );
        assert_eq!(
            call(lid),
            Call::SetLidAction {
                on_battery: "suspend".into(),
                on_power: "lock".into()
            }
        );
        assert!(
            plan(&a, "power.set_lid_action", Some("default|default"))
                .await
                .unwrap()
                .steps[0]
                .command
                .starts_with("sudo rm ")
        );
        for (action, subject, want) in [
            (
                "power.set_battery_turbo",
                Some("off"),
                Call::SetBatteryTurbo { on: false },
            ),
            (
                "storage.cap_journal",
                Some("500"),
                Call::SetJournalCap { mb: Some(500) },
            ),
            (
                "storage.cap_journal",
                Some("off"),
                Call::SetJournalCap { mb: None },
            ),
            (
                "storage.keep_fewer_snaps",
                Some("2"),
                Call::SetSnapRetain { keep: 2 },
            ),
            (
                "packages.remove_unused_flatpaks",
                Some("user"),
                Call::RemoveUnusedFlatpaks { system: false },
            ),
            (
                "system.raise_file_watches",
                Some("524288"),
                Call::SetFileWatchLimit {
                    value: 524_288,
                    keep: true,
                },
            ),
            (
                "system.enable_zswap",
                Some("on"),
                Call::SetZswap { on: true },
            ),
            (
                "storage.set_disk_scheduler",
                Some("sda|mq-deadline"),
                Call::SetDiskScheduler {
                    disk: "sda".into(),
                    scheduler: "mq-deadline".into(),
                },
            ),
            ("packages.finish_setup", None, Call::FinishPackageSetup),
            (
                "system.update_deecure",
                Some("0.1.0-alpha.4"),
                Call::InstallUpdate {
                    version: "0.1.0-alpha.4".into(),
                },
            ),
            (
                "system.set_auto_update",
                Some("on"),
                Call::SetAutoUpdate {
                    on: true,
                    check: deecure_core::update::CheckEvery::Daily,
                    channel: deecure_core::update::Channel::Prerelease,
                },
            ),
            (
                "packages.install_driver",
                Some("mesa-va-drivers"),
                Call::InstallPackages {
                    packages: vec!["mesa-va-drivers".into()],
                },
            ),
            (
                "system.rebuild_dkms",
                Some("6.8.0-47-generic"),
                Call::RebuildDkms {
                    kernel: "6.8.0-47-generic".into(),
                },
            ),
        ] {
            assert_eq!(
                call(plan(&a, action, subject).await.unwrap()),
                want,
                "{action}"
            );
        }
        for (action, subject) in [
            ("storage.cap_journal", "10"),
            ("system.update_deecure", "0.1.0; reboot"),
            ("system.update_deecure", "latest"),
            ("system.set_auto_update", "sometimes"),
            ("storage.keep_fewer_snaps", "1"),
            ("packages.remove_unused_flatpaks", "everyone"),
            ("system.raise_file_watches", "10"),
            ("storage.set_disk_scheduler", "sda|kyber"),
            ("power.set_lid_action", "hibernate|lock"),
            ("power.set_lid_action", "suspend"),
            ("power.set_battery_cpu_limit", "20"),
            ("power.set_battery_cpu_limit", "100"),
            ("power.set_suspend_mode", "deep"),
            ("power.set_suspend_mode", "deep|forever"),
            ("power.set_aspm_policy", "max|try"),
            ("power.set_charge_limits", "BAT0:80"),
            ("power.set_charge_limits", "BAT0:x:80"),
            ("power.set_turbo", "maybe"),
            ("power.set_radio", "nfc|on"),
            ("power.set_device_power_saving", "pci0000:00/0000:00:14.0"),
            ("power.set_keyboard_backlight", "bright"),
            ("power.set_wifi_power_save", "uuid|sometimes"),
        ] {
            assert!(
                plan(&a, action, Some(subject)).await.is_err(),
                "{action} {subject}"
            );
        }
        assert!(
            !Call::SetRadio {
                kind: "wlan".into(),
                on: true
            }
            .admin()
        );
        assert!(Call::SetTurbo { on: true }.admin());
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
