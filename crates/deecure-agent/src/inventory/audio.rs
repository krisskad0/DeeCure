// SPDX-License-Identifier: Apache-2.0

//! Sound devices and streams, from the PipeWire graph (`pw-dump`, read-only).

use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use super::command;

/// One output or input device.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AudioDevice {
    /// Internal name.
    pub name: String,
    /// Name for people, such as "Speaker + Headphones".
    pub description: String,
    /// Volume in percent, as the desktop shows it (cube root of the linear value).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    pub muted: bool,
    pub default: bool,
}

/// The `audio` inventory.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AudioInfo {
    /// Such as "PipeWire 1.0.5". Missing when no sound server answered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    pub outputs: Vec<AudioDevice>,
    pub inputs: Vec<AudioDevice>,
    /// Apps playing sound now.
    pub streams: Vec<String>,
}

/// Reads a `pw-dump` JSON array.
pub fn parse(dump: &str) -> AudioInfo {
    let Ok(Value::Array(objects)) = serde_json::from_str::<Value>(dump) else {
        return AudioInfo::default();
    };
    let mut info = AudioInfo::default();
    let mut defaults = (None, None);
    for o in &objects {
        let props = o.pointer("/info/props").or_else(|| o.get("props"));
        let prop = |k: &str| props.and_then(|p| p.get(k)).and_then(Value::as_str);
        match o.get("type").and_then(Value::as_str) {
            Some("PipeWire:Interface:Core") => {
                if let Some(v) = o.pointer("/info/version").and_then(Value::as_str) {
                    info.server = Some(format!("PipeWire {v}"));
                }
            }
            Some("PipeWire:Interface:Metadata") if prop("metadata.name") == Some("default") => {
                for m in o
                    .get("metadata")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    let name = m
                        .pointer("/value/name")
                        .and_then(Value::as_str)
                        .map(str::to_owned);
                    match m.get("key").and_then(Value::as_str) {
                        Some("default.audio.sink") => defaults.0 = name,
                        Some("default.audio.source") => defaults.1 = name,
                        _ => {}
                    }
                }
            }
            Some("PipeWire:Interface:Node") => {
                let class = prop("media.class").unwrap_or("");
                if class == "Stream/Output/Audio" {
                    if let Some(app) = prop("application.name").or(prop("node.name")) {
                        info.streams.push(app.to_owned());
                    }
                    continue;
                }
                let (Some(name), true) = (
                    prop("node.name"),
                    class == "Audio/Sink" || class == "Audio/Source",
                ) else {
                    continue;
                };
                let settings = o
                    .pointer("/info/params/Props")
                    .and_then(Value::as_array)
                    .and_then(|p| p.iter().find(|x| x.get("channelVolumes").is_some()));
                let linear = settings.and_then(|s| {
                    s.get("channelVolumes")?
                        .as_array()?
                        .iter()
                        .filter_map(Value::as_f64)
                        .reduce(f64::max)
                });
                let device = AudioDevice {
                    name: name.to_owned(),
                    description: prop("node.description").unwrap_or(name).to_owned(),
                    volume: linear.map(|v| (v.cbrt() * 100.0).round()),
                    muted: settings
                        .and_then(|s| s.get("mute")?.as_bool())
                        .unwrap_or(false),
                    default: false,
                };
                if class == "Audio/Sink" {
                    info.outputs.push(device)
                } else {
                    info.inputs.push(device)
                }
            }
            _ => {}
        }
    }
    for d in &mut info.outputs {
        d.default = defaults.0.as_deref() == Some(d.name.as_str());
    }
    for d in &mut info.inputs {
        d.default = defaults.1.as_deref() == Some(d.name.as_str());
    }
    info.outputs.sort_by(|a, b| {
        b.default
            .cmp(&a.default)
            .then_with(|| a.description.cmp(&b.description))
    });
    info.inputs.sort_by(|a, b| {
        b.default
            .cmp(&a.default)
            .then_with(|| a.description.cmp(&b.description))
    });
    info.streams.sort();
    info.streams.dedup();
    info
}

/// Asks PipeWire. Returns an empty inventory when it is not running.
pub async fn read() -> AudioInfo {
    match command::output("pw-dump", &[], Duration::from_secs(5)).await {
        Some(dump) => parse(&dump),
        None => AudioInfo::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_devices_defaults_and_streams() {
        let dump = r#"[
          {"type":"PipeWire:Interface:Core","info":{"version":"1.0.5"}},
          {"type":"PipeWire:Interface:Metadata","props":{"metadata.name":"default"},"metadata":[
            {"key":"default.audio.sink","value":{"name":"spk"}},{"key":"default.audio.source","value":{"name":"mic"}}]},
          {"type":"PipeWire:Interface:Node","info":{"props":{"media.class":"Audio/Sink","node.name":"hdmi","node.description":"HDMI"},
            "params":{"Props":[{"channelVolumes":[1.0,1.0],"mute":false}]}}},
          {"type":"PipeWire:Interface:Node","info":{"props":{"media.class":"Audio/Sink","node.name":"spk","node.description":"Speakers"},
            "params":{"Props":[{"volume":1.0},{"channelVolumes":[0.125,0.125],"mute":true}]}}},
          {"type":"PipeWire:Interface:Node","info":{"props":{"media.class":"Audio/Source","node.name":"mic","node.description":"Mic"}}},
          {"type":"PipeWire:Interface:Node","info":{"props":{"media.class":"Stream/Output/Audio","application.name":"Firefox"}}}
        ]"#;
        let a = parse(dump);
        assert_eq!(a.server.as_deref(), Some("PipeWire 1.0.5"));
        assert_eq!(a.outputs[0].description, "Speakers");
        assert!(a.outputs[0].default && a.outputs[0].muted);
        assert_eq!(a.outputs[0].volume, Some(50.0));
        assert_eq!(a.inputs[0].volume, None);
        assert_eq!(a.streams, ["Firefox"]);
        assert_eq!(parse("not json"), AudioInfo::default());
    }
}
