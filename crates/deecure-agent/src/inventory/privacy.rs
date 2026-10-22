// SPDX-License-Identifier: Apache-2.0

//! Privacy: which apps use the microphone, camera, or screen right now (PipeWire), what
//! snap and Flatpak apps may access, and the desktop's privacy settings. Read-only.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use super::command;
use crate::collect::SystemRoot;

/// Apps using a device now.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct InUse {
    pub microphone: Vec<String>,
    pub camera: Vec<String>,
    pub screen: Vec<String>,
}

/// What one sandboxed app may use.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AppAccess {
    pub app: String,
    /// `snap` or `flatpak`.
    pub kind: String,
    pub camera: bool,
    pub microphone: bool,
    pub location: bool,
    pub files: bool,
    pub network: bool,
}

/// One privacy setting and its value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Setting {
    pub label: String,
    pub description: String,
    pub on: bool,
}

/// The `privacy` inventory.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PrivacyInfo {
    pub in_use: InUse,
    pub apps: Vec<AppAccess>,
    pub settings: Vec<Setting>,
}

/// Recording streams in a `pw-dump`: audio input (microphone), video input from a camera,
/// and screen casts.
pub fn in_use(dump: &str) -> InUse {
    let mut out = InUse::default();
    let Ok(Value::Array(objects)) = serde_json::from_str::<Value>(dump) else {
        return out;
    };
    for o in &objects {
        let props = o.pointer("/info/props");
        let prop = |k: &str| {
            props
                .and_then(|p| p.get(k))
                .and_then(Value::as_str)
                .unwrap_or("")
        };
        let app = [
            prop("application.name"),
            prop("application.process.binary"),
            prop("node.name"),
        ]
        .into_iter()
        .find(|s| !s.is_empty())
        .unwrap_or("Unknown app")
        .to_owned();
        let screen = ["screen", "screencast", "xdg-desktop-portal"]
            .iter()
            .any(|w| {
                prop("node.name").contains(w) || prop("media.name").to_lowercase().contains(w)
            });
        match prop("media.class") {
            "Stream/Input/Audio" => out.microphone.push(app),
            "Stream/Input/Video" if screen => out.screen.push(app),
            "Stream/Input/Video" => out.camera.push(app),
            "Video/Source" if screen && prop("node.name").contains("screencast") => {
                out.screen.push(app)
            }
            _ => {}
        }
    }
    for list in [&mut out.microphone, &mut out.camera, &mut out.screen] {
        list.sort();
        list.dedup();
    }
    out
}

/// Reads `snap connections`: interface, plug (`app:plug`), slot, notes.
pub fn snap_access(text: &str) -> Vec<AppAccess> {
    let mut apps: BTreeMap<String, AppAccess> = BTreeMap::new();
    for l in text.lines().skip(1) {
        let f: Vec<&str> = l.split_whitespace().collect();
        let (Some(iface), Some(plug), Some(slot)) = (f.first(), f.get(1), f.get(2)) else {
            continue;
        };
        if *slot == "-" {
            continue; // not connected
        }
        let Some(app) = plug.split(':').next() else {
            continue;
        };
        let a = apps.entry(app.to_owned()).or_insert_with(|| AppAccess {
            app: app.to_owned(),
            kind: "snap".into(),
            ..AppAccess::default()
        });
        match *iface {
            "camera" => a.camera = true,
            "audio-record" => a.microphone = true,
            "location-observe" | "location-control" => a.location = true,
            "home" | "removable-media" | "personal-files" => a.files = true,
            "network" => a.network = true,
            _ => {}
        }
    }
    apps.into_values().collect()
}

/// Reads `flatpak info --show-permissions` for one app.
pub fn flatpak_access(app: &str, text: &str) -> AppAccess {
    let line = |key: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(&format!("{key}=")))
            .unwrap_or("")
    };
    let has = |list: &str, item: &str| list.split(';').any(|x| x == item);
    AppAccess {
        app: app.to_owned(),
        kind: "flatpak".into(),
        camera: has(line("devices"), "all"),
        microphone: has(line("sockets"), "pulseaudio"),
        location: text.contains("org.freedesktop.GeoClue2=talk"),
        files: ["home", "host", "xdg-documents", "xdg-download"]
            .iter()
            .any(|f| {
                line("filesystems")
                    .split(';')
                    .any(|x| x.trim_end_matches(":ro") == *f)
            }),
        network: has(line("shared"), "network"),
    }
}

/// Reads `gsettings list-recursively <schema>` lines into `key → value`.
fn gsettings(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter_map(|l| {
            let mut f = l.splitn(3, ' ');
            let (_, key, value) = (f.next()?, f.next()?, f.next()?);
            Some((key.to_owned(), value.trim().to_owned()))
        })
        .collect()
}

pub async fn read(root: &SystemRoot) -> PrivacyInfo {
    let in_use = command::output("pw-dump", &[], Duration::from_secs(5))
        .await
        .map(|d| in_use(&d))
        .unwrap_or_default();
    let mut apps = command::output("snap", &["connections"], Duration::from_secs(10))
        .await
        .map(|t| snap_access(&t))
        .unwrap_or_default();
    if root.path("/usr/bin/flatpak").exists()
        && let Some(list) = command::output(
            "flatpak",
            &["list", "--app", "--columns=application"],
            Duration::from_secs(10),
        )
        .await
    {
        for app in list.lines().map(str::trim).filter(|a| !a.is_empty()) {
            if let Some(info) = command::output(
                "flatpak",
                &["info", "--show-permissions", app],
                Duration::from_secs(5),
            )
            .await
            {
                apps.push(flatpak_access(app, &info));
            }
        }
    }
    let mut s = BTreeMap::new();
    for schema in [
        "org.gnome.desktop.privacy",
        "org.gnome.system.location",
        "org.gnome.desktop.screensaver",
        "org.gnome.desktop.notifications",
    ] {
        if let Some(text) = command::output(
            "gsettings",
            &["list-recursively", schema],
            Duration::from_secs(3),
        )
        .await
        {
            for (k, v) in gsettings(&text) {
                s.insert(format!("{schema}.{k}"), v);
            }
        }
    }
    let on = |key: &str| s.get(key).map(|v| v == "true");
    let mut settings = Vec::new();
    let mut add = |label: &str, description: &str, value: Option<bool>| {
        if let Some(on) = value {
            settings.push(Setting {
                label: label.into(),
                description: description.into(),
                on,
            });
        }
    };
    add(
        "Location services",
        "Apps can ask where you are",
        on("org.gnome.system.location.enabled"),
    );
    add(
        "Problem reporting",
        "Send crash reports to Canonical",
        on("org.gnome.desktop.privacy.report-technical-problems"),
    );
    add(
        "Usage statistics",
        "Send software usage data",
        on("org.gnome.desktop.privacy.send-software-usage-stats"),
    );
    add(
        "Remember recent files",
        "Apps can show files you opened recently",
        on("org.gnome.desktop.privacy.remember-recent-files"),
    );
    add(
        "Delete old Trash",
        "Empty items older than 30 days automatically",
        on("org.gnome.desktop.privacy.remove-old-trash-files"),
    );
    add(
        "Delete old temporary files",
        "Clean temporary files automatically",
        on("org.gnome.desktop.privacy.remove-old-temp-files"),
    );
    add(
        "Camera turned off",
        "Block every app from the camera",
        on("org.gnome.desktop.privacy.disable-camera"),
    );
    add(
        "Microphone turned off",
        "Block every app from the microphone",
        on("org.gnome.desktop.privacy.disable-microphone"),
    );
    add(
        "Screen lock",
        "Lock the screen when it blanks",
        on("org.gnome.desktop.screensaver.lock-enabled"),
    );
    add(
        "Notifications on the lock screen",
        "Others may read them while locked",
        on("org.gnome.desktop.notifications.show-in-lock-screen"),
    );
    let popcon = root
        .read("/etc/popularity-contest.conf")
        .ok()
        .map(|t| t.contains("PARTICIPATE=\"yes\""));
    add(
        "Package popularity contest",
        "Send a list of installed packages weekly",
        popcon,
    );
    PrivacyInfo {
        in_use,
        apps,
        settings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_recording_apps() {
        let dump = r#"[
          {"type":"PipeWire:Interface:Node","info":{"props":{"media.class":"Stream/Input/Audio","application.name":"Zoom"}}},
          {"type":"PipeWire:Interface:Node","info":{"props":{"media.class":"Stream/Input/Video","application.name":"Zoom"}}},
          {"type":"PipeWire:Interface:Node","info":{"props":{"media.class":"Stream/Output/Audio","application.name":"Firefox"}}}
        ]"#;
        let u = in_use(dump);
        assert_eq!(
            (u.microphone.as_slice(), u.camera.as_slice()),
            (
                ["Zoom".to_owned()].as_slice(),
                ["Zoom".to_owned()].as_slice()
            )
        );
        assert!(u.screen.is_empty());
    }

    #[test]
    fn reads_snap_and_flatpak_access() {
        let snaps = "Interface Plug Slot Notes\ncamera firefox:camera :camera -\naudio-record firefox:audio-record :audio-record -\nhome vlc:home :home -\nnetwork vlc:network :network -\ncamera vlc:camera - -\n";
        let a = snap_access(snaps);
        assert_eq!(
            a.iter()
                .map(|x| (x.app.as_str(), x.camera, x.files))
                .collect::<Vec<_>>(),
            [("firefox", true, false), ("vlc", false, true)]
        );
        let info = "[Context]\nshared=network;ipc;\nsockets=x11;wayland;pulseaudio;\ndevices=dri;\nfilesystems=xdg-download;\n";
        let f = flatpak_access("org.telegram.desktop", info);
        assert!(f.microphone && f.network && f.files && !f.camera);
    }
}
