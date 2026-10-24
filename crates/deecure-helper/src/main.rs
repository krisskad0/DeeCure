// SPDX-License-Identifier: Apache-2.0

//! The DeeCure admin helper. It runs as root, started by D-Bus only when needed, and
//! offers a fixed list of actions (docs/02-architecture/helper.md):
//!
//! - `Run(call)`: one admin fix step, a JSON [`Call`]. Unknown operations cannot even be read.
//! - `Read(what, arg)`: an admin read-out (SMART, firewall rules, crontabs, account status).
//!
//! Every call: polkit asks the person (each action has its own polkit ID), inputs are
//! checked against strict patterns and the live system, programs run with a fixed argument
//! list and a clean environment (never a shell), and a line goes to the audit log.
//! The helper exits after 2 minutes without calls.

mod actions;
mod power;
mod reads;
mod saved;

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::process::{ExitCode, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use deecure_core::VERSION;
use deecure_core::fix::Call;
use serde_json::{Value, json};
use tokio::sync::Mutex;
use zbus::fdo;
use zbus::zvariant::Value as Variant;

use actions::{Command, Probe};

const BUS_NAME: &str = "io.github.krisskad0.DeeCure.Helper";
const OBJECT: &str = "/io/github/krisskad0/DeeCure/Helper";
const AUDIT_LOG: &str = "/var/log/deecure/helper.log";
const UPOWER_CONF: &str = "/etc/UPower/UPower.conf";
const IDLE: Duration = Duration::from_secs(120);
/// Programs see only this PATH, so a changed environment cannot swap them.
const PATH: &str = "/usr/sbin:/usr/bin:/sbin:/bin";

/// The polkit action for a call: `io.github.krisskad0.deecure.<operation>`.
fn polkit_action(call: &Call) -> String {
    let op = serde_json::to_value(call)
        .ok()
        .and_then(|v| v.get("op")?.as_str().map(str::to_owned))
        .unwrap_or_default();
    format!("io.github.krisskad0.deecure.{}", op.replace('_', "-"))
}

struct State {
    last_call: std::sync::Mutex<Instant>,
    /// One write at a time: apt and systemctl must not race.
    writing: Mutex<()>,
}

struct Helper {
    state: Arc<State>,
}

/// Asks the live system, for input checks.
struct Live;

fn output(program: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new(program)
        .args(args)
        .env_clear()
        .env("PATH", PATH)
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

impl Probe for Live {
    fn unit_loaded(&self, unit: &str) -> bool {
        output(
            "/usr/bin/systemctl",
            &["show", "-p", "LoadState", "--value", "--", unit],
        )
        .is_some_and(|s| s.trim() == "loaded")
    }
    fn package_state(&self, name: &str) -> Option<String> {
        output(
            "/usr/bin/dpkg-query",
            &["-W", "-f=${db:Status-Abbrev}", "--", name],
        )
        .map(|s| s.trim().to_owned())
    }
    fn disabled_snaps(&self) -> Vec<(String, String)> {
        output("/usr/bin/snap", &["list", "--all"])
            .unwrap_or_default()
            .lines()
            .skip(1)
            .filter(|l| {
                l.split_whitespace()
                    .last()
                    .is_some_and(|n| n.split(',').any(|x| x == "disabled"))
            })
            .filter_map(|l| {
                let f: Vec<&str> = l.split_whitespace().collect();
                Some(((*f.first()?).to_owned(), (*f.get(2)?).to_owned()))
            })
            .collect()
    }
    fn running_kernel(&self) -> String {
        std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .unwrap_or_default()
            .trim()
            .to_owned()
    }
    fn installed_kernels(&self) -> Vec<String> {
        let key = |v: &str| {
            v.split(|c: char| !c.is_ascii_digit())
                .filter_map(|n| n.parse::<u64>().ok())
                .collect::<Vec<_>>()
        };
        let mut list: Vec<String> = std::fs::read_dir("/lib/modules")
            .map(|d| {
                d.flatten()
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .filter(|v| Path::new(&format!("/boot/vmlinuz-{v}")).exists())
                    .collect()
            })
            .unwrap_or_default();
        list.sort_by_key(|v| std::cmp::Reverse(key(v)));
        list
    }
    fn has(&self, program: &str) -> bool {
        Path::new(program).exists()
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis())
}

/// One line per call: time, caller, method, inputs, and result. Never secrets.
fn audit(uid: u32, method: &str, input: &Value, ok: bool, message: &str) {
    let line = json!({"time": now_ms(), "uid": uid, "method": method, "input": input, "ok": ok, "message": message});
    let _ = std::fs::create_dir_all("/var/log/deecure");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(AUDIT_LOG)
    {
        let _ = writeln!(f, "{line}");
    }
}

/// Runs the commands in order with a clean environment; stops at the first failure.
fn run_all(commands: &[Command]) -> Result<String, String> {
    let mut last = String::new();
    for c in commands {
        let out = std::process::Command::new(c.program)
            .args(&c.args)
            .env_clear()
            .env("PATH", PATH)
            .env("LC_ALL", "C")
            .env("DEBIAN_FRONTEND", "noninteractive")
            .stdin(Stdio::null())
            .output()
            .map_err(|e| format!("could not run {}: {e}", c.program))?;
        let text = String::from_utf8_lossy(&out.stdout).into_owned()
            + &String::from_utf8_lossy(&out.stderr);
        // The last lines are what a person needs; whole apt logs are too long.
        last = text
            .lines()
            .rev()
            .take(6)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        if !out.status.success() {
            return Err(format!("{} failed ({}): {last}", c.program, out.status));
        }
    }
    Ok(last)
}

/// Replaces a file in one step, so a crash never leaves it half written.
fn write_atomically(path: &str, text: &str) -> Result<(), String> {
    let tmp = format!("{path}.deecure-tmp");
    std::fs::write(&tmp, text)
        .and_then(|()| std::fs::rename(&tmp, path))
        .map_err(|e| format!("could not write {path}: {e}"))
}

impl Helper {
    fn touch(&self) {
        if let Ok(mut t) = self.state.last_call.lock() {
            *t = Instant::now();
        }
    }
}

async fn caller_uid(conn: &zbus::Connection, sender: &str) -> fdo::Result<u32> {
    let reply = conn
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "GetConnectionUnixUser",
            &(sender,),
        )
        .await?;
    Ok(reply.body().deserialize()?)
}

/// Asks polkit whether the caller may do this. polkit shows the password box if needed.
async fn authorize(conn: &zbus::Connection, sender: &str, action: &str) -> fdo::Result<()> {
    let mut details = HashMap::new();
    details.insert("name", Variant::from(sender));
    let subject = ("system-bus-name", details);
    let reply = conn
        .call_method(
            Some("org.freedesktop.PolicyKit1"),
            "/org/freedesktop/PolicyKit1/Authority",
            Some("org.freedesktop.PolicyKit1.Authority"),
            "CheckAuthorization",
            &(subject, action, HashMap::<&str, &str>::new(), 1u32, ""),
        )
        .await?;
    let (authorized, _, _): (bool, bool, HashMap<String, String>) = reply.body().deserialize()?;
    if authorized {
        Ok(())
    } else {
        Err(fdo::Error::AccessDenied("not authorized".into()))
    }
}

#[zbus::interface(name = "io.github.krisskad0.DeeCure.Helper1")]
impl Helper {
    /// Runs one admin fix step. Returns JSON `{"message": …, "undo": Call?}`.
    async fn run(
        &self,
        call: String,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> fdo::Result<String> {
        self.touch();
        let sender = hdr
            .sender()
            .map(|s| s.to_string())
            .ok_or_else(|| fdo::Error::AccessDenied("no sender".into()))?;
        let uid = caller_uid(conn, &sender).await?;
        let input: Value = serde_json::from_str(&call).unwrap_or(Value::Null);
        let parsed: Call = match serde_json::from_str(&call) {
            Ok(c) => c,
            Err(e) => {
                audit(uid, "Run", &input, false, "unknown call");
                return Err(fdo::Error::InvalidArgs(format!("unknown call: {e}")));
            }
        };
        if !parsed.admin() {
            audit(uid, "Run", &input, false, "not an admin call");
            return Err(fdo::Error::InvalidArgs("not an admin call".into()));
        }
        if let Err(e) = authorize(conn, &sender, &polkit_action(&parsed)).await {
            audit(uid, "Run", &input, false, "not authorized");
            return Err(e);
        }
        let _guard = self.state.writing.lock().await;
        let task = parsed.clone();
        let result =
            tokio::task::spawn_blocking(move || -> Result<(String, Option<Call>), String> {
                // Power settings are single writes to checked files in /sys, not programs.
                // They are also saved, so they come back after a restart (saved.rs).
                if let Some(plan) = power::plan(&task, Path::new("/")) {
                    let (writes, undo) = plan?;
                    let message = power::apply(&writes)?;
                    let message = match saved::remember(Path::new("/"), &task, &undo) {
                        Ok(()) => message,
                        Err(e) => format!("{message} (only until the next restart: {e})"),
                    };
                    return Ok((message, Some(undo)));
                }
                let commands = actions::prepare(&task, &Live)?;
                let mut undo = None;
                // The file as it was, put back if anything after the edit fails.
                let mut restore: Option<Option<String>> = None;
                if let Call::SetBatteryAction { action, percent } = &task {
                    let before = std::fs::read_to_string(UPOWER_CONF).ok();
                    let (new, old_action, old_percent) = actions::edit_upower(
                        before.as_deref().unwrap_or("[UPower]\n"),
                        action,
                        *percent,
                    );
                    write_atomically(UPOWER_CONF, &new)?;
                    restore = Some(before);
                    undo = Some(Call::SetBatteryAction {
                        action: old_action.unwrap_or_else(|| "HybridSleep".into()),
                        percent: old_percent.unwrap_or(2),
                    });
                }
                match run_all(&commands) {
                    Ok(message) => Ok((message, undo)),
                    Err(e) => match restore {
                        Some(before) => {
                            let put_back = match before {
                                Some(text) => write_atomically(UPOWER_CONF, &text),
                                None => {
                                    std::fs::remove_file(UPOWER_CONF).map_err(|e| e.to_string())
                                }
                            };
                            match put_back {
                                Ok(()) => Err(format!("{e}; {UPOWER_CONF} was put back as it was")),
                                Err(again) => Err(format!(
                                    "{e}; restoring {UPOWER_CONF} also failed: {again}"
                                )),
                            }
                        }
                        None => Err(e),
                    },
                }
            })
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        self.touch();
        match result {
            Ok((message, undo)) => {
                audit(uid, "Run", &input, true, &message);
                Ok(json!({"message": message, "undo": undo}).to_string())
            }
            Err(message) => {
                audit(uid, "Run", &input, false, &message);
                Err(fdo::Error::Failed(message))
            }
        }
    }

    /// An admin read-out: `smart` (arg: disk such as `nvme0n1`), `firewall_rules`,
    /// `crontabs`, `accounts`, `memory_modules`, or `serial`.
    async fn read(
        &self,
        what: String,
        arg: String,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> fdo::Result<String> {
        self.touch();
        let sender = hdr
            .sender()
            .map(|s| s.to_string())
            .ok_or_else(|| fdo::Error::AccessDenied("no sender".into()))?;
        let uid = caller_uid(conn, &sender).await?;
        let input = json!({"what": what, "arg": arg});
        if ![
            "smart",
            "firewall_rules",
            "crontabs",
            "accounts",
            "memory_modules",
            "serial",
        ]
        .contains(&what.as_str())
            || (what == "smart" && !reads::is_disk(&arg))
        {
            audit(uid, "Read", &input, false, "unknown read-out");
            return Err(fdo::Error::InvalidArgs("unknown read-out".into()));
        }
        authorize(conn, &sender, "io.github.krisskad0.deecure.read")
            .await
            .inspect_err(|_| audit(uid, "Read", &input, false, "not authorized"))?;
        let result = tokio::task::spawn_blocking(move || -> Result<String, String> {
            match what.as_str() {
                "smart" => output("/usr/sbin/smartctl", &["-j", "-a", &format!("/dev/{arg}")])
                    .or_else(|| {
                        // smartctl exits non-zero for some warnings but still prints JSON.
                        std::process::Command::new("/usr/sbin/smartctl")
                            .args(["-j", "-a", &format!("/dev/{arg}")])
                            .env_clear()
                            .env("PATH", PATH)
                            .output()
                            .ok()
                            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
                            .filter(|s| s.trim_start().starts_with('{'))
                    })
                    .ok_or_else(|| "smartctl is not installed (package smartmontools)".into()),
                "firewall_rules" => output("/usr/sbin/ufw", &["status", "verbose"])
                    .ok_or_else(|| "ufw is not installed".into()),
                "crontabs" => serde_json::to_string(&reads::all_crontabs(Path::new(
                    "/var/spool/cron/crontabs",
                )))
                .map_err(|e| e.to_string()),
                "accounts" => {
                    let users = reads::human_users(
                        &std::fs::read_to_string("/etc/passwd").unwrap_or_default(),
                    );
                    serde_json::to_string(&reads::shadow_status(
                        &std::fs::read_to_string("/etc/shadow").unwrap_or_default(),
                        &users,
                    ))
                    .map_err(|e| e.to_string())
                }
                "memory_modules" => serde_json::to_string(&reads::memory_modules(
                    &output("/usr/sbin/dmidecode", &["-t", "memory"]).unwrap_or_default(),
                ))
                .map_err(|e| e.to_string()),
                _ => Ok(std::fs::read_to_string("/sys/class/dmi/id/product_serial")
                    .unwrap_or_default()
                    .trim()
                    .to_owned()),
            }
        })
        .await
        .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        audit(
            uid,
            "Read",
            &input,
            result.is_ok(),
            result.as_ref().map_or_else(|e| e.as_str(), |_| "ok"),
        );
        result.map_err(fdo::Error::Failed)
    }

    /// The helper's version.
    #[zbus(property)]
    fn version(&self) -> String {
        VERSION.to_owned()
    }
}

fn main() -> ExitCode {
    if let Some(arg) = std::env::args().nth(1) {
        if arg == "--version" || arg == "-V" {
            println!("deecure-helper {VERSION}");
            return ExitCode::SUCCESS;
        }
        if arg == "--apply-saved" && std::env::args().len() == 2 {
            return apply_saved();
        }
        eprintln!(
            "deecure-helper: D-Bus starts this program; it takes no arguments except --version \
             (and --apply-saved, which deecure-power.service runs at boot)"
        );
        return ExitCode::from(2);
    }
    if !rustix::process::geteuid().is_root() {
        eprintln!("deecure-helper: must run as root (D-Bus starts it as root)");
        return ExitCode::FAILURE;
    }
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("deecure-helper: {e}");
            return ExitCode::FAILURE;
        }
    };
    match runtime.block_on(serve()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("deecure-helper: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Puts back the kept power settings at boot (deecure-power.service).
fn apply_saved() -> ExitCode {
    if !rustix::process::geteuid().is_root() {
        eprintln!("deecure-helper: must run as root");
        return ExitCode::FAILURE;
    }
    let root = Path::new("/");
    if let Err(e) = saved::trusted(&root.join(saved::SAVED)) {
        eprintln!("deecure-helper: {e}");
        audit(0, "ApplySaved", &Value::Null, false, &e);
        return ExitCode::FAILURE;
    }
    let outcomes = saved::apply_saved(root, Duration::from_secs(1));
    for o in &outcomes {
        eprintln!(
            "deecure-helper: {}: {}{}",
            o.setting,
            if o.ok { "" } else { "not applied: " },
            o.message
        );
        audit(
            0,
            "ApplySaved",
            &json!({"setting": o.setting}),
            o.ok,
            &o.message,
        );
    }
    if let Err(e) = saved::write_results(root, now_ms(), &outcomes) {
        eprintln!("deecure-helper: {e}");
    }
    // Settings that could not be applied are in the results; the boot itself is fine.
    ExitCode::SUCCESS
}

async fn serve() -> zbus::Result<()> {
    let state = Arc::new(State {
        last_call: std::sync::Mutex::new(Instant::now()),
        writing: Mutex::new(()),
    });
    let _conn = zbus::connection::Builder::system()?
        .name(BUS_NAME)?
        .serve_at(
            OBJECT,
            Helper {
                state: state.clone(),
            },
        )?
        .build()
        .await?;
    eprintln!("deecure-helper {VERSION}: ready");
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        let idle = state
            .last_call
            .lock()
            .map(|t| t.elapsed())
            .unwrap_or_default();
        // Never exit in the middle of a write.
        if idle >= IDLE && state.writing.try_lock().is_ok() {
            eprintln!("deecure-helper: idle, exiting");
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_call_has_its_own_polkit_action() {
        assert_eq!(
            polkit_action(&Call::CleanAptCache),
            "io.github.krisskad0.deecure.clean-apt-cache"
        );
        assert_eq!(
            polkit_action(&Call::SetServiceEnabled {
                unit: "x.service".into(),
                enabled: true
            }),
            "io.github.krisskad0.deecure.set-service-enabled"
        );
    }

    #[test]
    fn every_admin_call_is_in_the_polkit_policy() {
        let policy = include_str!("../../../packaging/helper/io.github.krisskad0.deecure.policy");
        let calls = [
            Call::CleanAptCache,
            Call::Autoremove,
            Call::VacuumJournal { keep_mb: 100 },
            Call::RemoveSnapRevision {
                name: "x".into(),
                revision: "1".into(),
            },
            Call::PurgeResidual { packages: vec![] },
            Call::UpgradePackages { packages: vec![] },
            Call::SetFirewall { on: true },
            Call::RestartService {
                unit: "x.service".into(),
            },
            Call::SetServiceEnabled {
                unit: "x.service".into(),
                enabled: true,
            },
            Call::RemoveKernels { versions: vec![] },
            Call::SetBatteryAction {
                action: "PowerOff".into(),
                percent: 5,
            },
            Call::SetChargeLimits {
                battery: "BAT0".into(),
                start: None,
                stop: 80,
            },
            Call::SetChargeMode {
                battery: "BAT0".into(),
                mode: "Standard".into(),
            },
            Call::SetTurbo { on: true },
            Call::SetDeviceWakeup {
                device: "x".into(),
                enabled: true,
            },
            Call::SetDevicePowerSaving {
                device: "x".into(),
                on: true,
            },
        ];
        for call in calls {
            assert!(call.admin(), "{call:?}");
            let action = polkit_action(&call);
            assert!(
                policy.contains(&format!("<action id=\"{action}\">")),
                "{action} is missing from the polkit policy"
            );
        }
    }

    #[test]
    fn runs_commands_without_a_shell_and_stops_on_failure() {
        let ok = run_all(&[Command {
            program: "/bin/echo",
            args: vec!["a;".into(), "$(reboot)".into()],
        }])
        .unwrap();
        assert_eq!(ok, "a; $(reboot)", "arguments are passed literally");
        let err = run_all(&[
            Command {
                program: "/bin/false",
                args: vec![],
            },
            Command {
                program: "/bin/echo",
                args: vec!["never".into()],
            },
        ])
        .unwrap_err();
        assert!(err.contains("/bin/false failed"));
    }
}
