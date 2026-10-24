// SPDX-License-Identifier: Apache-2.0

//! Fixes: what a fix will do (a [`Plan`] of [`Step`]s), how each step runs (a fixed
//! [`Call`]), and what happened ([`FixRun`]). See docs/02-architecture/fix-engine.md.
//!
//! A [`Call`] is a closed list of operations. User-level calls run as the user; admin
//! calls map one-to-one to methods of the root helper, which checks every input again.

use serde::{Deserialize, Serialize};

use crate::{FixActionId, RuleId, Timestamp};

/// Every fix DeeCure knows: ID, title, and the target it needs, if any.
pub const ACTIONS: &[(&str, &str, Option<&str>)] = &[
    ("storage.free_space", "Free up disk space", None),
    ("storage.empty_trash", "Empty Trash", None),
    ("storage.clear_thumbnails", "Clear thumbnail cache", None),
    ("storage.clean_apt_cache", "Clean APT package cache", None),
    (
        "storage.remove_old_snap_revisions",
        "Remove disabled snap revisions",
        None,
    ),
    ("storage.vacuum_journal", "Shrink the system log", None),
    (
        "storage.prune_docker_images",
        "Prune unused Docker images",
        None,
    ),
    (
        "storage.prune_docker_build_cache",
        "Clear Docker build cache",
        None,
    ),
    (
        "packages.autoremove",
        "Remove packages no longer needed",
        None,
    ),
    (
        "packages.purge_residual_config",
        "Remove leftover package settings",
        None,
    ),
    (
        "security.install_security_updates",
        "Install security updates",
        None,
    ),
    ("system.install_updates", "Install all updates", None),
    ("security.enable_firewall", "Turn on the firewall", None),
    ("system.remove_old_kernels", "Remove old kernels", None),
    ("services.restart", "Restart a service", Some("unit")),
    ("services.disable", "Turn off a service", Some("unit")),
    ("services.enable", "Turn on a service", Some("unit")),
    (
        "services.disable_startup_app",
        "Stop an app from starting at login",
        Some("app id"),
    ),
    (
        "services.enable_startup_app",
        "Start an app at login",
        Some("app id"),
    ),
    ("processes.end", "End a process", Some("pid:name")),
    (
        "power.set_profile",
        "Change the power mode",
        Some("power-saver|balanced|performance"),
    ),
    (
        "power.set_battery_action",
        "Power off cleanly when the battery is empty",
        None,
    ),
    (
        "power.set_charge_limits",
        "Change the battery charge limits",
        Some("battery:start:stop"),
    ),
    (
        "power.set_charge_mode",
        "Change the battery charge mode",
        Some("battery:mode"),
    ),
    (
        "power.set_turbo",
        "Turn CPU turbo on or off",
        Some("on|off"),
    ),
    (
        "power.set_device_wakeup",
        "Allow or stop a device waking the computer",
        Some("device|on|off"),
    ),
    (
        "power.set_device_power_saving",
        "Turn power saving on or off for a device",
        Some("device|on|off"),
    ),
    (
        "power.set_battery_cpu_limit",
        "Limit the CPU speed on battery",
        Some("percent|off"),
    ),
    (
        "power.set_aspm_policy",
        "Change the PCIe power saving policy",
        Some("policy|try|keep"),
    ),
    (
        "power.set_suspend_mode",
        "Change the suspend type",
        Some("mode|try|keep"),
    ),
    (
        "power.set_lid_action",
        "Change what closing the lid does",
        Some("battery|power"),
    ),
    (
        "power.set_battery_turbo",
        "Turn turbo off on battery",
        Some("on|off"),
    ),
    (
        "storage.cap_journal",
        "Cap the system log size",
        Some("megabytes|off"),
    ),
    (
        "storage.keep_fewer_snaps",
        "Keep fewer old snap versions",
        Some("count"),
    ),
    (
        "packages.remove_unused_flatpaks",
        "Remove unused Flatpak runtimes",
        Some("user|system"),
    ),
    (
        "system.raise_file_watches",
        "Raise the file-watch limit",
        Some("value"),
    ),
    (
        "system.enable_zswap",
        "Turn on compressed swap",
        Some("on|off"),
    ),
    (
        "storage.set_disk_scheduler",
        "Use a better scheduler for a hard disk",
        Some("disk|scheduler"),
    ),
    (
        "packages.finish_setup",
        "Finish a broken package install",
        None,
    ),
    (
        "packages.install_driver",
        "Install a driver package",
        Some("package"),
    ),
    (
        "system.rebuild_dkms",
        "Rebuild extra drivers for a kernel",
        Some("kernel"),
    ),
    (
        "system.update_firmware",
        "Install a firmware update",
        Some("device id"),
    ),
    (
        "power.set_keyboard_backlight",
        "Change the keyboard backlight",
        Some("level"),
    ),
    (
        "power.set_radio",
        "Turn a radio on or off",
        Some("kind|on|off"),
    ),
    (
        "power.set_wifi_power_save",
        "Turn Wi-Fi power saving on or off",
        Some("uuid|on|off|default"),
    ),
];

/// Whether a step can be taken back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Undo {
    /// DeeCure can put it back exactly.
    Full,
    /// What was removed comes back by itself when needed (caches).
    Regenerates,
    /// Gone for good; the user is warned.
    None,
}

/// One fixed operation. The executor knows how to run each; nothing else can be run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Call {
    // User level: no password.
    /// Empty the user's Trash.
    EmptyTrash,
    /// Delete the thumbnail cache in `~/.cache/thumbnails`.
    ClearThumbnails,
    /// `docker image prune --all --force`: images no container uses.
    DockerPruneImages,
    /// `docker builder prune --force`.
    DockerPruneBuildCache,
    /// Turn a login app on or off (a user override in `~/.config/autostart`).
    SetStartupApp {
        /// The `.desktop` file name.
        id: String,
        /// On or off.
        enabled: bool,
    },
    /// Ask one of the user's own processes to quit (SIGTERM).
    EndProcess {
        /// The process ID.
        pid: u32,
        /// Its name, checked again before signalling, so a reused PID is not hit.
        name: String,
    },
    /// Switch the power mode through power-profiles-daemon.
    SetPowerProfile {
        /// `power-saver`, `balanced`, or `performance`.
        profile: String,
    },
    /// Set the keyboard backlight through UPower.
    SetKeyboardBacklight {
        /// A level from 0 to the most the keyboard offers.
        level: u32,
    },
    /// Turn a radio on or off with rfkill.
    SetRadio {
        /// `wlan`, `bluetooth`, or `wwan`.
        kind: String,
        /// On or off.
        on: bool,
    },
    /// Set Wi-Fi power saving for one NetworkManager connection (`802-11-wireless.powersave`).
    SetWifiPowerSave {
        /// The connection's UUID.
        uuid: String,
        /// `0` (default), `2` (off), or `3` (on).
        value: u32,
    },

    // Admin: through the root helper.
    /// `apt-get clean`.
    CleanAptCache,
    /// `snap remove <name> --revision=<rev>` for a disabled revision.
    RemoveSnapRevision {
        /// Snap name.
        name: String,
        /// The disabled revision.
        revision: String,
    },
    /// `journalctl --vacuum-size=<mb>M`.
    VacuumJournal {
        /// Size to keep, in megabytes.
        keep_mb: u32,
    },
    /// `apt-get autoremove`.
    Autoremove,
    /// `apt-get purge` for packages that only left settings behind.
    PurgeResidual {
        /// Package names (each must be in the "config files" state).
        packages: Vec<String>,
    },
    /// `apt-get install --only-upgrade` for the given installed packages.
    UpgradePackages {
        /// Package names (each must be installed and upgradable).
        packages: Vec<String>,
    },
    /// Turn the ufw firewall on or off.
    SetFirewall {
        /// On or off.
        on: bool,
    },
    /// `systemctl restart <unit>`.
    RestartService {
        /// The unit, such as `bluetooth.service`.
        unit: String,
    },
    /// `systemctl enable --now` or `disable --now`.
    SetServiceEnabled {
        /// The unit.
        unit: String,
        /// Enable and start, or disable and stop.
        enabled: bool,
    },
    /// `apt-get purge` for old kernel packages (never the running one or the newest).
    RemoveKernels {
        /// Kernel versions, such as `6.8.0-41-generic`.
        versions: Vec<String>,
    },
    /// Set a battery's charge limits (`charge_control_start_threshold`, `…_end_threshold`).
    SetChargeLimits {
        /// The battery, such as `BAT0`.
        battery: String,
        /// Charging starts below this percent, when the battery has a start limit.
        start: Option<u32>,
        /// Charging stops at this percent.
        stop: u32,
    },
    /// Set a battery's charge mode (`charge_types`), such as `Adaptive` on Dell.
    SetChargeMode {
        /// The battery, such as `BAT0`.
        battery: String,
        /// One of the modes the battery offers.
        mode: String,
    },
    /// Turn CPU turbo on or off (`intel_pstate/no_turbo` or `cpufreq/boost`).
    SetTurbo {
        /// On or off.
        on: bool,
    },
    /// Allow or stop one device waking the computer from suspend (`power/wakeup`).
    SetDeviceWakeup {
        /// The device path under `/sys/devices`, such as `pci0000:00/0000:00:14.0`.
        device: String,
        /// Allowed to wake the computer.
        enabled: bool,
    },
    /// Turn runtime power saving on or off for one device (`power/control`: `auto` or `on`).
    SetDevicePowerSaving {
        /// The device path under `/sys/devices`.
        device: String,
        /// Power saving on (`auto`).
        on: bool,
    },
    /// Set the highest CPU speed on battery (`cpufreq/policy*/scaling_max_freq`). It is set
    /// again each time the computer is plugged in or unplugged, and at each start.
    SetBatteryCpuLimit {
        /// Percent of the CPU's top speed (30-99), or `None` for no limit.
        percent: Option<u32>,
    },
    /// Set the PCIe power saving policy (`/sys/module/pcie_aspm/parameters/policy`).
    SetAspmPolicy {
        /// One the kernel offers: `default`, `performance`, `powersave`, `powersupersave`.
        policy: String,
        /// Kept after restart; `false` is for this session only (and stops keeping it).
        keep: bool,
    },
    /// Set the suspend type (`/sys/power/mem_sleep`).
    SetSuspendMode {
        /// One the computer offers: `s2idle`, `shallow`, or `deep`.
        mode: String,
        /// Kept after restart; `false` is for this session only (and stops keeping it).
        keep: bool,
    },
    /// Set what closing the lid does (a logind drop-in; it applies when logind next starts,
    /// which is the next restart: restarting logind would end the desktop session).
    SetLidAction {
        /// On battery: `suspend`, `lock`, `ignore`, or `poweroff`; `default` with `on_power`
        /// also `default` removes the drop-in.
        on_battery: String,
        /// Plugged in, with the same choices.
        on_power: String,
    },
    /// Turn CPU turbo off while on battery (`on: false`), set again at each plug in and unplug.
    /// On mains, turbo follows the kept turbo setting, or is on.
    SetBatteryTurbo {
        /// Turbo on battery.
        on: bool,
    },
    /// Cap the system log (`SystemMaxUse=` in a journald drop-in), and shrink it to the cap now.
    /// `None` removes the drop-in.
    SetJournalCap {
        /// Megabytes, 100-10000.
        mb: Option<u32>,
    },
    /// How many versions of each snap snapd keeps (`snap set system refresh.retain=`).
    SetSnapRetain {
        /// 2-20.
        keep: u32,
    },
    /// `flatpak uninstall --unused`: runtimes no installed app uses.
    RemoveUnusedFlatpaks {
        /// The system installation (admin), or the user's own (no password).
        system: bool,
    },
    /// Set the file-watch limit (`fs.inotify.max_user_watches`) now, and keep it in
    /// `/etc/sysctl.d/60-deecure-inotify.conf` (`keep: false` removes that file).
    SetFileWatchLimit {
        /// 8192-4194304.
        value: u64,
        /// Kept in the settings file; `false` removes the file.
        keep: bool,
    },
    /// Turn compressed swap (zswap) on or off (`/sys/module/zswap/parameters/enabled`), kept
    /// after restart.
    SetZswap {
        /// On or off.
        on: bool,
    },
    /// Set a disk's I/O scheduler (`/sys/block/<disk>/queue/scheduler`), kept after restart.
    SetDiskScheduler {
        /// Such as `sda`.
        disk: String,
        /// One the disk offers, such as `mq-deadline`.
        scheduler: String,
    },
    /// `dpkg --configure -a`: finish package installs that stopped half way.
    FinishPackageSetup,
    /// `apt-get install` for a hardware video driver or a driver `ubuntu-drivers` offers.
    InstallPackages {
        /// Package names.
        packages: Vec<String>,
    },
    /// `dkms autoinstall -k <kernel>`: build the extra drivers for an installed kernel.
    RebuildDkms {
        /// Such as `6.8.0-47-generic`.
        kernel: String,
    },
    /// `fwupdmgr update <device>`: install the firmware update fwupd offers (fwupd asks for
    /// the password itself).
    UpdateFirmware {
        /// fwupd's device ID: 40 hex characters.
        device: String,
    },
    /// Set UPower's critical battery action.
    SetBatteryAction {
        /// `PowerOff`, `Hibernate`, or `HybridSleep`.
        action: String,
        /// Battery percent at which it happens (1-20).
        percent: u32,
    },
}

impl Call {
    /// True when the step needs the root helper.
    pub fn admin(&self) -> bool {
        !matches!(
            self,
            Call::EmptyTrash
                | Call::ClearThumbnails
                | Call::DockerPruneImages
                | Call::DockerPruneBuildCache
                | Call::SetStartupApp { .. }
                | Call::EndProcess { .. }
                | Call::SetPowerProfile { .. }
                | Call::SetKeyboardBacklight { .. }
                | Call::SetRadio { .. }
                | Call::SetWifiPowerSave { .. }
                | Call::RemoveUnusedFlatpaks { system: false }
                | Call::UpdateFirmware { .. }
        )
    }
}

/// One step of a plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    /// Unique within the plan, such as `apt`.
    pub id: String,
    /// What it does, in a few words.
    pub title: String,
    /// More detail, such as "61 downloaded .deb files".
    pub description: String,
    /// Bytes it frees, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// Can it be taken back?
    pub undo: Undo,
    /// Ticked in the preview by default.
    pub checked: bool,
    /// The command, as the user would type it, for the preview.
    pub command: String,
    /// What actually runs.
    pub call: Call,
}

/// Everything a fix will do, shown before anything runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    /// The fix.
    pub action: FixActionId,
    /// Such as "Free up disk space".
    pub title: String,
    /// Such as "Root filesystem ( / )".
    pub subtitle: String,
    /// The finding it fixes, if any; its rule is checked again afterwards.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<RuleId>,
    /// The finding's subject.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The steps, in the order they run.
    pub steps: Vec<Step>,
}

/// What happened to one step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepResult {
    /// The step's ID.
    pub id: String,
    /// The step's title.
    pub title: String,
    /// True when it worked.
    pub ok: bool,
    /// What happened, in plain words.
    pub message: String,
    /// The call that takes it back, for steps with a full undo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub undo: Option<Call>,
}

/// How a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    /// Every chosen step worked.
    Succeeded,
    /// A step failed; later steps did not run.
    Failed,
    /// It was taken back.
    Undone,
}

/// One time a fix ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixRun {
    /// Its number in the database; 0 before it is saved.
    #[serde(default)]
    pub id: i64,
    /// The fix.
    pub action: FixActionId,
    /// The plan's title.
    pub title: String,
    /// The finding it was for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<RuleId>,
    /// The finding's subject.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// When it started.
    pub started_at: Timestamp,
    /// When it finished.
    pub finished_at: Timestamp,
    /// How it ended.
    pub status: RunStatus,
    /// What each chosen step did.
    pub steps: Vec<StepResult>,
}

impl FixRun {
    /// The calls that take this run back, last step first. Empty when nothing can be undone.
    pub fn undo_calls(&self) -> Vec<Call> {
        self.steps
            .iter()
            .rev()
            .filter(|s| s.ok)
            .filter_map(|s| s.undo.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn calls_have_a_stable_shape_and_know_if_they_need_admin() {
        assert_eq!(
            serde_json::to_value(Call::RestartService {
                unit: "cups.service".into()
            })
            .unwrap(),
            json!({"op": "restart_service", "unit": "cups.service"})
        );
        assert_eq!(
            serde_json::to_value(Call::EmptyTrash).unwrap(),
            json!({"op": "empty_trash"})
        );
        assert!(Call::CleanAptCache.admin());
        assert!(!Call::EmptyTrash.admin());
        assert!(
            serde_json::from_value::<Call>(json!({"op": "rm_rf", "path": "/"})).is_err(),
            "unknown operations cannot be read"
        );
    }

    #[test]
    fn undo_runs_last_step_first_and_skips_failures() {
        let step = |id: &str, ok: bool, undo: Option<Call>| StepResult {
            id: id.into(),
            title: id.into(),
            ok,
            message: String::new(),
            undo,
        };
        let run = FixRun {
            id: 1,
            action: FixActionId::new("services.disable").unwrap(),
            title: "x".into(),
            rule: None,
            subject: None,
            started_at: Timestamp(0),
            finished_at: Timestamp(1),
            status: RunStatus::Failed,
            steps: vec![
                step("a", true, Some(Call::SetFirewall { on: false })),
                step(
                    "b",
                    true,
                    Some(Call::SetServiceEnabled {
                        unit: "x.service".into(),
                        enabled: true,
                    }),
                ),
                step("c", false, Some(Call::EmptyTrash)),
            ],
        };
        assert_eq!(
            run.undo_calls(),
            [
                Call::SetServiceEnabled {
                    unit: "x.service".into(),
                    enabled: true
                },
                Call::SetFirewall { on: false }
            ]
        );
    }
}
