// SPDX-License-Identifier: Apache-2.0

//! DeeCure's own updates (docs/02-architecture/updates.md): the agent checks GitHub's release
//! list (daily by default; the only time DeeCure contacts the internet, and nothing about the
//! computer is sent), tells the user once per new version, and says after an update that it
//! happened. Installing is the root helper's job (`InstallUpdate`), after the user's OK, or
//! its timer when the user turned automatic updates on.

use std::sync::Arc;
use std::time::Duration;

use deecure_core::update::{self, UpdateSettings, UpdateStatus};
use deecure_core::{Timestamp, VERSION, local};
use serde::{Deserialize, Serialize};

use crate::context::Agent;
use crate::inventory::command;
use crate::store;

/// The user's choices.
pub const SETTINGS: &str = "updates.settings";
/// The last check: when, what it found, and why it failed.
const LAST_CHECK: &str = "updates.last_check";
/// The newest version the user was told about.
const NOTIFIED: &str = "updates.notified";
/// The version that ran last, to say "DeeCure was updated" once.
const SEEN: &str = "updates.seen_version";
/// After a failed check, try again this much later.
const RETRY_MS: i64 = 3_600_000;
/// The root timer's settings and its last result.
const AUTO_FILE: &str = "/etc/deecure/update.json";
const AUTO_RESULT: &str = "/run/deecure/update-result.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct LastCheck {
    at: Option<Timestamp>,
    latest: Option<update::Release>,
    error: Option<String>,
}

fn release_list_url() -> String {
    format!(
        "https://api.github.com/repos/{}/releases?per_page=20",
        update::REPO
    )
}

/// GitHub's release list, with curl or wget (Ubuntu Desktop has wget but not always curl).
async fn fetch() -> Result<serde_json::Value, String> {
    let url = release_list_url();
    let limit = Duration::from_secs(40);
    let text = match command::output(
        "curl",
        &[
            "-fsSL",
            "--proto",
            "=https",
            "--max-time",
            "30",
            "-H",
            "Accept: application/vnd.github+json",
            &url,
        ],
        limit,
    )
    .await
    {
        Some(t) => t,
        None => command::output(
            "wget",
            &[
                "-qO-",
                "--https-only",
                "--timeout=30",
                "--tries=2",
                "--header=Accept: application/vnd.github+json",
                &url,
            ],
            limit,
        )
        .await
        .ok_or("could not reach GitHub (offline, or neither curl nor wget is installed)")?,
    };
    serde_json::from_str(&text).map_err(|_| "GitHub's answer was not a release list".into())
}

/// The user's choices.
pub fn settings(s: &store::Store) -> store::Result<UpdateSettings> {
    Ok(s.setting(SETTINGS)?.unwrap_or_default())
}

/// What the root timer is set to, from its file.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct AutoFile {
    /// Install by itself.
    pub auto: bool,
}

/// Everything the Settings page shows.
pub fn status(agent: &Agent) -> store::Result<UpdateStatus> {
    let s = store::lock(&agent.store);
    let settings = settings(&s)?;
    let last: LastCheck = s.setting(LAST_CHECK)?.unwrap_or_default();
    drop(s);
    let root = &agent.root;
    let available = last
        .latest
        .as_ref()
        .is_some_and(|r| update::compare(&r.version, VERSION) == Some(std::cmp::Ordering::Greater));
    // A DeeCure APT source means Software Updater brings the updates.
    let from_apt = std::fs::read_dir(root.path("/etc/apt/sources.list.d"))
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| e.file_name().to_string_lossy().contains("deecure"));
    let auto_ready = root
        .read(AUTO_FILE)
        .ok()
        .and_then(|t| serde_json::from_str::<AutoFile>(&t).ok())
        .is_some_and(|f| f.auto);
    let last_auto = root
        .read(AUTO_RESULT)
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.get("message")?.as_str().map(str::to_owned));
    Ok(UpdateStatus {
        settings,
        current: VERSION.to_owned(),
        checked_at: last.at,
        latest: last.latest,
        available,
        error: last.error,
        from_apt,
        auto_ready,
        last_auto,
    })
}

/// Checks GitHub now and saves what it found.
pub async fn check_now(agent: &Agent) -> store::Result<UpdateStatus> {
    let channel = settings(&store::lock(&agent.store))?.channel;
    let result = fetch().await;
    let last = match result {
        Ok(list) => LastCheck {
            at: Some(Timestamp::now()),
            latest: update::newest(&list, channel),
            error: None,
        },
        Err(e) => {
            let before: LastCheck = store::lock(&agent.store)
                .setting(LAST_CHECK)?
                .unwrap_or_default();
            LastCheck {
                at: Some(Timestamp::now()),
                latest: before.latest,
                error: Some(e),
            }
        }
    };
    store::lock(&agent.store).set_setting(LAST_CHECK, &last)?;
    status(agent)
}

/// Is a check due? Never when off; after a failure, again an hour later.
pub fn due(
    settings: &UpdateSettings,
    last_at: Option<Timestamp>,
    failed: bool,
    now: Timestamp,
) -> bool {
    let Some(period) = settings.check.period_ms() else {
        return false;
    };
    let wait = if failed { RETRY_MS.min(period) } else { period };
    last_at.is_none_or(|at| now.as_millis() - at.as_millis() >= wait)
}

async fn tell(agent: &Agent, key: &str, title: &str, body: &str) -> Result<(), String> {
    let quiet = {
        let s = store::lock(&agent.store);
        crate::notify::settings(&s)
            .map_err(|e| e.to_string())?
            .quiet_hours
    };
    let minute = local::parts(Timestamp::now()).map_or(12 * 60, |p| p.minute_of_day());
    if quiet.contains(minute) {
        return Err("quiet hours".into());
    }
    crate::notify::show(agent, title, body, false).await?;
    store::lock(&agent.store)
        .add_notification(key, Timestamp::now(), title, body)
        .map_err(|e| e.to_string())
}

/// One round: say "updated" once, check when due, and tell about a new version once.
pub async fn tick(agent: &Agent) -> Result<(), String> {
    let seen: Option<String> = store::lock(&agent.store)
        .setting(SEEN)
        .map_err(|e| e.to_string())?;
    if seen.as_deref() != Some(VERSION) {
        // The first run after installing says nothing; a later version change does.
        if let Some(before) = seen
            && tell(
                agent,
                "update:installed",
                &format!("DeeCure was updated to {VERSION}"),
                &format!("It was {before}. Open the app again if it was open."),
            )
            .await
            .is_err()
        {
            // Quiet hours or no desktop: try again next round.
            return Ok(());
        }
        store::lock(&agent.store)
            .set_setting(SEEN, &VERSION)
            .map_err(|e| e.to_string())?;
    }
    let (settings, last) = {
        let s = store::lock(&agent.store);
        let settings = settings(&s).map_err(|e| e.to_string())?;
        let last: LastCheck = s
            .setting(LAST_CHECK)
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        (settings, last)
    };
    if due(&settings, last.at, last.error.is_some(), Timestamp::now()) {
        check_now(agent).await.map_err(|e| e.to_string())?;
    }
    let st = status(agent).map_err(|e| e.to_string())?;
    let notified: Option<String> = store::lock(&agent.store)
        .setting(NOTIFIED)
        .map_err(|e| e.to_string())?;
    if let Some(latest) = st
        .latest
        .filter(|_| st.available && !st.settings.auto && !st.from_apt)
        && notified.as_deref() != Some(latest.version.as_str())
        && tell(
            agent,
            &format!("update:{}", latest.version),
            &format!("DeeCure {} is available", latest.version),
            "Open DeeCure, then Settings › About, to see what is new and install it.",
        )
        .await
        .is_ok()
    {
        store::lock(&agent.store)
            .set_setting(NOTIFIED, &latest.version)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// The first round waits this long after the agent starts, so a login (or a test run) never
/// goes online at once; "Check now" still works right away.
const FIRST_AFTER: Duration = Duration::from_secs(10 * 60);

/// Every 30 minutes, forever, starting 10 minutes after the agent starts.
pub async fn run(agent: Arc<Agent>) {
    tokio::time::sleep(FIRST_AFTER).await;
    let mut timer = tokio::time::interval(Duration::from_secs(30 * 60));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut failing = false;
    loop {
        timer.tick().await;
        match tick(&agent).await {
            Ok(()) => failing = false,
            Err(e) => {
                if !failing {
                    eprintln!("deecure-agent: update check: {e}");
                }
                failing = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deecure_core::update::CheckEvery;

    #[test]
    fn checks_are_due_by_the_chosen_period_and_retry_sooner_after_a_failure() {
        let now = Timestamp(100 * 86_400_000);
        let daily = UpdateSettings::default();
        let weekly = UpdateSettings {
            check: CheckEvery::Weekly,
            ..UpdateSettings::default()
        };
        let off = UpdateSettings {
            check: CheckEvery::Off,
            ..UpdateSettings::default()
        };
        let ago = |h: i64| Some(Timestamp(now.as_millis() - h * 3_600_000));
        assert!(due(&daily, None, false, now), "never checked");
        assert!(!due(&daily, ago(5), false, now));
        assert!(due(&daily, ago(25), false, now));
        assert!(!due(&weekly, ago(25), false, now));
        assert!(due(&weekly, ago(24 * 7), false, now));
        assert!(
            due(&daily, ago(2), true, now),
            "a failed check retries after an hour"
        );
        assert!(
            !due(&off, None, false, now),
            "off never contacts the internet"
        );
    }
}
