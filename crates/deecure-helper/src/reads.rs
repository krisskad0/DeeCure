// SPDX-License-Identifier: Apache-2.0

//! Admin read-outs: things only root may read. Each returns JSON or text, never secrets:
//! password hashes stay in /etc/shadow, only their status is reported.

use std::collections::BTreeMap;

use serde::Serialize;

/// One account's password status, from `/etc/shadow` (the hash itself is never returned).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AccountStatus {
    pub user: String,
    /// True when the password is locked (`!` or `*`).
    pub locked: bool,
    /// True when there is no password at all (an empty field).
    pub empty: bool,
    /// Days since 1970 when the password last changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changed_day: Option<u64>,
    /// Days a password may be kept, when set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_days: Option<u64>,
}

/// Reads `/etc/shadow` lines for the given users.
pub fn shadow_status(shadow: &str, users: &[String]) -> Vec<AccountStatus> {
    shadow
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split(':').collect();
            let user = (*f.first()?).to_owned();
            if !users.contains(&user) {
                return None;
            }
            let hash = f.get(1).copied().unwrap_or("");
            Some(AccountStatus {
                locked: hash.starts_with('!') || hash.starts_with('*'),
                empty: hash.is_empty(),
                changed_day: f.get(2).and_then(|v| v.parse().ok()),
                max_days: f
                    .get(4)
                    .and_then(|v| v.parse().ok())
                    .filter(|d| *d < 99_999),
                user,
            })
        })
        .collect()
}

/// Human users (UID 0 and 1000-59999) from `/etc/passwd`.
pub fn human_users(passwd: &str) -> Vec<String> {
    passwd
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split(':').collect();
            let uid: u32 = f.get(2)?.parse().ok()?;
            let name = (*f.first()?).to_owned();
            (uid == 0 || (1000..60_000).contains(&uid)).then_some(name)
        })
        .collect()
}

/// Each user's crontab from `/var/spool/cron/crontabs`.
pub fn all_crontabs(dir: &std::path::Path) -> BTreeMap<String, String> {
    std::fs::read_dir(dir)
        .map(|d| {
            d.flatten()
                .filter(|e| e.path().is_file())
                .filter_map(|e| {
                    Some((
                        e.file_name().to_string_lossy().into_owned(),
                        std::fs::read_to_string(e.path()).ok()?,
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Disk names `smartctl` may read: `sda`, `nvme0n1`, `mmcblk0`, and nothing else.
pub fn is_disk(name: &str) -> bool {
    let digits = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
    if let Some(rest) = name.strip_prefix("sd") {
        return (1..=3).contains(&rest.len()) && rest.chars().all(|c| c.is_ascii_lowercase());
    }
    if let Some(rest) = name.strip_prefix("nvme") {
        return rest
            .split_once('n')
            .is_some_and(|(a, b)| digits(a) && digits(b));
    }
    name.strip_prefix("mmcblk").is_some_and(digits)
}

/// The summary of `dmidecode -t memory`: one entry per filled slot.
pub fn memory_modules(text: &str) -> Vec<BTreeMap<String, String>> {
    let wanted = [
        "Size",
        "Type",
        "Speed",
        "Configured Memory Speed",
        "Form Factor",
        "Manufacturer",
        "Locator",
        "Part Number",
    ];
    let mut out = Vec::new();
    for block in text.split("\n\n").filter(|b| b.contains("Memory Device")) {
        let mut m = BTreeMap::new();
        for line in block.lines() {
            if let Some((k, v)) = line.trim().split_once(": ")
                && wanted.contains(&k)
            {
                m.insert(k.to_owned(), v.trim().to_owned());
            }
        }
        m.insert(
            "Filled".into(),
            (!m.get("Size").is_some_and(|s| s.starts_with("No Module"))).to_string(),
        );
        out.push(m);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_password_status_without_hashes() {
        let shadow = "root:!:19800:0:99999:7:::\nriya:$y$j9T$abc$def:19850:0:90:7:::\nguest::19000:0:99999:7:::\ndaemon:*:19000:0:99999:7:::\n";
        let users = human_users(
            "root:x:0:0::/root:/bin/bash\ndaemon:x:1:1::/:/usr/sbin/nologin\nriya:x:1000:1000::/home/riya:/bin/bash\nguest:x:1001:1001::/home/guest:/bin/bash\n",
        );
        let s = shadow_status(shadow, &users);
        assert_eq!(s.len(), 3);
        assert!(s[0].locked);
        assert_eq!(
            (s[1].locked, s[1].max_days, s[1].changed_day),
            (false, Some(90), Some(19850))
        );
        assert!(s[2].empty);
        let json = serde_json::to_string(&s).unwrap();
        assert!(!json.contains("$y$"), "hashes never leave the helper");
    }

    #[test]
    fn accepts_only_disk_names() {
        for good in ["sda", "nvme0n1", "mmcblk0", "sdab"] {
            assert!(is_disk(good), "{good}");
        }
        for bad in [
            "sda1",
            "../sda",
            "nvme0",
            "loop0",
            "dm-0",
            "sd",
            "nvme0n1p2",
        ] {
            assert!(!is_disk(bad), "{bad}");
        }
    }

    #[test]
    fn summarizes_memory_modules() {
        let text = "Handle 0x0001\nMemory Device\n\tSize: 8 GB\n\tType: LPDDR4\n\tSpeed: 4267 MT/s\n\tForm Factor: Row Of Chips\n\tLocator: Channel-0\n\nHandle 0x0002\nMemory Device\n\tSize: No Module Installed\n\tLocator: DIMM 1\n";
        let m = memory_modules(text);
        assert_eq!(m.len(), 2);
        assert_eq!(m[0]["Type"], "LPDDR4");
        assert_eq!(m[1]["Filled"], "false");
    }
}
