// SPDX-License-Identifier: Apache-2.0

//! What each admin action runs, after checking its inputs. Pure: the live system is
//! asked through [`Probe`], so every check is tested without root.
//!
//! Review rules (docs/02-architecture/security-model.md): no shell, fixed programs with
//! fixed arguments, every input checked against a strict pattern and the live system.

use deecure_core::fix::Call;

/// One program to run, with its arguments. Never a shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// An absolute path.
    pub program: &'static str,
    /// Its arguments, each one checked before it gets here.
    pub args: Vec<String>,
}

fn cmd(program: &'static str, args: &[&str]) -> Command {
    Command {
        program,
        args: args.iter().map(|a| (*a).to_owned()).collect(),
    }
}

/// What the helper may ask the live system before acting.
pub trait Probe {
    /// `systemctl show -p LoadState --value <unit>` is `loaded`.
    fn unit_loaded(&self, unit: &str) -> bool;
    /// The dpkg state of a package: `ii` installed, `rc` only settings left; `None` if unknown.
    fn package_state(&self, name: &str) -> Option<String>;
    /// `(name, revision)` of disabled snap revisions.
    fn disabled_snaps(&self) -> Vec<(String, String)>;
    /// The running kernel release.
    fn running_kernel(&self) -> String;
    /// Installed kernel versions, newest first.
    fn installed_kernels(&self) -> Vec<String>;
    /// True when the program exists (such as `/usr/sbin/ufw`).
    fn has(&self, program: &str) -> bool;
}

const APT: &str = "/usr/bin/apt-get";
const SYSTEMCTL: &str = "/usr/bin/systemctl";

fn is_unit(s: &str) -> bool {
    s.len() <= 200
        && s.ends_with(".service")
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "@._:-".contains(c))
        && !s.starts_with('-')
}

/// Debian package names: lowercase letters, digits, `+ - .`, at least two characters.
pub fn is_package(s: &str) -> bool {
    s.len() >= 2
        && s.len() <= 100
        && s.chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "+-.".contains(c))
}

fn is_snap(s: &str) -> bool {
    (1..=40).contains(&s.len())
        && s.chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn is_kernel(s: &str) -> bool {
    // 6.8.0-41-generic
    let mut parts = s.splitn(3, '-');
    let (Some(v), Some(abi), Some(flavor)) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    v.split('.').count() == 3
        && v.split('.')
            .all(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
        && !abi.is_empty()
        && abi.chars().all(|c| c.is_ascii_digit())
        && !flavor.is_empty()
        && flavor
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

const MAX_PACKAGES: usize = 500;

/// Checks a call and returns the commands that carry it out, in order.
pub fn prepare(call: &Call, sys: &dyn Probe) -> Result<Vec<Command>, String> {
    match call {
        Call::CleanAptCache => Ok(vec![cmd(APT, &["clean"])]),
        Call::Autoremove => Ok(vec![cmd(APT, &["-y", "autoremove"])]),
        Call::VacuumJournal { keep_mb } => {
            if !(50..=10_000).contains(keep_mb) {
                return Err("keep between 50 and 10000 MB of logs".into());
            }
            Ok(vec![Command {
                program: "/usr/bin/journalctl",
                args: vec![format!("--vacuum-size={keep_mb}M")],
            }])
        }
        Call::RemoveSnapRevision { name, revision } => {
            if !is_snap(name)
                || revision.is_empty()
                || !revision
                    .trim_start_matches('x')
                    .chars()
                    .all(|c| c.is_ascii_digit())
            {
                return Err("not a snap name and revision".into());
            }
            if !sys
                .disabled_snaps()
                .iter()
                .any(|(n, r)| n == name && r == revision)
            {
                return Err(format!(
                    "{name} revision {revision} is not a disabled revision"
                ));
            }
            Ok(vec![Command {
                program: "/usr/bin/snap",
                args: vec![
                    "remove".into(),
                    name.clone(),
                    format!("--revision={revision}"),
                ],
            }])
        }
        Call::PurgeResidual { packages } => {
            check_packages(packages, sys, "rc", "has no leftover settings")?;
            let mut args = vec!["-y".to_owned(), "purge".to_owned(), "--".to_owned()];
            args.extend(packages.iter().cloned());
            Ok(vec![Command { program: APT, args }])
        }
        Call::UpgradePackages { packages } => {
            check_packages(packages, sys, "ii", "is not installed")?;
            let mut args = vec![
                "-y".to_owned(),
                "install".to_owned(),
                "--only-upgrade".to_owned(),
                "--".to_owned(),
            ];
            args.extend(packages.iter().cloned());
            Ok(vec![Command { program: APT, args }])
        }
        Call::SetFirewall { on } => {
            if !sys.has("/usr/sbin/ufw") {
                return Err("ufw is not installed".into());
            }
            Ok(vec![if *on {
                cmd("/usr/sbin/ufw", &["--force", "enable"])
            } else {
                cmd("/usr/sbin/ufw", &["disable"])
            }])
        }
        Call::RestartService { unit } => {
            check_unit(unit, sys)?;
            Ok(vec![Command {
                program: SYSTEMCTL,
                args: vec!["restart".into(), "--".into(), unit.clone()],
            }])
        }
        Call::SetServiceEnabled { unit, enabled } => {
            check_unit(unit, sys)?;
            // Never switch off what the system needs to start or to sign in.
            const ESSENTIAL: &[&str] = &[
                "dbus.service",
                "systemd-logind.service",
                "systemd-journald.service",
                "systemd-udevd.service",
                "gdm.service",
                "polkit.service",
                "NetworkManager.service",
            ];
            if !enabled && ESSENTIAL.contains(&unit.as_str()) {
                return Err(format!(
                    "{unit} is essential and cannot be turned off from DeeCure"
                ));
            }
            let verb = if *enabled { "enable" } else { "disable" };
            Ok(vec![Command {
                program: SYSTEMCTL,
                args: vec![verb.into(), "--now".into(), "--".into(), unit.clone()],
            }])
        }
        Call::RemoveKernels { versions } => {
            if versions.is_empty() || versions.len() > 20 || !versions.iter().all(|v| is_kernel(v))
            {
                return Err("not a list of kernel versions".into());
            }
            let running = sys.running_kernel();
            let installed = sys.installed_kernels();
            let newest = installed.first().cloned().unwrap_or_default();
            for v in versions {
                if *v == running || *v == newest {
                    return Err(format!("{v} is the running or newest kernel and is kept"));
                }
                if !installed.contains(v) {
                    return Err(format!("{v} is not installed"));
                }
            }
            if installed.len().saturating_sub(versions.len()) < 2 {
                return Err("at least two kernels are always kept".into());
            }
            let mut args = vec!["-y".to_owned(), "purge".to_owned(), "--".to_owned()];
            for v in versions {
                for prefix in [
                    "linux-image",
                    "linux-image-unsigned",
                    "linux-modules",
                    "linux-modules-extra",
                    "linux-headers",
                ] {
                    let pkg = format!("{prefix}-{v}");
                    if sys.package_state(&pkg).as_deref() == Some("ii") {
                        args.push(pkg);
                    }
                }
            }
            Ok(vec![Command { program: APT, args }])
        }
        Call::SetBatteryAction { action, percent } => {
            if !["PowerOff", "Hibernate", "HybridSleep"].contains(&action.as_str())
                || !(1..=20).contains(percent)
            {
                return Err(
                    "the action must be PowerOff, Hibernate, or HybridSleep, at 1-20%".into(),
                );
            }
            // The file itself is edited by the helper, not by a program; then UPower reloads it.
            Ok(vec![cmd(SYSTEMCTL, &["try-restart", "upower.service"])])
        }
        other => Err(format!("{other:?} is not an admin action")),
    }
}

fn check_unit(unit: &str, sys: &dyn Probe) -> Result<(), String> {
    if !is_unit(unit) {
        return Err("not a service name".into());
    }
    if !sys.unit_loaded(unit) {
        return Err(format!("{unit} does not exist"));
    }
    Ok(())
}

fn check_packages(
    packages: &[String],
    sys: &dyn Probe,
    state: &str,
    problem: &str,
) -> Result<(), String> {
    if packages.is_empty() || packages.len() > MAX_PACKAGES {
        return Err("give between 1 and 500 packages".into());
    }
    for p in packages {
        if !is_package(p) {
            return Err(format!("{p:?} is not a package name"));
        }
        if sys.package_state(p).as_deref() != Some(state) {
            return Err(format!("{p} {problem}"));
        }
    }
    Ok(())
}

/// Sets `CriticalPowerAction` and `PercentageAction` in UPower.conf text, keeping every
/// other line. Returns the new text and the old values (for undo).
pub fn edit_upower(
    text: &str,
    action: &str,
    percent: u32,
) -> (String, Option<String>, Option<u32>) {
    let mut old_action = None;
    let mut old_percent = None;
    let mut seen = (false, false);
    let mut out: Vec<String> = text
        .lines()
        .map(|l| {
            if let Some(v) = l.trim().strip_prefix("CriticalPowerAction=") {
                old_action = Some(v.trim().to_owned());
                seen.0 = true;
                format!("CriticalPowerAction={action}")
            } else if let Some(v) = l.trim().strip_prefix("PercentageAction=") {
                old_percent = v.trim().parse().ok();
                seen.1 = true;
                format!("PercentageAction={percent}")
            } else {
                l.to_owned()
            }
        })
        .collect();
    if !seen.0 || !seen.1 {
        if !out.iter().any(|l| l.trim() == "[UPower]") {
            out.push("[UPower]".into());
        }
        if !seen.1 {
            out.push(format!("PercentageAction={percent}"));
        }
        if !seen.0 {
            out.push(format!("CriticalPowerAction={action}"));
        }
    }
    (out.join("\n") + "\n", old_action, old_percent)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake;
    impl Probe for Fake {
        fn unit_loaded(&self, unit: &str) -> bool {
            ["cups.service", "dbus.service", "bluetooth.service"].contains(&unit)
        }
        fn package_state(&self, name: &str) -> Option<String> {
            match name {
                "htop" | "linux-image-6.8.0-41-generic" | "linux-modules-6.8.0-41-generic" => {
                    Some("ii".into())
                }
                "oldapp" => Some("rc".into()),
                _ => None,
            }
        }
        fn disabled_snaps(&self) -> Vec<(String, String)> {
            vec![("firefox".into(), "4793".into())]
        }
        fn running_kernel(&self) -> String {
            "6.8.0-47-generic".into()
        }
        fn installed_kernels(&self) -> Vec<String> {
            vec![
                "6.8.0-47-generic".into(),
                "6.8.0-45-generic".into(),
                "6.8.0-41-generic".into(),
            ]
        }
        fn has(&self, _: &str) -> bool {
            true
        }
    }

    fn args(call: Call) -> Result<Vec<String>, String> {
        prepare(&call, &Fake).map(|c| {
            c.into_iter()
                .flat_map(|c| std::iter::once(c.program.to_owned()).chain(c.args))
                .collect()
        })
    }

    #[test]
    fn builds_fixed_commands() {
        assert_eq!(
            args(Call::CleanAptCache).unwrap(),
            ["/usr/bin/apt-get", "clean"]
        );
        assert_eq!(
            args(Call::RestartService {
                unit: "cups.service".into()
            })
            .unwrap(),
            ["/usr/bin/systemctl", "restart", "--", "cups.service"]
        );
        assert_eq!(
            args(Call::RemoveSnapRevision {
                name: "firefox".into(),
                revision: "4793".into()
            })
            .unwrap(),
            ["/usr/bin/snap", "remove", "firefox", "--revision=4793"]
        );
        assert_eq!(
            args(Call::PurgeResidual {
                packages: vec!["oldapp".into()]
            })
            .unwrap(),
            ["/usr/bin/apt-get", "-y", "purge", "--", "oldapp"]
        );
        assert_eq!(
            args(Call::RemoveKernels {
                versions: vec!["6.8.0-41-generic".into()]
            })
            .unwrap(),
            [
                "/usr/bin/apt-get",
                "-y",
                "purge",
                "--",
                "linux-image-6.8.0-41-generic",
                "linux-modules-6.8.0-41-generic"
            ]
        );
    }

    #[test]
    fn rejects_bad_or_dangerous_inputs() {
        for bad in [
            Call::RestartService {
                unit: "-rf.service".into(),
            },
            Call::RestartService {
                unit: "cups.service; reboot".into(),
            },
            Call::RestartService {
                unit: "missing.service".into(),
            },
            Call::SetServiceEnabled {
                unit: "dbus.service".into(),
                enabled: false,
            },
            Call::RemoveSnapRevision {
                name: "firefox".into(),
                revision: "4794".into(),
            },
            Call::RemoveSnapRevision {
                name: "../x".into(),
                revision: "1".into(),
            },
            Call::PurgeResidual {
                packages: vec!["htop".into()],
            },
            Call::PurgeResidual {
                packages: vec!["-o".into()],
            },
            Call::PurgeResidual { packages: vec![] },
            Call::UpgradePackages {
                packages: vec!["notinstalled".into()],
            },
            Call::RemoveKernels {
                versions: vec!["6.8.0-47-generic".into()],
            },
            Call::RemoveKernels {
                versions: vec!["6.8.0-45-generic; x".into()],
            },
            Call::RemoveKernels {
                versions: vec!["6.8.0-45-generic".into(), "6.8.0-41-generic".into()],
            },
            Call::VacuumJournal { keep_mb: 1 },
            Call::SetBatteryAction {
                action: "Reboot".into(),
                percent: 5,
            },
            Call::SetBatteryAction {
                action: "PowerOff".into(),
                percent: 90,
            },
            Call::EmptyTrash,
        ] {
            assert!(prepare(&bad, &Fake).is_err(), "{bad:?} should be refused");
        }
    }

    #[test]
    fn edits_upower_and_remembers_old_values() {
        let (text, action, percent) = edit_upower(
            "[UPower]\n# comment\nPercentageAction=2\nCriticalPowerAction=HybridSleep\n",
            "PowerOff",
            5,
        );
        assert_eq!(
            text,
            "[UPower]\n# comment\nPercentageAction=5\nCriticalPowerAction=PowerOff\n"
        );
        assert_eq!((action.as_deref(), percent), (Some("HybridSleep"), Some(2)));
        let (text, action, _) = edit_upower("[UPower]\n", "Hibernate", 4);
        assert!(
            text.contains("CriticalPowerAction=Hibernate") && text.contains("PercentageAction=4")
        );
        assert_eq!(action, None);
    }
}

/// Randomized tests: thousands of random and hostile inputs (shell characters, option
/// tricks such as `--root=/`, very long and non-ASCII text) against every check, with a
/// fake system that says yes to everything, so only the input checks stand in the way.
/// No extra dependencies: a small fixed-seed generator makes every run the same.
#[cfg(test)]
mod randomized {
    use super::*;

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            // xorshift64
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
        /// A string built from pieces that often break programs.
        fn hostile(&mut self) -> String {
            const PIECES: &[&str] = &[
                "a", "z", "0", "9", "-", "--", ".", "..", "/", "@", ":", "_", "+", " ", "\t", "\n",
                ";", "|", "&", "$", "`", "$(id)", "'", "\"", "\\", "*", "?", "<", ">", "\0", "é",
                "ß", "日", "\u{202e}", "service", ".service", "linux", "6.8.0", "-41", "-generic",
                "--root=/", "-rf", "x1", "nvme0n1", "sda", "mmcblk0", "A", "Z",
            ];
            let parts = self.below(12);
            let mut s = String::new();
            for _ in 0..parts {
                s.push_str(PIECES[self.below(PIECES.len() as u64) as usize]);
            }
            if self.below(50) == 0 {
                s = s.repeat(40); // very long
            }
            // Half the time: a valid name, unchanged or with one hostile piece inserted,
            // so the tests hit the edge between accepted and refused.
            if self.below(2) == 0 {
                const VALID: &[&str] = &[
                    "cups.service",
                    "dbus.service",
                    "htop",
                    "libc6",
                    "g++",
                    "firefox",
                    "core22",
                    "4793",
                    "6.8.0-41-generic",
                    "6.8.0-45-lowlatency",
                ];
                let mut base = VALID[self.below(VALID.len() as u64) as usize].to_owned();
                if self.below(2) == 0 {
                    let at = (self.below(base.len() as u64 + 1)) as usize;
                    if base.is_char_boundary(at) {
                        base.insert_str(at, &s);
                    }
                }
                return base;
            }
            s
        }
    }

    /// A fake system that confirms whatever it is asked about.
    struct Yes {
        snap: (String, String),
        kernels: Vec<String>,
    }
    impl Probe for Yes {
        fn unit_loaded(&self, _: &str) -> bool {
            true
        }
        fn package_state(&self, name: &str) -> Option<String> {
            Some(
                if name.len().is_multiple_of(2) {
                    "ii"
                } else {
                    "rc"
                }
                .into(),
            )
        }
        fn disabled_snaps(&self) -> Vec<(String, String)> {
            vec![self.snap.clone()]
        }
        fn running_kernel(&self) -> String {
            "6.8.0-99-generic".into()
        }
        fn installed_kernels(&self) -> Vec<String> {
            self.kernels.clone()
        }
        fn has(&self, _: &str) -> bool {
            true
        }
    }

    /// Characters an accepted input may contain: nothing a shell or an option parser
    /// could misread.
    fn safe(s: &str) -> bool {
        !s.is_empty()
            && !s.starts_with('-')
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || "@._:+-".contains(c))
    }

    #[test]
    fn accepted_inputs_contain_only_safe_characters() {
        let mut rng = Rng(0x00de_ec0e_5eed);
        for _ in 0..50_000 {
            let s = rng.hostile();
            for (name, ok) in [
                ("unit", is_unit(&s)),
                ("package", is_package(&s)),
                ("snap", is_snap(&s)),
                ("kernel", is_kernel(&s)),
                ("disk", crate::reads::is_disk(&s)),
            ] {
                if ok {
                    assert!(safe(&s), "{name} check accepted {s:?}");
                    assert!(s.len() <= 200, "{name} check accepted {} bytes", s.len());
                }
            }
        }
    }

    #[test]
    fn hostile_input_never_becomes_an_option_or_a_shell_character() {
        let mut rng = Rng(0x0bad_c0de_1234);
        let mut built_from_input = 0;
        for round in 0..20_000 {
            let a = rng.hostile();
            let b = rng.hostile();
            let list: Vec<String> = (0..rng.below(4)).map(|_| rng.hostile()).collect();
            let sys = Yes {
                snap: (a.clone(), b.clone()),
                kernels: list.clone(),
            };
            // Each call, with the inputs it really uses.
            let (call, used): (Call, Vec<String>) = match round % 8 {
                0 => (
                    Call::RemoveSnapRevision {
                        name: a.clone(),
                        revision: b.clone(),
                    },
                    vec![a.clone(), b.clone()],
                ),
                1 => (
                    Call::PurgeResidual {
                        packages: list.clone(),
                    },
                    list.clone(),
                ),
                2 => (
                    Call::UpgradePackages {
                        packages: list.clone(),
                    },
                    list.clone(),
                ),
                3 => (Call::RestartService { unit: a.clone() }, vec![a.clone()]),
                4 => (
                    Call::SetServiceEnabled {
                        unit: a.clone(),
                        enabled: round % 3 == 0,
                    },
                    vec![a.clone()],
                ),
                5 => (
                    Call::RemoveKernels {
                        versions: list.clone(),
                    },
                    list.clone(),
                ),
                6 => (
                    Call::SetBatteryAction {
                        action: a.clone(),
                        percent: rng.below(40) as u32,
                    },
                    vec![a.clone()],
                ),
                _ => (
                    Call::VacuumJournal {
                        keep_mb: rng.below(20_000) as u32,
                    },
                    vec![],
                ),
            };
            // It may refuse, but it must never panic.
            let Ok(commands) = prepare(&call, &sys) else {
                continue;
            };
            if !used.is_empty() {
                built_from_input += 1;
            }
            for c in &commands {
                assert!(
                    c.program.starts_with("/usr/"),
                    "{round}: program {}",
                    c.program
                );
                for arg in &c.args {
                    assert!(
                        !arg.contains('\0') && !arg.chars().any(char::is_whitespace),
                        "{round}: {arg:?}"
                    );
                    // An argument that is one of this call's inputs passed the strict checks.
                    if used.iter().any(|input| input == arg) {
                        assert!(
                            safe(arg),
                            "{round}: input {arg:?} reached {} unchecked",
                            c.program
                        );
                    }
                }
            }
        }
        // The generator must also make inputs that pass, or the test proves nothing.
        assert!(
            built_from_input > 100,
            "only {built_from_input} calls with input were accepted"
        );
    }
}
