// SPDX-License-Identifier: Apache-2.0

//! Docker containers with a port open to every network (`containers.exposed`), and those
//! ports (`containers.published_port`): the ufw firewall does not filter them.
//! Asked only when Docker is installed and running; without Docker nothing is exposed.

use std::collections::BTreeSet;
use std::io;
use std::time::Duration;

use deecure_core::{Timestamp, Unit};

use super::{Sample, SystemRoot, metric};
use crate::inventory::command;

const LIMIT: Duration = Duration::from_secs(20);

/// Running containers whose ports line (`docker ps --format {{.Ports}}`) maps a port on
/// every address: `0.0.0.0:5432->5432/tcp`, `:::5432->…`, or `[::]:5432->…`.
pub fn exposed(ports: &str) -> usize {
    ports
        .lines()
        .filter(|line| {
            line.split(", ").any(|p| {
                p.contains("->")
                    && (p.starts_with("0.0.0.0:") || p.starts_with(":::") || p.starts_with("[::]:"))
            })
        })
        .count()
}

/// The host ports that running containers publish on every address.
pub fn published_ports(ports: &str) -> BTreeSet<u16> {
    ports
        .lines()
        .flat_map(|line| line.split(", "))
        .filter_map(|p| {
            let (host, _) = p.split_once("->")?;
            let rest = host
                .strip_prefix("0.0.0.0:")
                .or_else(|| host.strip_prefix(":::"))
                .or_else(|| host.strip_prefix("[::]:"))?;
            rest.parse().ok()
        })
        .collect()
}

/// Counts exposed containers.
#[derive(Debug, Default)]
pub struct Containers;

impl Containers {
    /// The name used in logs.
    pub const NAME: &'static str = "containers";

    /// Asks Docker for running containers. A user who may not talk to Docker gets an
    /// error (and no value), not a "nothing exposed" that might be wrong.
    pub async fn collect(&mut self, root: &SystemRoot, now: Timestamp) -> io::Result<Sample> {
        let installed = root.path("/usr/bin/docker").exists();
        let running = root.path("/var/run/docker.sock").exists();
        let ports = if !installed || !running {
            String::new()
        } else {
            command::output("docker", &["ps", "--format", "{{.Ports}}"], LIMIT)
                .await
                .ok_or_else(|| {
                    io::Error::other("docker ps failed (is this user in the docker group?)")
                })?
        };
        let mut metrics = vec![metric(
            "containers.exposed",
            None,
            exposed(&ports) as f64,
            Unit::Count,
            now,
        )];
        for port in published_ports(&ports) {
            metrics.push(metric(
                "containers.published_port",
                Some(&port.to_string()),
                1.0,
                Unit::Count,
                now,
            ));
        }
        Ok(Sample {
            metrics,
            snapshots: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_containers_open_to_every_network() {
        let ps = "0.0.0.0:5432->5432/tcp, :::5432->5432/tcp
127.0.0.1:6379->6379/tcp

80/tcp
[::]:8080->80/tcp
";
        assert_eq!(exposed(ps), 2);
        assert_eq!(exposed(""), 0);
        assert_eq!(published_ports(ps), BTreeSet::from([5432, 8080]));
        assert!(published_ports("").is_empty());
    }
}
