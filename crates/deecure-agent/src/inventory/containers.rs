// SPDX-License-Identifier: Apache-2.0

//! Containers and virtual machines: Docker (asked only when its socket exists), and
//! VMs from libvirt and VirtualBox. Only read-only commands are used.

use std::collections::HashMap;
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use super::command;
use crate::collect::SystemRoot;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Container {
    pub name: String,
    pub image: String,
    /// As Docker says it, such as "Up 12 hours (healthy)".
    pub status: String,
    /// `running`, `exited`, `paused`, `restarting`, `created`.
    pub state: String,
    /// Such as "0.0.0.0:5432->5432/tcp".
    pub ports: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Image {
    pub repository: String,
    pub tag: String,
    /// As Docker says it, such as "432MB".
    pub size: String,
    pub created: String,
    pub containers: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Volume {
    pub name: String,
    pub size: String,
    /// How many containers use it.
    pub links: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Vm {
    pub name: String,
    /// `libvirt (system)`, `libvirt (user)`, or `VirtualBox`.
    pub engine: String,
    pub running: bool,
}

/// The `containers` inventory.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ContainersInfo {
    /// `running`, `not running`, `no access` (user not in the docker group), or `not installed`.
    pub docker: String,
    pub containers: Vec<Container>,
    pub images: Vec<Image>,
    pub volumes: Vec<Volume>,
    pub vms: Vec<Vm>,
}

fn lines_json(text: &str) -> Vec<Value> {
    text.lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

fn s(v: &Value, k: &str) -> String {
    v.get(k).and_then(Value::as_str).unwrap_or("").to_owned()
}

/// Parses `docker ps -a --format '{{json .}}'` and `docker stats --no-stream --format '{{json .}}'`.
pub fn parse_containers(ps: &str, stats: &str) -> Vec<Container> {
    let usage: HashMap<String, (String, String)> = lines_json(stats)
        .iter()
        .map(|v| {
            (
                s(v, "Name"),
                (
                    s(v, "CPUPerc"),
                    s(v, "MemUsage")
                        .split(" / ")
                        .next()
                        .unwrap_or("")
                        .to_owned(),
                ),
            )
        })
        .collect();
    let mut out: Vec<Container> = lines_json(ps)
        .iter()
        .map(|v| {
            let name = s(v, "Names");
            let u = usage.get(&name);
            Container {
                image: s(v, "Image"),
                status: s(v, "Status"),
                state: s(v, "State"),
                ports: s(v, "Ports")
                    .split(", ")
                    .filter(|p| !p.is_empty())
                    .map(str::to_owned)
                    .collect(),
                cpu: u.map(|u| u.0.clone()).filter(|c| !c.is_empty()),
                memory: u.map(|u| u.1.clone()).filter(|m| !m.is_empty()),
                name,
            }
        })
        .collect();
    out.sort_by(|a, b| {
        (a.state != "running")
            .cmp(&(b.state != "running"))
            .then_with(|| a.name.cmp(&b.name))
    });
    out
}

/// Parses `docker system df -v --format '{{json .}}'`.
pub fn parse_df(text: &str) -> (Vec<Image>, Vec<Volume>) {
    let Ok(v) = serde_json::from_str::<Value>(text.trim()) else {
        return (Vec::new(), Vec::new());
    };
    let images = v
        .get("Images")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|i| Image {
            repository: s(i, "Repository"),
            tag: s(i, "Tag"),
            size: s(i, "Size"),
            created: s(i, "CreatedSince"),
            containers: s(i, "Containers").parse().unwrap_or(0),
        })
        .collect();
    let volumes = v
        .get("Volumes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|x| Volume {
            name: s(x, "Name"),
            size: s(x, "Size"),
            links: s(x, "Links").parse().unwrap_or(0),
        })
        .collect();
    (images, volumes)
}

/// Parses `virsh list --all`: " Id   Name     State".
pub fn parse_virsh(text: &str, engine: &str) -> Vec<Vm> {
    text.lines()
        .skip(2)
        .filter_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            let name = (*f.get(1)?).to_owned();
            Some(Vm {
                name,
                engine: engine.to_owned(),
                running: f[2..].join(" ") == "running",
            })
        })
        .collect()
}

/// Parses `VBoxManage list vms`: `"win11-test" {uuid}`.
pub fn parse_vbox(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| l.split('"').nth(1))
        .map(str::to_owned)
        .collect()
}

const LIMIT: Duration = Duration::from_secs(15);

pub async fn read(root: &SystemRoot) -> ContainersInfo {
    let mut info = ContainersInfo::default();
    info.docker = if !root.path("/usr/bin/docker").exists() {
        "not installed".into()
    } else if !root.path("/var/run/docker.sock").exists() {
        "not running".into()
    } else {
        match command::output("docker", &["ps", "-a", "--format", "{{json .}}"], LIMIT).await {
            None => "no access".into(),
            Some(ps) => {
                let stats = command::output(
                    "docker",
                    &["stats", "--no-stream", "--format", "{{json .}}"],
                    LIMIT,
                )
                .await
                .unwrap_or_default();
                info.containers = parse_containers(&ps, &stats);
                let df = command::output(
                    "docker",
                    &["system", "df", "-v", "--format", "{{json .}}"],
                    LIMIT,
                )
                .await
                .unwrap_or_default();
                (info.images, info.volumes) = parse_df(&df);
                "running".into()
            }
        }
    };
    if root.path("/usr/bin/virsh").exists() {
        for (uri, engine) in [
            ("qemu:///system", "libvirt (system)"),
            ("qemu:///session", "libvirt (user)"),
        ] {
            if let Some(text) = command::output("virsh", &["-c", uri, "list", "--all"], LIMIT).await
            {
                info.vms.extend(parse_virsh(&text, engine));
            }
        }
    }
    if root.path("/usr/bin/VBoxManage").exists() {
        let all = parse_vbox(
            &command::output("VBoxManage", &["list", "vms"], LIMIT)
                .await
                .unwrap_or_default(),
        );
        let running = parse_vbox(
            &command::output("VBoxManage", &["list", "runningvms"], LIMIT)
                .await
                .unwrap_or_default(),
        );
        info.vms.extend(all.into_iter().map(|name| Vm {
            running: running.contains(&name),
            name,
            engine: "VirtualBox".into(),
        }));
    }
    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_docker_and_vms() {
        let ps = r#"{"Names":"redis-dev","Image":"redis:7","Status":"Up 12 hours","State":"running","Ports":"127.0.0.1:6379->6379/tcp"}
{"Names":"old","Image":"x","Status":"Exited (0) 2 days ago","State":"exited","Ports":""}
{"Names":"pg-dev","Image":"postgres:16","Status":"Up 12 hours","State":"running","Ports":"0.0.0.0:5432->5432/tcp, :::5432->5432/tcp"}"#;
        let stats = r#"{"Name":"pg-dev","CPUPerc":"0.20%","MemUsage":"142MiB / 7.6GiB"}"#;
        let c = parse_containers(ps, stats);
        assert_eq!(
            c.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(),
            ["pg-dev", "redis-dev", "old"]
        );
        assert_eq!(
            (c[0].ports.len(), c[0].memory.as_deref()),
            (2, Some("142MiB"))
        );
        let (images, volumes) = parse_df(
            r#"{"Images":[{"Repository":"postgres","Tag":"16","Size":"432MB","CreatedSince":"3 weeks ago","Containers":"1"}],"Volumes":[{"Name":"pgdata","Size":"980MB","Links":"1"}]}"#,
        );
        assert_eq!(
            (images[0].containers, volumes[0].name.as_str()),
            (1, "pgdata")
        );
        let vms = parse_virsh(
            " Id   Name        State\n-----------------------------\n 1    ubuntu-vm   running\n -    win11       shut off\n",
            "libvirt (system)",
        );
        assert_eq!(
            vms.iter()
                .map(|v| (v.name.as_str(), v.running))
                .collect::<Vec<_>>(),
            [("ubuntu-vm", true), ("win11", false)]
        );
        assert_eq!(parse_vbox("\"win11-test\" {1234}\n"), ["win11-test"]);
    }
}
