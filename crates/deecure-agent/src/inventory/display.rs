// SPDX-License-Identifier: Apache-2.0

//! Screens: every connector from `/sys/class/drm`, with the current mode, scale, and
//! primary screen from GNOME's display settings (read-only D-Bus) when GNOME runs.

use std::collections::HashMap;

use serde::Serialize;
use zbus::zvariant::OwnedValue;

use crate::collect::SystemRoot;
use crate::context::{Agent, Bus};

const MUTTER: &str = "org.gnome.Mutter.DisplayConfig";

/// One screen connector.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Output {
    /// Such as `eDP-1` or `HDMI-A-1`.
    pub connector: String,
    pub connected: bool,
    /// The screen's own name, from its EDID or GNOME.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// True for a laptop's own screen.
    pub builtin: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_hz: Option<f64>,
    /// 1.0 = 100%.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    pub primary: bool,
}

/// The `display` inventory.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct DisplayInfo {
    /// "Mutter" when GNOME's display settings answered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compositor: Option<String>,
    /// GNOME Shell version, such as "46.0".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_version: Option<String>,
    /// True when GNOME's fractional scaling is turned on.
    pub fractional_scaling: bool,
    pub outputs: Vec<Output>,
}

/// The monitor name from an EDID block: the descriptor tagged 0xFC.
pub fn edid_name(edid: &[u8]) -> Option<String> {
    (0..4).find_map(|i| {
        let d = edid.get(54 + i * 18..72 + i * 18)?;
        (d[..3] == [0, 0, 0] && d[3] == 0xFC).then(|| {
            String::from_utf8_lossy(&d[5..18])
                .trim_end_matches(['\n', ' ', '\0'])
                .to_owned()
        })
    })
}

fn connectors(root: &SystemRoot) -> Vec<Output> {
    let mut out: Vec<Output> = std::fs::read_dir(root.path("/sys/class/drm"))
        .map(|d| {
            d.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
        .into_iter()
        // `card1-eDP-1` → connector `eDP-1`; `card1` and `renderD128` are not connectors.
        .filter_map(|n| {
            n.split_once('-')
                .filter(|(card, _)| card.starts_with("card"))
                .map(|(_, c)| (n.clone(), c.to_owned()))
        })
        .map(|(dir, connector)| {
            let base = format!("/sys/class/drm/{dir}");
            let connected = root
                .read(&format!("{base}/status"))
                .is_ok_and(|s| s.trim() == "connected");
            Output {
                builtin: connector.starts_with("eDP")
                    || connector.starts_with("LVDS")
                    || connector.starts_with("DSI"),
                name: connected
                    .then(|| {
                        std::fs::read(root.path(&format!("{base}/edid")))
                            .ok()
                            .as_deref()
                            .and_then(edid_name)
                    })
                    .flatten(),
                connected,
                connector,
                ..Output::default()
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.connected
            .cmp(&a.connected)
            .then_with(|| a.connector.cmp(&b.connector))
    });
    out
}

type Mode = (
    String,
    i32,
    i32,
    f64,
    f64,
    Vec<f64>,
    HashMap<String, OwnedValue>,
);
type Spec = (String, String, String, String);
type Monitor = (Spec, Vec<Mode>, HashMap<String, OwnedValue>);
type Logical = (
    i32,
    i32,
    f64,
    u32,
    bool,
    Vec<Spec>,
    HashMap<String, OwnedValue>,
);

async fn mutter(agent: &Agent, outputs: &mut [Output]) -> zbus::Result<()> {
    let bus = agent.bus(Bus::Session).await?;
    let reply = bus
        .call_method(
            Some(MUTTER),
            "/org/gnome/Mutter/DisplayConfig",
            Some(MUTTER),
            "GetCurrentState",
            &(),
        )
        .await?;
    let (_, monitors, logical, _): (u32, Vec<Monitor>, Vec<Logical>, HashMap<String, OwnedValue>) =
        reply.body().deserialize()?;
    let flag = |props: &HashMap<String, OwnedValue>, key: &str| {
        props
            .get(key)
            .and_then(|v| bool::try_from(v).ok())
            .unwrap_or(false)
    };
    for (spec, modes, props) in monitors {
        let Some(out) = outputs.iter_mut().find(|o| o.connector == spec.0) else {
            continue;
        };
        out.connected = true;
        if let Some(name) = props
            .get("display-name")
            .and_then(|v| v.try_clone().ok())
            .and_then(|v| String::try_from(v).ok())
        {
            out.name = Some(name);
        }
        out.builtin |= flag(&props, "is-builtin");
        if let Some((_, w, h, hz, ..)) = modes.iter().find(|m| flag(&m.6, "is-current")) {
            out.width = u32::try_from(*w).ok();
            out.height = u32::try_from(*h).ok();
            out.refresh_hz = Some((hz * 100.0).round() / 100.0);
        }
        if let Some((_, _, scale, _, primary, ..)) =
            logical.iter().find(|l| l.5.iter().any(|s| s.0 == spec.0))
        {
            out.scale = Some(*scale);
            out.primary = *primary;
        }
    }
    Ok(())
}

async fn shell_version(agent: &Agent) -> Option<String> {
    let bus = agent.bus(Bus::Session).await.ok()?;
    let reply = bus
        .call_method(
            Some("org.gnome.Shell"),
            "/org/gnome/Shell",
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.gnome.Shell", "ShellVersion"),
        )
        .await
        .ok()?;
    let value: OwnedValue = reply.body().deserialize().ok()?;
    String::try_from(value).ok()
}

/// Reads the screens. GNOME is asked only when its display service already runs.
pub async fn read(agent: &Agent) -> DisplayInfo {
    let mut info = DisplayInfo {
        outputs: connectors(&agent.root),
        ..DisplayInfo::default()
    };
    if agent.is_running(Bus::Session, MUTTER).await {
        if mutter(agent, &mut info.outputs).await.is_ok() {
            info.compositor = Some("Mutter".into());
        } else {
            agent.drop_bus(Bus::Session).await;
        }
        info.shell_version = shell_version(agent).await;
        info.fractional_scaling = super::command::output(
            "gsettings",
            &["get", "org.gnome.mutter", "experimental-features"],
            std::time::Duration::from_secs(3),
        )
        .await
        .is_some_and(|v| v.contains("scale-monitor-framebuffer"));
    }
    info
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    fn edid(name: &str) -> Vec<u8> {
        let mut e = vec![0u8; 128];
        let d = &mut e[72..90];
        d[3] = 0xFC;
        let text = format!("{name}\n");
        d[5..5 + text.len()].copy_from_slice(text.as_bytes());
        e
    }

    #[test]
    fn names_from_edid() {
        assert_eq!(
            edid_name(&edid("DELL U2720Q")).as_deref(),
            Some("DELL U2720Q")
        );
        assert_eq!(edid_name(&[0; 10]), None);
    }

    #[test]
    fn lists_connectors_connected_first() {
        let root = fixture_root(
            "display",
            &[
                ("/sys/class/drm/card1-HDMI-A-1/status", "disconnected\n"),
                ("/sys/class/drm/card1-eDP-1/status", "connected\n"),
                ("/sys/class/drm/card1/dev", "226:1\n"),
                ("/sys/class/drm/renderD128/dev", "x"),
            ],
        );
        let outs = connectors(&root);
        assert_eq!(
            outs.iter()
                .map(|o| (o.connector.as_str(), o.connected, o.builtin))
                .collect::<Vec<_>>(),
            [("eDP-1", true, true), ("HDMI-A-1", false, false)]
        );
    }
}
