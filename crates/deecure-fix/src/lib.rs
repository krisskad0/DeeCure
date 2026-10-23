// SPDX-License-Identifier: Apache-2.0

//! Runs fix steps. It runs inside the person's session (the desktop app or the CLI), so
//! when an admin step goes to the root helper, polkit can show the password box there.
//!
//! User-level steps run here, as the person. Admin steps are sent to `deecure-helper`,
//! which checks them again. Steps run in order and stop at the first failure.

use std::path::{Path, PathBuf};
use std::time::Duration;

use deecure_core::Timestamp;
use deecure_core::fix::{Call, FixRun, Plan, RunStatus, StepResult};
use serde_json::Value;
use zbus::zvariant::{OwnedValue, Value as Variant};

const HELPER: &str = "io.github.krisskad0.DeeCure.Helper";
const HELPER_OBJECT: &str = "/io/github/krisskad0/DeeCure/Helper";
const HELPER_INTERFACE: &str = "io.github.krisskad0.DeeCure.Helper1";

/// Runs the chosen steps of a plan and returns what happened. `progress` is called after
/// each step, so a window can tick steps off as they finish.
pub async fn apply(
    plan: &Plan,
    chosen: &[String],
    mut progress: impl FnMut(&StepResult),
) -> FixRun {
    let started_at = Timestamp::now();
    let mut steps = Vec::new();
    let mut status = RunStatus::Succeeded;
    for step in plan.steps.iter().filter(|s| chosen.contains(&s.id)) {
        let result = match run_call(&step.call).await {
            Ok((message, undo)) => StepResult {
                id: step.id.clone(),
                title: step.title.clone(),
                ok: true,
                message,
                undo,
            },
            Err(message) => StepResult {
                id: step.id.clone(),
                title: step.title.clone(),
                ok: false,
                message,
                undo: None,
            },
        };
        progress(&result);
        let failed = !result.ok;
        steps.push(result);
        if failed {
            status = RunStatus::Failed;
            break;
        }
    }
    FixRun {
        id: 0,
        action: plan.action.clone(),
        title: plan.title.clone(),
        rule: plan.rule.clone(),
        subject: plan.subject.clone(),
        started_at,
        finished_at: Timestamp::now(),
        status,
        steps,
    }
}

/// Takes a run back: runs each undo call, last step first. Stops at the first failure.
pub async fn undo(run: &FixRun) -> Result<Vec<String>, String> {
    let calls = run.undo_calls();
    if calls.is_empty() {
        return Err("this fix cannot be undone".into());
    }
    let mut done = Vec::new();
    for call in calls {
        let (message, _) = run_call(&call).await?;
        done.push(message);
    }
    Ok(done)
}

/// The inverse of a call DeeCure can always compute itself.
pub fn inverse(call: &Call) -> Option<Call> {
    match call {
        Call::SetFirewall { on } => Some(Call::SetFirewall { on: !on }),
        Call::SetServiceEnabled { unit, enabled } => Some(Call::SetServiceEnabled {
            unit: unit.clone(),
            enabled: !enabled,
        }),
        Call::SetStartupApp { id, enabled } => Some(Call::SetStartupApp {
            id: id.clone(),
            enabled: !enabled,
        }),
        _ => None,
    }
}

/// Runs one call. Returns a message and, when it can be taken back, the undo call.
pub async fn run_call(call: &Call) -> Result<(String, Option<Call>), String> {
    if call.admin() {
        return helper_run(call).await;
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or("HOME is not set")?;
    match call {
        Call::EmptyTrash => {
            let trash = home.join(".local/share/Trash");
            let (removed, failed) = clear_folder(&trash.join("files"));
            clear_folder(&trash.join("info"));
            clear_folder(&trash.join("expunged"));
            if failed > 0 {
                Err(format!("{failed} items in the Trash could not be removed"))
            } else {
                Ok((format!("Trash emptied ({removed} items)"), None))
            }
        }
        Call::ClearThumbnails => {
            let (removed, _) = clear_folder(&home.join(".cache/thumbnails"));
            Ok((format!("Thumbnail cache cleared ({removed} folders)"), None))
        }
        Call::DockerPruneImages => program("docker", &["image", "prune", "--all", "--force"])
            .await
            .map(|m| (m, None)),
        Call::DockerPruneBuildCache => program("docker", &["builder", "prune", "--force"])
            .await
            .map(|m| (m, None)),
        Call::SetStartupApp { id, enabled } => {
            set_startup_app(&home, id, *enabled).map(|m| (m, inverse(call)))
        }
        Call::EndProcess { pid, name } => end_process(*pid, name).map(|m| (m, None)),
        Call::SetPowerProfile { profile } => set_power_profile(profile).await,
        _ => Err("not a user-level call".into()),
    }
}

/// Removes everything inside `dir` (not `dir` itself). Returns (removed, failed).
fn clear_folder(dir: &Path) -> (usize, usize) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return (0, 0);
    };
    let (mut removed, mut failed) = (0, 0);
    for e in entries.flatten() {
        let path = e.path();
        // Never follow a link out of the folder: remove the link itself.
        let result = match e.file_type() {
            Ok(t) if t.is_dir() => std::fs::remove_dir_all(&path),
            _ => std::fs::remove_file(&path),
        };
        if result.is_ok() {
            removed += 1
        } else {
            failed += 1
        }
    }
    (removed, failed)
}

async fn program(name: &str, args: &[&str]) -> Result<String, String> {
    let out = tokio::time::timeout(
        Duration::from_secs(600),
        tokio::process::Command::new(name)
            .args(args)
            .env("LC_ALL", "C")
            .kill_on_drop(true)
            .output(),
    )
    .await
    .map_err(|_| format!("{name} took too long"))?
    .map_err(|e| format!("could not run {name}: {e}"))?;
    let text =
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
    let last = text
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("done")
        .to_owned();
    if out.status.success() {
        Ok(last)
    } else {
        Err(format!("{name} failed: {last}"))
    }
}

/// A `.desktop` file name with no path in it.
fn is_desktop_id(id: &str) -> bool {
    id.ends_with(".desktop")
        && id.len() <= 200
        && !id.starts_with('.')
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._@-+".contains(c))
}

/// Sets `X-GNOME-Autostart-enabled` in the `[Desktop Entry]` group, keeping everything else.
pub fn set_autostart_flag(text: &str, enabled: bool) -> String {
    let mut out = Vec::new();
    let mut in_entry = false;
    let mut written = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if in_entry && !written {
                out.push(format!("X-GNOME-Autostart-enabled={enabled}"));
                written = true;
            }
            in_entry = trimmed == "[Desktop Entry]";
        } else if in_entry && trimmed.starts_with("X-GNOME-Autostart-enabled=") {
            out.push(format!("X-GNOME-Autostart-enabled={enabled}"));
            written = true;
            continue;
        } else if in_entry && trimmed.starts_with("Hidden=") {
            // Hidden=true would win over the flag; drop it so the flag decides.
            continue;
        }
        out.push(line.to_owned());
    }
    if !written {
        out.push(format!("X-GNOME-Autostart-enabled={enabled}"));
    }
    out.join("\n") + "\n"
}

fn set_startup_app(home: &Path, id: &str, enabled: bool) -> Result<String, String> {
    if !is_desktop_id(id) {
        return Err("not a startup app ID".into());
    }
    let dir = home.join(".config/autostart");
    let user = dir.join(id);
    let source = if user.exists() {
        user.clone()
    } else {
        PathBuf::from("/etc/xdg/autostart").join(id)
    };
    let text = std::fs::read_to_string(&source)
        .map_err(|e| format!("cannot read {}: {e}", source.display()))?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(&user, set_autostart_flag(&text, enabled))
        .map_err(|e| format!("cannot write {}: {e}", user.display()))?;
    Ok(format!(
        "{id} {} at login",
        if enabled {
            "starts"
        } else {
            "no longer starts"
        }
    ))
}

fn end_process(pid: u32, name: &str) -> Result<String, String> {
    let status = std::fs::read_to_string(format!("/proc/{pid}/status"))
        .map_err(|_| format!("process {pid} has already ended"))?;
    let field = |k: &str| {
        status
            .lines()
            .find_map(|l| l.strip_prefix(k))
            .map(str::trim)
            .unwrap_or("")
    };
    let comm = field("Name:");
    // Linux cuts names at 15 characters; a reused PID with another program is refused.
    if comm != name && !(comm.len() == 15 && name.starts_with(comm)) {
        return Err(format!(
            "process {pid} is now {comm}, not {name}; nothing was done"
        ));
    }
    let uid: u32 = field("Uid:")
        .split_whitespace()
        .next()
        .and_then(|u| u.parse().ok())
        .unwrap_or(u32::MAX);
    if uid != rustix::process::getuid().as_raw() {
        return Err(format!(
            "{name} belongs to another user; ending it needs admin rights"
        ));
    }
    let pid = rustix::process::Pid::from_raw(i32::try_from(pid).map_err(|_| "bad process ID")?)
        .ok_or("bad process ID")?;
    rustix::process::kill_process(pid, rustix::process::Signal::TERM)
        .map_err(|e| format!("could not end {name}: {e}"))?;
    Ok(format!("asked {name} to quit"))
}

async fn set_power_profile(profile: &str) -> Result<(String, Option<Call>), String> {
    let bus = zbus::Connection::system()
        .await
        .map_err(|e| e.to_string())?;
    for (service, object) in [
        (
            "org.freedesktop.UPower.PowerProfiles",
            "/org/freedesktop/UPower/PowerProfiles",
        ),
        ("net.hadess.PowerProfiles", "/net/hadess/PowerProfiles"),
    ] {
        let Ok(reply) = bus
            .call_method(
                Some(service),
                object,
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &(service, "ActiveProfile"),
            )
            .await
        else {
            continue;
        };
        let old: Option<String> = reply
            .body()
            .deserialize::<OwnedValue>()
            .ok()
            .and_then(|v| String::try_from(v).ok());
        bus.call_method(
            Some(service),
            object,
            Some("org.freedesktop.DBus.Properties"),
            "Set",
            &(service, "ActiveProfile", Variant::from(profile)),
        )
        .await
        .map_err(|e| format!("could not change the power mode: {e}"))?;
        return Ok((
            format!("power mode is now {profile}"),
            old.map(|p| Call::SetPowerProfile { profile: p }),
        ));
    }
    Err("the power profiles service is not running".into())
}

/// Sends an admin call to the root helper. polkit may show the password box.
async fn helper_run(call: &Call) -> Result<(String, Option<Call>), String> {
    let bus = zbus::Connection::system()
        .await
        .map_err(|e| e.to_string())?;
    let json = serde_json::to_string(call).map_err(|e| e.to_string())?;
    let reply = bus
        .call_method(
            Some(HELPER),
            HELPER_OBJECT,
            Some(HELPER_INTERFACE),
            "Run",
            &(json,),
        )
        .await
        .map_err(|e| helper_error(&e))?;
    let text: String = reply.body().deserialize().map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    let message = v
        .get("message")
        .and_then(Value::as_str)
        .filter(|m| !m.is_empty())
        .unwrap_or("done")
        .to_owned();
    let undo = v
        .get("undo")
        .and_then(|u| serde_json::from_value::<Call>(u.clone()).ok())
        .or_else(|| inverse(call));
    Ok((message, undo))
}

/// Asks the helper for an admin read-out (see `deecure-helper`'s `Read`).
pub async fn helper_read(what: &str, arg: &str) -> Result<String, String> {
    let bus = zbus::Connection::system()
        .await
        .map_err(|e| e.to_string())?;
    let reply = bus
        .call_method(
            Some(HELPER),
            HELPER_OBJECT,
            Some(HELPER_INTERFACE),
            "Read",
            &(what, arg),
        )
        .await
        .map_err(|e| helper_error(&e))?;
    reply
        .body()
        .deserialize::<String>()
        .map_err(|e| e.to_string())
}

/// True when the helper is installed (its D-Bus service can be started).
pub async fn helper_installed() -> bool {
    let Ok(bus) = zbus::Connection::system().await else {
        return false;
    };
    let Ok(reply) = bus
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "ListActivatableNames",
            &(),
        )
        .await
    else {
        return false;
    };
    reply
        .body()
        .deserialize::<Vec<String>>()
        .is_ok_and(|names| names.iter().any(|n| n == HELPER))
}

/// Plain words for helper errors.
fn helper_error(e: &zbus::Error) -> String {
    match e {
        zbus::Error::MethodError(name, message, _) => {
            let name = name.as_str();
            if name.ends_with("AccessDenied") {
                "Not approved: the password box was closed or the password was wrong.".into()
            } else if name.ends_with("ServiceUnknown") || name.ends_with("NameHasNoOwner") {
                "The DeeCure admin helper is not installed, so admin fixes are not available."
                    .into()
            } else {
                message.clone().unwrap_or_else(|| name.to_owned())
            }
        }
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggles_the_autostart_flag_and_keeps_the_rest() {
        let text = "[Desktop Entry]\nName=Telegram\nExec=telegram -startintray\nHidden=true\n[Desktop Action x]\nName=Other\n";
        let off = set_autostart_flag(text, false);
        assert_eq!(
            off,
            "[Desktop Entry]\nName=Telegram\nExec=telegram -startintray\nX-GNOME-Autostart-enabled=false\n[Desktop Action x]\nName=Other\n"
        );
        let on = set_autostart_flag(&off, true);
        assert!(on.contains("X-GNOME-Autostart-enabled=true") && !on.contains("=false"));
        assert!(
            set_autostart_flag("[Desktop Entry]\nName=A\n", false)
                .ends_with("X-GNOME-Autostart-enabled=false\n")
        );
    }

    #[test]
    fn refuses_bad_ids_and_other_processes() {
        assert!(!is_desktop_id("../../.bashrc"));
        assert!(!is_desktop_id("x.desktop/../y"));
        assert!(is_desktop_id("org.telegram.desktop.desktop"));
        // PID 1 belongs to root and is not "chrome".
        assert!(end_process(1, "chrome").is_err());
        assert!(end_process(u32::MAX, "x").is_err());
    }

    #[test]
    fn knows_simple_inverses() {
        assert_eq!(
            inverse(&Call::SetFirewall { on: true }),
            Some(Call::SetFirewall { on: false })
        );
        assert_eq!(inverse(&Call::CleanAptCache), None);
    }

    #[test]
    fn clears_a_folder_without_following_links() {
        let dir = std::env::temp_dir().join(format!("deecure-fix-{}", std::process::id()));
        let outside =
            std::env::temp_dir().join(format!("deecure-fix-outside-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("keep.txt"), "x").unwrap();
        std::fs::write(dir.join("a.txt"), "x").unwrap();
        std::os::unix::fs::symlink(&outside, dir.join("link")).unwrap();
        assert_eq!(clear_folder(&dir), (3, 0));
        assert!(
            outside.join("keep.txt").exists(),
            "the link's target is untouched"
        );
        std::fs::remove_dir_all(&dir).unwrap();
        std::fs::remove_dir_all(&outside).unwrap();
    }
}
