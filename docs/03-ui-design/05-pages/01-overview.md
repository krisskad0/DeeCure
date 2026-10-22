# Overview (`overview`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Monitoring · Icon: `grid` · Status badge: {crit:Critical} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Summary, Issues, Health Score

## Overview › Summary — `overview/summary`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Health score | 82 | /100 | {warn:Warning} | 9 open issues | Usage bar 82% | `overview/score` |
| CPU | 12 | % | — | Load 0.84 · 12 threads | Sparkline slot 1, gen[3, 14, 8, 6, 2, 60, 12] | `performance/cpu` |
| Memory | 5.9 | GB | — | 78% of 7.6 GB | Sparkline slot 2, gen[5, 70, 8, 3, 40, 95, 78] | `performance/memory` |
| Disk ( / ) | 94 | % | {crit:Critical} | 13.8 GB free of 238 GB | Usage bar 94.2% | `storage/overview` |
| Network | 1.2 | MB/s | — | ↑ 0.3 MB/s · Wi-Fi | Sparkline slot 3, gen[9, 0.8, 0.6, 0.8, 0, 6, 1.2] | `performance/network` |
| Battery | 64 | % | {warn:Warning} | Health 71% · 3 h 10 m left | Usage bar 64% | `power/battery` |

2. **Line chart** — "CPU and memory utilization" · span 8
- Info tooltip: "How busy the processor and memory were. Change the time range at the top to zoom out."
- Source chip (Expert): `/proc/stat · /proc/meminfo`
- Height 300px · Y axis 0 to 100 · unit "%"
- Series: CPU (slot 1, gen[3, 14, 8, 6, 2, 60, 12]); Memory (slot 2, gen[5, 70, 8, 3, 40, 95, 78])
- X axis: time (current range)

3. **Status list** — "Subsystems" · span 4
- Info tooltip: "Status of every area SysDoc checks. Select a row to open it."

| Icon | Label | Metric | Badge | Opens |
| --- | --- | --- | --- | --- |
| `chip` | Hardware | CPU 54 °C | {ok:Healthy} | `hardware/summary` |
| `bat` | Power | Health 71% | {warn:Warning} | `power/battery` |
| `sys` | System | Reboot required | {info:Info} | `system/overview` |
| `cog` | Services | 1 failed | {crit:Critical} | `services/system` |
| `cal` | Scheduled Jobs | 10 jobs | {ok:Healthy} | `jobs/calendar` |
| `pkg` | Packages | 12 updates | {warn:Warning} | `packages/installed` |
| `cube` | Containers | 1 exposed port | {warn:Warning} | `containers/containers` |
| `db` | Storage | 94% used | {crit:Critical} | `storage/overview` |
| `backup` | Backups | 21 days ago | {warn:Warning} | `backups/jobs` |
| `wifi` | Network | Wi-Fi −52 dBm | {ok:Healthy} | `network/interfaces` |
| `users` | Accounts | 2 users | {ok:Healthy} | `accounts/users` |
| `shield` | Security | 4 updates | {warn:Warning} | `security/overview` |
| `eye` | Privacy | No active access | {ok:Healthy} | `privacy/access` |

4. **Data table** — "Active issues" · span 8
- Subtitle: "9 open"
- Header actions: [View all](go:overview/issues) (link style)

| Severity | Issue | Category | Detected | (action) |
| --- | --- | --- | --- | --- |
| {crit:Critical} | **Root filesystem almost full** · / at 94% · 15.2 GB reclaimable | Storage | 14 Oct | [**Resolve**](act:fix:storage) |
| {crit:Critical} | **bluetooth.service crash loop** · 14 restarts today · SIGSEGV | Services | Today 09:12 | [**Inspect**](act:bt) |
| {warn:Warning} | **Security updates pending** · 4 packages · 2 high-severity CVEs | Security | Today 06:21 | [Install](act:fix:sec) |
| {warn:Warning} | **Battery health degraded** · 71% of design capacity | Power | 02 Sep | [View](go:power/battery) |
| {warn:Warning} | **Backup overdue** · Last successful backup 21 days ago | Backups | 07 Oct | [View](go:backups/jobs) |
| {warn:Warning} | **Out-of-memory kill** · chrome renderer · Mon 16:12 | Logs | Mon 16:12 | [View](go:logs/kernel) |

5. **Gauge + horizontal bars** — "Health score" · span 4
- Info tooltip: "Starts at 100. Each open issue removes points from its area."
- Header actions: [Details](go:overview/score) (link style)

Gauge: value **82**, subtitle "out of 100", arc color `warn` (by score: ≥90 ok, ≥75 warn, else crit). Below it, horizontal bars (max 10, warn color): Storage −8, Services −5, Security −3, Power −1, Backups −1 (only categories with a deduction).

6. **Donut chart** — "Disk usage ( / )" · span 4
- Subtitle: "238 GB"
- Source chip (Expert): `df -h / · du`
- Header actions: [Open](go:storage/overview) (link style)
- Center: "94%" / "used" · unit "GB"

| Segment | Value | Color | Display value |
| --- | --- | --- | --- |
| Home | 128 | slot 1 | — |
| Apps | 46 | slot 2 | — |
| System | 22 | slot 3 | — |
| Containers | 13 | slot 4 | — |
| Reclaimable | 15.2 | kc | — |
| Free | 13.8 | kn | — |

7. **Data table** — "Top processes" · span 4
- Subtitle: "by memory"
- Source chip (Expert): `ps -eo comm,%cpu,rss`
- Header actions: [View all](go:processes/all) (link style)

| Process | CPU [n] | Memory [n] |
| --- | --- | --- |
| chrome | 4.2% | 842 MB |
| chrome (renderer) | 2.6% | 612 MB |
| gnome-shell | 3.8% | 410 MB |
| code | 1.9% | 486 MB |
| telegram-desktop | 0.4% | 280 MB |
| dropbox | 0.3% | 210 MB |

8. **Key-value list** — "System" · span 4
- Source chip (Expert): `hostnamectl`

| Key | Value |
| --- | --- |
| Hostname | riya-laptop |
| OS | Ubuntu 24.04.1 LTS |
| Kernel | `6.8.0-47-generic` |
| Uptime | 12 h 36 m |
| Last boot | Today 08:58 |
| Reboot required | {warn:Yes} |

9. **Activity timeline** — "Recent activity" · span 6
- Header actions: [View all](go:activity/timeline) (link style)

| Time | Title | Detail | Dot |
| --- | --- | --- | --- |
| 10:40 | Unattended upgrade: libc6, openssl, sudo | Reboot required | info |
| 09:12 | bluetooth.service entered crash loop | SIGSEGV · 14 restarts | crit |
| 08:58 | Boot completed in 41.2 s | Kernel 6.8.0-47-generic | ok |
| 02:14 | Unclean shutdown | Battery depleted | warn |
| Tue 18:41 | linux-firmware upgraded | …0ubuntu2.5 | default |
| Tue 18:40 | Kernel upgraded 6.8.0-45 → 6.8.0-47 | Software Updater | default |

10. **Heatmap (day × hour)** — "Errors and warnings by hour" · span 6
- Subtitle: "last 7 days"
- Source chip (Expert): `journalctl -p warning`
- Rows: Thu, Fri, Sat, Sun, Mon, Tue, Wed · Columns: hours 00–23 · unit "events"
- Values (row by row, 24 per row):

```
Thu  1 0 0 1 1 0 0 0 1 1 0 0 0 0 0 0 0 1 1 1 2 0 0 1
Fri  0 1 0 0 0 1 0 1 0 1 1 0 1 0 1 0 0 0 1 0 0 0 0 0
Sat  1 0 0 0 1 1 1 0 0 0 0 1 0 1 1 0 0 0 1 0 0 0 0 1
Sun  1 0 1 2 1 0 1 0 0 0 0 0 0 0 0 1 0 0 0 0 0 1 1 0
Mon  0 0 0 0 0 0 0 0 2 1 1 0 0 0 0 0 7 1 0 0 0 0 0 0
Tue  1 0 0 1 1 0 0 0 1 0 0 1 0 0 0 0 0 1 0 0 2 1 0 0
Wed  1 1 3 0 0 2 0 1 1 9 0 0 1 0 2 0 0 1 0 0 0 6 0 2
```

## Overview › Issues — `overview/issues`

Tab count pill: 9 (red).

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Critical | 2 |  | {crit:Critical} |   | — | — |
| Warning | 4 |  | {warn:Warning} |   | — | — |
| Info | 3 |  | — |   | — | — |
| Resolved | 11 |  | — | last 30 days | — | — |
| Mean time to resolve | 1.8 | days | — |   | — | — |

2. **Data table** — "All issues" · span 12
- Search: "Search issues"
- Filter chips on column 1: All / Critical / Warning / Info
- Table ID: `issues-all`

| Severity | Issue | Category | First seen | Count [n] | Status | (action) |
| --- | --- | --- | --- | --- | --- | --- |
| {crit:Critical} | **Root filesystem almost full** · / at 94% · 15.2 GB reclaimable | Storage | 14 Oct | — | {off:Open} | [**Resolve**](act:fix:storage) |
| {crit:Critical} | **bluetooth.service crash loop** · 14 restarts today · SIGSEGV | Services | Today 09:12 | 14 | {off:Open} | [**Inspect**](act:bt) |
| {warn:Warning} | **Security updates pending** · 4 packages · 2 high-severity CVEs | Security | Today 06:21 | 4 | {off:Open} | [Install](act:fix:sec) |
| {warn:Warning} | **Battery health degraded** · 71% of design capacity | Power | 02 Sep | — | {off:Open} | [View](go:power/battery) |
| {warn:Warning} | **Backup overdue** · Last successful backup 21 days ago | Backups | 07 Oct | — | {off:Open} | [View](go:backups/jobs) |
| {warn:Warning} | **Out-of-memory kill** · chrome renderer · Mon 16:12 | Logs | Mon 16:12 | 1 | {off:Open} | [View](go:logs/kernel) |
| {info:Info} | **Reboot required** · libc6, openssl, sudo updated | System | Today 10:40 | — | {off:Open} | [Details](go:system/updates) |
| {info:Info} | **Low-battery action not set** · Unclean shutdown at 02:14 today | Power | Today 02:14 | 1 | {off:Open} | [Configure](act:fix:batt) |
| {info:Info} | **Missed timer run** · e2scrub_all.timer · Sun 03:10 | Scheduled Jobs | Sun 03:10 | 1 | {off:Open} | [View](go:jobs/timers) |

## Overview › Health Score — `overview/score`

1. **Line chart** — "Health score trend" · span 8
- Subtitle: "30 days"
- Height 240px · Y axis 60 to 100
- Series: Score (slot 1, data [96,96,95,95,94,94,93,93,92,92,92,91,91,90,90,89,89,88,88,87,87,87,86,82,81,81,80,80,79,82])
- X labels: 30 labels: 22 Sept, 23 Sept, 24 Sept … 20 Oct, 21 Oct

2. **Gauge + horizontal bars** — "Current score" · span 4

Gauge: value **82**, subtitle "out of 100", arc color `warn` (by score: ≥90 ok, ≥75 warn, else crit). Caption under the gauge (muted, centered): "90+ Healthy · 75–89 Fair · below 75 At risk".

3. **Data table** — "Score breakdown" · span 12

| Category | Weight [n] | Deduction [n] | Open findings | Status |
| --- | --- | --- | --- | --- |
| Storage | 15 | −8 | Root filesystem almost full | {crit:Critical} |
| Services | 15 | −5 | bluetooth.service crash loop | {crit:Critical} |
| Security | 15 | −3 | 4 security updates pending | {warn:Warning} |
| Power | 10 | −1 | Battery health 71% | {warn:Warning} |
| Backups | 10 | −1 | Last backup 21 days ago | {warn:Warning} |
| Hardware | 10 | 0 | — | {ok:Healthy} |
| Network | 10 | 0 | — | {ok:Healthy} |
| Accounts | 5 | 0 | — | {ok:Healthy} |
| Privacy | 5 | 0 | — | {ok:Healthy} |
| Packages | 5 | 0 | — | {ok:Healthy} |

---

[Index](README.md) · Next: [Performance](02-performance.md) →
