// SPDX-License-Identifier: Apache-2.0

//! Services and boot in depth for the Expert level (docs/01-product/expert-coverage.md,
//! section 10), from systemd's own tools, read only when asked:
//!
//! - `boot_detail`: the boot's critical chain (what the desktop really waited for) and how long
//!   every unit took (`systemd-analyze critical-chain`, `blame`).
//! - `unit_files`: each system service's unit file, its local and package overrides, what turns
//!   it on, its restart policy and restarts, its on/off state, and who masked it. (Ubuntu
//!   packages turn services on with their own scripts, not systemd presets, so the preset is not
//!   a reliable "Ubuntu default" and is not used.)
//! - `service_security`: how locked down each service is (`systemd-analyze security`).

use std::time::Duration;

use serde::Serialize;

use super::command;

const LIMIT: Duration = Duration::from_secs(10);

/// One step of the critical chain.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChainStep {
    pub unit: String,
    /// When it became active and how long it took to start, in ms after the kernel started.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub took_ms: Option<u64>,
    /// 0 for the target at the top.
    pub depth: usize,
}

/// What `boot_detail` answers.
#[derive(Debug, Default, Serialize)]
pub struct BootDetail {
    pub critical_chain: Vec<ChainStep>,
    /// Every unit with the time it took to start, slowest first.
    pub blame: Vec<(String, u64)>,
}

/// One system service's unit file facts.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct UnitFile {
    pub unit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Changes on top of the unit file made on this computer (in `/etc` or `/run`), and the
    /// ones its package ships (in `/usr/lib`).
    pub local_overrides: Vec<String>,
    pub package_overrides: Vec<String>,
    /// What turns it on, such as `multi-user.target`.
    pub wanted_by: Vec<String>,
    /// `no`, `on-failure`, `always`, …
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<String>,
    pub restarts: u64,
    /// `enabled`, `disabled`, `masked`, `static`, …
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// For a masked service: `local` (masked on this computer, in `/etc`), `runtime` (until the
    /// next restart, in `/run`), or `package` (its package ships it masked).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub masked_by: Option<&'static str>,
}

/// One service's exposure score: 0 is locked down, 10 has every right.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Exposure {
    pub unit: String,
    pub score: f64,
    /// `OK`, `MEDIUM`, `EXPOSED`, or `UNSAFE`.
    pub level: String,
}

/// `12.107s`, `41ms`, `1min 2.500s`, `2h 1min` in ms.
pub fn parse_span(text: &str) -> Option<u64> {
    let mut total = 0f64;
    let mut any = false;
    for part in text.split_whitespace() {
        let digits: String = part
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let unit = &part[digits.len()..];
        let value: f64 = digits.parse().ok()?;
        total += value
            * match unit {
                "us" | "µs" => 0.001,
                "ms" => 1.0,
                "s" => 1000.0,
                "min" => 60_000.0,
                "h" => 3_600_000.0,
                _ => return None,
            };
        any = true;
    }
    any.then_some(total.round() as u64)
}

const UNIT_KINDS: &[&str] = &[
    ".target",
    ".service",
    ".mount",
    ".socket",
    ".device",
    ".slice",
    ".scope",
    ".timer",
    ".path",
    ".swap",
    ".automount",
];

/// Parses `systemd-analyze critical-chain`.
pub fn parse_chain(text: &str) -> Vec<ChainStep> {
    text.lines()
        .filter_map(|line| {
            let start = line.find(|c: char| c.is_ascii_alphanumeric())?;
            // Only lines that start with a unit name (the headers are sentences).
            let first = line[start..].split_whitespace().next()?;
            if !UNIT_KINDS.iter().any(|k| first.ends_with(k)) {
                return None;
            }
            // Each level adds two characters of tree drawing ("└─" or "  ").
            let depth = line[..start].chars().count() / 2;
            let rest = &line[start..];
            let unit = rest.split_whitespace().next()?.to_owned();
            let at_ms = rest
                .split_once('@')
                .and_then(|(_, a)| parse_span(a.split('+').next()?.trim()));
            let took_ms = rest.split_once('+').and_then(|(_, t)| parse_span(t.trim()));
            Some(ChainStep {
                unit,
                at_ms,
                took_ms,
                depth,
            })
        })
        .collect()
}

/// Parses `systemd-analyze blame`: a time, then the unit.
pub fn parse_blame(text: &str) -> Vec<(String, u64)> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            let i = line.rfind(' ')?;
            let (time, unit) = line.split_at(i);
            Some((unit.trim().to_owned(), parse_span(time.trim())?))
        })
        .collect()
}

/// Parses `systemctl show` output: blocks separated by blank lines.
pub fn parse_show(text: &str) -> Vec<UnitFile> {
    text.split("\n\n")
        .filter_map(|block| {
            let get = |k: &str| {
                block
                    .lines()
                    .find_map(|l| l.strip_prefix(k)?.strip_prefix('='))
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
            };
            let list = |k: &str| {
                get(k)
                    .map(|v| v.split_whitespace().map(str::to_owned).collect::<Vec<_>>())
                    .unwrap_or_default()
            };
            let (package_overrides, local_overrides): (Vec<String>, Vec<String>) =
                list("DropInPaths")
                    .into_iter()
                    .partition(|p| p.starts_with("/usr/") || p.starts_with("/lib/"));
            Some(UnitFile {
                unit: get("Id")?.to_owned(),
                file: get("FragmentPath").map(str::to_owned),
                local_overrides,
                package_overrides,
                wanted_by: list("WantedBy"),
                restart: get("Restart").map(str::to_owned),
                restarts: get("NRestarts").and_then(|v| v.parse().ok()).unwrap_or(0),
                state: get("UnitFileState").map(str::to_owned),
                masked_by: None,
            })
        })
        .collect()
}

/// Parses `systemd-analyze security`: unit, score, level, and a face.
pub fn parse_security(text: &str) -> Vec<Exposure> {
    text.lines()
        .skip(1)
        .filter_map(|line| {
            let f: Vec<&str> = line.split_whitespace().collect();
            Some(Exposure {
                unit: f.first()?.to_string(),
                score: f.get(1)?.parse().ok()?,
                level: f.get(2)?.to_string(),
            })
        })
        .collect()
}

/// Reads `boot_detail`.
pub async fn read_boot() -> BootDetail {
    BootDetail {
        critical_chain: command::output(
            "systemd-analyze",
            &["critical-chain", "--no-pager"],
            LIMIT,
        )
        .await
        .map(|t| parse_chain(&t))
        .unwrap_or_default(),
        blame: command::output("systemd-analyze", &["blame", "--no-pager"], LIMIT)
            .await
            .map(|t| parse_blame(&t))
            .unwrap_or_default(),
    }
}

/// Reads `unit_files` for every system service systemd knows.
pub async fn read_units() -> Vec<UnitFile> {
    let Some(list) = command::output(
        "systemctl",
        &[
            "list-units",
            "--type=service",
            "--all",
            "--no-legend",
            "--plain",
            "--no-pager",
        ],
        LIMIT,
    )
    .await
    else {
        return Vec::new();
    };
    let mut names: Vec<String> = list
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .filter(|n| n.ends_with(".service"))
        .map(str::to_owned)
        .collect();
    // Masked services are not loaded, so they are listed separately.
    if let Some(masked) = command::output(
        "systemctl",
        &[
            "list-unit-files",
            "--type=service",
            "--state=masked",
            "--no-legend",
            "--no-pager",
        ],
        LIMIT,
    )
    .await
    {
        names.extend(
            masked
                .lines()
                .filter_map(|l| l.split_whitespace().next())
                .map(str::to_owned),
        );
    }
    names.sort();
    names.dedup();
    let mut out = Vec::new();
    for chunk in names.chunks(100) {
        let mut args = vec![
            "show",
            "--no-pager",
            "-p",
            "Id,FragmentPath,DropInPaths,WantedBy,Restart,NRestarts,UnitFileState",
        ];
        args.extend(chunk.iter().map(String::as_str));
        if let Some(text) = command::output("systemctl", &args, LIMIT).await {
            out.extend(parse_show(&text));
        }
    }
    for u in out
        .iter_mut()
        .filter(|u| u.state.as_deref() == Some("masked"))
    {
        u.masked_by = Some(masked_by(&crate::collect::SystemRoot::real(), &u.unit));
    }
    out
}

/// Where a masked unit's mask is: a link to `/dev/null` in `/etc` (local), `/run` (until the next
/// restart), or only in the package's own folder.
fn masked_by(root: &crate::collect::SystemRoot, unit: &str) -> &'static str {
    let is_mask = |dir: &str| {
        std::fs::read_link(root.path(&format!("{dir}/{unit}")))
            .is_ok_and(|t| t == std::path::Path::new("/dev/null"))
    };
    if is_mask("/etc/systemd/system") {
        "local"
    } else if is_mask("/run/systemd/system") {
        "runtime"
    } else {
        "package"
    }
}

/// Reads `service_security`.
pub async fn read_security() -> Vec<Exposure> {
    command::output("systemd-analyze", &["security", "--no-pager"], LIMIT)
        .await
        .map(|t| parse_security(&t))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans() {
        assert_eq!(parse_span("12.107s"), Some(12_107));
        assert_eq!(parse_span("41ms"), Some(41));
        assert_eq!(parse_span("1min 2.500s"), Some(62_500));
        assert_eq!(parse_span("900us"), Some(1));
        assert_eq!(parse_span("soon"), None);
    }

    #[test]
    fn critical_chain_with_depth() {
        let t = "The time when unit became active or started is printed after the \"@\" character.\nThe time the unit took to start is printed after the \"+\" character.\n\ngraphical.target @12.107s\n└─multi-user.target @12.107s\n  └─kerneloops.service @12.064s +41ms\n    └─network-online.target @12.044s\n      └─NetworkManager-wait-online.service @6.054s +5.988s\n";
        let c = parse_chain(t);
        assert_eq!(c.len(), 5);
        assert_eq!(
            (c[0].unit.as_str(), c[0].depth, c[0].at_ms),
            ("graphical.target", 0, Some(12_107))
        );
        assert_eq!(
            (c[4].unit.as_str(), c[4].depth, c[4].took_ms),
            ("NetworkManager-wait-online.service", 4, Some(5_988))
        );
        assert_eq!(c[1].took_ms, None);
    }

    #[test]
    fn blame_show_and_security() {
        assert_eq!(
            parse_blame("5.988s NetworkManager-wait-online.service\n 1min 3s snapd.service\n"),
            [
                ("NetworkManager-wait-online.service".to_owned(), 5988),
                ("snapd.service".to_owned(), 63_000)
            ]
        );
        let u = parse_show(
            "Restart=on-failure\nNRestarts=2\nId=cups.service\nWantedBy=multi-user.target printer.target\nFragmentPath=/usr/lib/systemd/system/cups.service\nDropInPaths=/etc/systemd/system/cups.service.d/override.conf /usr/lib/systemd/system/cups.service.d/debian.conf\nUnitFileState=disabled\n\nId=alsa-utils.service\nUnitFileState=masked\n",
        );
        assert_eq!(u.len(), 2);
        assert_eq!(u[0].wanted_by, ["multi-user.target", "printer.target"]);
        assert_eq!(
            u[0].local_overrides,
            ["/etc/systemd/system/cups.service.d/override.conf"]
        );
        assert_eq!(
            u[0].package_overrides,
            ["/usr/lib/systemd/system/cups.service.d/debian.conf"]
        );
        assert_eq!(
            (u[0].restarts, u[0].state.as_deref()),
            (2, Some("disabled"))
        );
        assert_eq!(
            (u[1].file.as_ref(), u[1].state.as_deref()),
            (None, Some("masked"))
        );
        let s = parse_security(
            "UNIT                                  EXPOSURE PREDICATE HAPPY\nNetworkManager.service                     7.8 EXPOSED   🙁\n",
        );
        assert_eq!(
            s,
            [Exposure {
                unit: "NetworkManager.service".into(),
                score: 7.8,
                level: "EXPOSED".into()
            }]
        );
    }

    #[test]
    fn masks_are_told_apart_by_their_place() {
        let root = crate::collect::test_support::fixture_root(
            "masked",
            &[("/usr/lib/systemd/system/sudo.service", "")],
        );
        std::fs::create_dir_all(root.path("/etc/systemd/system")).unwrap();
        std::os::unix::fs::symlink("/dev/null", root.path("/etc/systemd/system/saned.service"))
            .unwrap();
        assert_eq!(masked_by(&root, "saned.service"), "local");
        assert_eq!(masked_by(&root, "sudo.service"), "package");
    }
}
