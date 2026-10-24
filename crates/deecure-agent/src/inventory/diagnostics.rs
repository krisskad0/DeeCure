// SPDX-License-Identifier: Apache-2.0

//! On-demand checks the user starts: why the computer is slow, and whether the network
//! works. They only measure: a second of sampling, a few pings, a DNS lookup, and short
//! TCP connections. Nothing is changed.

use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use serde::Serialize;

use super::command;
use super::processes::ProcessSampler;
use crate::collect::{Collector, Sensors, SystemRoot};
use crate::context::Agent;

/// One measured step of a check.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Step {
    pub name: String,
    /// `ok`, `warn`, `crit`, or `na`.
    pub result: String,
    /// The measured value in words, such as "23% of the time".
    pub value: String,
}

/// What a check found.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Report {
    pub steps: Vec<Step>,
    /// One sentence: the main finding.
    pub verdict: String,
    /// `ok`, `warn`, or `crit`.
    pub tone: String,
    /// Busiest programs during the check (performance check only).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub top: Vec<(String, f64)>,
}

fn step(name: &str, result: &str, value: String) -> Step {
    Step {
        name: name.into(),
        result: result.into(),
        value,
    }
}

fn grade(value: f64, warn: f64, crit: f64) -> &'static str {
    if value >= crit {
        "crit"
    } else if value >= warn {
        "warn"
    } else {
        "ok"
    }
}

fn cpu_busy(stat: &str) -> Option<(u64, u64)> {
    let f: Vec<u64> = stat
        .lines()
        .next()?
        .split_whitespace()
        .skip(1)
        .filter_map(|v| v.parse().ok())
        .collect();
    let idle = f.get(3)? + f.get(4).unwrap_or(&0);
    Some((f.iter().take(8).sum::<u64>(), idle))
}

fn vmstat(text: &str, key: &str) -> u64 {
    text.lines()
        .find_map(|l| l.strip_prefix(key)?.trim().parse().ok())
        .unwrap_or(0)
}

fn some_avg10(text: &str) -> Option<f64> {
    text.lines()
        .find(|l| l.starts_with("some "))?
        .split_whitespace()
        .find_map(|f| f.strip_prefix("avg10="))?
        .parse()
        .ok()
}

/// Finds the main reason the computer is slow.
pub async fn performance(agent: &Agent) -> Report {
    let root: &SystemRoot = &agent.root;
    let read = |p: &str| root.read(p).unwrap_or_default();
    let (stat0, vm0, disk0) = (
        read("/proc/stat"),
        read("/proc/vmstat"),
        read("/proc/diskstats"),
    );
    let mut sampler = ProcessSampler::default();
    let _ = sampler.read(root);
    let start = Instant::now();
    tokio::time::sleep(Duration::from_millis(1500)).await;
    let seconds = start.elapsed().as_secs_f64();
    let (stat1, vm1, disk1) = (
        read("/proc/stat"),
        read("/proc/vmstat"),
        read("/proc/diskstats"),
    );
    let procs = sampler.read(root).unwrap_or_default();

    let cpu = match (cpu_busy(&stat0), cpu_busy(&stat1)) {
        (Some((t0, i0)), Some((t1, i1))) if t1 > t0 => {
            100.0 * (1.0 - (i1 - i0) as f64 / (t1 - t0) as f64)
        }
        _ => 0.0,
    };
    let meminfo = read("/proc/meminfo");
    let kb = |k: &str| {
        meminfo
            .lines()
            .find_map(|l| {
                l.strip_prefix(k)?
                    .trim()
                    .trim_end_matches("kB")
                    .trim()
                    .parse::<f64>()
                    .ok()
            })
            .unwrap_or(0.0)
    };
    let mem = if kb("MemTotal:") > 0.0 {
        100.0 * (1.0 - kb("MemAvailable:") / kb("MemTotal:"))
    } else {
        0.0
    };
    let pressure = some_avg10(&read("/proc/pressure/memory"));
    let swap = (vmstat(&vm1, "pswpin ") + vmstat(&vm1, "pswpout "))
        .saturating_sub(vmstat(&vm0, "pswpin ") + vmstat(&vm0, "pswpout ")) as f64
        / seconds;
    // Busy time of the busiest whole disk, from field 13 of /proc/diskstats (ms doing I/O).
    let busy = |text: &str| -> Vec<(String, u64)> {
        text.lines()
            .filter_map(|l| {
                let f: Vec<&str> = l.split_whitespace().collect();
                let name = *f.get(2)?;
                root.path(&format!("/sys/block/{name}/device"))
                    .exists()
                    .then(|| {
                        (
                            name.to_owned(),
                            f.get(12).and_then(|v| v.parse().ok()).unwrap_or(0),
                        )
                    })
            })
            .collect()
    };
    let (b0, b1) = (busy(&disk0), busy(&disk1));
    let disk = b1
        .iter()
        .filter_map(|(n, v)| {
            b0.iter()
                .find(|(m, _)| m == n)
                .map(|(_, w)| v.saturating_sub(*w) as f64 / (seconds * 10.0))
        })
        .fold(0.0f64, f64::max)
        .min(100.0);
    let temp = Sensors::default()
        .collect(root, deecure_core::Timestamp::now())
        .ok()
        .and_then(|s| {
            s.metrics
                .iter()
                .find(|m| m.name == "temp.cpu")
                .map(|m| m.value)
        });

    let mut steps = vec![
        step(
            "CPU load",
            grade(cpu, 70.0, 90.0),
            format!("{cpu:.0}% busy"),
        ),
        step("Memory", grade(mem, 85.0, 95.0), format!("{mem:.0}% used")),
        match pressure {
            Some(p) => step(
                "Memory pressure",
                grade(p, 5.0, 20.0),
                format!("tasks waited {p:.1}% of the time"),
            ),
            None => step(
                "Memory pressure",
                "na",
                "not reported by this kernel".into(),
            ),
        },
        step(
            "Swap activity",
            grade(swap, 50.0, 500.0),
            format!("{swap:.0} pages per second"),
        ),
        step(
            "Disk I/O",
            grade(disk, 60.0, 90.0),
            format!("busiest disk {disk:.0}% busy"),
        ),
        match temp {
            Some(t) => step(
                "Temperature",
                grade(t, 85.0, 95.0),
                format!("CPU {t:.0} °C"),
            ),
            None => step("Temperature", "na", "no CPU sensor".into()),
        },
    ];
    let top: Vec<(String, f64)> = procs
        .iter()
        .take(5)
        .map(|p| (p.name.clone(), p.cpu_percent))
        .collect();
    let hog = top.first().filter(|(_, c)| *c >= 50.0).cloned();
    steps.push(step(
        "Top processes",
        if hog.is_some() { "warn" } else { "ok" },
        top.iter()
            .take(3)
            .map(|(n, c)| format!("{n} {c:.0}%"))
            .collect::<Vec<_>>()
            .join(", "),
    ));

    let worst = |name: &str| {
        steps
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.result.as_str())
            == Some("crit")
    };
    let (verdict, tone) = if worst("Memory pressure")
        || worst("Swap activity")
        || (worst("Memory") && swap > 50.0)
    {
        ("Memory is the bottleneck: apps wait for memory and the system swaps to disk. Close large apps or add memory.".to_owned(), "crit")
    } else if worst("Temperature") {
        (
            "The CPU is very hot and slows itself down to cool off. Check the fans and air vents."
                .into(),
            "crit",
        )
    } else if worst("Disk I/O") {
        (
            "The disk is busy almost all the time, so apps wait for it.".into(),
            "crit",
        )
    } else if worst("CPU load") {
        (
            match &hog {
                Some((n, c)) => {
                    format!("The processor is busy, mostly with {n} ({c:.0}% of one CPU).")
                }
                None => "The processor is busy with many programs at once.".into(),
            },
            "crit",
        )
    } else if steps.iter().any(|s| s.result == "warn") {
        (
            "Something is a little high; see the steps below. Nothing is critical.".into(),
            "warn",
        )
    } else {
        ("No bottleneck found. If the computer feels slow, run the check again while it happens.".into(), "ok")
    };
    Report {
        steps,
        verdict,
        tone: tone.into(),
        top,
    }
}

/// Average round trip and loss from `ping -c N` output.
pub fn parse_ping(text: &str) -> (Option<f64>, Option<f64>) {
    let loss = text.lines().find_map(|l| {
        l.split(", ")
            .find(|p| p.ends_with("packet loss"))
            .and_then(|p| p.trim_end_matches("% packet loss").parse().ok())
    });
    let avg = text
        .lines()
        .find_map(|l| l.strip_prefix("rtt min/avg/max/mdev = "))
        .and_then(|v| v.split('/').nth(1)?.parse().ok());
    (avg, loss)
}

fn connect_ms(addr: SocketAddr) -> Option<f64> {
    let start = Instant::now();
    TcpStream::connect_timeout(&addr, Duration::from_secs(3)).ok()?;
    Some(start.elapsed().as_secs_f64() * 1000.0)
}

/// Checks the way to the internet, one hop at a time.
pub async fn network(agent: &Agent) -> Report {
    let gateway =
        super::net::default_gateway(&agent.root.read("/proc/net/route").unwrap_or_default());
    let mut steps = Vec::new();
    match &gateway {
        Some(g) => {
            let (avg, loss) = parse_ping(
                &command::output_any(
                    "ping",
                    &["-c", "3", "-i", "0.3", "-W", "1", g],
                    Duration::from_secs(6),
                )
                .await
                .unwrap_or_default(),
            );
            steps.push(match avg {
                Some(ms) => step(
                    "Gateway",
                    grade(ms, 20.0, 100.0),
                    format!("{g} answers in {ms:.0} ms"),
                ),
                None => step("Gateway", "crit", format!("{g} does not answer")),
            });
            if let Some(l) = loss {
                steps.push(step(
                    "Packet loss",
                    grade(l, 1.0, 10.0),
                    format!("{l:.0}% lost to the gateway"),
                ));
            }
        }
        None => steps.push(step(
            "Gateway",
            "crit",
            "no default route: not connected to a network".into(),
        )),
    }
    let started = Instant::now();
    let resolved = tokio::task::spawn_blocking(|| {
        "ubuntu.com:443"
            .to_socket_addrs()
            .map(|a| a.collect::<Vec<_>>())
    })
    .await
    .ok()
    .and_then(Result::ok);
    let dns_ms = started.elapsed().as_secs_f64() * 1000.0;
    steps.push(match &resolved {
        Some(list) if !list.is_empty() => step(
            "DNS resolution",
            grade(dns_ms, 200.0, 1000.0),
            format!("ubuntu.com found in {dns_ms:.0} ms"),
        ),
        _ => step("DNS resolution", "crit", "names cannot be looked up".into()),
    });
    let v4 = resolved
        .as_ref()
        .and_then(|l| l.iter().find(|a| a.is_ipv4()).copied());
    let latency = match v4 {
        Some(a) => tokio::task::spawn_blocking(move || connect_ms(a))
            .await
            .ok()
            .flatten(),
        None => None,
    };
    steps.push(match latency {
        Some(ms) => step(
            "Internet latency",
            grade(ms, 150.0, 500.0),
            format!("{ms:.0} ms to ubuntu.com"),
        ),
        None => step(
            "Internet latency",
            "crit",
            "cannot reach the internet".into(),
        ),
    });
    let v6 = resolved
        .as_ref()
        .and_then(|l| l.iter().find(|a| a.is_ipv6()).copied());
    let ipv6 = match v6 {
        Some(a) => tokio::task::spawn_blocking(move || connect_ms(a))
            .await
            .ok()
            .flatten(),
        None => None,
    };
    steps.push(match ipv6 {
        Some(ms) => step("IPv6", "ok", format!("works ({ms:.0} ms)")),
        None => step(
            "IPv6",
            "na",
            "not available on this network (this is common)".into(),
        ),
    });
    let failed: Vec<&Step> = steps.iter().filter(|s| s.result == "crit").collect();
    let (verdict, tone) = match failed.first() {
        Some(s) if s.name == "Gateway" => (
            "The router does not answer. Check Wi-Fi or the cable.".to_owned(),
            "crit",
        ),
        Some(s) if s.name == "DNS resolution" => (
            "The network works, but names cannot be looked up. Try another DNS server.".to_owned(),
            "crit",
        ),
        Some(_) => (
            "The local network works, but the internet is not reachable.".to_owned(),
            "crit",
        ),
        None if steps.iter().any(|s| s.result == "warn") => (
            "The network works, but it is slow or loses packets.".to_owned(),
            "warn",
        ),
        None => (
            format!(
                "Network is healthy: all {} checks passed.",
                steps.iter().filter(|s| s.result == "ok").count()
            ),
            "ok",
        ),
    };
    Report {
        steps,
        verdict,
        tone: tone.into(),
        top: Vec::new(),
    }
}

/// Facts about DeeCure itself, for Settings › Data and About.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AgentInfo {
    pub version: String,
    pub database: String,
    /// The database and its write-ahead log, in bytes.
    pub database_bytes: u64,
    pub rules: usize,
    pub rules_dir: String,
    /// Seconds the agent has run.
    pub uptime_seconds: u64,
    /// Memory the agent uses, in bytes.
    pub memory_bytes: u64,
}

pub fn agent_info(agent: &Agent) -> Option<AgentInfo> {
    let facts = agent.facts.get()?;
    let size = |p: String| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
    let db = facts.database.to_string_lossy().into_owned();
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    Some(AgentInfo {
        version: deecure_core::VERSION.to_owned(),
        database_bytes: size(db.clone()) + size(format!("{db}-wal")),
        database: db,
        rules: facts.rules,
        rules_dir: facts.rules_dir.to_string_lossy().into_owned(),
        uptime_seconds: facts.started.elapsed().as_secs(),
        memory_bytes: status
            .lines()
            .find_map(|l| l.strip_prefix("VmRSS:"))
            .and_then(|v| v.trim().trim_end_matches("kB").trim().parse::<u64>().ok())
            .map_or(0, |kb| kb * 1024),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_ping_and_grades() {
        let out = "3 packets transmitted, 3 received, 0% packet loss, time 602ms\nrtt min/avg/max/mdev = 1.9/2.4/3.1/0.4 ms\n";
        assert_eq!(parse_ping(out), (Some(2.4), Some(0.0)));
        assert_eq!(
            parse_ping("3 packets transmitted, 0 received, 100% packet loss, time 2ms\n"),
            (None, Some(100.0))
        );
        assert_eq!(
            (
                grade(10.0, 20.0, 100.0),
                grade(50.0, 20.0, 100.0),
                grade(200.0, 20.0, 100.0)
            ),
            ("ok", "warn", "crit")
        );
        assert_eq!(
            some_avg10("some avg10=23.40 avg60=1 avg300=0 total=1\n"),
            Some(23.4)
        );
        assert_eq!(cpu_busy("cpu  100 0 50 800 50 0 0 0\n"), Some((1000, 850)));
    }
}
