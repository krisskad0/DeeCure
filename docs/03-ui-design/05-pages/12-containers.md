# Containers (`containers`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Workloads · Icon: `cube` · Status badge: {warn:Warning} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Containers, Images, Volumes, Virtual Machines

## Containers › Containers: `containers/containers`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Running | 2 |  | - |   | - | - |
| Stopped | 1 |  | - |   | - | - |
| Images | 11 |  | - | 9.0 GB | - | - |
| Volumes | 3 |  | - | 1.1 GB | - | - |
| Exposed ports | 1 |  | {warn:Warning} | 5432 on all interfaces | - | - |

2. **Data table**: "Docker containers" · span 12
- Source chip (Expert): `docker ps -a`
- Inspect: row click opens the row-inspect drawer with actions "Restart", "Stop", "Logs"

| Name | Image | Status | Ports | CPU [n] | Memory [n] |
| --- | --- | --- | --- | --- | --- |
| pg-dev | `postgres:16` | {ok:Up 12 hours} | `0.0.0.0:5432→5432` {warn:LAN} | 0.2% | 142 MB |
| redis-dev | `redis:7-alpine` | {ok:Up 12 hours} | `127.0.0.1:6379→6379` | 0.1% | 12 MB |
| api-test | `node:20-alpine` | {off:Exited (0) 2 days ago} | - | - | - |

3. **Alert banner**: "Port 5432 bypasses the UFW firewall" · span 12
- State: `warn` · Title: "Port 5432 bypasses the UFW firewall" · Text: "Docker publishes ports with its own iptables rules. Bind to 127.0.0.1 to keep it local." · Buttons: [View firewall](go:network/firewall)

## Containers › Images: `containers/images`

1. **Donut chart**: "Image storage" · span 4
- Center: "9.0" / "GB" · unit "GB"

| Segment | Value | Color | Display value |
| --- | --- | --- | --- |
| In use | 5 | slot 1 | - |
| Unused | 4 | kw | - |

2. **Data table**: "Images" · span 8
- Source chip (Expert): `docker images`
- Header actions: [Prune unused · 4.0 GB](act:fix:storage)

| Repository | Tag | Size [n] | Created | In use |
| --- | --- | --- | --- | --- |
| postgres | 16 | 432 MB | 3 weeks ago | ✓ |
| redis | 7-alpine | 41 MB | 1 month ago | ✓ |
| node | 20-alpine | 135 MB | 2 weeks ago | ✓ |
| node | 18 | 1.1 GB | 4 months ago | - |
| python | 3.11 | 1.0 GB | 5 months ago | - |
| mongo | 7 | 760 MB | 3 months ago | - |
| nginx | alpine | 48 MB | 2 months ago | - |
| <none> | <none> | 1.1 GB | dangling | - |

## Containers › Volumes: `containers/volumes`

1. **Data table**: "Volumes" · span 12
- Source chip (Expert): `docker volume ls · docker system df -v`

| Name | Driver | Size [n] | Used by |
| --- | --- | --- | --- |
| `pgdata` | local | 980 MB | pg-dev |
| `redisdata` | local | 64 MB | redis-dev |
| `api-node-modules` | local | 86 MB | api-test |

## Containers › Virtual Machines: `containers/vms`

1. **Data table**: "Virtual machines" · span 12
- Source chip (Expert): `VBoxManage list vms · virsh list --all`

| Name | Engine | Guest OS | State | vCPU [n] | Memory [n] | Disk [n] |
| --- | --- | --- | --- | --- | --- | --- |
| win11-test | VirtualBox 7.0 | Windows 11 | {off:Powered off} | 2 | 4 GB | 2.9 GB |

---

← Previous: [Scheduled Jobs](11-jobs.md) · [Index](README.md) · Next: [Packages](13-packages.md) →
