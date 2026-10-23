// SPDX-License-Identifier: Apache-2.0

//! Use per app and per service, from the kernel's control groups (cgroup v2), for the Expert
//! level (docs/01-product/expert-coverage.md, sections 2, 3, and 10). GNOME starts every app in
//! its own group, so an app's many processes count once. Memory, CPU time, and pressure are
//! readable without admin rights; disk use is counted for system services (the desktop does
//! not count it per app). Totals since start, with the reading time, so the app works out rates.

use serde::Serialize;

use crate::collect::SystemRoot;

/// One app or service.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Group {
    /// `service` (system), `user-service`, or `app`.
    pub kind: &'static str,
    /// The systemd unit, such as `cups.service` or `app-gnome-org.gnome.Nautilus-1234.scope`.
    pub unit: String,
    /// A name for people: the app's name from its desktop file, or the unit without its suffix.
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<u64>,
    /// Of `memory`: the app's own memory (`anon`), and file cache charged to it (`file`),
    /// which the kernel can free at once.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_own: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_cache: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_peak: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap: Option<u64>,
    /// Share of the last 10 seconds some of its tasks waited for memory, in percent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_pressure: Option<f64>,
    /// CPU time used since it started, in microseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_usec: Option<u64>,
    /// Bytes read from and written to disks since it started (system services only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_read: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_write: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks: Option<u64>,
}

/// What `cgroups` answers.
#[derive(Debug, Default, Serialize)]
pub struct Groups {
    /// When this was read, in ms since 1970.
    pub at: u64,
    pub groups: Vec<Group>,
}

fn text(root: &SystemRoot, path: &str) -> Option<String> {
    root.read(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

/// Undoes systemd's escaping in unit names: `\x2d` is `-`.
pub fn unescape(name: &str) -> String {
    let mut out = String::new();
    let mut rest = name;
    while let Some(i) = rest.find("\\x") {
        out.push_str(&rest[..i]);
        match rest
            .get(i + 2..i + 4)
            .and_then(|h| u8::from_str_radix(h, 16).ok())
        {
            Some(b) => {
                out.push(b as char);
                rest = &rest[i + 4..];
            }
            None => {
                out.push_str("\\x");
                rest = &rest[i + 2..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// The app ID inside a GNOME app unit: `app-gnome-org.gnome.Nautilus-1234.scope` gives
/// `org.gnome.Nautilus`; `app-com.google.Chrome-5273.scope` gives `com.google.Chrome`.
pub fn app_id(unit: &str) -> Option<String> {
    let base = unit
        .strip_suffix(".scope")
        .or_else(|| unit.strip_suffix(".service"))?;
    let rest = base.strip_prefix("app-")?;
    let rest = rest
        .strip_prefix("gnome-")
        .or_else(|| rest.strip_prefix("flatpak-"))
        .unwrap_or(rest);
    // A trailing `-<number>` (the launch ID) or `@<instance>` is not part of the app.
    let rest = rest.split('@').next().unwrap_or(rest);
    let rest = match rest.rsplit_once('-') {
        Some((head, tail))
            if !tail.is_empty()
                && tail.chars().all(|c| c.is_ascii_hexdigit())
                && !head.is_empty() =>
        {
            head
        }
        _ => rest,
    };
    Some(unescape(rest))
}

/// The `Name=` of an app's desktop file, from the places Ubuntu keeps them.
fn desktop_name(root: &SystemRoot, home: Option<&str>, id: &str) -> Option<String> {
    let mut dirs = vec![
        "/usr/share/applications".to_owned(),
        "/usr/local/share/applications".to_owned(),
        "/var/lib/snapd/desktop/applications".to_owned(),
        "/var/lib/flatpak/exports/share/applications".to_owned(),
    ];
    if let Some(home) = home {
        dirs.push(format!("{home}/.local/share/applications"));
        dirs.push(format!(
            "{home}/.local/share/flatpak/exports/share/applications"
        ));
    }
    dirs.iter().find_map(|d| {
        let file = root.read(&format!("{d}/{id}.desktop")).ok()?;
        file.lines()
            .find_map(|l| l.strip_prefix("Name=").map(|n| n.trim().to_owned()))
            .filter(|n| !n.is_empty())
    })
}

fn sum_io(text: &str) -> (u64, u64) {
    text.split_whitespace()
        .fold((0, 0), |(r, w), field| match field.split_once('=') {
            Some(("rbytes", v)) => (r + v.parse::<u64>().unwrap_or(0), w),
            Some(("wbytes", v)) => (r, w + v.parse::<u64>().unwrap_or(0)),
            _ => (r, w),
        })
}

fn read_group(
    root: &SystemRoot,
    dir: &str,
    unit: &str,
    kind: &'static str,
    name: String,
) -> Option<Group> {
    let at = |f: &str| format!("{dir}/{f}");
    let memory = text(root, &at("memory.current")).and_then(|v| v.parse().ok());
    let cpu_usec = text(root, &at("cpu.stat")).and_then(|t| {
        t.lines()
            .find_map(|l| l.strip_prefix("usage_usec ")?.trim().parse().ok())
    });
    if memory.is_none() && cpu_usec.is_none() {
        return None;
    }
    let io = text(root, &at("io.stat")).map(|t| sum_io(&t));
    let stat = text(root, &at("memory.stat")).unwrap_or_default();
    let stat_value = |key: &str| {
        stat.lines().find_map(|l| {
            l.strip_prefix(key)?
                .strip_prefix(' ')?
                .trim()
                .parse::<u64>()
                .ok()
        })
    };
    Some(Group {
        kind,
        unit: unit.to_owned(),
        name,
        memory,
        memory_own: stat_value("anon"),
        memory_cache: stat_value("file"),
        memory_peak: text(root, &at("memory.peak")).and_then(|v| v.parse().ok()),
        swap: text(root, &at("memory.swap.current")).and_then(|v| v.parse().ok()),
        memory_pressure: text(root, &at("memory.pressure")).and_then(|t| {
            t.lines()
                .next()?
                .split_whitespace()
                .find_map(|f| f.strip_prefix("avg10=")?.parse().ok())
        }),
        cpu_usec,
        disk_read: io.map(|(r, _)| r),
        disk_write: io.map(|(_, w)| w),
        tasks: text(root, &at("pids.current")).and_then(|v| v.parse().ok()),
    })
}

/// Units inside `dir`, going one level into `.slice` folders (such as `system-getty.slice`).
fn units(root: &SystemRoot, dir: &str) -> Vec<(String, String)> {
    let list = |d: &str| -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(root.path(d))
            .map(|r| {
                r.flatten()
                    .filter(|e| e.path().is_dir())
                    .filter_map(|e| e.file_name().to_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        v.sort();
        v
    };
    let mut out = Vec::new();
    for name in list(dir) {
        let path = format!("{dir}/{name}");
        if name.ends_with(".slice") {
            out.extend(
                list(&path)
                    .into_iter()
                    .filter(|n| n.ends_with(".service") || n.ends_with(".scope"))
                    .map(|n| (format!("{path}/{n}"), n)),
            );
        } else if name.ends_with(".service") || name.ends_with(".scope") {
            out.push((path, name));
        }
    }
    out
}

fn uid(root: &SystemRoot) -> Option<u32> {
    root.read("/proc/self/status").ok()?.lines().find_map(|l| {
        l.strip_prefix("Uid:")?
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    })
}

/// Reads every system service, user service, and app with its use.
pub fn read(root: &SystemRoot) -> Groups {
    let cg = "/sys/fs/cgroup";
    let home = std::env::var("HOME").ok();
    let mut groups: Vec<Group> = units(root, &format!("{cg}/system.slice"))
        .into_iter()
        .filter_map(|(dir, unit)| {
            let name = unescape(unit.trim_end_matches(".service"));
            read_group(root, &dir, &unit, "service", name)
        })
        .collect();
    if let Some(uid) = uid(root) {
        let user = format!("{cg}/user.slice/user-{uid}.slice/user@{uid}.service");
        for slice in ["app.slice", "session.slice", "background.slice"] {
            for (dir, unit) in units(root, &format!("{user}/{slice}")) {
                let (kind, name) = match app_id(&unit) {
                    Some(id) => (
                        "app",
                        desktop_name(root, home.as_deref(), &id).unwrap_or(id),
                    ),
                    None => (
                        "user-service",
                        unescape(unit.trim_end_matches(".service").trim_end_matches(".scope")),
                    ),
                };
                groups.extend(read_group(root, &dir, &unit, kind, name));
            }
        }
    }
    let at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    Groups { at, groups }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn unit_names_become_app_ids() {
        assert_eq!(
            unescape("org.gnome.Evolution\\x2dalarm\\x2dnotify"),
            "org.gnome.Evolution-alarm-notify"
        );
        assert_eq!(unescape("broken\\xZZ"), "broken\\xZZ");
        assert_eq!(
            app_id("app-gnome-org.gnome.Nautilus-1234.scope").as_deref(),
            Some("org.gnome.Nautilus")
        );
        assert_eq!(
            app_id("app-com.google.Chrome-5273.scope").as_deref(),
            Some("com.google.Chrome")
        );
        assert_eq!(
            app_id("app-gnome-chrome\\x2dompif\\x2dProfile_1-5273.scope").as_deref(),
            Some("chrome-ompif-Profile_1")
        );
        assert_eq!(
            app_id("app-flatpak-org.mozilla.firefox-a1b2c3.scope").as_deref(),
            Some("org.mozilla.firefox")
        );
        assert_eq!(
            app_id("app-gnome-org.gnome.Terminal@1.service").as_deref(),
            Some("org.gnome.Terminal")
        );
        assert_eq!(app_id("cups.service"), None);
    }

    #[test]
    fn reads_services_and_apps_with_their_names() {
        let cg = "/sys/fs/cgroup";
        let user = "/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service";
        let root = fixture_root(
            "cgroups",
            &[
                (
                    "/proc/self/status",
                    "Name:\tx\nUid:\t1000\t1000\t1000\t1000\n",
                ),
                (
                    &format!("{cg}/system.slice/cups.service/memory.current"),
                    "4096\n",
                ),
                (
                    &format!("{cg}/system.slice/cups.service/cpu.stat"),
                    "usage_usec 1500\nuser_usec 1000\n",
                ),
                (
                    &format!("{cg}/system.slice/cups.service/io.stat"),
                    "259:0 rbytes=100 wbytes=50 rios=1\n8:0 rbytes=10 wbytes=5\n",
                ),
                (
                    &format!(
                        "{cg}/system.slice/system-getty.slice/getty@tty1.service/memory.current"
                    ),
                    "1024\n",
                ),
                (
                    &format!("{cg}/system.slice/empty.mount/memory.current"),
                    "1\n",
                ),
                (
                    &format!(
                        "{user}/app.slice/app-gnome-org.gnome.Nautilus-42.scope/memory.current"
                    ),
                    "8192\n",
                ),
                (
                    &format!("{user}/app.slice/app-gnome-org.gnome.Nautilus-42.scope/memory.stat"),
                    "anon 6000\nfile 2000\nanon_thp 0\n",
                ),
                (
                    &format!(
                        "{user}/app.slice/app-gnome-org.gnome.Nautilus-42.scope/memory.pressure"
                    ),
                    "some avg10=1.50 avg60=0.00 avg300=0.00 total=1\nfull avg10=0.00\n",
                ),
                (
                    "/usr/share/applications/org.gnome.Nautilus.desktop",
                    "[Desktop Entry]\nName=Files\nExec=nautilus\n",
                ),
                (
                    &format!("{user}/session.slice/pipewire.service/memory.current"),
                    "2048\n",
                ),
            ],
        );
        let g = read(&root).groups;
        let names: Vec<(&str, &str)> = g.iter().map(|x| (x.kind, x.name.as_str())).collect();
        assert_eq!(
            names,
            [
                ("service", "cups"),
                ("service", "getty@tty1"),
                ("app", "Files"),
                ("user-service", "pipewire")
            ]
        );
        assert_eq!(
            (g[0].disk_read, g[0].disk_write, g[0].cpu_usec),
            (Some(110), Some(55), Some(1500))
        );
        assert_eq!(g[2].memory_pressure, Some(1.5));
        assert_eq!(
            (g[2].memory_own, g[2].memory_cache),
            (Some(6000), Some(2000))
        );
        assert_eq!(g[2].disk_read, None, "apps have no disk counters");
    }
}
