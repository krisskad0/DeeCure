// SPDX-License-Identifier: Apache-2.0

//! Built-in devices, screens, and sound hardware for the Expert level
//! (docs/01-product/expert-coverage.md, sections 17 and 18): cameras, the fingerprint reader,
//! network chips (driver, firmware, Ethernet speed and energy saving), touchpads and other input,
//! batteries of mice and headsets, printers, each screen from its own EDID (maker, model, year,
//! size, refresh range, HDR), backlights, and sound cards with their codecs. No admin rights.

use std::time::Duration;

use serde::Serialize;

use super::command;
use crate::collect::SystemRoot;

const LIMIT: Duration = Duration::from_secs(5);

/// One camera.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Camera {
    pub node: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
}

/// A network chip.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct NetChip {
    pub interface: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bus: Option<String>,
    /// Ethernet: speed and duplex now, the fastest mode it supports, and whether a cable is in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fastest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<bool>,
    /// Energy-efficient Ethernet, such as `enabled - inactive`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy_saving: Option<String>,
}

/// A battery of a mouse, keyboard, headset, or other device, from UPower.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct DeviceBattery {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent: Option<f64>,
}

/// One printer from CUPS.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Printer {
    pub name: String,
    /// CUPS's words, such as `is idle` or `disabled`.
    pub state: String,
    pub default: bool,
    pub jobs: usize,
}

/// One screen, from its EDID.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Screen {
    pub connector: String,
    /// The three-letter maker code, such as `LGD` (LG Display).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub product_code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,
    /// Physical size in centimetres.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width_cm: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_cm: Option<u8>,
    /// Preferred mode, such as 1920 × 1080.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native: Option<(u32, u32)>,
    /// The refresh range it accepts (variable refresh between these, when it has it).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_min: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_max: Option<u8>,
    /// Colour depth per channel, in bits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bits: Option<u8>,
    /// HDR transfer functions it supports, such as `HDR10 (PQ)` and `HLG`.
    pub hdr: Vec<String>,
}

/// One backlight.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Backlight {
    pub name: String,
    pub level: u64,
    pub max: u64,
    /// `raw`, `platform`, or `firmware`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// One sound card.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct SoundCard {
    pub index: u32,
    pub name: String,
    pub codecs: Vec<String>,
}

/// What `devices_detail` answers.
#[derive(Debug, Default, Serialize)]
pub struct DevicesDetail {
    pub cameras: Vec<Camera>,
    /// Missing when fprintd is not installed; empty device name when there is no reader.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<Fingerprint>,
    pub network: Vec<NetChip>,
    /// Touchpads, touch screens, and pens.
    pub pointing: Vec<String>,
    pub batteries: Vec<DeviceBattery>,
    /// Missing when CUPS is not installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub printers: Option<Vec<Printer>>,
    pub screens: Vec<Screen>,
    pub backlights: Vec<Backlight>,
    pub sound: Vec<SoundCard>,
}

/// The fingerprint reader and the fingers enrolled for the user.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Fingerprint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    pub enrolled: Vec<String>,
}

fn text(root: &SystemRoot, path: &str) -> Option<String> {
    root.read(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn names(root: &SystemRoot, dir: &str) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(root.path(dir))
        .map(|d| {
            d.flatten()
                .filter_map(|e| e.file_name().to_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// Decodes the parts of an EDID this app shows (base block, and a CTA-861 extension for HDR).
pub fn parse_edid(e: &[u8]) -> Option<Screen> {
    if e.len() < 128 || e[0..8] != [0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0] {
        return None;
    }
    let id = u16::from_be_bytes([e[8], e[9]]);
    let letter = |n: u16| char::from(b'A' + ((n & 0x1f) as u8).saturating_sub(1));
    let maker = Some(format!(
        "{}{}{}",
        letter(id >> 10),
        letter(id >> 5),
        letter(id)
    ));
    let product_code = u16::from_le_bytes([e[10], e[11]]);
    let year = (e[17] > 0).then(|| 1990 + e[17] as u16);
    let digital = e[20] & 0x80 != 0;
    let bits = (digital && e[19] >= 4)
        .then(|| match (e[20] >> 4) & 0x7 {
            1 => 6,
            2 => 8,
            3 => 10,
            4 => 12,
            5 => 14,
            6 => 16,
            _ => 0,
        })
        .filter(|b| *b > 0);
    let mut screen = Screen {
        maker,
        product_code,
        year,
        width_cm: (e[21] > 0).then_some(e[21]),
        height_cm: (e[22] > 0).then_some(e[22]),
        bits,
        ..Default::default()
    };
    for d in e[54..126].chunks(18) {
        if d[0] != 0 || d[1] != 0 {
            // A detailed timing: the first one is the preferred mode.
            if screen.native.is_none() {
                let h = d[2] as u32 | ((d[4] as u32 & 0xf0) << 4);
                let v = d[5] as u32 | ((d[7] as u32 & 0xf0) << 4);
                screen.native = Some((h, v));
            }
            continue;
        }
        // Text ends at a newline; some panels put a binary byte between two words.
        let ascii = || {
            let line = d[5..18].split(|b| *b == b'\n').next().unwrap_or(&[]);
            line.split(|b| !(0x20..0x7f).contains(b))
                .map(|w| String::from_utf8_lossy(w).trim().to_owned())
                .filter(|w| !w.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        };
        match d[3] {
            0xfc => screen.model = Some(ascii()).filter(|m| !m.is_empty()),
            0xfd => {
                screen.refresh_min = Some(d[5]);
                screen.refresh_max = Some(d[6]);
            }
            0xfe if screen.model.is_none() => {
                screen.model = Some(ascii()).filter(|m| !m.is_empty())
            }
            _ => {}
        }
    }
    // CTA-861 extensions: an HDR static metadata block (extended tag 6) lists transfer functions.
    for ext in e[128..]
        .chunks(128)
        .filter(|x| x.len() == 128 && x[0] == 0x02)
    {
        let end = (ext[2] as usize).min(127);
        let mut at = 4;
        while at < end {
            let tag = ext[at] >> 5;
            let len = (ext[at] & 0x1f) as usize;
            if tag == 7 && len >= 2 && ext.get(at + 1) == Some(&0x06) {
                let eotf = ext.get(at + 2).copied().unwrap_or(0);
                for (bit, name) in [
                    (2, "HDR10 (PQ)"),
                    (3, "HLG"),
                    (1, "HDR (traditional gamma)"),
                ] {
                    if eotf & (1 << bit) != 0 {
                        screen.hdr.push(name.to_owned());
                    }
                }
            }
            at += len + 1;
        }
    }
    Some(screen)
}

/// `ethtool -i` and `ethtool` lines as key → value.
fn ethtool_fields(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
        .collect()
}

async fn network(root: &SystemRoot) -> Vec<NetChip> {
    let mut out = Vec::new();
    for name in names(root, "/sys/class/net") {
        // Physical chips only: they have a device.
        if !root.path(&format!("/sys/class/net/{name}/device")).exists() {
            continue;
        }
        let info = command::output("ethtool", &["-i", &name], LIMIT)
            .await
            .map(|t| ethtool_fields(&t))
            .unwrap_or_default();
        let get = |list: &[(String, String)], k: &str| {
            list.iter()
                .find(|(key, _)| key == k)
                .map(|(_, v)| v.clone())
                .filter(|v| !v.is_empty() && v != "N/A")
        };
        let wireless = root
            .path(&format!("/sys/class/net/{name}/wireless"))
            .exists();
        let mut chip = NetChip {
            interface: name.clone(),
            driver: get(&info, "driver"),
            firmware: get(&info, "firmware-version"),
            bus: get(&info, "bus-info"),
            ..Default::default()
        };
        if !wireless {
            let link = command::output("ethtool", &[&name], LIMIT)
                .await
                .unwrap_or_default();
            let fields = ethtool_fields(&link);
            chip.speed = get(&fields, "Speed").filter(|s| !s.starts_with("Unknown"));
            chip.duplex = get(&fields, "Duplex").filter(|s| !s.starts_with("Unknown"));
            chip.link = get(&fields, "Link detected").map(|v| v == "yes");
            // The supported modes run over several lines: take the fastest number.
            chip.fastest = link
                .split("Supported link modes:")
                .nth(1)
                .and_then(|rest| rest.split("Supported pause").next())
                .and_then(|modes| {
                    modes
                        .split_whitespace()
                        .filter_map(|m| m.split("base").next()?.parse::<u64>().ok())
                        .max()
                })
                .map(|mbps| {
                    if mbps >= 1000 {
                        format!("{} Gb/s", mbps / 1000)
                    } else {
                        format!("{mbps} Mb/s")
                    }
                });
            chip.energy_saving = command::output("ethtool", &["--show-eee", &name], LIMIT)
                .await
                .and_then(|t| {
                    t.lines().find_map(|l| {
                        l.trim()
                            .strip_prefix("EEE status:")
                            .map(|v| v.trim().to_owned())
                    })
                });
        }
        out.push(chip);
    }
    out
}

/// `upower -d` blocks for devices other than the system battery and power supplies.
pub fn parse_upower(text: &str) -> Vec<DeviceBattery> {
    text.split("Device: ")
        .skip(1)
        .filter_map(|block| {
            let kinds = [
                "mouse",
                "keyboard",
                "headset",
                "headphones",
                "gaming-input",
                "pen",
                "tablet",
                "phone",
                "touchpad",
                "speakers",
                "other",
            ];
            let kind = block
                .lines()
                .map(str::trim)
                .find(|l| kinds.contains(l))?
                .to_owned();
            let get = |k: &str| {
                block
                    .lines()
                    .find_map(|l| l.trim().strip_prefix(k).map(|v| v.trim().to_owned()))
            };
            Some(DeviceBattery {
                kind,
                model: get("model:"),
                percent: get("percentage:").and_then(|p| p.trim_end_matches('%').parse().ok()),
            })
        })
        .collect()
}

/// `lpstat -p -d` and `lpstat -o` into printers with their job count.
pub fn parse_printers(status: &str, jobs: &str) -> Vec<Printer> {
    let default = status
        .lines()
        .find_map(|l| l.strip_prefix("system default destination:"))
        .map(|d| d.trim().to_owned());
    status
        .lines()
        .filter_map(|l| {
            let rest = l.strip_prefix("printer ")?;
            let (name, state) = rest.split_once(' ')?;
            let state = state.split('.').next().unwrap_or(state).trim().to_owned();
            Some(Printer {
                jobs: jobs
                    .lines()
                    .filter(|j| j.starts_with(&format!("{name}-")))
                    .count(),
                default: default.as_deref() == Some(name),
                name: name.to_owned(),
                state,
            })
        })
        .collect()
}

/// Reads `devices_detail`.
pub async fn read(root: &SystemRoot) -> DevicesDetail {
    let cameras = names(root, "/sys/class/video4linux")
        .into_iter()
        // Capture nodes have index 0; the others carry metadata.
        .filter(|n| {
            text(root, &format!("/sys/class/video4linux/{n}/index")).is_none_or(|i| i == "0")
        })
        .filter_map(|n| {
            Some(Camera {
                // The kernel cuts names at 32 characters, often as "Name: Na".
                name: text(root, &format!("/sys/class/video4linux/{n}/name"))?
                    .split(": ")
                    .next()?
                    .to_owned(),
                driver: super::hardware::driver_of(
                    &root.path(&format!("/sys/class/video4linux/{n}/device")),
                ),
                node: n,
            })
        })
        .collect();
    let user = std::env::var("USER").unwrap_or_default();
    let fingerprint = command::output_any("fprintd-list", &[&user], LIMIT)
        .await
        .map(|t| Fingerprint {
            // "Fingerprints for user x on Goodix MOC Fingerprint Sensor (press):"
            device: t.lines().find_map(|l| {
                l.split(" on ").nth(1).map(|d| {
                    d.trim_end_matches(':')
                        .split(" (")
                        .next()
                        .unwrap_or(d)
                        .trim()
                        .to_owned()
                })
            }),
            enrolled: t
                .lines()
                .filter_map(|l| {
                    l.trim()
                        .strip_prefix("- #")
                        .and_then(|r| r.split_once(": "))
                        .map(|(_, f)| f.to_owned())
                })
                .collect(),
        });
    let inputs = root.read("/proc/bus/input/devices").unwrap_or_default();
    let mut pointing: Vec<String> = inputs
        .lines()
        .filter_map(|l| {
            l.strip_prefix("N: Name=\"")?
                .strip_suffix('"')
                .map(str::to_owned)
        })
        .filter(|n| {
            [
                "touchpad",
                "touchscreen",
                "touch screen",
                "pen",
                "stylus",
                "trackpoint",
                "wacom",
            ]
            .iter()
            .any(|k| n.to_lowercase().contains(k))
        })
        .collect();
    pointing.dedup();
    let printers = match command::output_any("lpstat", &["-p", "-d"], LIMIT).await {
        Some(status) => Some(parse_printers(
            &status,
            &command::output_any("lpstat", &["-o"], LIMIT)
                .await
                .unwrap_or_default(),
        )),
        None => None,
    };
    let screens = names(root, "/sys/class/drm")
        .into_iter()
        .filter(|n| {
            n.contains('-')
                && text(root, &format!("/sys/class/drm/{n}/status")).as_deref() == Some("connected")
        })
        .filter_map(|n| {
            let edid = std::fs::read(root.path(&format!("/sys/class/drm/{n}/edid"))).ok()?;
            let mut s = parse_edid(&edid)?;
            s.connector = n.split_once('-').map(|(_, c)| c.to_owned()).unwrap_or(n);
            Some(s)
        })
        .collect();
    let backlights = names(root, "/sys/class/backlight")
        .into_iter()
        .filter_map(|n| {
            let at = |f: &str| format!("/sys/class/backlight/{n}/{f}");
            Some(Backlight {
                // The level set; some drivers report `actual_brightness` as 0.
                level: text(root, &at("brightness"))?.parse().ok()?,
                max: text(root, &at("max_brightness"))?.parse().ok()?,
                kind: text(root, &at("type")),
                name: n,
            })
        })
        .collect();
    let cards = root.read("/proc/asound/cards").unwrap_or_default();
    let sound = cards
        .lines()
        .filter_map(|l| {
            let (index, rest) = l.trim().split_once(' ')?;
            let index: u32 = index.parse().ok()?;
            let name = rest.split_once("]:").map(|(_, n)| n.trim().to_owned())?;
            let codecs = names(root, &format!("/proc/asound/card{index}"))
                .into_iter()
                .filter(|f| f.starts_with("codec#"))
                .filter_map(|f| {
                    root.read(&format!("/proc/asound/card{index}/{f}"))
                        .ok()?
                        .lines()
                        .find_map(|l| l.strip_prefix("Codec: ").map(str::to_owned))
                })
                .collect();
            Some(SoundCard {
                index,
                name,
                codecs,
            })
        })
        .collect();
    DevicesDetail {
        cameras,
        fingerprint,
        network: network(root).await,
        pointing,
        batteries: command::output("upower", &["-d"], LIMIT)
            .await
            .map(|t| parse_upower(&t))
            .unwrap_or_default(),
        printers,
        screens,
        backlights,
        sound,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edid() -> Vec<u8> {
        let mut e = vec![0u8; 256];
        e[0..8].copy_from_slice(&[0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0]);
        e[8..10].copy_from_slice(&[0x30, 0xe4]); // LGD
        e[10..12].copy_from_slice(&0x0753u16.to_le_bytes());
        e[17] = 32; // 2022
        e[18] = 1;
        e[19] = 4;
        e[20] = 0x80 | (2 << 4); // digital, 8 bits
        e[21] = 31;
        e[22] = 17;
        // Detailed timing 1920 × 1080.
        e[54..72].copy_from_slice(&[
            0x3a, 0x02, 0x80, 0x18, 0x71, 0x38, 0x2d, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        // Name and range limits.
        let mut name = vec![0, 0, 0, 0xfc, 0];
        name.extend(b"LG\x01Panel\n    ");
        e[72..90].copy_from_slice(&name);
        e[90..108].copy_from_slice(&[0, 0, 0, 0xfd, 0, 48, 120, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        // CTA extension with an HDR static metadata block (PQ and HLG).
        e[126] = 1;
        e[128] = 0x02;
        e[129] = 3;
        e[130] = 8;
        e[132..136].copy_from_slice(&[(7 << 5) | 3, 0x06, 0b1100, 0]);
        e
    }

    #[test]
    fn edid_gives_the_screen_facts() {
        let s = parse_edid(&edid()).unwrap();
        assert_eq!(s.maker.as_deref(), Some("LGD"));
        assert_eq!(
            (s.product_code, s.year, s.width_cm, s.height_cm),
            (0x0753, Some(2022), Some(31), Some(17))
        );
        assert_eq!(s.native, Some((1920, 1080)));
        assert_eq!(
            s.model.as_deref(),
            Some("LG Panel"),
            "a binary byte between words is a space"
        );
        assert_eq!(
            (s.refresh_min, s.refresh_max, s.bits),
            (Some(48), Some(120), Some(8))
        );
        assert_eq!(s.hdr, ["HDR10 (PQ)", "HLG"]);
        assert!(parse_edid(&[0u8; 10]).is_none());
    }

    #[test]
    fn upower_and_printers() {
        let u = parse_upower(
            "Device: /org/freedesktop/UPower/devices/battery_BAT0\n  native-path: BAT0\n  battery\n    percentage: 84%\nDevice: /org/freedesktop/UPower/devices/mouse_hidpp\n  model: MX Master 3\n  mouse\n    percentage: 55%\n",
        );
        assert_eq!(
            u,
            [DeviceBattery {
                kind: "mouse".into(),
                model: Some("MX Master 3".into()),
                percent: Some(55.0)
            }]
        );
        let p = parse_printers(
            "printer Office is idle.  enabled since Mon\nprinter Home disabled since Tue -\nsystem default destination: Office\n",
            "Office-12 user 1024 Mon\nOffice-13 user 2048 Mon\n",
        );
        assert_eq!(p.len(), 2);
        assert_eq!(
            (
                p[0].name.as_str(),
                p[0].state.as_str(),
                p[0].default,
                p[0].jobs
            ),
            ("Office", "is idle", true, 2)
        );
        assert_eq!(p[1].state, "disabled since Tue -");
    }
}
