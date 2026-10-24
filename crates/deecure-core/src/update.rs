// SPDX-License-Identifier: Apache-2.0

//! DeeCure's own updates (docs/02-architecture/updates.md): the user's choices, the newest
//! release on GitHub, and version order. The agent checks; the root helper installs, after
//! checking every package.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::Timestamp;

/// The repository releases come from. Never a user input.
pub const REPO: &str = "krisskad0/DeeCure";

/// How often to look for a new release.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckEvery {
    /// Never: DeeCure does not contact the internet.
    Off,
    /// Once a day (the default).
    #[default]
    Daily,
    /// Once a week.
    Weekly,
}

impl CheckEvery {
    /// The time between checks, in milliseconds; `None` when off.
    pub fn period_ms(self) -> Option<i64> {
        match self {
            CheckEvery::Off => None,
            CheckEvery::Daily => Some(86_400_000),
            CheckEvery::Weekly => Some(7 * 86_400_000),
        }
    }
}

/// Which releases to offer.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    /// Pre-releases too (the default while there is no stable release).
    #[default]
    Prerelease,
    /// Stable releases only.
    Stable,
}

/// The user's update choices.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UpdateSettings {
    /// How often to check.
    pub check: CheckEvery,
    /// Which releases to offer.
    pub channel: Channel,
    /// Install by itself (the root timer does it); otherwise ask first.
    pub auto: bool,
}

/// One release on GitHub.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    /// Such as `0.1.0-alpha.4`.
    pub version: String,
    /// A pre-release (alpha or beta).
    pub prerelease: bool,
    /// When it was published, as GitHub writes it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    /// The release page.
    pub url: String,
    /// The release notes, cut short.
    #[serde(default)]
    pub notes: String,
}

/// What the update check knows.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateStatus {
    /// The user's choices.
    pub settings: UpdateSettings,
    /// The version installed now.
    pub current: String,
    /// When the last check finished.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked_at: Option<Timestamp>,
    /// The newest release for the channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest: Option<Release>,
    /// The newest release is newer than this one.
    pub available: bool,
    /// Why the last check failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// DeeCure comes from an APT repository, so Software Updater installs its updates.
    pub from_apt: bool,
    /// Automatic updates are set up for the root timer (`/etc/deecure/update.json`).
    pub auto_ready: bool,
    /// What the last automatic update did, from `/run/deecure/update-result.json`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_auto: Option<String>,
}

/// A version such as `0.1.0` or `0.1.0-alpha.3`: three numbers, then an optional pre-release
/// of letters, digits, and dots.
pub fn valid_version(v: &str) -> bool {
    parse(v).is_some() && v.len() <= 40
}

fn parse(v: &str) -> Option<([u64; 3], Vec<&str>)> {
    let (core, pre) = match v.split_once('-') {
        Some((c, p)) => (c, Some(p)),
        None => (v, None),
    };
    let mut nums = [0u64; 3];
    let mut parts = core.split('.');
    for n in &mut nums {
        let p = parts.next()?;
        if p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        *n = p.parse().ok()?;
    }
    if parts.next().is_some() {
        return None;
    }
    let pre: Vec<&str> = match pre {
        None => Vec::new(),
        Some(p) => {
            let ids: Vec<&str> = p.split('.').collect();
            if ids
                .iter()
                .any(|i| i.is_empty() || !i.chars().all(|c| c.is_ascii_alphanumeric()))
            {
                return None;
            }
            ids
        }
    };
    Some((nums, pre))
}

/// Semantic version order: `0.1.0-alpha.3` < `0.1.0-alpha.10` < `0.1.0-beta.1` < `0.1.0`.
/// `None` when either is not a version.
pub fn compare(a: &str, b: &str) -> Option<Ordering> {
    let (an, ap) = parse(a)?;
    let (bn, bp) = parse(b)?;
    Some(
        an.cmp(&bn)
            .then_with(|| match (ap.is_empty(), bp.is_empty()) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                (false, false) => {
                    for (x, y) in ap.iter().zip(&bp) {
                        let o = match (x.parse::<u64>(), y.parse::<u64>()) {
                            (Ok(i), Ok(j)) => i.cmp(&j),
                            (Ok(_), Err(_)) => Ordering::Less,
                            (Err(_), Ok(_)) => Ordering::Greater,
                            (Err(_), Err(_)) => x.cmp(y),
                        };
                        if o != Ordering::Equal {
                            return o;
                        }
                    }
                    ap.len().cmp(&bp.len())
                }
            }),
    )
}

/// The Debian package version of a release: `0.1.0-alpha.3` becomes `0.1.0~alpha.3-1`.
pub fn deb_version(v: &str) -> String {
    format!("{}-1", v.replacen('-', "~", 1))
}

/// The newest release for the channel from GitHub's release list (`/releases` JSON): not a
/// draft, a valid version, with the packages and checksums attached.
pub fn newest(list: &serde_json::Value, channel: Channel) -> Option<Release> {
    let mut best: Option<Release> = None;
    for r in list.as_array()? {
        let get = |k: &str| r.get(k);
        if get("draft").and_then(serde_json::Value::as_bool) == Some(true) {
            continue;
        }
        let prerelease = get("prerelease")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if prerelease && channel == Channel::Stable {
            continue;
        }
        let Some(version) = get("tag_name")
            .and_then(serde_json::Value::as_str)
            .and_then(|t| t.strip_prefix('v'))
            .filter(|v| valid_version(v))
        else {
            continue;
        };
        let assets: Vec<&str> = get("assets")
            .and_then(serde_json::Value::as_array)
            .map(|a| a.iter().filter_map(|x| x.get("name")?.as_str()).collect())
            .unwrap_or_default();
        let complete = assets.contains(&"SHA256SUMS")
            && assets
                .iter()
                .any(|n| n.starts_with("deecure_") && n.ends_with("_amd64.deb"));
        if !complete {
            continue;
        }
        if best
            .as_ref()
            .is_some_and(|b| compare(version, &b.version) != Some(Ordering::Greater))
        {
            continue;
        }
        let notes: String = get("body")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .chars()
            .take(2000)
            .collect();
        best = Some(Release {
            version: version.to_owned(),
            prerelease,
            published_at: get("published_at")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            url: format!("https://github.com/{REPO}/releases/tag/v{version}"),
            notes,
        });
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn versions_follow_semantic_order() {
        let lt = |a, b| assert_eq!(compare(a, b), Some(Ordering::Less), "{a} < {b}");
        lt("0.1.0-alpha.3", "0.1.0-alpha.4");
        lt("0.1.0-alpha.4", "0.1.0-alpha.10");
        lt("0.1.0-alpha.10", "0.1.0-beta.1");
        lt("0.1.0-beta.1", "0.1.0");
        lt("0.1.0", "0.1.1");
        lt("0.9.9", "0.10.0");
        assert_eq!(compare("1.2.3", "1.2.3"), Some(Ordering::Equal));
        for bad in [
            "",
            "1.2",
            "1.2.3.4",
            "v1.2.3",
            "1.2.3-",
            "1.2.3-a..b",
            "1.2.3-$(x)",
            "1.2.x",
        ] {
            assert!(!valid_version(bad), "{bad}");
        }
        assert_eq!(deb_version("0.1.0-alpha.3"), "0.1.0~alpha.3-1");
        assert_eq!(deb_version("0.1.0"), "0.1.0-1");
    }

    #[test]
    fn the_newest_complete_release_for_the_channel_wins() {
        let rel = |tag: &str, pre: bool, draft: bool, assets: &[&str]| {
            json!({"tag_name": tag, "prerelease": pre, "draft": draft, "body": "notes", "published_at": "2026-10-24T10:00:00Z",
                   "assets": assets.iter().map(|n| json!({"name": n})).collect::<Vec<_>>()})
        };
        let full = ["SHA256SUMS", "deecure_0.1.0.alpha.4-1_amd64.deb"];
        let list = json!([
            rel("v0.1.0-alpha.5", true, true, &full),
            rel("v0.1.0-alpha.4", true, false, &full),
            rel("v0.1.0-alpha.10", true, false, &["SHA256SUMS"]),
            rel("v0.0.9", false, false, &full),
            rel("nightly", true, false, &full),
        ]);
        let pre = newest(&list, Channel::Prerelease).unwrap();
        assert_eq!(
            pre.version, "0.1.0-alpha.4",
            "drafts and releases without packages are skipped"
        );
        assert_eq!(
            pre.url,
            "https://github.com/krisskad0/DeeCure/releases/tag/v0.1.0-alpha.4"
        );
        assert_eq!(newest(&list, Channel::Stable).unwrap().version, "0.0.9");
        assert!(newest(&json!([]), Channel::Stable).is_none());
    }
}
