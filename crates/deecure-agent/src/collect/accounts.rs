// SPDX-License-Identifier: Apache-2.0

//! Accounts from `/etc/passwd` (readable by everyone; `/etc/shadow` is never read).

use std::io;

use deecure_core::{Timestamp, Unit};

use super::{Collector, Sample, SystemRoot, invalid, metric};

/// Shells that mean "this account cannot sign in".
const NO_LOGIN: &[&str] = &["nologin", "false", "sync", "shutdown", "halt"];

/// `(people, extra_root)`: accounts a person can sign in with (UID 1000 and up, a real
/// shell, not `nobody`), and accounts other than `root` with UID 0 (full admin power).
pub fn count(passwd: &str) -> (usize, usize) {
    let mut people = 0;
    let mut extra_root = 0;
    for line in passwd.lines() {
        let f: Vec<&str> = line.split(':').collect();
        let (Some(name), Some(uid), Some(shell)) = (f.first(), f.get(2), f.get(6)) else {
            continue;
        };
        let Ok(uid) = uid.parse::<u32>() else {
            continue;
        };
        let shell = shell.rsplit('/').next().unwrap_or("");
        if uid == 0 && *name != "root" && !name.starts_with('#') {
            extra_root += 1;
        }
        if uid >= 1000 && uid != 65534 && !shell.is_empty() && !NO_LOGIN.contains(&shell) {
            people += 1;
        }
    }
    (people, extra_root)
}

/// `accounts.people` and `accounts.uid0_extra`.
#[derive(Debug, Default)]
pub struct Accounts;

impl Collector for Accounts {
    fn name(&self) -> &'static str {
        "accounts"
    }

    fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let passwd = root.read("/etc/passwd")?;
        if !passwd.lines().any(|l| l.starts_with("root:")) {
            return Err(invalid("/etc/passwd", "no root account"));
        }
        let (people, extra_root) = count(&passwd);
        Ok(Sample {
            metrics: vec![
                metric("accounts.people", None, people as f64, Unit::Count, now),
                metric(
                    "accounts.uid0_extra",
                    None,
                    extra_root as f64,
                    Unit::Count,
                    now,
                ),
            ],
            snapshots: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_people_and_extra_root_accounts() {
        let passwd = "root:x:0:0:root:/root:/bin/bash
daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin
riya:x:1000:1000:Riya,,,:/home/riya:/bin/bash
guest:x:1001:1001::/home/guest:/usr/bin/zsh
nobody:x:65534:65534:nobody:/nonexistent:/usr/sbin/nologin
backup:x:1002:1002::/home/backup:/bin/false
toor:x:0:0::/root:/bin/sh
broken line
";
        assert_eq!(count(passwd), (2, 1));
    }
}
