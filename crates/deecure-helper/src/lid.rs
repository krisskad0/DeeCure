// SPDX-License-Identifier: Apache-2.0

//! What closing the lid does: one logind drop-in, `/etc/systemd/logind.conf.d/50-deecure-lid.conf`.
//!
//! logind reads it when it starts, so it applies after the next restart. The helper never
//! restarts logind: that would end every desktop session. Only four actions are offered;
//! hibernation is left out because it needs swap and firmware support this cannot check.

use std::path::Path;

use deecure_core::fix::Call;

/// The drop-in, under the system root.
pub const FILE: &str = "etc/systemd/logind.conf.d/50-deecure-lid.conf";
const ACTIONS: [&str; 4] = ["suspend", "lock", "ignore", "poweroff"];

/// The drop-in's text, or `None` to remove it.
fn text(on_battery: &str, on_power: &str) -> Option<String> {
    (on_battery != "default" || on_power != "default").then(|| {
        let mut t = String::from(
            "# Written by DeeCure (Power > Power Profile). It applies after the next restart.\n\
             # Delete this file to go back to the system's own setting.\n[Login]\n",
        );
        if on_battery != "default" {
            t += &format!("HandleLidSwitch={on_battery}\n");
        }
        if on_power != "default" {
            t += &format!("HandleLidSwitchExternalPower={on_power}\n");
        }
        t
    })
}

fn get(text: &str, key: &str) -> String {
    text.lines()
        .find_map(|l| l.trim().strip_prefix(key)?.strip_prefix('='))
        .map_or_else(|| "default".to_owned(), |v| v.trim().to_owned())
}

/// The drop-in's new text (`None`: remove it) and the call that undoes the change.
pub type Planned = (Option<String>, Call);

/// Checks the call and returns the new text (`None`: remove the file) and the undo call.
/// `None` when the call is not the lid action.
pub fn plan(call: &Call, root: &Path) -> Option<Result<Planned, String>> {
    let Call::SetLidAction {
        on_battery,
        on_power,
    } = call
    else {
        return None;
    };
    let ok = |v: &str| v == "default" || ACTIONS.contains(&v);
    if !ok(on_battery) || !ok(on_power) {
        return Some(Err(
            "the lid action must be suspend, lock, ignore, poweroff, or default".into(),
        ));
    }
    let before = std::fs::read_to_string(root.join(FILE)).unwrap_or_default();
    Some(Ok((
        text(on_battery, on_power),
        Call::SetLidAction {
            on_battery: get(&before, "HandleLidSwitch"),
            on_power: get(&before, "HandleLidSwitchExternalPower"),
        },
    )))
}

/// Writes (in one step) or removes the drop-in.
pub fn apply(root: &Path, text: Option<&str>) -> Result<String, String> {
    let path = root.join(FILE);
    match text {
        Some(t) => {
            let dir = path.parent().ok_or("no folder")?;
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("could not create {}: {e}", dir.display()))?;
            let tmp = path.with_extension("deecure-tmp");
            std::fs::write(&tmp, t)
                .and_then(|()| std::fs::rename(&tmp, &path))
                .map_err(|e| format!("could not write {}: {e}", path.display()))?;
            Ok(format!("wrote /{FILE}; it applies after the next restart"))
        }
        None => match std::fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                Err(format!("could not remove {}: {e}", path.display()))
            }
            _ => Ok(format!(
                "removed /{FILE}; the system's own setting applies after the next restart"
            )),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lid(b: &str, p: &str) -> Call {
        Call::SetLidAction {
            on_battery: b.into(),
            on_power: p.into(),
        }
    }

    #[test]
    fn writes_only_known_actions_and_undoes_to_what_was_there() {
        let root = std::env::temp_dir().join(format!("deecure-helper-lid-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        assert!(plan(&Call::CleanAptCache, &root).is_none());
        assert!(plan(&lid("hibernate", "lock"), &root).unwrap().is_err());
        assert!(
            plan(&lid("suspend\nHandlePowerKey=poweroff", "lock"), &root)
                .unwrap()
                .is_err()
        );

        let (text, undo) = plan(&lid("suspend", "lock"), &root).unwrap().unwrap();
        assert_eq!(undo, lid("default", "default"), "there was no file");
        let t = text.unwrap();
        assert!(
            t.contains("[Login]\nHandleLidSwitch=suspend\nHandleLidSwitchExternalPower=lock\n")
        );
        apply(&root, Some(&t)).unwrap();

        let (text, undo) = plan(&lid("default", "ignore"), &root).unwrap().unwrap();
        assert_eq!(undo, lid("suspend", "lock"));
        assert!(!text.unwrap().contains("HandleLidSwitch="));

        let (text, _) = plan(&lid("default", "default"), &root).unwrap().unwrap();
        assert!(text.is_none());
        apply(&root, None).unwrap();
        assert!(!root.join(FILE).exists());
        apply(&root, None).unwrap();
    }
}
