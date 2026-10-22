// SPDX-License-Identifier: Apache-2.0

//! systemd units read live over D-Bus: user services, memory per system service, and
//! timers. Also startup applications from autostart files. All read-only.

use std::collections::{BTreeMap, HashMap};

use serde::Serialize;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

use crate::collect::SystemRoot;
use crate::context::{Agent, Bus};

const SYSTEMD: &str = "org.freedesktop.systemd1";
const MANAGER: &str = "/org/freedesktop/systemd1";

type UnitRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    OwnedObjectPath,
    u32,
    String,
    OwnedObjectPath,
);
type Props = HashMap<String, OwnedValue>;

fn get<T: TryFrom<OwnedValue>>(p: &Props, key: &str) -> Option<T> {
    T::try_from(p.get(key)?.try_clone().ok()?).ok()
}

async fn list_units(bus: &zbus::Connection) -> zbus::Result<Vec<UnitRow>> {
    let reply = bus
        .call_method(
            Some(SYSTEMD),
            MANAGER,
            Some("org.freedesktop.systemd1.Manager"),
            "ListUnits",
            &(),
        )
        .await?;
    reply.body().deserialize()
}

async fn get_all(bus: &zbus::Connection, path: &str, interface: &str) -> Option<Props> {
    let reply = bus
        .call_method(
            Some(SYSTEMD),
            path,
            Some("org.freedesktop.DBus.Properties"),
            "GetAll",
            &(interface,),
        )
        .await
        .ok()?;
    reply.body().deserialize().ok()
}

/// systemd says "not known" with the largest number.
fn known(bytes: Option<u64>) -> Option<u64> {
    bytes.filter(|b| *b != u64::MAX)
}

/// One user service.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UserService {
    pub name: String,
    pub description: String,
    pub active: String,
    pub sub: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restarts: Option<u32>,
}

/// The user's own services (`systemctl --user`).
pub async fn user_services(agent: &Agent) -> Result<Vec<UserService>, String> {
    let bus = agent
        .bus(Bus::Session)
        .await
        .map_err(|e| format!("session bus: {e}"))?;
    let rows = match list_units(&bus).await {
        Ok(rows) => rows,
        Err(e) => {
            agent.drop_bus(Bus::Session).await;
            return Err(format!("user systemd: {e}"));
        }
    };
    let mut out = Vec::new();
    for (name, description, _load, active, sub, _, path, ..) in
        rows.into_iter().filter(|r| r.0.ends_with(".service"))
    {
        let unit = get_all(&bus, path.as_str(), "org.freedesktop.systemd1.Unit")
            .await
            .unwrap_or_default();
        let service = get_all(&bus, path.as_str(), "org.freedesktop.systemd1.Service")
            .await
            .unwrap_or_default();
        out.push(UserService {
            enabled: get::<String>(&unit, "UnitFileState").filter(|s| !s.is_empty()),
            memory: known(get(&service, "MemoryCurrent")),
            restarts: get(&service, "NRestarts"),
            name,
            description,
            active,
            sub,
        });
    }
    out.sort_by(|a, b| {
        (a.active != "failed")
            .cmp(&(b.active != "failed"))
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(out)
}

/// Memory in use by each running system service, by name.
pub async fn service_memory(agent: &Agent) -> Result<BTreeMap<String, u64>, String> {
    let bus = agent
        .bus(Bus::System)
        .await
        .map_err(|e| format!("system bus: {e}"))?;
    let rows = list_units(&bus).await.map_err(|e| e.to_string())?;
    let mut out = BTreeMap::new();
    for (name, _, _, active, _, _, path, ..) in rows {
        if !name.ends_with(".service") || active != "active" {
            continue;
        }
        let Some(props) = get_all(&bus, path.as_str(), "org.freedesktop.systemd1.Service").await
        else {
            continue;
        };
        if let Some(bytes) = known(get(&props, "MemoryCurrent")) {
            out.insert(name, bytes);
        }
    }
    Ok(out)
}

/// One timer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Timer {
    pub name: String,
    /// `system` or `user`.
    pub scope: String,
    /// The unit it starts.
    pub activates: String,
    /// Calendar rules such as `*-*-* 06:00:00`, and time rules such as `OnBootSec=15min`.
    pub schedule: Vec<String>,
    /// Milliseconds since 1970.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<i64>,
    /// The started unit's last result: `success`, `exit-code`, `timeout`, …
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    pub persistent: bool,
}

fn usec_to_ms(us: Option<u64>) -> Option<i64> {
    us.filter(|u| *u > 0 && *u != u64::MAX)
        .and_then(|u| i64::try_from(u / 1000).ok())
}

/// "1d", "15min", "1h 30min" from microseconds, for monotonic timer rules.
fn span(us: u64) -> String {
    let s = us / 1_000_000;
    match s {
        s if s % 86_400 == 0 => format!("{}d", s / 86_400),
        s if s % 3600 == 0 => format!("{}h", s / 3600),
        s if s % 60 == 0 => format!("{}min", s / 60),
        s => format!("{s}s"),
    }
}

async fn timers_on(agent: &Agent, which: Bus) -> Vec<Timer> {
    let Ok(bus) = agent.bus(which).await else {
        return Vec::new();
    };
    let Ok(rows) = list_units(&bus).await else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (name, _, _, _, _, _, path, ..) in rows.into_iter().filter(|r| r.0.ends_with(".timer")) {
        let Some(t) = get_all(&bus, path.as_str(), "org.freedesktop.systemd1.Timer").await else {
            continue;
        };
        let activates: String = get(&t, "Unit").unwrap_or_default();
        let mut schedule: Vec<String> = get::<Vec<(String, String, u64)>>(&t, "TimersCalendar")
            .unwrap_or_default()
            .into_iter()
            .map(|(_, spec, _)| spec)
            .collect();
        schedule.extend(
            get::<Vec<(String, u64, u64)>>(&t, "TimersMonotonic")
                .unwrap_or_default()
                .into_iter()
                .map(|(base, us, _)| format!("{base}={}", span(us))),
        );
        // The result belongs to the started service; it may not be loaded yet.
        let result = match bus
            .call_method(
                Some(SYSTEMD),
                MANAGER,
                Some("org.freedesktop.systemd1.Manager"),
                "GetUnit",
                &(activates.as_str(),),
            )
            .await
        {
            Ok(reply) => match reply.body().deserialize::<OwnedObjectPath>() {
                Ok(p) => get_all(&bus, p.as_str(), "org.freedesktop.systemd1.Service")
                    .await
                    .and_then(|s| get::<String>(&s, "Result")),
                Err(_) => None,
            },
            Err(_) => None,
        };
        out.push(Timer {
            name,
            scope: if which == Bus::System {
                "system"
            } else {
                "user"
            }
            .into(),
            activates,
            schedule,
            next: usec_to_ms(get(&t, "NextElapseUSecRealtime")),
            last: usec_to_ms(get(&t, "LastTriggerUSec")),
            result,
            persistent: get(&t, "Persistent").unwrap_or(false),
        });
    }
    out
}

/// System and user timers, soonest first.
pub async fn timers(agent: &Agent) -> Vec<Timer> {
    let mut all = timers_on(agent, Bus::System).await;
    all.extend(timers_on(agent, Bus::Session).await);
    all.sort_by_key(|t| t.next.unwrap_or(i64::MAX));
    all
}

/// One app started at login.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StartupApp {
    /// The `.desktop` file name.
    pub id: String,
    pub name: String,
    pub command: String,
    /// `User` (in `~/.config/autostart`) or `System` (in `/etc/xdg/autostart`).
    pub scope: String,
    pub enabled: bool,
}

/// Reads a `[Desktop Entry]` group into keys.
pub fn desktop_entry(text: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let mut in_entry = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
        } else if in_entry && let Some((k, v)) = line.split_once('=') {
            out.entry(k.trim().to_owned())
                .or_insert_with(|| v.trim().to_owned());
        }
    }
    out
}

/// Apps started at login in a GNOME session. A user file overrides the system file of
/// the same name; entries hidden from GNOME or marked `Hidden` are disabled or left out.
pub fn startup(root: &SystemRoot, home: Option<&std::path::Path>) -> Vec<StartupApp> {
    let read = |dir: std::path::PathBuf| -> Vec<(String, HashMap<String, String>)> {
        std::fs::read_dir(dir)
            .map(|d| {
                d.flatten()
                    .filter(|e| e.path().extension().is_some_and(|x| x == "desktop"))
                    .filter_map(|e| {
                        Some((
                            e.file_name().to_string_lossy().into_owned(),
                            desktop_entry(&std::fs::read_to_string(e.path()).ok()?),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut apps: BTreeMap<String, StartupApp> = BTreeMap::new();
    let sources = [("System", root.path("/etc/xdg/autostart"))]
        .into_iter()
        .chain(home.map(|h| ("User", h.join(".config/autostart"))));
    for (scope, dir) in sources {
        for (id, e) in read(dir) {
            let shown_in_gnome = e
                .get("OnlyShowIn")
                .is_none_or(|v| v.split(';').any(|d| d == "GNOME"))
                && e.get("NotShowIn")
                    .is_none_or(|v| !v.split(';').any(|d| d == "GNOME"));
            let enabled = e.get("Hidden").map(String::as_str) != Some("true")
                && e.get("X-GNOME-Autostart-enabled").map(String::as_str) != Some("false");
            // GNOME's own helpers (settings daemons) are part of the desktop, not apps.
            if !shown_in_gnome || id.starts_with("org.gnome.SettingsDaemon.") {
                apps.remove(&id);
                continue;
            }
            apps.insert(
                id.clone(),
                StartupApp {
                    name: e.get("Name").cloned().unwrap_or_else(|| id.clone()),
                    command: e.get("Exec").cloned().unwrap_or_default(),
                    scope: scope.into(),
                    enabled,
                    id,
                },
            );
        }
    }
    apps.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn formats_timer_spans() {
        assert_eq!(span(86_400_000_000), "1d");
        assert_eq!(span(900_000_000), "15min");
        assert_eq!(span(5_400_000_000), "90min");
        assert_eq!(span(7_000_000), "7s");
    }

    #[test]
    fn user_autostart_overrides_system() {
        let root = fixture_root(
            "autostart",
            &[
                (
                    "/etc/xdg/autostart/nm-applet.desktop",
                    "[Desktop Entry]\nName=Network\nExec=nm-applet\nNotShowIn=KDE;\n",
                ),
                (
                    "/etc/xdg/autostart/kde-only.desktop",
                    "[Desktop Entry]\nName=K\nExec=k\nOnlyShowIn=KDE;\n",
                ),
                (
                    "/etc/xdg/autostart/org.gnome.SettingsDaemon.Power.desktop",
                    "[Desktop Entry]\nName=Power\nExec=gsd-power\n",
                ),
                (
                    "/home/riya/.config/autostart/nm-applet.desktop",
                    "[Desktop Entry]\nName=Network\nExec=nm-applet\nX-GNOME-Autostart-enabled=false\n[Desktop Action x]\nName=Ignored\n",
                ),
                (
                    "/home/riya/.config/autostart/telegram.desktop",
                    "[Desktop Entry]\nName=Telegram Desktop\nExec=telegram-desktop -startintray\n",
                ),
            ],
        );
        let apps = startup(&root, Some(&root.path("/home/riya")));
        let summary: Vec<(&str, &str, bool)> = apps
            .iter()
            .map(|a| (a.name.as_str(), a.scope.as_str(), a.enabled))
            .collect();
        assert_eq!(
            summary,
            [
                ("Network", "User", false),
                ("Telegram Desktop", "User", true)
            ]
        );
    }
}
