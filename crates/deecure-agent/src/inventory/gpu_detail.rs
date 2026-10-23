// SPDX-License-Identifier: Apache-2.0

//! Graphics in depth for the Expert level (docs/01-product/expert-coverage.md, section 5).
//!
//! `gpu_detail` (read every few seconds while open): each card's clock now and its limits, time
//! in power saving, why it is slowed down, power drawn, whether it sleeps, its screens, and the
//! graphics use of each of the user's apps (from DRM `fdinfo`; Intel, AMD, and other DRM
//! drivers that report it).
//!
//! `graphics_stack` (read every few minutes): the desktop session type, Mesa and NVIDIA driver
//! versions, the OpenGL renderer, the video-decoding drivers installed, and the kernel's
//! graphics firmware messages. No admin rights for any of it.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::Serialize;

use super::command;
use crate::collect::SystemRoot;

const LIMIT: Duration = Duration::from_secs(3);

/// One graphics card.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct CardDetail {
    /// The DRM card, such as `card1`.
    pub card: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pci_address: Option<String>,
    /// The card the computer started with; on laptops with two cards, the other one is the
    /// second card that should sleep when unused.
    pub boot_vga: bool,
    /// Clock now, and the lowest, highest allowed, highest possible, and boost clocks, in MHz.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mhz_now: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mhz_min: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mhz_max: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mhz_hardware_max: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mhz_boost: Option<u64>,
    /// Time in its deepest power saving since start, in ms (Intel RC6).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_saving_ms: Option<u64>,
    /// Why it is slowed down right now, such as `pl1` (power limit) or `thermal` (Intel).
    pub slowed_by: Vec<String>,
    /// Power drawn now, in watts (AMD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watts: Option<f64>,
    /// `active` or `suspended`: a sleeping card uses almost no power.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_status: Option<String>,
    /// Each screen connector and whether something is plugged in.
    pub connectors: Vec<(String, String)>,
}

/// The graphics use of one app's connection to a card.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct GpuClient {
    pub pid: u32,
    pub name: String,
    /// The card's PCI address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pci_address: Option<String>,
    /// Busy time per engine (`render`, `video`, `copy`, …) since it connected, in ns.
    pub engines: BTreeMap<String, u64>,
    /// Graphics memory it holds, in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<u64>,
}

/// What `gpu_detail` answers.
#[derive(Debug, Default, Serialize)]
pub struct GpuDetail {
    /// When this was read, in ms since 1970.
    pub at: u64,
    pub cards: Vec<CardDetail>,
    pub clients: Vec<GpuClient>,
}

/// What `graphics_stack` answers.
#[derive(Debug, Default, Serialize)]
pub struct GraphicsStack {
    /// `wayland`, `x11`, or `tty`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesa_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvidia_version: Option<String>,
    /// From `glxinfo -B`, when it is installed and a screen is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renderer: Option<String>,
    /// The renderer is software (`llvmpipe`, `softpipe`): slow, and no graphics card is used.
    pub software_rendering: bool,
    /// Video-decoding (VA-API) drivers installed, such as `iHD` or `radeonsi`.
    pub video_drivers: Vec<String>,
    /// Formats the card decodes in hardware, from `vainfo` when it is installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_decode: Option<Vec<String>>,
    /// The kernel's graphics firmware lines from this boot (loaded, or failed to load).
    pub firmware: Vec<String>,
}

fn text(root: &SystemRoot, path: &str) -> Option<String> {
    root.read(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn number(root: &SystemRoot, path: &str) -> Option<u64> {
    text(root, path)?.parse().ok()
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// The line marked `*` in an AMD `pp_dpm_sclk` list, such as `1: 1200Mhz *`.
pub fn amd_current_mhz(text: &str) -> Option<u64> {
    let line = text.lines().find(|l| l.trim_end().ends_with('*'))?;
    let value = line.split_whitespace().nth(1)?;
    value
        .trim_end_matches(|c: char| c.is_ascii_alphabetic())
        .parse()
        .ok()
}

fn card(root: &SystemRoot, name: &str, all: &[String]) -> CardDetail {
    let base = format!("/sys/class/drm/{name}");
    let device = root.path(&format!("{base}/device"));
    let pci_address = std::fs::canonicalize(&device)
        .ok()
        .and_then(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_owned));
    let driver = super::hardware::driver_of(&device);
    let at = |f: &str| format!("{base}/{f}");
    let mut c = CardDetail {
        card: name.to_owned(),
        // Intel i915.
        mhz_now: number(root, &at("gt_act_freq_mhz"))
            .or_else(|| number(root, &at("gt_cur_freq_mhz"))),
        mhz_min: number(root, &at("gt_min_freq_mhz")),
        mhz_max: number(root, &at("gt_max_freq_mhz")),
        mhz_hardware_max: number(root, &at("gt_RP0_freq_mhz")),
        mhz_boost: number(root, &at("gt_boost_freq_mhz")),
        power_saving_ms: number(root, &at("gt/gt0/rc6_residency_ms")),
        runtime_status: text(root, &at("device/power/runtime_status")),
        boot_vga: text(root, &at("device/boot_vga")).as_deref() == Some("1"),
        ..Default::default()
    };
    // Intel xe.
    if c.mhz_now.is_none() {
        let f = |n: &str| number(root, &at(&format!("device/tile0/gt0/freq0/{n}")));
        c.mhz_now = f("act_freq").or_else(|| f("cur_freq"));
        c.mhz_min = f("min_freq");
        c.mhz_max = f("max_freq");
        c.mhz_hardware_max = f("rp0_freq");
    }
    // AMD.
    if c.mhz_now.is_none() {
        c.mhz_now = text(root, &at("device/pp_dpm_sclk")).and_then(|t| amd_current_mhz(&t));
    }
    let gt = format!("{base}/gt/gt0");
    let mut reasons: Vec<String> = std::fs::read_dir(root.path(&gt))
        .map(|d| {
            d.flatten()
                .filter_map(|e| {
                    e.file_name()
                        .to_str()?
                        .strip_prefix("throttle_reason_")
                        .map(str::to_owned)
                })
                .collect()
        })
        .unwrap_or_default();
    reasons.retain(|r| {
        r != "status" && text(root, &format!("{gt}/throttle_reason_{r}")).as_deref() == Some("1")
    });
    reasons.sort();
    c.slowed_by = reasons;
    c.watts = std::fs::read_dir(root.path(&at("device/hwmon")))
        .ok()
        .and_then(|d| {
            d.flatten()
                .find_map(|e| std::fs::read_to_string(e.path().join("power1_average")).ok())
        })
        .and_then(|v| v.trim().parse::<f64>().ok())
        .map(|uw| uw / 1e6);
    c.connectors = all
        .iter()
        .filter_map(|n| {
            let connector = n.strip_prefix(&format!("{name}-"))?;
            Some((
                connector.to_owned(),
                text(root, &format!("/sys/class/drm/{n}/status"))
                    .unwrap_or_else(|| "unknown".into()),
            ))
        })
        .collect();
    c.driver = driver;
    c.pci_address = pci_address;
    c
}

/// The `drm-*` keys of one `fdinfo` file that describe a graphics client.
pub fn parse_fdinfo(
    text: &str,
) -> Option<(String, Option<String>, BTreeMap<String, u64>, Option<u64>)> {
    let mut client = None;
    let mut pdev = None;
    let mut engines = BTreeMap::new();
    let mut memory: Option<u64> = None;
    for line in text.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        if key == "drm-client-id" {
            client = Some(value.to_owned());
        } else if key == "drm-pdev" {
            pdev = Some(value.to_owned());
        } else if let Some(engine) = key
            .strip_prefix("drm-engine-")
            .filter(|e| !e.starts_with("capacity-"))
        {
            if let Some(ns) = value
                .strip_suffix(" ns")
                .and_then(|v| v.trim().parse::<u64>().ok())
            {
                engines.insert(engine.to_owned(), ns);
            }
        } else if key.starts_with("drm-resident-") {
            let mut parts = value.split_whitespace();
            let n: u64 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
            let factor = match parts.next() {
                Some("KiB") => 1024,
                Some("MiB") => 1024 * 1024,
                Some("GiB") => 1024 * 1024 * 1024,
                _ => 1,
            };
            memory = Some(memory.unwrap_or(0) + n * factor);
        }
    }
    Some((client?, pdev, engines, memory))
}

/// Every graphics client of the user's processes, each client once.
fn clients(root: &SystemRoot) -> Vec<GpuClient> {
    let pids: Vec<u32> = std::fs::read_dir(root.path("/proc"))
        .map(|d| {
            d.flatten()
                .filter_map(|e| e.file_name().to_str()?.parse().ok())
                .collect()
        })
        .unwrap_or_default();
    let mut seen = std::collections::BTreeSet::new();
    let mut out: Vec<GpuClient> = Vec::new();
    for pid in pids {
        let Ok(fds) = std::fs::read_dir(root.path(&format!("/proc/{pid}/fd"))) else {
            continue;
        };
        for fd in fds.flatten() {
            let is_drm = std::fs::read_link(fd.path()).is_ok_and(|t| t.starts_with("/dev/dri/"));
            if !is_drm {
                continue;
            }
            let Some(n) = fd.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let Some((client, pdev, engines, memory)) = root
                .read(&format!("/proc/{pid}/fdinfo/{n}"))
                .ok()
                .and_then(|t| parse_fdinfo(&t))
            else {
                continue;
            };
            // A client shared by several file handles or processes is counted once.
            if !seen.insert((pdev.clone(), client)) {
                continue;
            }
            let name = root
                .read(&format!("/proc/{pid}/comm"))
                .map(|c| c.trim().to_owned())
                .unwrap_or_else(|_| pid.to_string());
            match out
                .iter_mut()
                .find(|c| c.pid == pid && c.pci_address == pdev)
            {
                Some(c) => {
                    for (e, ns) in engines {
                        *c.engines.entry(e).or_insert(0) += ns;
                    }
                    c.memory = match (c.memory, memory) {
                        (Some(a), Some(b)) => Some(a + b),
                        (a, b) => a.or(b),
                    };
                }
                None => out.push(GpuClient {
                    pid,
                    name,
                    pci_address: pdev,
                    engines,
                    memory,
                }),
            }
        }
    }
    out
}

/// Reads `gpu_detail`.
pub fn read(root: &SystemRoot) -> GpuDetail {
    let mut names: Vec<String> = std::fs::read_dir(root.path("/sys/class/drm"))
        .map(|d| {
            d.flatten()
                .filter_map(|e| e.file_name().to_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    let cards: Vec<CardDetail> = names
        .iter()
        .filter(|n| {
            n.starts_with("card") && n.len() > 4 && n[4..].chars().all(|c| c.is_ascii_digit())
        })
        .map(|n| card(root, n, &names))
        .collect();
    GpuDetail {
        at: now_ms(),
        cards,
        clients: clients(root),
    }
}

/// Decode profiles from `vainfo`: lines such as `VAProfileH264Main : VAEntrypointVLD`.
pub fn vainfo_decode(text: &str) -> Vec<String> {
    let mut out: Vec<String> = text
        .lines()
        .filter(|l| l.contains("VAEntrypointVLD"))
        .filter_map(|l| {
            l.split(':')
                .next()
                .map(|p| p.trim().trim_start_matches("VAProfile").to_owned())
        })
        .filter(|p| !p.is_empty() && p != "None")
        .collect();
    out.dedup();
    out
}

/// The desktop session type of the user's graphical session, from logind.
async fn session_type() -> Option<String> {
    if let Ok(t) = std::env::var("XDG_SESSION_TYPE")
        && !t.is_empty()
    {
        return Some(t);
    }
    let uid = std::fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find_map(|l| {
            l.strip_prefix("Uid:")?
                .split_whitespace()
                .next()
                .map(str::to_owned)
        })?;
    let display = command::output(
        "loginctl",
        &["show-user", &uid, "-p", "Display", "--value"],
        LIMIT,
    )
    .await?;
    let display = display.trim();
    if display.is_empty() {
        return None;
    }
    let kind = command::output(
        "loginctl",
        &["show-session", display, "-p", "Type", "--value"],
        LIMIT,
    )
    .await?;
    Some(kind.trim().to_owned()).filter(|k| !k.is_empty())
}

/// Reads `graphics_stack`.
pub async fn read_stack(root: &SystemRoot) -> GraphicsStack {
    let mut video_drivers: Vec<String> =
        std::fs::read_dir(root.path("/usr/lib/x86_64-linux-gnu/dri"))
            .map(|d| {
                d.flatten()
                    .filter_map(|e| {
                        e.file_name()
                            .to_str()?
                            .strip_suffix("_drv_video.so")
                            .map(str::to_owned)
                    })
                    .collect()
            })
            .unwrap_or_default();
    video_drivers.sort();
    let renderer = command::output("glxinfo", &["-B"], LIMIT)
        .await
        .and_then(|t| {
            t.lines().find_map(|l| {
                l.trim()
                    .strip_prefix("OpenGL renderer string:")
                    .map(|r| r.trim().to_owned())
            })
        });
    let firmware = command::output("journalctl", &["-k", "-b", "-o", "cat", "--no-pager", "--grep", "GuC|HuC|DMC|GSC|amdgpu.*firmware|nouveau.*firmware|Direct firmware load.*(i915|xe|amdgpu|nouveau|radeon)"], LIMIT)
        .await
        .map(|t| {
            // The same lines repeat after each resume; keep the first of each.
            let mut seen = std::collections::BTreeSet::new();
            t.lines()
                .map(|l| l.trim().to_owned())
                .filter(|l| !l.is_empty() && !l.starts_with("-- ") && seen.insert(l.clone()))
                .take(20)
                .collect()
        })
        .unwrap_or_default();
    GraphicsStack {
        session_type: session_type().await,
        mesa_version: command::output(
            "dpkg-query",
            &["-W", "-f=${Version}", "libgl1-mesa-dri"],
            LIMIT,
        )
        .await
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty()),
        nvidia_version: text(root, "/sys/module/nvidia/version"),
        software_rendering: renderer
            .as_deref()
            .is_some_and(|r| r.contains("llvmpipe") || r.contains("softpipe")),
        renderer,
        video_drivers,
        video_decode: command::output("vainfo", &[], LIMIT)
            .await
            .map(|t| vainfo_decode(&t)),
        firmware,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn fdinfo_gives_engines_and_memory() {
        let (client, pdev, engines, memory) = parse_fdinfo(
            "pos:\t0\ndrm-driver:\ti915\ndrm-client-id:\t17\ndrm-pdev:\t0000:00:02.0\ndrm-resident-system0:\t194328 KiB\ndrm-resident-stolen-system0:\t0\ndrm-engine-render:\t293668527516 ns\ndrm-engine-video:\t0 ns\ndrm-engine-capacity-video:\t2\n",
        )
        .unwrap();
        assert_eq!(
            (client.as_str(), pdev.as_deref()),
            ("17", Some("0000:00:02.0"))
        );
        assert_eq!(engines.get("render"), Some(&293_668_527_516));
        assert!(!engines.contains_key("capacity-video"));
        assert_eq!(memory, Some(194_328 * 1024));
        assert!(
            parse_fdinfo("pos: 0\nflags: 02\n").is_none(),
            "not a graphics client"
        );
    }

    #[test]
    fn amd_and_vainfo_formats() {
        assert_eq!(
            amd_current_mhz("0: 500Mhz\n1: 1200Mhz *\n2: 2000Mhz\n"),
            Some(1200)
        );
        assert_eq!(amd_current_mhz("0: 500Mhz\n"), None);
        assert_eq!(
            vainfo_decode(
                "      VAProfileNone                   :\tVAEntrypointVideoProc\n      VAProfileH264Main               :\tVAEntrypointVLD\n      VAProfileH264Main               :\tVAEntrypointEncSlice\n      VAProfileHEVCMain               :\tVAEntrypointVLD\n"
            ),
            ["H264Main", "HEVCMain"]
        );
    }

    #[test]
    fn reads_an_intel_card_with_its_screens() {
        let c = "/sys/class/drm/card1";
        let root = fixture_root(
            "gpu-detail",
            &[
                (&format!("{c}/gt_act_freq_mhz"), "300\n"),
                (&format!("{c}/gt_min_freq_mhz"), "100\n"),
                (&format!("{c}/gt_max_freq_mhz"), "1250\n"),
                (&format!("{c}/gt_RP0_freq_mhz"), "1250\n"),
                (&format!("{c}/gt/gt0/rc6_residency_ms"), "102485033\n"),
                (&format!("{c}/gt/gt0/throttle_reason_status"), "1\n"),
                (&format!("{c}/gt/gt0/throttle_reason_pl1"), "1\n"),
                (&format!("{c}/gt/gt0/throttle_reason_thermal"), "0\n"),
                (&format!("{c}/device/power/runtime_status"), "active\n"),
                ("/sys/class/drm/card1-eDP-1/status", "connected\n"),
                ("/sys/class/drm/card1-HDMI-A-1/status", "disconnected\n"),
                ("/sys/class/drm/renderD128/dev", "226:128\n"),
            ],
        );
        let d = read(&root);
        assert_eq!(
            d.cards.len(),
            1,
            "connectors and render nodes are not cards"
        );
        let card = &d.cards[0];
        assert_eq!(
            (card.mhz_now, card.mhz_max, card.mhz_hardware_max),
            (Some(300), Some(1250), Some(1250))
        );
        assert_eq!(card.power_saving_ms, Some(102_485_033));
        assert_eq!(card.slowed_by, ["pl1"]);
        assert_eq!(
            card.connectors,
            [
                ("HDMI-A-1".to_owned(), "disconnected".to_owned()),
                ("eDP-1".to_owned(), "connected".to_owned())
            ]
        );
        assert_eq!(card.watts, None);
    }
}
