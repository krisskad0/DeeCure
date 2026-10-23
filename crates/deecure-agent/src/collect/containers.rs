// SPDX-License-Identifier: Apache-2.0

//! Docker containers with a port open to every network (`containers.exposed`).
//! Asked only when Docker is installed and running; without Docker nothing is exposed.

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
        let count = if !installed || !running {
            0
        } else {
            let ports = command::output("docker", &["ps", "--format", "{{.Ports}}"], LIMIT)
                .await
                .ok_or_else(|| {
                    io::Error::other("docker ps failed (is this user in the docker group?)")
                })?;
            exposed(&ports)
        };
        Ok(Sample {
            metrics: vec![metric(
                "containers.exposed",
                None,
                count as f64,
                Unit::Count,
                now,
            )],
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
    }
}
