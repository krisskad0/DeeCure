# System (`system`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: System · Icon: `sys` · Status badge: {info:Info} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Overview, Boot, Updates, Kernel, Time & Locale

## System › Overview: `system/overview`

1. **Key-value list**: "Operating system" · span 6
- Source chip (Expert): `/etc/os-release · uname -a`

| Key | Value |
| --- | --- |
| Distribution | Ubuntu 24.04.1 LTS (Noble Numbat) |
| Kernel | `6.8.0-47-generic` |
| Architecture | x86_64 |
| Desktop | GNOME 46 · Wayland |
| Init system | systemd 255 |
| Hostname | riya-laptop |
| Virtualization | None (bare metal) |
| Machine ID | `4f1c9d…a72e` |

2. **Key-value list**: "Status" · span 6

| Key | Value |
| --- | --- |
| Uptime | 12 h 36 m |
| Last boot | Today 08:58 · 41.2 s |
| Reboot required | {warn:Yes · libc6, openssl, sudo} |
| Load average | `0.84 0.92 1.10` |
| Users logged in | 1 |
| Running services | 142 |
| Ubuntu Pro | {off:Not attached} |
| Livepatch | {off:Disabled} |

3. **Lifecycle bar ([Lifecycle bar](../03-components/20-lifecycle-bar.md))**: "Release lifecycle" · span 12
- Subtitle: "Ubuntu 24.04 LTS"
- Info tooltip: "How long this Ubuntu version gets security updates."

Fill and "Today" marker at 50.4%. Left label "Released · Apr 2024", right label "Standard support ends · Apr 2029".

## System › Boot: `system/boot`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Last boot | 41.2 | s | {warn:Warning} | Average 28.6 s | - | - |
| Firmware | 8.1 | s | - |   | - | - |
| Loader | 2.4 | s | - |   | - | - |
| Kernel | 3.9 | s | - |   | - | - |
| Userspace | 26.8 | s | - |   | - | - |

2. **Horizontal bar list**: "Slowest units" · span 6
- Source chip (Expert): `systemd-analyze blame`

| Label | Value | Shown as |
| --- | --- | --- |
| `snapd.service` | 6.2 | 6.2 s |
| `NetworkManager-wait-online.service` | 5.1 | 5.1 s |
| `plymouth-quit-wait.service` | 4.4 | 4.4 s |
| `docker.service` | 3.1 | 3.1 s |
| `dev-nvme0n1p2.device` | 2.2 | 2.2 s |
| `apt-daily.service` | 1.7 | 1.7 s |
| `systemd-udev-settle.service` | 1.2 | 1.2 s |

3. **Bar chart**: "Boot time history" · span 6
- Subtitle: "last 10 boots"
- Height 220px · Y max auto · unit "s" · highlighted (warn + value label): index 9
- Bars (label: value): 8 Oct: 29; 10 Oct: 28; 12 Oct: 31; 13 Oct: 27; 14 Oct: 28; 15 Oct: 30; 16 Oct: 29; 17 Oct: 28; 19 Oct: 28; Today: 41.2


Expert adds: **What the desktop waited for** (the boot's critical chain, indented by depth, the slowest step marked) and **Every unit's start time**: chips for how many units started, took over 1 s, over 100 ms, and under 100 ms, with the full list under **Show all** (the slowest are already in "Slowest units").

## System › Updates: `system/updates`

Header actions: [**Install security updates**](act:fix:sec) (hidden once done), then Refresh.

Tab count pill: 12.

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Pending | 12 |  | {warn:Warning} |   | - | - |
| Security | 4 |  | {warn:Warning} |   | - | - |
| Last check | 2 | h ago | - |   | - | - |
| Automatic security updates | On |  | - |   | - | - |
| Reboot required | Yes |  | {info:Info} |   | - | - |

2. **Data table**: "Pending updates" · span 12
- Source chip (Expert): `apt list --upgradable`
- Row selection · bulk button [Install selected](act:fix:sec)
- Total count shown: 12
- Table ID: `pending`

| Package | Installed [x] | Candidate | Priority | Size [n] |
| --- | --- | --- | --- | --- |
| openssl | `3.0.13-0ubuntu3.4` | `3.0.13-0ubuntu3.5` | {warn:Security} | 1.2 MB |
| libssl3t64 | `3.0.13-0ubuntu3.4` | `3.0.13-0ubuntu3.5` | {warn:Security} | 1.9 MB |
| sudo | `1.9.15p5-3ubuntu5.1` | `1.9.15p5-3ubuntu5.2` | {warn:Security} | 0.9 MB |
| linux-firmware | `…0ubuntu2.5` | `…0ubuntu2.6` | {warn:Security} | 298 MB |
| google-chrome-stable | `130.0.6723.58` | `130.0.6723.69` | {off:Regular} | 112 MB |
| git | `1:2.43.0-1ubuntu7.1` | `1:2.43.0-1ubuntu7.2` | {off:Regular} | 3.7 MB |
| docker-ce | `5:27.3.1` | `5:27.3.2` | {off:Regular} | 25 MB |
| code (snap) | `1.94.2` | `1.95.0` | {off:Regular} | 106 MB |

3. **Data table**: "Update history" · span 12

| Date | Packages [n] | Source | Result |
| --- | --- | --- | --- |
| Today 10:40 | 3 | unattended-upgrades | {ok:Success} |
| Tue 18:40 | 6 | Software Updater | {ok:Success} |
| Thu 06:24 | 2 | unattended-upgrades | {ok:Success} |

"Pending updates" shows chips per origin, a note when a new kernel waits (restart after installing), and up to 8 rows (security fixes first) with the version change. The table, with its search, is under "Show all".

## System › Kernel: `system/kernel`

1. **Running kernel** · span 5
- The version, large; chips: "Not tainted" (or "Tainted (N)", warn), "245 modules loaded", "installed 7 Oct 2026".
- "Started with these options": the boot command line split into one code chip per option.

2. **Kernel list**: "Installed kernels" · span 7
- Subtitle: "2 · 522 MB in /boot". Header action: **Remove old…** when there are old kernels.
- One row per kernel: version, status badge (Running ok, Fallback info, Old off), and "261 MB · installed 7 Oct 2026"; a left edge in the status colour.

3. **Ranked bars**: "Kernel modules" · span 12 (Beginner and Advanced; Expert shows "Loaded modules" instead, so modules show once)
- Subtitle: "245 loaded · the 10 biggest". Each bar: module name, a second line on who uses it ("used by rfcomm, bnep, btusb +1", "in use 55 times", or "not in use"), and its size.
- **Show all 245 modules** opens the full searchable table.

Expert adds:
- **KPI row:** file watches (with a bar), open files, kernel lockdown, clock offset.
- **Kernel settings from files** · span 12: a green "As shipped" line when every setting comes from Ubuntu's packages unchanged and in effect; otherwise "Worth a look" lists each setting set on this computer, from a changed package file, changed while running, or replaced by a later file, with why and the file. Then a chip per package with how many it sets, and **Show all N settings** for the full table.
- **File watches by program** · span 6: the top 8 as ranked bars with PID; **Show all** when there are more.
- **Time sync** · span 6.
- **Loaded modules** · span 6: a green "All signed" line, or a warning list of modules from outside the kernel or unsigned (taint letters); then "Drive the most devices" as ranked bars; **Show all** for the full table.
- **Extra drivers (DKMS)** · span 6.

## System › Time & Locale: `system/time`

1. **Key-value list**: "Date and time" · span 6
- Source chip (Expert): `timedatectl`

| Key | Value |
| --- | --- |
| Local time | Wed 21 Oct 2026 21:34 IST |
| Time zone | Asia/Kolkata (IST, +0530) |
| NTP synchronized | {ok:Yes} |
| Time server | ntp.ubuntu.com |
| RTC in local time | No |

2. **Key-value list**: "Locale" · span 6
- Source chip (Expert): `localectl`

| Key | Value |
| --- | --- |
| System locale | `LANG=en_IN.UTF-8` |
| Keyboard layout | us |
| Input method | IBus |
| Units | Metric · °C |

---

← Previous: [Activity](04-activity.md) · [Index](README.md) · Next: [Hardware](06-hardware.md) →
