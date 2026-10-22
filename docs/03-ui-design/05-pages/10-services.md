# Services (`services`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Workloads · Icon: `cog` · Status badge: {crit:Critical} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: System Services, User Services, Startup Applications

## Services › System Services — `services/system`

Tab count pill: 1 (red).

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Running | 142 |  | — |   | — | — |
| Failed | 1 |  | {crit:Critical} | bluetooth.service | — | — |
| Inactive | 38 |  | — |   | — | — |
| Enabled at boot | 61 |  | — |   | — | — |
| Restarts today | 14 |  | {warn:Warning} |   | — | — |

2. **Alert banner** — "bluetooth.service is in a crash loop" · span 12
- State: `crit` · Title: "bluetooth.service is in a crash loop" · Text: "14 restarts since 09:12 · exit status 11 (SIGSEGV)" · Buttons: [**Inspect**](act:bt)

3. **Data table** — "Units" · span 12
- Source chip (Expert): `systemctl list-units --type=service --all`
- Search: "Filter units"
- Filter chips on column 2: All / active / failed / inactive
- Page size 14
- Total count shown: 181
- Inspect: row click opens the row-inspect drawer with actions "Restart", "Stop", "Disable"
- Table ID: `svcs`

| Unit | Active | Sub | Enabled | Restarts [n] | Memory [n] | Row action |
| --- | --- | --- | --- | --- | --- | --- |
| **bluetooth.service** · Bluetooth service | {crit:failed} | auto-restart | {info:enabled} | 14 | — | `bt` |
| **NetworkManager.service** · Network Manager | {ok:active} | running | {info:enabled} | 0 | 22 MB |  |
| **docker.service** · Docker Application Container Engine | {ok:active} | running | {info:enabled} | 0 | 96 MB |  |
| **containerd.service** · containerd container runtime | {ok:active} | running | {info:enabled} | 0 | 52 MB |  |
| **snapd.service** · Snap Daemon | {ok:active} | running | {info:enabled} | 0 | 44 MB |  |
| **gdm.service** · GNOME Display Manager | {ok:active} | running | {info:enabled} | 0 | 9 MB |  |
| **systemd-journald.service** · Journal Service | {ok:active} | running | static | 0 | 38 MB |  |
| **systemd-resolved.service** · Network Name Resolution | {ok:active} | running | {info:enabled} | 0 | 8 MB |  |
| **systemd-timesyncd.service** · Network Time Synchronization | {ok:active} | running | {info:enabled} | 0 | 4 MB |  |
| **cups.service** · CUPS Scheduler | {ok:active} | running | {info:enabled} | 0 | 7 MB |  |
| **cups-browsed.service** · Make remote CUPS printers available locally | {off:inactive} | dead | disabled | 0 | — |  |
| **ufw.service** · Uncomplicated firewall | {ok:active} | exited | {info:enabled} | 0 | — |  |
| **apparmor.service** · Load AppArmor profiles | {ok:active} | exited | {info:enabled} | 0 | — |  |
| **unattended-upgrades.service** · Unattended Upgrades Shutdown | {ok:active} | running | {info:enabled} | 0 | 12 MB |  |
| **power-profiles-daemon.service** · Power Profiles daemon | {ok:active} | running | {info:enabled} | 0 | 6 MB |  |
| **thermald.service** · Thermal Daemon Service | {ok:active} | running | {info:enabled} | 0 | 5 MB |  |
| **avahi-daemon.service** · Avahi mDNS/DNS-SD Stack | {ok:active} | running | {info:enabled} | 0 | 3 MB |  |
| **ModemManager.service** · Modem Manager | {ok:active} | running | {info:enabled} | 0 | 8 MB |  |
| **packagekit.service** · PackageKit Daemon | {ok:active} | running | static | 0 | 28 MB |  |
| **accounts-daemon.service** · Accounts Service | {ok:active} | running | {info:enabled} | 0 | 6 MB |  |
| **polkit.service** · Authorization Manager | {ok:active} | running | static | 0 | 9 MB |  |
| **rsyslog.service** · System Logging Service | {ok:active} | running | {info:enabled} | 0 | 5 MB |  |
| **cron.service** · Regular background program processing daemon | {ok:active} | running | {info:enabled} | 0 | 3 MB |  |
| **anacron.service** · Run anacron jobs | {off:inactive} | dead | {info:enabled} | 0 | — |  |
| **wpa_supplicant.service** · WPA supplicant | {ok:active} | running | {info:enabled} | 0 | 7 MB |  |
| **fwupd.service** · Firmware update daemon | {off:inactive} | dead | static | 0 | — |  |
| **kerneloops.service** · Collects kernel crash signatures | {ok:active} | running | {info:enabled} | 0 | 2 MB |  |

## Services › User Services — `services/user`

1. **Data table** — "User units" · span 12
- Subtitle: "riya"
- Source chip (Expert): `systemctl --user`

| Unit | Active | Sub | Memory [n] |
| --- | --- | --- | --- |
| **pipewire.service** · PipeWire Multimedia Service | {ok:active} | running | 18 MB |
| **pipewire-pulse.service** · PipeWire PulseAudio | {ok:active} | running | 9 MB |
| **wireplumber.service** · Multimedia Service Session Manager | {ok:active} | running | 12 MB |
| **xdg-desktop-portal.service** · Portal service | {ok:active} | running | 7 MB |
| **tracker-miner-fs-3.service** · Tracker file system data miner | {ok:active} | running | 96 MB |
| **gvfs-daemon.service** · Virtual filesystem service | {ok:active} | running | 6 MB |
| **gnome-keyring-daemon.service** · GNOME Keyring | {ok:active} | running | 8 MB |
| **org.gnome.Shell@wayland.service** · GNOME Shell on Wayland | {ok:active} | running | 410 MB |
| **snap.telegram-desktop.service** · Telegram (snap) | {ok:active} | running | — |

## Services › Startup Applications — `services/startup`

1. **Data table** — "Started at login" · span 12
- Source chip (Expert): `~/.config/autostart · /etc/xdg/autostart`

| Name | Scope | Command [x] | Startup impact [n] | Enabled |
| --- | --- | --- | --- | --- |
| Telegram Desktop | User | `telegram-desktop -startintray` | [meter:ok 57% "1.2 s"] | [toggle:on] |
| Dropbox | User | `dropbox start -i` | [meter:ok 43% "0.9 s"] | [toggle:on] |
| Google Chrome | User | `google-chrome --no-startup-window` | [meter:warn 100% "2.1 s"] | [toggle:on] |
| GNOME Software | System | `gnome-software --gapplication-service` | [meter:ok 29% "0.6 s"] | [toggle:on] |
| Snap user daemon | System | `snap userd --autostart` | [meter:ok 10% "0.2 s"] | [toggle:on] |

---

← Previous: [Processes](09-processes.md) · [Index](README.md) · Next: [Scheduled Jobs](11-jobs.md) →
