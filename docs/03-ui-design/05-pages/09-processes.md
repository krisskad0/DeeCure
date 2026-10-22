# Processes (`processes`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Workloads · Icon: `layers` · Status badge: {ok:Healthy} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: All Processes, Top Consumers

## Processes › All Processes — `processes/all`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Total | 312 |  | — |   | — | — |
| Running | 3 |  | — |   | — | — |
| Sleeping | 309 |  | — |   | — | — |
| Zombie | 0 |  | — |   | — | — |
| Threads | 1,204 |  | — |   | — | — |

2. **Data table** — "Processes" · span 12
- Source chip (Expert): `ps aux`
- Search: "Filter by name, PID or user"
- Filter chips on column 3: All / riya / root
- Page size 14
- Total count shown: 312
- Inspect: row click opens the row-inspect drawer with actions "End process", "Open Performance"
- Table ID: `procs`

| Name | PID [n] | User | State | CPU [n] | Memory [n] | Threads [n, ,x] | Started [x] |
| --- | --- | --- | --- | --- | --- | --- | --- |
| chrome | 4121 | riya | Sleeping | 4.2% | 842 MB | 38 | 08:59 |
| chrome (renderer) | 4388 | riya | Sleeping | 2.6% | 612 MB | 19 | 09:01 |
| gnome-shell | 1874 | riya | {ok:Running} | 3.8% | 410 MB | 24 | 08:59 |
| code | 5102 | riya | Sleeping | 1.9% | 486 MB | 31 | 15:24 |
| telegram-desktop | 2230 | riya | Sleeping | 0.4% | 280 MB | 22 | 08:59 |
| dropbox | 2301 | riya | Sleeping | 0.3% | 210 MB | 61 | 08:59 |
| postgres | 2811 | 999 | Sleeping | 0.2% | 142 MB | 1 | 09:00 |
| dockerd | 1320 | root | Sleeping | 0.3% | 96 MB | 18 | 08:58 |
| tracker-miner-fs-3 | 2051 | riya | Sleeping | 0.2% | 96 MB | 5 | 08:59 |
| Xwayland | 1990 | riya | Sleeping | 0.6% | 88 MB | 6 | 08:59 |
| gnome-terminal-server | 6012 | riya | Sleeping | 0.3% | 64 MB | 5 | 20:12 |
| containerd | 1288 | root | Sleeping | 0.1% | 52 MB | 14 | 08:58 |
| snapd | 1043 | root | Sleeping | 0.1% | 44 MB | 16 | 08:58 |
| systemd-journald | 412 | root | Sleeping | 0.1% | 38 MB | 1 | 08:58 |
| NetworkManager | 1011 | root | Sleeping | 0.1% | 22 MB | 4 | 08:58 |
| pipewire | 1702 | riya | Sleeping | 0.5% | 18 MB | 3 | 08:59 |
| redis-server | 2840 | 999 | Sleeping | 0.1% | 12 MB | 5 | 09:00 |
| systemd | 1 | root | Sleeping | 0.0% | 14 MB | 1 | 08:58 |
| bluetoothd | 7740 | root | {ok:Running} | 0.1% | 6 MB | 1 | 21:20 |
| bash | 6024 | riya | Sleeping | 0.0% | 5 MB | 1 | 20:12 |
| top | 6110 | riya | {ok:Running} | 0.3% | 4 MB | 1 | 21:30 |
| cron | 1036 | root | Sleeping | 0.0% | 3 MB | 1 | 08:58 |

## Processes › Top Consumers — `processes/top`

1. **Horizontal bar list** — "By CPU" · span 6
- Subtitle: "now"

| Label | Value | Shown as |
| --- | --- | --- |
| chrome | 4.2 | 4.2% |
| gnome-shell | 3.8 | 3.8% |
| chrome (renderer) | 2.6 | 2.6% |
| code | 1.9 | 1.9% |
| Xwayland | 0.6 | 0.6% |
| pipewire | 0.5 | 0.5% |
| telegram-desktop | 0.4 | 0.4% |
| dropbox | 0.3 | 0.3% |

2. **Horizontal bar list** — "By memory" · span 6
- Subtitle: "resident"

| Label | Value | Shown as |
| --- | --- | --- |
| chrome | 842 | 842 MB |
| chrome (renderer) | 612 | 612 MB |
| gnome-shell | 410 | 410 MB |
| code | 486 | 486 MB |
| telegram-desktop | 280 | 280 MB |
| dropbox | 210 | 210 MB |
| postgres | 142 | 142 MB |
| dockerd | 96 | 96 MB |

---

← Previous: [Display & Audio](08-display.md) · [Index](README.md) · Next: [Services](10-services.md) →
