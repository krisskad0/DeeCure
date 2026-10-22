// SPDX-License-Identifier: Apache-2.0

//! Installed software from every source (APT, snap, Flatpak, AppImage), software
//! sources, and what could be removed. `apt-get -s` only simulates: it changes nothing.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use serde::Serialize;

use super::command;
use crate::collect::SystemRoot;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    /// `apt`, `snap`, `flatpak`, or `appimage`.
    pub source: String,
    /// Bytes on disk.
    pub size: u64,
    /// True when a person asked for it (not pulled in as a dependency).
    pub manual: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub summary: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub section: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Source {
    /// Such as `http://archive.ubuntu.com/ubuntu noble main restricted`.
    pub line: String,
    /// `Official`, `PPA`, or `Third-party`.
    pub kind: String,
    /// False only when the source is marked `trusted=yes`: apt then skips signature checks.
    pub signed: bool,
    /// True when the source names its own key (`signed-by`), so that key cannot sign
    /// packages for other sources.
    pub own_key: bool,
    pub enabled: bool,
    /// The file it is written in.
    pub file: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Removable {
    pub action: String,
    pub title: String,
    pub count: usize,
    pub size: u64,
    pub names: Vec<String>,
    /// `reinstallable` or `irreversible`.
    pub undo: String,
}

/// The `packages` inventory.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PackagesInfo {
    pub packages: Vec<Package>,
    pub sources: Vec<Source>,
    pub removable: Vec<Removable>,
}

/// Reads `/var/lib/dpkg/status`; `auto` holds packages APT installed as dependencies.
/// Returns installed packages, and the names of removed packages that left config files.
pub fn parse_dpkg(status: &str, auto: &HashSet<String>) -> (Vec<Package>, Vec<String>) {
    let mut installed = Vec::new();
    let mut residual = Vec::new();
    for block in status.split("\n\n") {
        let field = |k: &str| {
            block
                .lines()
                .find_map(|l| l.strip_prefix(&format!("{k}: ")))
                .unwrap_or("")
        };
        let name = field("Package");
        if name.is_empty() {
            continue;
        }
        match field("Status") {
            "install ok installed" => installed.push(Package {
                name: name.to_owned(),
                version: field("Version").to_owned(),
                source: "apt".into(),
                size: field("Installed-Size").parse::<u64>().unwrap_or(0) * 1024,
                manual: !auto.contains(name),
                summary: field("Description").chars().take(90).collect(),
                section: field("Section").to_owned(),
            }),
            "deinstall ok config-files" => residual.push(name.to_owned()),
            _ => {}
        }
    }
    (installed, residual)
}

/// Package names APT marked as automatically installed, from `extended_states`.
pub fn auto_installed(text: &str) -> HashSet<String> {
    text.split("\n\n")
        .filter(|b| b.lines().any(|l| l.trim() == "Auto-Installed: 1"))
        .filter_map(|b| {
            b.lines()
                .find_map(|l| l.strip_prefix("Package: "))
                .map(str::to_owned)
        })
        .collect()
}

fn kind_of(uri: &str) -> &'static str {
    if uri.contains("ppa.launchpad") {
        "PPA"
    } else if uri.contains("ubuntu.com")
        || uri.contains("canonical.com")
        || uri.contains("ubuntu-esm")
    {
        "Official"
    } else {
        "Third-party"
    }
}

/// Reads one-line `.list` files and deb822 `.sources` files.
pub fn parse_sources(file: &str, text: &str) -> Vec<Source> {
    let mut out = Vec::new();
    if file.ends_with(".sources") {
        for block in text.split("\n\n") {
            let field = |k: &str| {
                block
                    .lines()
                    .find_map(|l| l.strip_prefix(&format!("{k}:")))
                    .map(str::trim)
                    .unwrap_or("")
            };
            if field("Types").is_empty() || field("URIs").is_empty() {
                continue;
            }
            let uri = field("URIs");
            out.push(Source {
                line: format!("{uri} {} {}", field("Suites"), field("Components"))
                    .trim()
                    .to_owned(),
                kind: kind_of(uri).into(),
                signed: field("Trusted") != "yes",
                own_key: !field("Signed-By").is_empty(),
                enabled: field("Enabled") != "no",
                file: file.to_owned(),
            });
        }
    } else {
        for raw in text.lines() {
            let (enabled, line) = match raw.trim().strip_prefix('#') {
                Some(rest) if rest.trim_start().starts_with("deb ") => (false, rest.trim()),
                Some(_) => continue,
                None => (true, raw.trim()),
            };
            if !line.starts_with("deb ") {
                continue;
            }
            let own_key = line.contains("signed-by=");
            let signed = !line.contains("trusted=yes");
            // Drop the "[arch=amd64 signed-by=…]" options for display.
            let shown: String = line
                .trim_start_matches("deb ")
                .split_whitespace()
                .filter(|w| !w.starts_with('[') && !w.ends_with(']') && !w.contains('='))
                .collect::<Vec<_>>()
                .join(" ");
            let uri = shown.split_whitespace().next().unwrap_or("");
            out.push(Source {
                kind: kind_of(uri).into(),
                line: shown.clone(),
                signed,
                own_key,
                enabled,
                file: file.to_owned(),
            });
        }
    }
    out
}

/// "Remv linux-headers-6.8.0-45 [6.8.0-45.45]" lines of `apt-get -s autoremove`.
pub fn autoremove(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| l.strip_prefix("Remv "))
        .filter_map(|l| l.split_whitespace().next())
        .map(str::to_owned)
        .collect()
}

fn file_size(path: &std::path::Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

pub async fn read(root: &SystemRoot) -> PackagesInfo {
    let auto = auto_installed(
        &root
            .read("/var/lib/apt/extended_states")
            .unwrap_or_default(),
    );
    let (mut packages, residual) = parse_dpkg(
        &root.read("/var/lib/dpkg/status").unwrap_or_default(),
        &auto,
    );
    let apt_sizes: HashMap<String, u64> =
        packages.iter().map(|p| (p.name.clone(), p.size)).collect();
    let mut snap_disabled = 0u64;
    let mut snap_disabled_names = Vec::new();
    if let Some(list) = command::output("snap", &["list", "--all"], Duration::from_secs(10)).await {
        for l in list.lines().skip(1) {
            let f: Vec<&str> = l.split_whitespace().collect();
            let (Some(name), Some(version), Some(rev)) = (f.first(), f.get(1), f.get(2)) else {
                continue;
            };
            let size = file_size(&root.path(&format!("/var/lib/snapd/snaps/{name}_{rev}.snap")));
            if f.last()
                .is_some_and(|n| n.split(',').any(|x| x == "disabled"))
            {
                snap_disabled += size;
                snap_disabled_names.push(format!("{name} (rev {rev})"));
                continue;
            }
            packages.push(Package {
                name: (*name).to_owned(),
                version: (*version).to_owned(),
                source: "snap".into(),
                size,
                manual: !f
                    .last()
                    .is_some_and(|n| n.contains("base") || n.contains("snapd")),
                summary: String::new(),
                section: String::new(),
            });
        }
    }
    if root.path("/usr/bin/flatpak").exists()
        && let Some(list) = command::output(
            "flatpak",
            &["list", "--app", "--columns=application,version,size"],
            Duration::from_secs(10),
        )
        .await
    {
        for l in list.lines() {
            let f: Vec<&str> = l.split('\t').collect();
            let Some(app) = f.first().filter(|a| !a.is_empty()) else {
                continue;
            };
            packages.push(Package {
                name: (*app).to_owned(),
                version: f.get(1).copied().unwrap_or("").to_owned(),
                source: "flatpak".into(),
                size: f.get(2).map(|s| human_size(s)).unwrap_or(0),
                manual: true,
                summary: String::new(),
                section: String::new(),
            });
        }
    }
    if let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) {
        for dir in ["Applications", "Downloads", ".local/bin", "bin"] {
            for e in std::fs::read_dir(home.join(dir))
                .map(|d| d.flatten().collect::<Vec<_>>())
                .unwrap_or_default()
            {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.to_lowercase().ends_with(".appimage") {
                    packages.push(Package {
                        version: String::new(),
                        source: "appimage".into(),
                        size: file_size(&e.path()),
                        manual: true,
                        summary: format!("~/{dir}"),
                        section: String::new(),
                        name,
                    });
                }
            }
        }
    }
    packages.sort_by(|a, b| a.name.cmp(&b.name));

    let mut sources = parse_sources(
        "/etc/apt/sources.list",
        &root.read("/etc/apt/sources.list").unwrap_or_default(),
    );
    let mut files: Vec<_> = std::fs::read_dir(root.path("/etc/apt/sources.list.d"))
        .map(|d| d.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    files.sort();
    for f in files {
        let name = f
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if name.ends_with(".list") || name.ends_with(".sources") {
            sources.extend(parse_sources(
                &format!("/etc/apt/sources.list.d/{name}"),
                &std::fs::read_to_string(&f).unwrap_or_default(),
            ));
        }
    }

    let mut removable = Vec::new();
    let unneeded = autoremove(
        &command::output("apt-get", &["-s", "autoremove"], Duration::from_secs(30))
            .await
            .unwrap_or_default(),
    );
    removable.push(Removable {
        action: "packages.autoremove".into(),
        title: "Packages no longer needed".into(),
        count: unneeded.len(),
        size: unneeded.iter().filter_map(|n| apt_sizes.get(n)).sum(),
        names: unneeded,
        undo: "reinstallable".into(),
    });
    removable.push(Removable {
        action: "storage.remove_old_snap_revisions".into(),
        title: "Disabled snap revisions".into(),
        count: snap_disabled_names.len(),
        size: snap_disabled,
        names: snap_disabled_names,
        undo: "irreversible".into(),
    });
    removable.push(Removable {
        action: "packages.purge_residual_config".into(),
        title: "Settings left by removed packages".into(),
        count: residual.len(),
        size: 0,
        names: residual,
        undo: "irreversible".into(),
    });
    removable.retain(|r| r.count > 0);
    PackagesInfo {
        packages,
        sources,
        removable,
    }
}

/// "1.2 GB" or "340.5 MB" (flatpak's sizes) → bytes.
fn human_size(text: &str) -> u64 {
    let mut f = text.split_whitespace();
    let value: f64 = f
        .next()
        .and_then(|v| v.replace(',', ".").parse().ok())
        .unwrap_or(0.0);
    let unit = match f.next().unwrap_or("") {
        "kB" | "KB" => 1e3,
        "MB" => 1e6,
        "GB" => 1e9,
        _ => 1.0,
    };
    (value * unit) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_dpkg_and_auto_marks() {
        let auto = auto_installed(
            "Package: libfoo\nArchitecture: amd64\nAuto-Installed: 1\n\nPackage: htop\nAuto-Installed: 0\n",
        );
        let status = "Package: htop\nStatus: install ok installed\nInstalled-Size: 300\nSection: utils\nVersion: 3.3.0\nDescription: interactive viewer\n more\n\nPackage: libfoo\nStatus: install ok installed\nInstalled-Size: 10\nVersion: 1\n\nPackage: oldapp\nStatus: deinstall ok config-files\nVersion: 2\n";
        let (pkgs, residual) = parse_dpkg(status, &auto);
        assert_eq!(
            pkgs.iter()
                .map(|p| (p.name.as_str(), p.manual, p.size))
                .collect::<Vec<_>>(),
            [("htop", true, 300 * 1024), ("libfoo", false, 10 * 1024)]
        );
        assert_eq!(pkgs[0].summary, "interactive viewer");
        assert_eq!(residual, ["oldapp"]);
    }

    #[test]
    fn reads_both_source_formats() {
        let list = parse_sources(
            "a.list",
            "deb [arch=amd64 signed-by=/usr/share/keyrings/x.gpg] https://packages.microsoft.com/repos/code stable main\n# deb http://archive.ubuntu.com/ubuntu noble main\n# a comment\n",
        );
        assert_eq!(
            (
                list[0].line.as_str(),
                list[0].kind.as_str(),
                list[0].signed,
                list[0].enabled
            ),
            (
                "https://packages.microsoft.com/repos/code stable main",
                "Third-party",
                true,
                true
            )
        );
        assert!(!list[1].enabled);
        let deb822 = parse_sources(
            "ubuntu.sources",
            "Types: deb\nURIs: http://archive.ubuntu.com/ubuntu/\nSuites: noble noble-updates\nComponents: main restricted\nSigned-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg\n\nTypes: deb\nURIs: https://ppa.launchpadcontent.net/x/y/ubuntu\nSuites: noble\nComponents: main\nEnabled: no\n",
        );
        assert_eq!(
            (deb822[0].kind.as_str(), deb822[0].signed),
            ("Official", true)
        );
        assert_eq!((deb822[1].kind.as_str(), deb822[1].enabled), ("PPA", false));
        assert_eq!(
            autoremove("Reading...\nRemv linux-headers-6.8.0-45 [6.8.0-45.45]\nRemv libx [1]\n"),
            ["linux-headers-6.8.0-45", "libx"]
        );
        assert_eq!(human_size("1.2 GB"), 1_200_000_000);
    }
}
