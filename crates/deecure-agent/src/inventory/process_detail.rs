// SPDX-License-Identifier: Apache-2.0

//! Everything about one process, for the Expert level's process drawer
//! (docs/01-product/expert-coverage.md, section 9): where it comes from (parents, unit, app,
//! program file, package), how it is scheduled (priority, disk priority, CPUs it may use), its
//! limits and open files, its busiest threads, its sandbox (AppArmor, capabilities, seccomp,
//! namespaces), and its costs (memory, disk, graphics, faults). Other users' processes show
//! only what the kernel lets everyone read; each missing part says so by being absent.

use std::time::Duration;

use serde::Serialize;

use super::command;
use crate::collect::SystemRoot;

const LIMIT: Duration = Duration::from_secs(3);

/// A process in the chain from this one up to the first process.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Ancestor {
    pub pid: u32,
    pub name: String,
}

/// One thread with its CPU time.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Thread {
    pub tid: u32,
    pub name: String,
    pub cpu_seconds: f64,
}

/// One resource limit.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Limit {
    pub name: String,
    pub soft: String,
    pub hard: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

/// What `process_detail` answers.
#[derive(Debug, Default, Serialize)]
pub struct ProcessDetail {
    pub pid: u32,
    pub name: String,
    /// State letter: R running, S sleeping, D waiting for disk, Z zombie, …
    pub state: String,
    pub user_id: Option<u32>,
    /// Parent, its parent, and so on, nearest first.
    pub ancestors: Vec<Ancestor>,
    pub children: Vec<Ancestor>,
    pub command_line: Vec<String>,
    /// The program file and working folder (the user's own processes only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    /// The package that installed the program file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// The control group, such as `/user.slice/…/app-gnome-org.gnome.Nautilus-42.scope`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cgroup: Option<String>,
    /// When it started (ms since 1970) and the CPU time it used, in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<u64>,
    pub cpu_seconds: f64,
    pub nice: i64,
    /// `normal`, `batch`, `idle`, `fifo`, `round-robin`, or `deadline`.
    pub policy: String,
    pub realtime_priority: u64,
    /// From `ionice`, such as `best-effort: prio 4`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus_allowed: Option<String>,
    pub threads: u64,
    /// The busiest threads by CPU time, at most 10.
    pub top_threads: Vec<Thread>,
    pub voluntary_switches: Option<u64>,
    pub forced_switches: Option<u64>,
    /// Open files and how many of them are network connections (own processes only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_files: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sockets: Option<u64>,
    pub limits: Vec<Limit>,
    /// AppArmor label, such as `snap.firefox.firefox (enforce)` or `unconfined`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apparmor: Option<String>,
    /// Effective capabilities by name; empty for ordinary programs.
    pub capabilities: Vec<String>,
    /// `none`, `strict`, or `filter`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seccomp: Option<String>,
    pub no_new_privileges: bool,
    /// Namespaces it does not share with the agent, such as `net` or `mnt`: a sandbox or container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub own_namespaces: Option<Vec<String>>,
    pub rss: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pss: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uss: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap: Option<u64>,
    pub major_faults: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oom_score: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oom_score_adj: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_read: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_write: Option<u64>,
    /// Graphics busy time since it connected, in ns, summed over engines.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_ns: Option<u64>,
}

/// Linux capability names by bit number (`include/uapi/linux/capability.h`).
const CAPABILITIES: &[&str] = &[
    "chown",
    "dac_override",
    "dac_read_search",
    "fowner",
    "fsetid",
    "kill",
    "setgid",
    "setuid",
    "setpcap",
    "linux_immutable",
    "net_bind_service",
    "net_broadcast",
    "net_admin",
    "net_raw",
    "ipc_lock",
    "ipc_owner",
    "sys_module",
    "sys_rawio",
    "sys_chroot",
    "sys_ptrace",
    "sys_pacct",
    "sys_admin",
    "sys_boot",
    "sys_nice",
    "sys_resource",
    "sys_time",
    "sys_tty_config",
    "mknod",
    "lease",
    "audit_write",
    "audit_control",
    "setfcap",
    "mac_override",
    "mac_admin",
    "syslog",
    "wake_alarm",
    "block_suspend",
    "audit_read",
    "perfmon",
    "bpf",
    "checkpoint_restore",
];

/// Capability names from a `CapEff` hex mask.
pub fn capabilities(mask: &str) -> Vec<String> {
    let Ok(bits) = u64::from_str_radix(mask.trim(), 16) else {
        return Vec::new();
    };
    (0..64)
        .filter(|b| bits & (1 << b) != 0)
        .map(|b| {
            CAPABILITIES
                .get(b)
                .map_or_else(|| format!("cap_{b}"), |n| (*n).to_owned())
        })
        .collect()
}

/// The fields of `/proc/<pid>/stat` after the name in brackets (field 3 onwards).
fn stat_fields(stat: &str) -> Option<(String, Vec<String>)> {
    let name = stat.get(stat.find('(')? + 1..stat.rfind(')')?)?.to_owned();
    let rest = stat.get(stat.rfind(')')? + 2..)?;
    Some((name, rest.split_whitespace().map(str::to_owned).collect()))
}

/// `/proc/<pid>/limits`: columns are fixed-width; the name runs until two spaces.
pub fn parse_limits(text: &str) -> Vec<Limit> {
    text.lines()
        .skip(1)
        .filter_map(|line| {
            let name = line.get(..26)?.trim().to_owned();
            let rest: Vec<&str> = line.get(26..)?.split_whitespace().collect();
            Some(Limit {
                name,
                soft: rest.first()?.to_string(),
                hard: rest.get(1)?.to_string(),
                unit: rest.get(2).map(|u| u.to_string()),
            })
        })
        .collect()
}

fn status_value<'a>(status: &'a str, key: &str) -> Option<&'a str> {
    status
        .lines()
        .find_map(|l| l.strip_prefix(key)?.strip_prefix(':'))
        .map(str::trim)
}

fn policy_name(code: &str) -> &'static str {
    match code {
        "0" => "normal",
        "1" => "fifo",
        "2" => "round-robin",
        "3" => "batch",
        "5" => "idle",
        "6" => "deadline",
        _ => "unknown",
    }
}

/// Reads `process_detail` for `pid`. `Err` when the process does not exist (any more).
pub async fn read(root: &SystemRoot, pid: u32) -> Result<ProcessDetail, String> {
    let p = |f: &str| format!("/proc/{pid}/{f}");
    let stat = root
        .read(&p("stat"))
        .map_err(|_| format!("process {pid} is not running"))?;
    let (name, f) = stat_fields(&stat).ok_or("the process list could not be read")?;
    let field = |n: usize| f.get(n - 3).map(String::as_str);
    let num = |n: usize| field(n).and_then(|v| v.parse::<u64>().ok());
    let status = root.read(&p("status")).unwrap_or_default();
    let ticks = rustix::param::clock_ticks_per_second().max(1) as f64;

    // Parents, nearest first, up to the first process.
    let mut ancestors = Vec::new();
    let mut parent = num(4).unwrap_or(0) as u32;
    while parent > 0 && ancestors.len() < 32 {
        let Ok(s) = root.read(&format!("/proc/{parent}/stat")) else {
            break;
        };
        let Some((n, pf)) = stat_fields(&s) else {
            break;
        };
        ancestors.push(Ancestor {
            pid: parent,
            name: n,
        });
        parent = pf.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
    }
    let mut children = Vec::new();
    for e in std::fs::read_dir(root.path("/proc"))
        .into_iter()
        .flatten()
        .flatten()
    {
        let Some(child) = e.file_name().to_str().and_then(|n| n.parse::<u32>().ok()) else {
            continue;
        };
        if let Ok(s) = root.read(&format!("/proc/{child}/stat"))
            && let Some((n, cf)) = stat_fields(&s)
            && cf.get(1).and_then(|v| v.parse::<u32>().ok()) == Some(pid)
        {
            children.push(Ancestor {
                pid: child,
                name: n,
            });
        }
    }
    children.sort_by_key(|c| c.pid);

    let program = std::fs::read_link(root.path(&p("exe"))).ok().map(|x| {
        x.to_string_lossy()
            .trim_end_matches(" (deleted)")
            .to_owned()
    });
    let package = match &program {
        Some(path) => command::output_any("dpkg", &["-S", path], LIMIT)
            .await
            .and_then(|o| {
                o.lines()
                    .next()
                    .and_then(|l| l.split_once(':'))
                    .map(|(pkg, _)| pkg.trim().to_owned())
            }),
        None => None,
    };
    let boot_s: Option<u64> = root.read("/proc/stat").ok().and_then(|s| {
        s.lines()
            .find_map(|l| l.strip_prefix("btime ")?.trim().parse().ok())
    });
    let started_at = match (boot_s, num(22)) {
        (Some(b), Some(start)) => Some((b as f64 * 1000.0 + start as f64 / ticks * 1000.0) as u64),
        _ => None,
    };

    let mut top_threads: Vec<Thread> = std::fs::read_dir(root.path(&p("task")))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let tid: u32 = e.file_name().to_str()?.parse().ok()?;
            let (n, tf) = stat_fields(&root.read(&format!("/proc/{pid}/task/{tid}/stat")).ok()?)?;
            let t = |i: usize| {
                tf.get(i - 3)
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0)
            };
            Some(Thread {
                tid,
                name: n,
                cpu_seconds: (t(14) + t(15)) as f64 / ticks,
            })
        })
        .collect();
    top_threads.sort_by(|a, b| b.cpu_seconds.total_cmp(&a.cpu_seconds));
    top_threads.truncate(10);

    let fds: Option<Vec<String>> = std::fs::read_dir(root.path(&p("fd"))).ok().map(|d| {
        d.flatten()
            .filter_map(|e| std::fs::read_link(e.path()).ok())
            .map(|t| t.to_string_lossy().into_owned())
            .collect()
    });
    let gpu_ns = std::fs::read_dir(root.path(&p("fd"))).ok().map(|d| {
        let mut seen = std::collections::BTreeSet::new();
        d.flatten()
            .filter(|e| std::fs::read_link(e.path()).is_ok_and(|t| t.starts_with("/dev/dri/")))
            .filter_map(|e| {
                root.read(&p(&format!("fdinfo/{}", e.file_name().to_string_lossy())))
                    .ok()
            })
            .filter_map(|t| super::gpu_detail::parse_fdinfo(&t))
            .filter(|c| seen.insert((c.pdev.clone(), c.client.clone())))
            .map(|c| c.engines.values().sum::<u64>())
            .sum::<u64>()
    });

    let own_namespaces = ["mnt", "net", "pid", "user", "uts", "ipc", "cgroup"]
        .iter()
        .map(|ns| {
            let theirs = std::fs::read_link(root.path(&p(&format!("ns/{ns}")))).ok()?;
            let ours = std::fs::read_link(root.path(&format!("/proc/self/ns/{ns}"))).ok()?;
            Some((theirs != ours).then(|| (*ns).to_owned()))
        })
        .collect::<Option<Vec<_>>>()
        .map(|v| v.into_iter().flatten().collect());
    let rollup = root
        .read(&p("smaps_rollup"))
        .ok()
        .map(|t| super::memory_detail::parse_rollup(&t));
    let io = root
        .read(&p("io"))
        .ok()
        .and_then(|t| super::process_io::parse_io(&t));
    let apparmor = root
        .read(&p("attr/apparmor/current"))
        .or_else(|_| root.read(&p("attr/current")))
        .ok()
        .map(|a| a.trim_end_matches('\0').trim().to_owned())
        .filter(|a| !a.is_empty());

    Ok(ProcessDetail {
        pid,
        name,
        state: field(3).unwrap_or("?").to_owned(),
        user_id: status_value(&status, "Uid")
            .and_then(|u| u.split_whitespace().next()?.parse().ok()),
        ancestors,
        children,
        command_line: root
            .read(&p("cmdline"))
            .map(|c| {
                c.split('\0')
                    .filter(|a| !a.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        folder: std::fs::read_link(root.path(&p("cwd")))
            .ok()
            .map(|x| x.to_string_lossy().into_owned()),
        program,
        package,
        cgroup: root.read(&p("cgroup")).ok().and_then(|c| {
            c.lines()
                .find_map(|l| l.strip_prefix("0::").map(str::to_owned))
        }),
        started_at,
        cpu_seconds: (num(14).unwrap_or(0) + num(15).unwrap_or(0)) as f64 / ticks,
        nice: field(19).and_then(|v| v.parse().ok()).unwrap_or(0),
        policy: policy_name(field(41).unwrap_or("0")).to_owned(),
        realtime_priority: num(40).unwrap_or(0),
        disk_priority: command::output("ionice", &["-p", &pid.to_string()], LIMIT)
            .await
            .map(|o| o.trim().to_owned())
            .filter(|o| !o.is_empty()),
        cpus_allowed: status_value(&status, "Cpus_allowed_list").map(str::to_owned),
        threads: num(20).unwrap_or(0),
        top_threads,
        voluntary_switches: status_value(&status, "voluntary_ctxt_switches")
            .and_then(|v| v.parse().ok()),
        forced_switches: status_value(&status, "nonvoluntary_ctxt_switches")
            .and_then(|v| v.parse().ok()),
        open_files: fds.as_ref().map(|f| f.len() as u64),
        sockets: fds
            .as_ref()
            .map(|f| f.iter().filter(|t| t.starts_with("socket:")).count() as u64),
        limits: root
            .read(&p("limits"))
            .map(|t| parse_limits(&t))
            .unwrap_or_default(),
        apparmor,
        capabilities: status_value(&status, "CapEff")
            .map(capabilities)
            .unwrap_or_default(),
        seccomp: status_value(&status, "Seccomp").map(|s| match s {
            "0" => "none".to_owned(),
            "1" => "strict".to_owned(),
            "2" => "filter".to_owned(),
            other => other.to_owned(),
        }),
        no_new_privileges: status_value(&status, "NoNewPrivs") == Some("1"),
        own_namespaces,
        rss: root
            .read(&p("statm"))
            .ok()
            .and_then(|s| s.split_whitespace().nth(1)?.parse::<u64>().ok())
            .unwrap_or(0)
            * 4096,
        pss: rollup.and_then(|r| r.0),
        uss: rollup.and_then(|r| r.1),
        swap: rollup.and_then(|r| r.2),
        major_faults: num(12).unwrap_or(0),
        oom_score: root
            .read(&p("oom_score"))
            .ok()
            .and_then(|v| v.trim().parse().ok()),
        oom_score_adj: root
            .read(&p("oom_score_adj"))
            .ok()
            .and_then(|v| v.trim().parse().ok()),
        disk_read: io.map(|i| i.0),
        disk_write: io.map(|i| i.1),
        gpu_ns,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::test_support::fixture_root;

    #[test]
    fn capabilities_and_limits() {
        assert!(capabilities("0000000000000000").is_empty());
        assert_eq!(capabilities("0000000000003000"), ["net_admin", "net_raw"]);
        assert_eq!(capabilities("zz"), Vec::<String>::new());
        let l = parse_limits(
            "Limit                     Soft Limit           Hard Limit           Units     \nMax open files            1024                 524288               files     \nMax cpu time              unlimited            unlimited            seconds   \n",
        );
        assert_eq!(
            l[0],
            Limit {
                name: "Max open files".into(),
                soft: "1024".into(),
                hard: "524288".into(),
                unit: Some("files".into())
            }
        );
        assert_eq!(l[1].soft, "unlimited");
    }

    #[tokio::test]
    async fn reads_a_process_with_its_family_and_threads() {
        // Fields after the name: state ppid pgrp session tty tpgid flags minflt cminflt majflt
        // cmajflt utime stime cutime cstime priority nice threads itrealvalue starttime …
        let stat = |pid: u32, name: &str, ppid: u32| {
            format!(
                "{pid} ({name}) S {ppid} 1 1 0 -1 0 10 0 7 0 300 100 0 0 20 5 3 0 1000 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n"
            )
        };
        let root = fixture_root(
            "process-detail",
            &[
                ("/proc/1/stat", &stat(1, "systemd", 0)),
                ("/proc/10/stat", &stat(10, "gnome-shell", 1)),
                ("/proc/20/stat", &stat(20, "Web Content", 10)),
                ("/proc/30/stat", &stat(30, "child", 20)),
                (
                    "/proc/20/status",
                    "Name:\tx\nUid:\t1000\t1000\t1000\t1000\nCpus_allowed_list:\t0-11\nCapEff:\t0000000000000000\nSeccomp:\t2\nNoNewPrivs:\t1\nvoluntary_ctxt_switches:\t50\nnonvoluntary_ctxt_switches:\t5\n",
                ),
                (
                    "/proc/20/cmdline",
                    "/usr/lib/firefox/firefox\0-contentproc\0",
                ),
                (
                    "/proc/20/cgroup",
                    "0::/user.slice/user-1000.slice/user@1000.service/app.slice/snap.firefox.firefox-1.scope\n",
                ),
                (
                    "/proc/20/limits",
                    "Limit                     Soft Limit           Hard Limit           Units     \nMax open files            1024                 524288               files     \n",
                ),
                ("/proc/20/oom_score", "800\n"),
                ("/proc/20/oom_score_adj", "300\n"),
                (
                    "/proc/20/attr/apparmor/current",
                    "snap.firefox.firefox (enforce)\n",
                ),
                ("/proc/20/task/20/stat", &stat(20, "Web Content", 10)),
                (
                    "/proc/20/task/21/stat",
                    "21 (Renderer) S 10 1 1 0 -1 0 0 0 0 0 900 100 0 0 20 5 1 0 1000\n",
                ),
                ("/proc/stat", "cpu 1 2 3\nbtime 1700000000\n"),
            ],
        );
        let d = read(&root, 20).await.unwrap();
        assert_eq!(d.name, "Web Content");
        assert_eq!(
            d.ancestors
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>(),
            ["gnome-shell", "systemd"]
        );
        assert_eq!(d.children.iter().map(|c| c.pid).collect::<Vec<_>>(), [30]);
        assert_eq!(d.command_line, ["/usr/lib/firefox/firefox", "-contentproc"]);
        assert_eq!((d.nice, d.threads, d.major_faults), (5, 3, 7));
        assert_eq!(d.seccomp.as_deref(), Some("filter"));
        assert!(d.no_new_privileges && d.capabilities.is_empty());
        assert_eq!(
            d.apparmor.as_deref(),
            Some("snap.firefox.firefox (enforce)")
        );
        assert_eq!((d.oom_score, d.oom_score_adj), (Some(800), Some(300)));
        assert_eq!(d.top_threads[0].name, "Renderer", "busiest thread first");
        assert_eq!(d.limits.len(), 1);
        assert!(d.cgroup.unwrap().ends_with("snap.firefox.firefox-1.scope"));
        assert!(d.started_at.unwrap() > 1_700_000_000_000);
        assert!(read(&root, 99).await.is_err(), "a process that is gone");
    }
}
