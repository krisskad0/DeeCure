# Logs (`logs`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Monitoring · Icon: `logs` · Status badge: {warn:Warning} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Journal, Errors, Crashes, Kernel, Boots

## Logs › Journal: `logs/journal`

1. **Bar chart**: "Log volume" · span 12
- Subtitle: "events per hour, last 24 h"
- Source chip (Expert): `journalctl --since -24h`
- Height 150px · Y max auto · highlighted (warn + value label): index 8
- Bars (label: value · tooltip label): 00: 14 · "00:00"; 01: 9 · "01:00"; 02: 41 · "02:00"; 03: 22 · "03:00"; 04: 6 · "04:00"; 05: 4 · "05:00"; 06: 5 · "06:00"; 07: 6 · "07:00"; 08: 88 · "08:00"; 09: 64 · "09:00"; 10: 31 · "10:00"; 11: 18 · "11:00"; 12: 22 · "12:00"; 13: 19 · "13:00"; 14: 16 · "14:00"; 15: 34 · "15:00"; 16: 21 · "16:00"; 17: 14 · "17:00"; 18: 26 · "18:00"; 19: 19 · "19:00"; 20: 31 · "20:00"; 21: 46 · "21:00"; 22: 12 · "22:00"; 23: 8 · "23:00"

2. **Data table**: "Journal entries" · span 12
- Search: "Filter messages"
- Filter chips on column 2: All / error / warning / info
- Page size 12
- Table ID: `journal`

| Time | Priority | Unit | Message [w] |
| --- | --- | --- | --- |
| `21:31:02` | {off:info} | `NetworkManager` | `device (wlp0s20f3): state change: activated` |
| `21:20:15` | {off:info} | `systemd` | `bluetooth.service: Scheduled restart job, restart counter is at 14.` |
| `21:20:14` | {warn:warning} | `systemd` | `bluetooth.service: Main process exited, code=dumped, status=11/SEGV` |
| `21:20:14` | {crit:error} | `kernel` | `bluetoothd[7702]: segfault at 0 ip 000055d1c2a4 sp 00007ffd error 4` |
| `21:05:44` | {off:info} | `gnome-shell` | `Overwriting existing binding of keysym 31 with keysym 31` |
| `20:12:03` | {off:info} | `systemd` | `Started gnome-terminal-server.service` |
| `18:52:10` | {off:info} | `systemd` | `Starting apt-daily.service - Daily apt download activities...` |
| `15:22:31` | {off:info} | `snapd` | `Installed "code" snap` |
| `10:40:02` | {off:info} | `unattended-upgrades` | `Packages that were upgraded: libc6 openssl sudo` |
| `09:12:44` | {crit:error} | `kernel` | `bluetoothd[1342]: segfault at 0 ip 000055d1c2a4 error 4` |
| `09:00:17` | {crit:error} | `kernel` | `ACPI BIOS Error (bug): Could not resolve symbol [\_SB.PC00.LPCB.EC0], AE_NOT_FOUND` |
| `08:58:41` | {off:info} | `systemd` | `Startup finished in 8.1s (firmware) + 2.4s (loader) + 3.9s (kernel) + 26.8s (userspace) = 41.2s.` |
| `08:58:05` | {off:info} | `kernel` | `Linux version 6.8.0-47-generic (buildd@lcy02-amd64-061)` |
| `02:13:58` | {warn:warning} | `upowerd` | `battery BAT0 at 1%, critical action not configured` |
| `02:09:12` | {warn:warning} | `upowerd` | `battery BAT0 is critically low (2%)` |
| `Mon 16:12:31` | {crit:error} | `kernel` | `Out of memory: Killed process 30412 (chrome) anon-rss:1843212kB` |
| `Mon 16:12:30` | {warn:warning} | `systemd-oomd` | `Memory pressure for user@1000.service is high` |
| `Mon 10:02:11` | {off:info} | `kernel` | `usb 3-2: new SuperSpeed USB device number 4 using xhci_hcd` |
| `Mon 09:44:09` | {off:info} | `kernel` | `usb 3-2: USB disconnect, device number 3` |
| `Sun 03:10:00` | {off:notice} | `systemd` | `e2scrub_all.timer: skipped, system suspended` |

## Logs › Errors: `logs/errors`

Tab count pill: 37.

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Errors | 37 |  | {warn:Warning} | last 7 days | - | - |
| Unique messages | 6 |  | - |   | - | - |
| Needs attention | 2 |  | {crit:Critical} |   | - | - |
| Known harmless | 29 |  | - | ACPI, gnome-shell | - | - |

2. **Data table**: "Grouped errors" · span 12

| Message | Unit | Count [n] | Last seen | Classification | Row action |
| --- | --- | --- | --- | --- | --- |
| **Segmentation fault (SIGSEGV)** · bluetoothd | `bluetooth.service` | 14 | Today 21:20 | {crit:Needs attention} | `bt` |
| **Out of memory: Killed process** · chrome renderer | `kernel` | 1 | Mon 16:12 | {warn:Needs attention} |  |
| **ACPI BIOS Error (AE_NOT_FOUND)** · Firmware table bug | `kernel` | 18 | Today 09:00 | {off:Harmless} |  |
| **Overwriting existing binding of keysym** · Keyboard shortcut | `gnome-shell` | 11 | Today 21:05 | {off:Harmless} |  |
| **USB disconnect** · usb 3-2 | `kernel` | 2 | Mon 09:44 | {info:Review} |  |
| **Failed to start Network Time Synchronization** · Retried successfully | `systemd` | 1 | Sat 08:12 | {off:Harmless} |  |

3. **Heatmap (day × hour)**: "Errors by day and hour" · span 12
- Source chip (Expert): `journalctl -p err`
- Rows: Thu, Fri, Sat, Sun, Mon, Tue, Wed · Columns: hours 00-23 · unit "errors"
- Values (row by row, 24 per row):

```
Thu  1 1 0 0 1 0 0 1 0 0 0 0 0 1 0 0 0 0 0 1 0 0 0 0
Fri  0 1 0 0 0 1 0 0 1 0 0 0 0 1 0 0 1 0 0 1 0 0 0 0
Sat  0 0 0 1 1 0 1 1 0 0 0 0 0 0 1 1 0 0 0 0 0 0 1 0
Sun  0 1 0 0 1 0 0 0 0 0 1 0 0 0 0 0 1 1 0 0 0 0 0 0
Mon  0 0 1 0 0 1 0 0 1 0 0 1 0 0 1 0 3 0 1 0 0 0 0 0
Tue  1 1 0 0 1 0 0 0 1 0 0 1 0 1 0 0 0 0 1 1 0 1 0 0
Wed  2 0 1 1 0 0 1 0 0 9 1 1 1 0 0 0 0 1 0 0 0 5 1 0
```

## Logs › Crashes: `logs/crashes`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Crashes | 16 |  | - | last 7 days | - | - |
| Applications | 3 |  | - |   | - | - |
| Core dumps stored | 420 | MB | - |   | - | - |

2. **Data table**: "Crash reports" · span 12
- Source chip (Expert): `coredumpctl list · /var/crash`

| Time | Application | Signal | Source | Count [n] | Status | Row action |
| --- | --- | --- | --- | --- | --- | --- |
| Today 21:20 | `bluetoothd` | SIGSEGV | systemd-coredump | 14 | {crit:Recurring} | `bt` |
| Mon 16:12 | `chrome (renderer)` | SIGKILL (OOM) | kernel | 1 | {warn:Resolved} |  |
| Mon 11:47 | `gnome-calculator` | SIGABRT | apport | 1 | {off:Reported} |  |

## Logs › Kernel: `logs/kernel`

1. **Data table**: "Kernel messages" · span 12
- Source chip (Expert): `journalctl -k`
- Search: "Filter kernel log"

| Time | Level | Subsystem | Message [w] |
| --- | --- | --- | --- |
| 21:20:14 | {crit:err} | traps | `bluetoothd[7702]: segfault at 0 ip 000055d1c2a4 error 4` |
| Mon 16:12 | {crit:err} | oom | `Out of memory: Killed process 30412 (chrome)` |
| 09:00:17 | {crit:err} | ACPI | `ACPI BIOS Error (bug): AE_NOT_FOUND` |
| 08:58:07 | {off:info} | iwlwifi | `loaded firmware version 89.202a2f7b.0 so-a0-gf-a0-89.ucode` |
| 08:58:06 | {off:info} | nvme | `nvme0: 8/0/0 default/read/poll queues` |
| Mon 10:02 | {off:info} | usb | `usb 3-2: new SuperSpeed USB device number 4` |
| Mon 09:44 | {warn:warn} | usb | `usb 3-2: USB disconnect, device number 3` |

## Logs › Boots: `logs/boots`

1. **Bar chart**: "Boot duration" · span 6
- Subtitle: "last 10 boots"
- Source chip (Expert): `systemd-analyze`
- Height 200px · Y max auto · unit "s" · highlighted (warn + value label): index 9
- Bars (label: value): 8 Oct: 29; 10 Oct: 28; 12 Oct: 31; 13 Oct: 27; 14 Oct: 28; 15 Oct: 30; 16 Oct: 29; 17 Oct: 28; 19 Oct: 28; Today: 41.2

2. **Data table**: "Boot history" · span 6
- Source chip (Expert): `journalctl --list-boots`

| Started | Ended | Duration | Shutdown | Kernel |
| --- | --- | --- | --- | --- |
| Today 08:58 | - | 12 h 36 m | {info:Running} | `6.8.0-47` |
| Mon 08:41 | Today 02:14 | 1 d 17 h | {crit:Unclean} | `6.8.0-45` |
| Sat 10:12 | Mon 08:40 | 1 d 22 h | {ok:Clean} | `6.8.0-45` |
| Thu 09:01 | Sat 10:11 | 2 d 1 h | {ok:Clean} | `6.8.0-45` |

---

← Previous: [Performance](02-performance.md) · [Index](README.md) · Next: [Activity](04-activity.md) →
