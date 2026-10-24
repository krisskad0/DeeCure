// SPDX-License-Identifier: Apache-2.0

//! Installing DeeCure's own updates (docs/02-architecture/updates.md), as `install.sh` does:
//!
//! 1. Only from DeeCure's own GitHub release page, built from a checked version number; never
//!    a URL from outside.
//! 2. `SHA256SUMS` first; each package must be listed there and match its checksum.
//! 3. Each package's own name, version, and architecture must be the ones expected.
//! 4. Only a newer version than the one installed.
//! 5. apt installs them (and the Ubuntu libraries they need).
//!
//! The checksums come from the same release as the packages, so they prove the download is
//! whole, not who made it: the same trust as the one-command install, until releases are
//! signed (decision #12). Automatic updates (`--auto-update`, run by `deecure-update.timer`)
//! do the same, only when the user turned them on, and not on a metered connection.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use deecure_core::fix::Call;
use deecure_core::update::{self, Channel, CheckEvery};
use serde::{Deserialize, Serialize};

use crate::actions::Command;

/// The automatic update settings, under the system root.
pub const AUTO_FILE: &str = "etc/deecure/update.json";
/// What the last automatic update did.
pub const RESULT_FILE: &str = "run/deecure/update-result.json";
/// When the last automatic attempt ran.
const STAMP_FILE: &str = "var/lib/deecure/update-stamp";
/// Where packages are downloaded; apt reads them as its own user, so it is world-readable.
const CACHE: &str = "/var/cache/deecure/update";
const PACKAGES: [&str; 2] = ["deecure", "deecure-desktop"];

/// The root timer's settings.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AutoSettings {
    pub auto: bool,
    pub check: CheckEvery,
    pub channel: Channel,
}

/// A file name GitHub may use for a package: letters, digits, and `._+~-`, never a path.
fn safe_name(n: &str) -> bool {
    !n.is_empty()
        && n.len() <= 120
        && !n.starts_with('.')
        && n.chars()
            .all(|c| c.is_ascii_alphanumeric() || "._+~-".contains(c))
}

/// `(checksum, file name)` from a `SHA256SUMS` file; bad lines are left out.
pub fn parse_sums(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|l| {
            let (hash, name) = l.split_once(char::is_whitespace)?;
            let name = name.trim().trim_start_matches('*');
            (hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()) && safe_name(name))
                .then(|| (hash.to_ascii_lowercase(), name.to_owned()))
        })
        .collect()
}

/// The package file for `package` in the checksums: `<package>_<version>_<arch>.deb`, where
/// GitHub may have written `~` as `.`.
pub fn file_for(
    sums: &[(String, String)],
    package: &str,
    version: &str,
    arch: &str,
) -> Option<(String, String)> {
    let deb = update::deb_version(version);
    let want = [
        format!("{package}_{deb}_{arch}.deb"),
        format!("{package}_{}_{arch}.deb", deb.replace('~', ".")),
    ];
    sums.iter().find(|(_, n)| want.contains(n)).cloned()
}

/// The package's own fields (`dpkg-deb -f`) must be the ones expected.
pub fn fields_match(fields: &str, package: &str, version: &str, arch: &str) -> Result<(), String> {
    let get = |k: &str| {
        fields.lines().find_map(|l| {
            l.strip_prefix(k)?
                .strip_prefix(':')
                .map(|v| v.trim().to_owned())
        })
    };
    let deb = update::deb_version(version);
    for (k, want) in [
        ("Package", package),
        ("Version", deb.as_str()),
        ("Architecture", arch),
    ] {
        let have = get(k).unwrap_or_default();
        if have != want {
            return Err(format!(
                "the downloaded {package} says {k} {have}, not {want}"
            ));
        }
    }
    Ok(())
}

fn run(program: &str, args: &[&str]) -> Result<String, String> {
    let out = std::process::Command::new(program)
        .args(args)
        .env_clear()
        .env("PATH", crate::PATH)
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("could not run {program}: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(format!(
            "{program} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// Downloads one file over HTTPS only, with curl or wget.
fn download(url: &str, to: &Path) -> Result<(), String> {
    let to = to.to_string_lossy();
    if Path::new("/usr/bin/curl").exists() {
        run(
            "/usr/bin/curl",
            &[
                "-fsSL",
                "--proto",
                "=https",
                "--tlsv1.2",
                "--max-time",
                "600",
                "--retry",
                "2",
                "-o",
                &to,
                url,
            ],
        )
        .map(|_| ())
    } else if Path::new("/usr/bin/wget").exists() {
        run(
            "/usr/bin/wget",
            &[
                "-q",
                "--https-only",
                "--tries=2",
                "--timeout=60",
                "-O",
                &to,
                url,
            ],
        )
        .map(|_| ())
    } else {
        Err("neither curl nor wget is installed".into())
    }
}

fn installed(package: &str) -> Option<String> {
    let out = run(
        "/usr/bin/dpkg-query",
        &["-W", "-f=${db:Status-Abbrev} ${Version}", "--", package],
    )
    .ok()?;
    let (state, version) = out.trim().split_once(' ')?;
    state.starts_with("ii").then(|| version.to_owned())
}

/// Downloads, checks, and installs a release. Returns what it did.
pub fn install(version: &str) -> Result<String, String> {
    if !update::valid_version(version) {
        return Err("not a DeeCure version".into());
    }
    let deb = update::deb_version(version);
    let current = installed("deecure").ok_or("DeeCure is not installed as a package here")?;
    if run(
        "/usr/bin/dpkg",
        &["--compare-versions", &deb, "gt", &current],
    )
    .is_err()
    {
        return Err(format!(
            "{version} is not newer than the installed {current}"
        ));
    }
    let arch = run("/usr/bin/dpkg", &["--print-architecture"])?
        .trim()
        .to_owned();
    let dir = PathBuf::from(CACHE);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("could not create {CACHE}: {e}"))?;
    let result = (|| {
        set_mode(&dir, 0o755)?;
        let base = format!(
            "https://github.com/{}/releases/download/v{version}",
            update::REPO
        );
        let sums_path = dir.join("SHA256SUMS");
        download(&format!("{base}/SHA256SUMS"), &sums_path)?;
        let sums = parse_sums(&std::fs::read_to_string(&sums_path).map_err(|e| e.to_string())?);
        let mut files = Vec::new();
        for package in PACKAGES {
            // The desktop app only where it is installed (a server has the agent alone).
            if package != "deecure" && installed(package).is_none() {
                continue;
            }
            let (hash, name) = file_for(&sums, package, version, &arch).ok_or(format!(
                "release {version} has no {package} package for {arch}"
            ))?;
            let path = dir.join(&name);
            download(&format!("{base}/{name}"), &path)?;
            let actual = run("/usr/bin/sha256sum", &[&path.to_string_lossy()])?;
            if actual.split_whitespace().next() != Some(hash.as_str()) {
                return Err(format!(
                    "{name} does not match its checksum; nothing was installed"
                ));
            }
            let fields = run(
                "/usr/bin/dpkg-deb",
                &[
                    "-f",
                    &path.to_string_lossy(),
                    "Package",
                    "Version",
                    "Architecture",
                ],
            )?;
            fields_match(&fields, package, version, &arch)?;
            set_mode(&path, 0o644)?;
            files.push(path.to_string_lossy().into_owned());
        }
        let mut args = vec![
            "-y".to_owned(),
            "--no-install-recommends".to_owned(),
            "install".to_owned(),
            "--".to_owned(),
        ];
        args.extend(files);
        crate::run_all(&[Command {
            program: "/usr/bin/apt-get",
            args,
        }])?;
        Ok(format!("DeeCure {version} is installed (it was {current})"))
    })();
    let _ = std::fs::remove_dir_all(&dir);
    result
}

fn set_mode(path: &Path, mode: u32) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).map_err(|e| e.to_string())
}

/// Writes or removes the automatic update settings. Returns the call that undoes it.
pub fn set_auto(
    root: &Path,
    on: bool,
    check: CheckEvery,
    channel: Channel,
) -> Result<(String, Call), String> {
    let path = root.join(AUTO_FILE);
    let before: AutoSettings = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    let undo = Call::SetAutoUpdate {
        on: before.auto,
        check: before.check,
        channel: before.channel,
    };
    if on {
        if check == CheckEvery::Off {
            return Err("automatic updates need a daily or weekly check".into());
        }
        let text = serde_json::to_string_pretty(&AutoSettings {
            auto: true,
            check,
            channel,
        })
        .map_err(|e| e.to_string())?;
        crate::tuning::write(&path, Some(&text))?;
        Ok(("automatic updates are on".into(), undo))
    } else {
        crate::tuning::write(&path, None)?;
        Ok(("automatic updates are off".into(), undo))
    }
}

/// Is an automatic attempt due, by the chosen period and the last attempt?
pub fn auto_due(check: CheckEvery, last_ms: Option<i64>, now_ms: i64) -> bool {
    match check.period_ms() {
        None => false,
        Some(p) => last_ms.is_none_or(|t| now_ms - t >= p),
    }
}

fn metered() -> bool {
    run(
        "/usr/bin/nmcli",
        &["-t", "-f", "GENERAL.METERED", "device", "show"],
    )
    .is_ok_and(|o| {
        o.lines()
            .any(|l| l.ends_with(":yes") || l.ends_with(":yes (guessed)"))
    })
}

/// One automatic run (deecure-update.service): returns the message for the result file.
pub fn auto_run(root: &Path, now_ms: i64) -> Result<Option<String>, String> {
    let path = root.join(AUTO_FILE);
    crate::saved::trusted(&path)?;
    let settings: AutoSettings =
        serde_json::from_str(&std::fs::read_to_string(&path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("{AUTO_FILE}: {e}"))?;
    if !settings.auto {
        return Ok(None);
    }
    let stamp = root.join(STAMP_FILE);
    let last = std::fs::read_to_string(&stamp)
        .ok()
        .and_then(|t| t.trim().parse().ok());
    if !auto_due(settings.check, last, now_ms) {
        return Ok(None);
    }
    if metered() {
        return Ok(Some("waiting: the connection is metered".into()));
    }
    crate::tuning::write(&stamp, Some(&now_ms.to_string()))?;
    let list_path = PathBuf::from("/var/cache/deecure/releases.json");
    if let Some(dir) = list_path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    download(
        &format!(
            "https://api.github.com/repos/{}/releases?per_page=20",
            update::REPO
        ),
        &list_path,
    )?;
    let list: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&list_path).map_err(|e| e.to_string())?)
            .map_err(|_| "GitHub's answer was not a release list".to_owned())?;
    let _ = std::fs::remove_file(&list_path);
    let Some(latest) = update::newest(&list, settings.channel) else {
        return Ok(Some("no release found".into()));
    };
    let current = installed("deecure").unwrap_or_default();
    if run(
        "/usr/bin/dpkg",
        &[
            "--compare-versions",
            &update::deb_version(&latest.version),
            "gt",
            &current,
        ],
    )
    .is_err()
    {
        return Ok(Some(format!("up to date ({current})")));
    }
    install(&latest.version).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUMS: &str = "3b85692d9f3588fc70c540e7114e403c8f001812de9b74b2a07f1006a5b20ec6  deecure-desktop_0.1.0.alpha.4-1_amd64.deb\n\
                        1e71d02cdabc65ba5ebff310ddde01b64b01ec010bbc961d4e896993d678de38  deecure_0.1.0.alpha.4-1_amd64.deb\n\
                        365c7a6ac6f3545e1b65dc896fe013a420a088b8d37710a5df9a0396bcee2f19  ../../etc/passwd\n\
                        xyz  deecure_bad.deb\n";

    #[test]
    fn only_listed_package_files_with_safe_names_are_used() {
        let sums = parse_sums(SUMS);
        assert_eq!(sums.len(), 2, "a path and a bad checksum are left out");
        let (hash, name) = file_for(&sums, "deecure", "0.1.0-alpha.4", "amd64").unwrap();
        assert_eq!(name, "deecure_0.1.0.alpha.4-1_amd64.deb");
        assert!(hash.starts_with("1e71d02c"));
        assert_eq!(
            file_for(&sums, "deecure-desktop", "0.1.0-alpha.4", "amd64")
                .unwrap()
                .1,
            "deecure-desktop_0.1.0.alpha.4-1_amd64.deb"
        );
        assert!(file_for(&sums, "deecure", "0.1.0-alpha.5", "amd64").is_none());
        assert!(file_for(&sums, "deecure", "0.1.0-alpha.4", "arm64").is_none());
    }

    #[test]
    fn a_package_must_say_what_it_should_be() {
        let ok = "Package: deecure\nVersion: 0.1.0~alpha.4-1\nArchitecture: amd64\n";
        assert!(fields_match(ok, "deecure", "0.1.0-alpha.4", "amd64").is_ok());
        let other = "Package: deecure\nVersion: 0.1.0~alpha.3-1\nArchitecture: amd64\n";
        assert!(
            fields_match(other, "deecure", "0.1.0-alpha.4", "amd64")
                .unwrap_err()
                .contains("Version")
        );
        let renamed = "Package: evil\nVersion: 0.1.0~alpha.4-1\nArchitecture: amd64\n";
        assert!(fields_match(renamed, "deecure", "0.1.0-alpha.4", "amd64").is_err());
    }

    #[test]
    fn automatic_settings_need_a_check_and_follow_their_period() {
        let root =
            std::env::temp_dir().join(format!("deecure-helper-update-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        assert!(set_auto(&root, true, CheckEvery::Off, Channel::Prerelease).is_err());
        let (_, undo) = set_auto(&root, true, CheckEvery::Daily, Channel::Prerelease).unwrap();
        assert_eq!(
            undo,
            Call::SetAutoUpdate {
                on: false,
                check: CheckEvery::Daily,
                channel: Channel::Prerelease
            }
        );
        let saved: AutoSettings =
            serde_json::from_str(&std::fs::read_to_string(root.join(AUTO_FILE)).unwrap()).unwrap();
        assert!(saved.auto);
        set_auto(&root, false, CheckEvery::Daily, Channel::Prerelease).unwrap();
        assert!(!root.join(AUTO_FILE).exists());
        let day = 86_400_000;
        assert!(auto_due(CheckEvery::Daily, None, day));
        assert!(!auto_due(CheckEvery::Daily, Some(day), day + 3_600_000));
        assert!(auto_due(CheckEvery::Weekly, Some(0), 7 * day));
        assert!(!auto_due(CheckEvery::Off, None, day));
    }
}
