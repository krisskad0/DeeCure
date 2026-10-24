// SPDX-License-Identifier: Apache-2.0

//! Software in depth for the Expert level (docs/01-product/expert-coverage.md, section 13):
//!
//! - `software_health`: held packages, half-installed packages, packages from no repository,
//!   every snap revision with its notes and the refresh schedule, and Flatpak runtimes no
//!   installed app uses (by runtime name).
//! - `package_detail` (its own method): one package's facts, what it needs, what needs it, where
//!   it can come from, and its changed files.
//! - `package_verify` and `package_verify_run`: every installed file checked against its
//!   package's checksum. It runs only when asked, at the lowest CPU and disk priority, and can
//!   take minutes; the answer says while it runs.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use serde::Serialize;

use super::command;
use crate::collect::SystemRoot;

const LIMIT: Duration = Duration::from_secs(20);
/// The full check reads every installed file.
const VERIFY_LIMIT: Duration = Duration::from_secs(30 * 60);

/// One snap revision.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Snap {
    pub name: String,
    pub version: String,
    pub revision: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    /// `base`, `classic`, `disabled` (an old revision kept), `held`, `devmode`, …
    pub notes: Vec<String>,
}

/// One Flatpak runtime.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Runtime {
    pub id: String,
    pub branch: String,
    /// `system` or `user`.
    pub installation: String,
    /// An installed app uses it, or it extends a runtime an app uses.
    pub used: bool,
}

/// What `software_health` answers.
#[derive(Debug, Default, Serialize)]
pub struct SoftwareHealth {
    pub held: Vec<String>,
    /// `dpkg --audit` lines: packages that are half installed or half removed.
    pub broken: Vec<String>,
    /// Packages no configured repository offers, with their version.
    pub local: Vec<(String, String)>,
    /// Missing when snapd does not run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snaps: Option<Vec<Snap>>,
    /// `timer`, `last`, `next`, and `hold`, from `snap refresh --time`.
    pub snap_refresh: BTreeMap<String, String>,
    /// Missing when Flatpak is not installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtimes: Option<Vec<Runtime>>,
}

/// What `package_detail` answers.
#[derive(Debug, Default, Serialize)]
pub struct PackageDetail {
    pub name: String,
    pub fields: BTreeMap<String, String>,
    /// Installed by hand (`false` when only as a dependency).
    pub manual: bool,
    /// What it needs (Depends and Pre-Depends), and the installed packages that need it.
    pub needs: Vec<String>,
    pub needed_by: Vec<String>,
    /// `apt-cache policy`: installed and candidate versions, and each version's sources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate: Option<String>,
    pub sources: Vec<(String, Vec<String>)>,
    pub files: usize,
    /// Files that differ from the package (`dpkg -V`), with what differs.
    pub changed_files: Vec<(String, String)>,
}

/// A file that differs from its package.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FileProblem {
    pub file: String,
    /// `missing`, `changed`, or `unreadable` (needs admin rights to check).
    pub problem: &'static str,
    /// A configuration file, which people are expected to change.
    pub config: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
}

/// What `package_verify` answers.
#[derive(Debug, Clone, Default, Serialize)]
pub struct VerifyAnswer {
    pub running: bool,
    /// When the last check finished, in ms since 1970, and how long it took.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub took_s: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problems: Option<Vec<FileProblem>>,
    /// The check could not finish (time limit, or dpkg failed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed: Option<String>,
}

/// The last full check and whether one runs.
#[derive(Debug, Default)]
pub struct VerifyState {
    /// What the last check found, or that one runs.
    pub answer: VerifyAnswer,
}

/// `snap list --all` rows.
pub fn parse_snaps(text: &str) -> Vec<Snap> {
    text.lines()
        .skip(1)
        .filter_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            Some(Snap {
                name: f.first()?.to_string(),
                version: f.get(1)?.to_string(),
                revision: f.get(2)?.to_string(),
                tracking: f.get(3).filter(|t| **t != "-").map(|t| t.to_string()),
                publisher: f.get(4).map(|p| p.trim_end_matches(['*', '✓']).to_string()),
                notes: f
                    .get(5)
                    .filter(|n| **n != "-")
                    .map(|n| n.split(',').map(str::to_owned).collect())
                    .unwrap_or_default(),
            })
        })
        .collect()
}

/// Extension points (`[Extension org.freedesktop.Platform.GL]`) in `flatpak info --show-metadata`.
pub fn extension_points(metadata: &str) -> Vec<String> {
    metadata
        .lines()
        .filter_map(|l| {
            l.trim()
                .strip_prefix("[Extension ")?
                .strip_suffix(']')
                .map(str::to_owned)
        })
        .collect()
}

/// Marks the runtimes that an installed app uses, or that fill an extension point an app or a
/// used runtime declares (such as a GL driver). Any branch counts, so nothing used is ever
/// called unused.
pub fn mark_runtimes(runtimes: &mut [Runtime], app_runtimes: &[String], points: &[String]) {
    // App runtimes look like `org.gnome.Platform/x86_64/46`.
    let used: Vec<(String, String)> = app_runtimes
        .iter()
        .filter_map(|r| {
            let mut p = r.split('/');
            Some((p.next()?.to_owned(), p.nth(1)?.to_owned()))
        })
        .collect();
    for r in runtimes.iter_mut() {
        r.used = used
            .iter()
            .any(|(id, branch)| r.id == *id && r.branch == *branch)
            || points
                .iter()
                .any(|p| r.id == *p || r.id.starts_with(&format!("{p}.")));
    }
}

/// Reads `software_health`.
pub async fn read_health() -> SoftwareHealth {
    let local = command::output("apt", &["list", "--installed"], LIMIT)
        .await
        .map(|t| {
            t.lines()
                .filter(|l| l.contains(",local]") || l.ends_with("[installed,local]"))
                .filter_map(|l| {
                    let (name, rest) = l.split_once('/')?;
                    Some((name.to_owned(), rest.split_whitespace().nth(1)?.to_owned()))
                })
                .collect()
        })
        .unwrap_or_default();
    let snaps = command::output(
        "snap",
        &["list", "--all", "--unicode=never", "--color=never"],
        LIMIT,
    )
    .await
    .map(|t| parse_snaps(&t));
    let snap_refresh = command::output("snap", &["refresh", "--time"], LIMIT)
        .await
        .map(|t| {
            t.lines()
                .filter_map(|l| l.split_once(':'))
                .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
                .collect()
        })
        .unwrap_or_default();
    let runtimes = match command::output(
        "flatpak",
        &[
            "list",
            "--runtime",
            "--columns=application,branch,installation",
        ],
        LIMIT,
    )
    .await
    {
        Some(list) => {
            let mut r: Vec<Runtime> = list
                .lines()
                .filter_map(|l| {
                    let f: Vec<&str> = l.split('\t').collect();
                    Some(Runtime {
                        id: f.first()?.trim().to_owned(),
                        branch: f.get(1)?.trim().to_owned(),
                        installation: f.get(2).map_or("system", |s| s.trim()).to_owned(),
                        used: false,
                    })
                })
                .collect();
            let listed = command::output(
                "flatpak",
                &["list", "--app", "--columns=application,runtime"],
                LIMIT,
            )
            .await
            .unwrap_or_default();
            let apps: Vec<(String, String)> = listed
                .lines()
                .filter_map(|l| l.split_once('\t'))
                .map(|(a, r)| (a.trim().to_owned(), r.trim().to_owned()))
                .collect();
            let mut points = Vec::new();
            let mut refs: Vec<String> = apps.iter().map(|(a, _)| a.clone()).collect();
            refs.extend(apps.iter().map(|(_, r)| format!("runtime/{r}")));
            refs.sort();
            refs.dedup();
            for reference in &refs {
                if let Some(meta) =
                    command::output("flatpak", &["info", "--show-metadata", reference], LIMIT).await
                {
                    points.extend(extension_points(&meta));
                }
            }
            let app_runtimes: Vec<String> = apps.into_iter().map(|(_, r)| r).collect();
            mark_runtimes(&mut r, &app_runtimes, &points);
            Some(r)
        }
        None => None,
    };
    SoftwareHealth {
        held: command::output("apt-mark", &["showhold"], LIMIT)
            .await
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .filter(|l| !l.is_empty())
            .collect(),
        broken: command::output_any("dpkg", &["--audit"], LIMIT)
            .await
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .filter(|l| !l.trim().is_empty())
            .collect(),
        local,
        snaps,
        snap_refresh,
        runtimes,
    }
}

/// The paragraph of one package in `dpkg-query -s` output, as field → value.
pub fn status_paragraph(status: &str, name: &str) -> Option<BTreeMap<String, String>> {
    status.split("\n\n").find_map(|p| {
        let first = p.lines().next()?;
        if first.strip_prefix("Package: ")?.trim() != name {
            return None;
        }
        let mut fields = BTreeMap::new();
        let mut last: Option<String> = None;
        for line in p.lines() {
            if line.starts_with(' ') {
                if let Some(k) = &last
                    && k != "Description"
                    && let Some(v) = fields.get_mut(k)
                {
                    *v = format!("{v}\n{}", line.trim());
                }
                continue;
            }
            if let Some((k, v)) = line.split_once(':') {
                fields.insert(k.to_owned(), v.trim().to_owned());
                last = Some(k.to_owned());
            }
        }
        Some(fields)
    })
}

/// Package names in a Depends-style field: `libc6 (>= 2.34), libfoo | libbar` gives all three.
pub fn depends_names(field: &str) -> Vec<String> {
    field
        .split([',', '|'])
        .filter_map(|d| d.split_whitespace().next())
        .map(|n| n.split(':').next().unwrap_or(n).to_owned())
        .filter(|n| !n.is_empty())
        .collect()
}

/// What `apt-cache policy` says about one package.
#[derive(Debug, Default, PartialEq)]
pub struct Policy {
    pub installed: Option<String>,
    pub candidate: Option<String>,
    /// Each version with the sources that offer it.
    pub sources: Vec<(String, Vec<String>)>,
}

/// Parses `apt-cache policy name`: installed, candidate, and each version's sources.
pub fn parse_policy(text: &str) -> Policy {
    let get = |k: &str| {
        text.lines()
            .find_map(|l| l.trim().strip_prefix(k).map(|v| v.trim().to_owned()))
            .filter(|v| v != "(none)")
    };
    let mut sources: Vec<(String, Vec<String>)> = Vec::new();
    let mut in_table = false;
    for line in text.lines() {
        if line.trim() == "Version table:" {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }
        let t = line.trim_start_matches([' ', '*']).trim();
        // Version lines have 5 spaces or "*** " before them; source lines have 8 or more.
        let indent = line.len() - line.trim_start().len();
        if indent <= 5 {
            if let Some(v) = t.split_whitespace().next() {
                sources.push((v.to_owned(), Vec::new()));
            }
        } else if let Some(last) = sources.last_mut() {
            let src = t.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
            last.1.push(if src.is_empty() { t.to_owned() } else { src });
        }
    }
    Policy {
        installed: get("Installed:"),
        candidate: get("Candidate:"),
        sources,
    }
}

/// What `dpkg -V` lines say: `??5?????? c /etc/x` (a changed configuration file), `missing   /usr/x`,
/// and `?????????   /boot/x` or `… (Permission denied)` (could not be read without admin rights).
/// Paths may contain spaces.
pub fn parse_verify(text: &str) -> Vec<FileProblem> {
    text.lines()
        .filter_map(|l| {
            if let Some(rest) = l.strip_prefix("missing") {
                let rest = rest.trim();
                let (path, denied) = match rest.strip_suffix(" (Permission denied)") {
                    Some(p) => (p, true),
                    None => (rest, false),
                };
                let (config, path) = match path.strip_prefix("c ") {
                    Some(p) => (true, p.trim()),
                    None => (false, path),
                };
                return path.starts_with('/').then(|| FileProblem {
                    file: path.to_owned(),
                    problem: if denied { "unreadable" } else { "missing" },
                    config,
                    package: None,
                });
            }
            let flags = l.get(..9)?;
            let rest = l.get(9..)?.trim_start();
            let (config, path) = match rest.strip_prefix("c ") {
                Some(p) => (true, p.trim_start()),
                None => (false, rest),
            };
            if !path.starts_with('/') {
                return None;
            }
            let problem = if flags.chars().all(|c| c == '?') {
                "unreadable"
            } else if flags.get(2..3) == Some("5") {
                "changed"
            } else {
                return None;
            };
            Some(FileProblem {
                file: path.to_owned(),
                problem,
                config,
                package: None,
            })
        })
        .collect()
}

/// The answers `package_detail` is built from, so the parsing is tested without dpkg.
pub struct PackageAnswers<'a> {
    /// `dpkg-query -s name`.
    pub status: &'a str,
    /// `dpkg-query -W` lines: package, status, and its Pre-Depends and Depends.
    pub listing: &'a str,
    /// `/var/lib/apt/extended_states`.
    pub extended: &'a str,
    /// `apt-cache policy name`.
    pub policy: &'a str,
    /// `dpkg-query -L name`.
    pub files: &'a str,
    /// `dpkg -V name`.
    pub verify: &'a str,
}

/// Builds `package_detail` from the answers of dpkg-query, apt-cache, and dpkg -V. (The dpkg
/// database is never read directly: its format is dpkg's own business.)
pub fn assemble(name: &str, a: &PackageAnswers) -> Result<PackageDetail, String> {
    let fields = status_paragraph(a.status, name)
        .ok_or_else(|| format!("package {name} is not installed"))?;
    if !fields.get("Status").is_some_and(|s| {
        s.ends_with("installed") && !s.contains("not-installed") && !s.contains("config-files")
    }) {
        return Err(format!("package {name} is not installed"));
    }
    let needs = ["Pre-Depends", "Depends"]
        .iter()
        .filter_map(|k| fields.get(*k))
        .flat_map(|f| depends_names(f))
        .collect();
    let needed_by = a
        .listing
        .lines()
        .filter_map(|l| {
            let mut f = l.splitn(3, '\t');
            let (pkg, state, deps) = (f.next()?, f.next()?, f.next().unwrap_or(""));
            (state.starts_with("ii") && depends_names(deps).iter().any(|d| d == name))
                .then(|| pkg.to_owned())
        })
        .collect();
    let manual = !a.extended.split("\n\n").any(|p| {
        p.lines().any(|l| l.trim() == format!("Package: {name}")) && p.contains("Auto-Installed: 1")
    });
    let Policy {
        installed,
        candidate,
        sources,
    } = parse_policy(a.policy);
    let changed_files = parse_verify(a.verify)
        .into_iter()
        .map(|p| {
            (
                p.file,
                format!(
                    "{}{}",
                    p.problem,
                    if p.config { " (configuration)" } else { "" }
                ),
            )
        })
        .collect();
    Ok(PackageDetail {
        name: name.to_owned(),
        manual,
        needs,
        needed_by,
        installed,
        candidate,
        sources,
        files: a
            .files
            .lines()
            .filter(|l| l.starts_with('/') && *l != "/.")
            .count(),
        changed_files,
        fields: fields
            .into_iter()
            .filter(|(k, _)| {
                [
                    "Version",
                    "Installed-Size",
                    "Section",
                    "Maintainer",
                    "Homepage",
                    "Architecture",
                    "Source",
                    "Description",
                ]
                .contains(&k.as_str())
            })
            .collect(),
    })
}

/// Reads one package for `package_detail`.
pub async fn package_detail(root: &SystemRoot, name: &str) -> Result<PackageDetail, String> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "+-.:".contains(c))
    {
        return Err(format!("'{name}' is not a package name"));
    }
    let status = command::output_any("dpkg-query", &["-s", name], LIMIT)
        .await
        .unwrap_or_default();
    let listing = command::output(
        "dpkg-query",
        &[
            "-W",
            "-f=${Package}\t${db:Status-Abbrev}\t${Pre-Depends}, ${Depends}\n",
        ],
        LIMIT,
    )
    .await
    .unwrap_or_default();
    let extended = root
        .read("/var/lib/apt/extended_states")
        .unwrap_or_default();
    let policy = command::output("apt-cache", &["policy", name], LIMIT)
        .await
        .unwrap_or_default();
    let files = command::output_any("dpkg-query", &["-L", name], LIMIT)
        .await
        .unwrap_or_default();
    let verify = command::output_any("dpkg", &["-V", name], LIMIT)
        .await
        .unwrap_or_default();
    assemble(
        name,
        &PackageAnswers {
            status: &status,
            listing: &listing,
            extended: &extended,
            policy: &policy,
            files: &files,
            verify: &verify,
        },
    )
}

/// Starts the full file check, unless one runs.
pub fn start_verify(state: &std::sync::Arc<std::sync::Mutex<VerifyState>>) {
    let mut guard = state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if guard.answer.running {
        return;
    }
    guard.answer.running = true;
    let shared = state.clone();
    tokio::spawn(async move {
        let started = Instant::now();
        // Lowest CPU and disk priority, so the computer stays responsive.
        let output = command::output_any(
            "nice",
            &["-n", "19", "ionice", "-c", "3", "dpkg", "--verify"],
            VERIFY_LIMIT,
        )
        .await;
        let mut problems = output.as_deref().map(parse_verify);
        if let Some(list) = problems.as_mut().filter(|l| !l.is_empty()) {
            let files: Vec<&str> = list.iter().take(500).map(|p| p.file.as_str()).collect();
            let mut args = vec!["-S"];
            args.extend(&files);
            let owners = command::output_any("dpkg", &args, LIMIT)
                .await
                .unwrap_or_default();
            for p in list.iter_mut() {
                p.package = owners.lines().find_map(|l| {
                    l.split_once(": ")
                        .filter(|(_, f)| f.trim() == p.file)
                        .map(|(pkg, _)| pkg.split(':').next().unwrap_or(pkg).to_owned())
                });
            }
        }
        let mut guard = shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.answer = VerifyAnswer {
            running: false,
            finished_at: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
            ),
            took_s: Some(started.elapsed().as_secs()),
            failed: output
                .is_none()
                .then(|| "dpkg --verify did not finish (time limit or error)".to_owned()),
            problems,
        };
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn snaps_and_runtimes() {
        let s = parse_snaps(
            "Name    Version   Rev   Tracking       Publisher     Notes\ncore18  20260204  2999  latest/stable  canonical**   base,disabled\nfirefox 131.0     5000  latest/stable  mozilla✓      -\n",
        );
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].notes, ["base", "disabled"]);
        assert_eq!(s[1].publisher.as_deref(), Some("mozilla"));
        assert!(s[1].notes.is_empty());
        let mut r = vec![
            Runtime {
                id: "org.gnome.Platform".into(),
                branch: "46".into(),
                installation: "system".into(),
                used: false,
            },
            Runtime {
                id: "org.gnome.Platform.Locale".into(),
                branch: "46".into(),
                installation: "system".into(),
                used: false,
            },
            Runtime {
                id: "org.freedesktop.Platform.GL.default".into(),
                branch: "24.08".into(),
                installation: "system".into(),
                used: false,
            },
            Runtime {
                id: "org.gnome.Platform".into(),
                branch: "44".into(),
                installation: "system".into(),
                used: false,
            },
        ];
        mark_runtimes(
            &mut r,
            &["org.gnome.Platform/x86_64/46".into()],
            &[
                "org.gnome.Platform.Locale".into(),
                "org.freedesktop.Platform.GL".into(),
            ],
        );
        assert_eq!(
            r.iter().map(|x| x.used).collect::<Vec<_>>(),
            [true, true, true, false],
            "the GL driver fills an extension point; the old branch is unused"
        );
        assert_eq!(
            extension_points(
                "[Application]\nruntime=x\n[Extension org.freedesktop.Platform.GL]\nversions=24.08\n"
            ),
            ["org.freedesktop.Platform.GL"]
        );
    }

    #[test]
    fn dpkg_and_apt_parsers() {
        assert_eq!(
            depends_names("libc6 (>= 2.34), libfoo | libbar:any, python3:any"),
            ["libc6", "libfoo", "libbar", "python3"]
        );
        let Policy {
            installed: i,
            candidate: c,
            sources: s,
        } = parse_policy(
            "htop:\n  Installed: 3.3.0-4build1\n  Candidate: 3.3.0-4build1\n  Version table:\n *** 3.3.0-4build1 500\n        500 http://archive.ubuntu.com/ubuntu noble/main amd64 Packages\n        100 /var/lib/dpkg/status\n",
        );
        assert_eq!(
            (i.as_deref(), c.as_deref()),
            (Some("3.3.0-4build1"), Some("3.3.0-4build1"))
        );
        assert_eq!(s[0].0, "3.3.0-4build1");
        assert_eq!(
            s[0].1,
            [
                "http://archive.ubuntu.com/ubuntu noble/main amd64 Packages",
                "/var/lib/dpkg/status"
            ]
        );
        let v = parse_verify(
            "??5?????? c /etc/sysctl.d/10-ptrace.conf\nmissing     /usr/share/doc/x/README\n??5??????   /usr/bin/my tool\n????????? c /etc/ok\n?????????   /boot/vmlinuz-7.0.0-38-generic\nmissing     /var/lib/polkit-1/localauthority (Permission denied)\n",
        );
        assert_eq!(
            v.iter()
                .map(|p| (p.file.as_str(), p.problem, p.config))
                .collect::<Vec<_>>(),
            [
                ("/etc/sysctl.d/10-ptrace.conf", "changed", true),
                ("/usr/share/doc/x/README", "missing", false),
                ("/usr/bin/my tool", "changed", false),
                ("/etc/ok", "unreadable", true),
                ("/boot/vmlinuz-7.0.0-38-generic", "unreadable", false),
                ("/var/lib/polkit-1/localauthority", "unreadable", false),
            ]
        );
    }

    #[test]
    fn package_detail_from_dpkg_query() {
        let answers = |status: &'static str| PackageAnswers {
            status,
            listing: "htop\tii \t, libc6 (>= 2.34), libncursesw6\ntool\tii \t, htop\ngone\trc \t, htop\n",
            extended: "Package: htop\nArchitecture: amd64\nAuto-Installed: 1\n",
            policy: "",
            files: "/.\n/usr\n/usr/bin/htop\n",
            verify: "",
        };
        let d = assemble("htop", &answers("Package: htop\nStatus: install ok installed\nVersion: 3.3.0\nDepends: libc6 (>= 2.34), libncursesw6\nDescription: process viewer\n longer text\n")).unwrap();
        assert_eq!(d.needs, ["libc6", "libncursesw6"]);
        assert_eq!(d.needed_by, ["tool"], "removed packages do not count");
        assert!(!d.manual);
        assert_eq!(d.files, 2);
        assert_eq!(
            d.fields.get("Description").map(String::as_str),
            Some("process viewer")
        );
        assert!(
            assemble(
                "htop",
                &answers("dpkg-query: package 'htop' is not installed\n")
            )
            .is_err()
        );
        assert!(
            assemble(
                "htop",
                &answers("Package: htop\nStatus: deinstall ok config-files\n")
            )
            .is_err()
        );
    }

    #[tokio::test]
    async fn only_package_names_are_accepted() {
        let root = fixture_root("package-detail", &[]);
        assert!(package_detail(&root, "rm -rf").await.is_err());
        assert!(package_detail(&root, "").await.is_err());
    }
}
