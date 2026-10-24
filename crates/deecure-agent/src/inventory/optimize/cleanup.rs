// SPDX-License-Identifier: Apache-2.0

//! Space, packages, updates, the firewall, and system settings (Optimize batch 2).

use std::collections::HashMap;

use deecure_core::{SnapshotKind, Timestamp};

use super::super::{kernel_detail, packages, software_detail, storage, storage_detail};
use super::{Check, count, fix};
use crate::collect::SystemRoot;
use crate::context::Agent;
use crate::store;

const MB: u64 = 1_000_000;
const GB: u64 = 1_000_000_000;

/// The size from which each cleanup part is offered (docs/01-product/optimize-catalog.md).
pub fn part_limit(action: &str) -> Option<u64> {
    Some(match action {
        "storage.empty_trash" | "storage.remove_old_snap_revisions" => GB,
        "storage.clear_thumbnails" | "storage.clean_apt_cache" | "storage.vacuum_journal" => {
            500 * MB
        }
        "storage.prune_docker_images" | "storage.prune_docker_build_cache" => 2 * GB,
        _ => return None,
    })
}

fn size(bytes: u64) -> String {
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else {
        format!("{} MB", bytes / MB)
    }
}

/// Points of a stored metric since `since_ms` ago, oldest first.
pub fn series(agent: &Agent, name: &str, since_ms: i64) -> Vec<(i64, f64)> {
    let now = Timestamp::now();
    store::lock(&agent.store)
        .metrics(name, None, Timestamp(now.0 - since_ms), now)
        .unwrap_or_default()
        .into_iter()
        .map(|m| (m.at.0, m.value))
        .collect()
}

/// The newest stored value of a metric from the last two hours.
pub fn latest(agent: &Agent, name: &str) -> Option<f64> {
    series(agent, name, 2 * 3_600_000).last().map(|(_, v)| *v)
}

/// True when a past fix run finished a step with this ID (or ID prefix), such as `journal`.
fn cleaned_before(agent: &Agent, step: &str) -> bool {
    store::lock(&agent.store)
        .fix_runs(200)
        .unwrap_or_default()
        .iter()
        .any(|r| r.steps.iter().any(|s| s.ok && s.id.starts_with(step)))
}

pub async fn checks(agent: &Agent) -> Vec<Check> {
    let root = &agent.root;
    let mut out = Vec::new();
    let space = storage::read(root).await;
    for c in &space.candidates {
        let Some(limit) = part_limit(&c.action) else {
            continue;
        };
        let over = c.size >= limit;
        out.push(fix(
            &format!("space:{}", c.action),
            &c.title,
            format!(
                "{} can be freed ({}); offered from {}.",
                size(c.size),
                c.details,
                size(limit)
            ),
            over,
            &c.action,
            None,
            if c.needs_password { "admin" } else { "you" },
        ));
    }
    let journal_used = space
        .candidates
        .iter()
        .find(|c| c.action == "storage.vacuum_journal")
        .map(|c| c.size + 200 * 1024 * 1024);
    out.push(journal_cap(agent, root, journal_used));
    let health = software_detail::read_health().await;
    out.push(snap_versions(agent, &space, &health));

    let pkgs = packages::read(root).await;
    for (action, title) in [
        ("packages.autoremove", "Remove packages no longer needed"),
        (
            "packages.purge_residual_config",
            "Remove leftover settings of removed packages",
        ),
    ] {
        let r = pkgs.removable.iter().find(|r| r.action == action);
        out.push(fix(
            &format!("packages:{action}"),
            title,
            match r {
                Some(r) if r.count > 0 => {
                    format!("{}: {}.", count(r.count, "package"), r.names.join(", "))
                }
                _ => "apt lists none.".into(),
            },
            r.is_some_and(|r| r.count > 0),
            action,
            None,
            "admin",
        ));
    }
    out.push(updates(agent));
    out.push(firewall(agent));
    out.push(fix(
        "packages.broken",
        "Finish a broken package install",
        if health.broken.is_empty() {
            "dpkg --audit reports no half-installed packages.".into()
        } else {
            format!("dpkg --audit reports: {}.", health.broken.join("; "))
        },
        !health.broken.is_empty(),
        "packages.finish_setup",
        None,
        "admin",
    ));
    out.extend(flatpaks(&health));
    out.push(file_watches(root).await);
    out.extend(hard_disks(root));
    out
}

fn journal_cap(agent: &Agent, root: &SystemRoot, used: Option<u64>) -> Check {
    let title = "Cap the system log size from now on";
    let capped = [
        "/etc/systemd/journald.conf",
        "/etc/systemd/journald.conf.d/50-deecure-size.conf",
    ]
    .iter()
    .filter_map(|f| root.read(f).ok())
    .any(|t| {
        t.lines()
            .any(|l| l.trim_start().starts_with("SystemMaxUse=") && !l.trim().ends_with('='))
    });
    let Some(used) = used else {
        return fix(
            "storage.journal_cap",
            title,
            "The log size is not known yet.".into(),
            false,
            "",
            None,
            "admin",
        );
    };
    let again = cleaned_before(agent, "journal");
    let offered = used > GB && again && !capped;
    fix(
        "storage.journal_cap",
        title,
        if capped {
            format!("A size cap is set; the log uses {}.", size(used))
        } else if used > GB && !again {
            format!(
                "The log uses {}. Shrink it once first (Storage › Cleanup); the cap is offered if it grows over 1 GB again.",
                size(used)
            )
        } else {
            format!(
                "The log uses {}{}.",
                size(used),
                if offered {
                    " again after it was shrunk, and no cap is set"
                } else {
                    ""
                }
            )
        },
        offered,
        "storage.cap_journal",
        Some("500".into()),
        "admin",
    )
}

fn snap_versions(
    agent: &Agent,
    space: &storage::StorageInfo,
    health: &software_detail::SoftwareHealth,
) -> Check {
    let title = "Keep fewer old snap versions from now on";
    let Some(snaps) = &health.snaps else {
        return fix(
            "storage.snap_retain",
            title,
            "snapd is not running.".into(),
            false,
            "",
            None,
            "admin",
        );
    };
    let mut per: HashMap<&str, u32> = HashMap::new();
    for s in snaps {
        *per.entry(s.name.as_str()).or_insert(0) += 1;
    }
    let most = per.values().copied().max().unwrap_or(0);
    let old = space
        .candidates
        .iter()
        .find(|c| c.action == "storage.remove_old_snap_revisions")
        .map_or(0, |c| c.size);
    let again = cleaned_before(agent, "snap");
    let offered = most >= 3 && old > GB && again;
    fix(
        "storage.snap_retain",
        title,
        if most < 3 {
            format!("snapd keeps at most {most} versions of a snap, the lowest it allows.")
        } else {
            format!(
                "snapd keeps up to {most} versions of a snap; old versions use {}{}.",
                size(old),
                if offered {
                    " again after a clean-up"
                } else {
                    ""
                }
            )
        },
        offered,
        "storage.keep_fewer_snaps",
        Some("2".into()),
        "admin",
    )
}

fn updates(agent: &Agent) -> Check {
    let info: Option<crate::collect::UpdatesInfo> = store::lock(&agent.store)
        .latest_snapshot(SnapshotKind::Updates)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_value(s.data).ok());
    let Some(info) = info else {
        return fix(
            "system.updates",
            "Install updates",
            "Updates were not checked yet.".into(),
            false,
            "",
            None,
            "admin",
        );
    };
    let security = info.pending.iter().filter(|p| p.security).count();
    let all = info.pending.len();
    if security > 0 {
        fix(
            "system.updates",
            "Install security updates",
            format!(
                "{} waiting ({} in all).",
                count(security, "security update"),
                count(all, "update")
            ),
            true,
            "security.install_security_updates",
            None,
            "admin",
        )
    } else {
        fix(
            "system.updates",
            "Install updates",
            if all == 0 {
                "Everything is up to date.".into()
            } else {
                format!(
                    "{} waiting, none of them security updates.",
                    count(all, "update")
                )
            },
            all > 0,
            "system.install_updates",
            None,
            "admin",
        )
    }
}

fn firewall(agent: &Agent) -> Check {
    let on = latest(agent, "firewall.enabled");
    fix(
        "security.firewall",
        "Turn on the firewall",
        match on {
            None => "ufw is not installed, or not read yet.".into(),
            Some(v) if v >= 1.0 => "The firewall (ufw) is on.".into(),
            Some(_) => "The firewall (ufw) is installed but off.".into(),
        },
        on == Some(0.0),
        "security.enable_firewall",
        None,
        "admin",
    )
}

fn flatpaks(health: &software_detail::SoftwareHealth) -> Vec<Check> {
    let Some(runtimes) = &health.runtimes else {
        return Vec::new();
    };
    ["user", "system"]
        .iter()
        .map(|inst| {
            let unused: Vec<String> = runtimes
                .iter()
                .filter(|r| !r.used && r.installation == *inst)
                .map(|r| format!("{} {}", r.id, r.branch))
                .collect();
            fix(
                &format!("packages.flatpak:{inst}"),
                &format!("Remove unused Flatpak runtimes ({inst})"),
                if unused.is_empty() {
                    format!("No unused {inst} runtimes.")
                } else {
                    format!("No installed app uses: {}.", unused.join(", "))
                },
                !unused.is_empty(),
                "packages.remove_unused_flatpaks",
                Some((*inst).to_owned()),
                if *inst == "system" { "admin" } else { "you" },
            )
        })
        .collect()
}

/// The new file-watch limit: twice the old one, at least 524288 (what many IDEs ask for).
pub fn raised_watch_limit(limit: u64) -> u64 {
    (limit * 2).clamp(524_288, 4_194_304)
}

async fn file_watches(root: &SystemRoot) -> Check {
    let k = kernel_detail::read(root).await;
    let title = "Raise the file-watch limit";
    let Some(limit) = k.watch_limit.filter(|l| *l > 0) else {
        return fix(
            "system.file_watches",
            title,
            "The file-watch limit is not readable.".into(),
            false,
            "",
            None,
            "admin",
        );
    };
    let pct = k.watches_used * 100 / limit;
    let new = raised_watch_limit(limit);
    fix(
        "system.file_watches",
        title,
        format!(
            "Your apps use {} of {limit} file watches ({pct}%){}.",
            k.watches_used,
            if pct > 80 {
                format!("; near the limit, apps stop seeing file changes. New limit: {new}")
            } else {
                String::new()
            }
        ),
        pct > 80 && new > limit,
        "system.raise_file_watches",
        Some(new.to_string()),
        "admin",
    )
}

/// Spinning disks (not USB sticks, which often say they spin) left on the `none` scheduler.
fn hard_disks(root: &SystemRoot) -> Vec<Check> {
    storage_detail::read(root)
        .disks
        .into_iter()
        .filter(|d| d.rotational)
        .filter(|d| {
            let removable = root
                .read(&format!("/sys/block/{}/removable", d.name))
                .is_ok_and(|v| v.trim() == "1");
            let usb = std::fs::canonicalize(root.path(&format!("/sys/block/{}", d.name)))
                .is_ok_and(|p| p.to_string_lossy().contains("/usb"));
            !removable && !usb
        })
        .map(|d| {
            let better = ["mq-deadline", "bfq"]
                .into_iter()
                .find(|s| d.schedulers.iter().any(|x| x == s));
            let offered = d.scheduler.as_deref() == Some("none") && better.is_some();
            fix(
                &format!("storage.scheduler:{}", d.name),
                &format!("Use a better scheduler for hard disk {}", d.name),
                format!(
                    "{} is a spinning disk using the {} scheduler{}.",
                    d.name,
                    d.scheduler.as_deref().unwrap_or("unknown"),
                    if offered {
                        "; on a hard disk, requests in disk order are much faster"
                    } else {
                        ""
                    }
                ),
                offered,
                "storage.set_disk_scheduler",
                better.map(|b| format!("{}|{b}", d.name)),
                "admin",
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_part_has_its_limit_and_the_watch_limit_doubles() {
        assert_eq!(part_limit("storage.empty_trash"), Some(GB));
        assert_eq!(part_limit("storage.vacuum_journal"), Some(500 * MB));
        assert_eq!(part_limit("storage.free_space"), None);
        assert_eq!(raised_watch_limit(65_536), 524_288);
        assert_eq!(raised_watch_limit(1_000_000), 2_000_000);
        assert_eq!(raised_watch_limit(4_000_000), 4_194_304);
    }
}
