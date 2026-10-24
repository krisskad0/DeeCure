// SPDX-License-Identifier: Apache-2.0

//! What closing the lid does, read the way logind reads its settings (`man logind.conf`):
//! the first `logind.conf` found in `/etc`, `/run`, `/usr/local/lib`, `/usr/lib`, then every
//! `logind.conf.d/*.conf` drop-in from those folders in file name order (a file in `/etc` replaces
//! one with the same name in `/run`, and so on). Later values win. The value logind runs with now
//! comes from its D-Bus properties: the files only take effect when logind starts.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::collect::SystemRoot;
use crate::context::{Agent, Bus};

/// The drop-in DeeCure writes (through the admin helper).
pub const OURS: &str = "/etc/systemd/logind.conf.d/50-deecure-lid.conf";

/// Folders in rising priority.
const DIRS: [&str; 4] = [
    "/usr/lib/systemd",
    "/usr/local/lib/systemd",
    "/run/systemd",
    "/etc/systemd",
];

/// What closing the lid does.
#[derive(Debug, Default, Serialize)]
pub struct Lid {
    /// On battery (`HandleLidSwitch`, default `suspend`).
    pub on_battery: String,
    /// Plugged in (`HandleLidSwitchExternalPower`, default: the same as on battery).
    pub on_power: String,
    /// The file each value comes from; none means logind's built-in default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_battery_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_power_from: Option<String>,
    /// DeeCure's own drop-in exists.
    pub ours: bool,
    /// What logind runs with now; differs from the files until the next restart.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub running_battery: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub running_power: Option<String>,
}

/// logind's files in the order it reads them: `(path, text)`.
pub fn files(root: &SystemRoot) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Some((path, text)) = DIRS.iter().rev().find_map(|d| {
        let path = format!("{d}/logind.conf");
        root.read(&path).ok().map(|t| (path, t))
    }) {
        out.push((path, text));
    }
    let mut dropins = BTreeMap::new();
    for d in DIRS {
        let dir = format!("{d}/logind.conf.d");
        for e in std::fs::read_dir(root.path(&dir))
            .into_iter()
            .flatten()
            .flatten()
        {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.ends_with(".conf") {
                dropins.insert(name.clone(), format!("{dir}/{name}"));
            }
        }
    }
    out.extend(
        dropins
            .into_values()
            .filter_map(|path| root.read(&path).ok().map(|t| (path, t))),
    );
    out
}

/// The last `key=value` for `key` in a file, outside comments.
fn value(text: &str, key: &str) -> Option<String> {
    text.lines()
        .rev()
        .map(str::trim)
        .filter(|l| !l.starts_with('#') && !l.starts_with(';'))
        .find_map(|l| l.strip_prefix(key)?.trim_start().strip_prefix('='))
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

/// The value logind uses for `key`, and the file it comes from.
fn setting(files: &[(String, String)], key: &str) -> Option<(String, String)> {
    files
        .iter()
        .rev()
        .find_map(|(path, text)| Some((value(text, key)?, path.clone())))
}

/// The lid settings from the files.
pub fn lid(root: &SystemRoot) -> Lid {
    let files = files(root);
    let battery = setting(&files, "HandleLidSwitch");
    let power = setting(&files, "HandleLidSwitchExternalPower");
    let on_battery = battery.as_ref().map_or("suspend", |(v, _)| v).to_owned();
    Lid {
        on_power: power
            .as_ref()
            .map_or(on_battery.clone(), |(v, _)| v.clone()),
        on_battery,
        on_battery_from: battery.map(|(_, f)| f),
        on_power_from: power.map(|(_, f)| f),
        ours: root.path(OURS).exists(),
        running_battery: None,
        running_power: None,
    }
}

/// What logind runs with now. Never starts logind.
pub async fn running(agent: &Agent) -> (Option<String>, Option<String>) {
    const LOGIN: &str = "org.freedesktop.login1";
    if !agent.is_running(Bus::System, LOGIN).await {
        return (None, None);
    }
    let Ok(bus) = agent.bus(Bus::System).await else {
        return (None, None);
    };
    let get = |name: &'static str| {
        let bus = bus.clone();
        async move {
            let reply = bus
                .call_method(
                    Some(LOGIN),
                    "/org/freedesktop/login1",
                    Some("org.freedesktop.DBus.Properties"),
                    "Get",
                    &("org.freedesktop.login1.Manager", name),
                )
                .await
                .ok()?;
            let v: zbus::zvariant::OwnedValue = reply.body().deserialize().ok()?;
            String::try_from(v).ok()
        }
    };
    (
        get("HandleLidSwitch").await,
        get("HandleLidSwitchExternalPower").await,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn later_files_win_and_etc_replaces_the_same_name() {
        let root = fixture_root(
            "logind-lid",
            &[
                (
                    "/etc/systemd/logind.conf",
                    "[Login]\n#HandleLidSwitch=ignore\nHandleLidSwitch=suspend\n",
                ),
                (
                    "/usr/lib/systemd/logind.conf.d/10-vendor.conf",
                    "[Login]\nHandleLidSwitchExternalPower=ignore\n",
                ),
                (
                    "/usr/lib/systemd/logind.conf.d/50-deecure-lid.conf",
                    "[Login]\nHandleLidSwitch=poweroff\n",
                ),
                (
                    "/etc/systemd/logind.conf.d/50-deecure-lid.conf",
                    "[Login]\nHandleLidSwitch=lock\n",
                ),
            ],
        );
        let l = lid(&root);
        assert_eq!(l.on_battery, "lock");
        assert_eq!(l.on_battery_from.as_deref(), Some(OURS));
        assert_eq!(l.on_power, "ignore");
        assert_eq!(
            l.on_power_from.as_deref(),
            Some("/usr/lib/systemd/logind.conf.d/10-vendor.conf")
        );
        assert!(l.ours);
    }

    #[test]
    fn without_files_logind_defaults_apply() {
        let root = fixture_root(
            "logind-none",
            &[(
                "/usr/lib/systemd/logind.conf",
                "[Login]\n#HandleLidSwitch=suspend\n",
            )],
        );
        let l = lid(&root);
        assert_eq!(
            (l.on_battery.as_str(), l.on_power.as_str()),
            ("suspend", "suspend")
        );
        assert!(l.on_battery_from.is_none() && !l.ours);
        // Plugged in follows the battery value when it is not set.
        let root = fixture_root(
            "logind-follow",
            &[(
                "/etc/systemd/logind.conf",
                "[Login]\nHandleLidSwitch=lock\n",
            )],
        );
        assert_eq!(lid(&root).on_power, "lock");
    }
}
