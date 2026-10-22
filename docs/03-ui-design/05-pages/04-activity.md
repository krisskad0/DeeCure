# Activity (`activity`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Monitoring · Icon: `hist` · Status badge: {ok:Healthy} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Timeline, Package History, Config Changes, Device Events, SysDoc Actions

## Activity › Timeline — `activity/timeline`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Changes | 18 |  | — | last 7 days | — | — |
| Package changes | 9 |  | — |   | — | — |
| Configuration changes | 3 |  | — |   | — | — |
| Sign-ins | 14 |  | — |   | — | — |
| SysDoc actions | 1 |  | — |   | — | — |

2. **Activity timeline** — "Change timeline" · span 8

| Time | Title | Detail | Dot |
| --- | --- | --- | --- |
| 10:40 | Unattended upgrade: libc6, openssl, sudo | System · Reboot required | info |
| 09:12 | bluetooth.service entered crash loop | Services · SIGSEGV · 14 restarts | crit |
| 08:58 | Boot completed in 41.2 s | System · Kernel 6.8.0-47-generic | ok |
| 02:14 | Unclean shutdown | Power · Battery depleted | warn |
| Tue 18:41 | linux-firmware upgraded | Packages · …0ubuntu2.5 | default |
| Tue 18:40 | Kernel upgraded 6.8.0-45 → 6.8.0-47 | Packages · Software Updater | default |
| Tue 15:22 | Installed snap: code 1.94.2 | Packages · 340 MB | default |
| Tue 11:05 | Disabled cups-browsed.service | SysDoc · Reversible | info |
| Mon 21:30 | crontab changed (riya): +backup-photos.sh | Jobs · 0 21 * * * | default |
| Mon 16:12 | OOM kill: chrome renderer (pid 30412) | Logs · RSS 1.8 GB | warn |
| Mon 10:02 | USB storage connected: PHOTOS (64 GB) | Devices · usb 3-2 | default |

3. **Donut chart** — "Changes by type" · span 4
- Subtitle: "7 days"
- Center: "18" / "changes"

| Segment | Value | Color | Display value |
| --- | --- | --- | --- |
| Packages | 9 | slot 1 | — |
| Configuration | 3 | slot 2 | — |
| Services | 2 | slot 3 | — |
| Devices | 3 | slot 4 | — |
| Jobs | 1 | slot 5 | — |

## Activity › Package History — `activity/packages`

1. **Data table** — "Package history" · span 12
- Source chip (Expert): `/var/log/apt/history.log · snap changes`
- Search: "Search packages"
- Filter chips on column 2: All / Install / Upgrade / Remove

| Date | Action | Package | From | To | Source |
| --- | --- | --- | --- | --- | --- |
| Today 10:40 | {info:Upgrade} | libc6 | `2.39-0ubuntu8.2` | `2.39-0ubuntu8.3` | unattended-upgrades |
| Today 10:40 | {info:Upgrade} | openssl | `3.0.13-0ubuntu3.3` | `3.0.13-0ubuntu3.4` | unattended-upgrades |
| Today 10:40 | {info:Upgrade} | sudo | `1.9.15p5-3ubuntu5` | `1.9.15p5-3ubuntu5.1` | unattended-upgrades |
| Tue 18:40 | {info:Upgrade} | linux-image-generic | `6.8.0-45` | `6.8.0-47` | Software Updater |
| Tue 18:41 | {info:Upgrade} | linux-firmware | `…0ubuntu2.4` | `…0ubuntu2.5` | Software Updater |
| Tue 15:22 | {ok:Install} | code (snap) | — | `1.94.2` | snap |
| Sat 19:03 | {ok:Install} | com.spotify.Client | — | `1.2.45` | flatpak |
| Fri 12:20 | {crit:Remove} | thunderbird (snap) | `128.3.1` | — | snap |
| Thu 09:30 | {ok:Install} | httpie | — | `3.2.2` | pip |

## Activity › Config Changes — `activity/config`

1. **Data table** — "Configuration changes" · span 12
- Source chip (Expert): `inotify watcher · /etc`

| Time | File | Change | Changed by | (action) |
| --- | --- | --- | --- | --- |
| Tue 11:05 | `/etc/systemd/system/multi-user.target.wants/cups-browsed.service` | Removed (symlink) | SysDoc | [View diff](act:toast:Symlink removed by systemctl disable cups-browsed) |
| Tue 15:20 | `/etc/apt/sources.list.d/vscode.list` | Added | apt | [View diff](act:toast:+ deb [arch=amd64] https://packages.microsoft.com/repos/code stable main) |
| Mon 21:30 | `/var/spool/cron/crontabs/riya` | ✓ | riya | [View diff](act:toast:+ 0 21 * * * ~/bin/backup-photos.sh) |

## Activity › Device Events — `activity/devices`

1. **Data table** — "Device events" · span 12
- Source chip (Expert): `udev · journal`

| Time | Event | Device | Bus | ID |
| --- | --- | --- | --- | --- |
| Today 21:20 | {warn:Disconnected} | Wireless earbuds | Bluetooth | `A4:C1:38:..:5E` |
| Today 21:20 | {ok:Connected} | Wireless earbuds | Bluetooth | `A4:C1:38:..:5E` |
| Mon 10:02 | {ok:Connected} | USB storage PHOTOS (64 GB) | USB 3.2 | `0781:5583` |
| Mon 09:44 | {warn:Disconnected} | USB storage PHOTOS (64 GB) | USB 3.2 | `0781:5583` |
| Sat 18:30 | {ok:Connected} | USB mouse | USB 2.0 | `046d:c077` |

## Activity › SysDoc Actions — `activity/sysdoc`

1. **Data table** — "Remediation log" · span 12
- Info tooltip: "Every change SysDoc made, with undo where possible."

| Time | Action | Target | Result | Undo |
| --- | --- | --- | --- | --- |
| Tue 11:05 | Disabled service | `cups-browsed.service` | {ok:Succeeded} | [Undo](act:undo:cups) |

---

← Previous: [Logs](03-logs.md) · [Index](README.md) · Next: [System](05-system.md) →
