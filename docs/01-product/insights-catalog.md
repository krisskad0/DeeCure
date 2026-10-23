# Insights catalog

[Docs](../README.md) › [Product](README.md)

Every insight DeeCure shows, and what it needs. This is the checklist the collectors are built from.

| Access | Meaning |
| --- | --- |
| **Yes** | Readable by your own account. Works in Basic access, today. |
| **Admin** | Needs root. Read through the admin helper in Full access ([Phase 2](../05-roadmap/phase-2-fix-safely.md)); the password is asked each time. In Basic access, or without the helper, the app says how to turn it on. |
| **No** | Linux does not expose it (for example password strength). The app says so. |

Sources are listed in [Collectors](../02-architecture/collectors.md).

## Monitoring

| Insight | Access | Page |
| --- | --- | --- |
| CPU use (total, by type, per core), load, clock speed, context switches | Yes | Performance › CPU |
| Memory use and composition, swap, top memory users | Yes | Performance › Memory |
| Disk throughput, IOPS, busy time, wait time per disk | Yes | Performance › Disk I/O |
| Disk I/O per process | Admin | Performance › Disk I/O |
| Network throughput, data today, connections | Yes | Performance › Network |
| Network traffic per app | Admin | Network › Traffic |
| GPU model and driver; load and video memory (amdgpu) | Yes | Performance › GPU |
| GPU load on Intel and NVIDIA | Admin | Performance › GPU |
| Pressure stall information (CPU, memory, I/O) | Yes | Performance › Pressure |
| Journal entries, error groups, kernel log | Yes (in the `adm` group, the default for the first user) | Logs |
| Crashes (kernel, systemd, apport) | Yes | Logs › Crashes |
| Boot history and unclean shutdowns | Yes | Logs › Boots |
| Package, config-file, and USB device changes | Yes | Activity |

## System

| Insight | Access | Page |
| --- | --- | --- |
| OS, release support dates, kernel, uptime, virtualization, Ubuntu Pro, Livepatch | Yes | System › Overview |
| Boot phase times and slowest units | Yes | System › Boot |
| Pending updates, security updates, automatic updates, update history | Yes | System › Updates |
| Installed kernels, modules, taint | Yes | System › Kernel |
| Time zone, NTP sync, locale, keyboard | Yes | System › Time & Locale |
| Machine, chassis, BIOS, Secure Boot, TPM | Yes | Hardware › Summary, Firmware |
| Serial number | Admin | Hardware › Summary |
| Processor model, hybrid cores, cache, features | Yes | Hardware › Processor |
| Memory module type, speed, slots | Admin | Hardware › Memory |
| Disks: model, type, size, temperature (NVMe) | Yes | Hardware › Storage Devices |
| SMART health, wear, errors | Admin | Hardware › Storage Devices |
| Temperatures and fans | Yes | Hardware › Sensors |
| PCI, USB, input, and Bluetooth devices with drivers | Yes | Hardware › Peripherals |
| Firmware updates (fwupd, when it runs) | Yes | Hardware › Firmware |
| Battery charge, health, capacity, cycles, charge limit | Yes | Power › Battery |
| Per-process power use | Admin | Power › Battery |
| Power mode, battery policy, suspend and lid settings | Yes | Power › Power Profile |
| Sleep history | Yes | Power › Sleep |
| Screens, resolution, refresh, scale (GNOME) | Yes | Display & Audio |
| Sound devices, volumes, streams (PipeWire) | Yes | Display & Audio |

## Workloads, software, and data

| Insight | Access | Page |
| --- | --- | --- |
| Processes with CPU, memory, swap, threads, start time | Yes (other users' command lines too) | Processes |
| System services, memory per service, user services, startup apps | Yes | Services |
| Cron, timers, anacron, at, and job runs | Yes (your crontab and the system ones) | Scheduled Jobs |
| Other users' crontabs | Admin | Scheduled Jobs › Cron |
| Docker containers, images, volumes (in the `docker` group); VMs | Yes | Containers |
| Packages from APT, snap, Flatpak, AppImage; sources; what could be removed | Yes | Packages |
| Disk space, inodes, forecast, encryption, fstab | Yes | Storage |
| Folder sizes | Yes for your home; folders of other users and root are skipped | Storage › Disk Usage |
| Cleanup candidates | Yes | Storage › Cleanup |
| Backups (Déjà Dup, Timeshift) | Yes | Backups |

## Network, people, security, privacy

| Insight | Access | Page |
| --- | --- | --- |
| Interfaces, addresses, Wi-Fi signal, connectivity, DNS, VPNs | Yes | Network |
| Listening ports and who can reach them | Yes | Network › Open Ports |
| Which system program owns a port | Admin | Network › Open Ports |
| Firewall on/off and default policy | Yes | Network › Firewall |
| Firewall rules | Admin | Network › Firewall |
| Users, groups, sessions, sign-ins, sudo use, SSH keys | Yes | Accounts |
| Password strength | No (only salted hashes exist) | Accounts › Users |
| Password age, locked accounts | Admin | Accounts › Users |
| Kernel hardening, AppArmor, SUID programs, malware signs, screen lock | Yes | Security |
| CVE numbers for pending fixes | Yes, with the Ubuntu security database (later) | Security › Vulnerabilities |
| Full file verification against package checksums | Admin (long scan) | Security › File Integrity |
| Apps using camera, microphone, screen now | Yes | Privacy › Device Access |
| Snap and Flatpak permissions; privacy settings | Yes | Privacy |

## Checks that turn into findings (rules)

| Rule | Access |
| --- | --- |
All 16 rules work in Basic access. Every one of the 13 areas in Overview › Subsystems has at least one.

| Area | Rule | Access |
| --- | --- | --- |
| Storage | `storage.root_almost_full`, `storage.filling_up` (7-day forecast) | Yes |
| Services | `services.unit_failed` | Yes |
| Scheduled Jobs | `jobs.job_failed` (a timer-started service failed) | Yes |
| Hardware | `hardware.cpu_hot` | Yes (needs a CPU sensor) |
| Power | `power.battery_worn` | Yes |
| System | `system.reboot_required` | Yes |
| Packages | `packages.updates_available` | Yes |
| Containers | `containers.port_exposed` | Yes (in the `docker` group) |
| Backups | `backups.none`, `backups.overdue` | Yes |
| Network | `network.offline` | Yes |
| Accounts | `accounts.extra_root` (another UID 0 account) | Yes (`/etc/passwd` only) |
| Security | `security.updates_pending`, `security.firewall_off` | Yes |
| Privacy | `privacy.popcon` (popularity contest on) | Yes |

## History, predictions, and reports (Phase 3)

| Insight | Access | Where |
| --- | --- | --- |
| What changed: packages (apt, dpkg, snap, Flatpak), config files with diffs, USB devices, boots and kernel changes, unclean shutdowns, crashes, failed services, DeeCure's fixes | Yes | Activity, `deecure history` |
| Disk full in about N days | Yes | Storage, rule `storage.filling_up` |
| Desktop notifications and the weekly summary | Yes | Settings › Notifications, Overview › Health Score |
| A report of any period, as text or HTML | Yes | `deecure report`, Settings › Data › Export |

---

[Index](README.md)
