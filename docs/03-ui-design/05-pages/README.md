# Pages

[Docs](../../README.md) › [UI design](../README.md)

Each insight category is its own page. Each page has sub-pages, shown as tabs under the page title and as a tree in the sidebar. The page names follow common Linux and admin-tool terms.

Each file below covers one page. Each sub-page lists its header actions (if any) and its widgets in grid order. Read [Notation](../01-foundations/notation.md) first.

## Index

| # | Group | Page | Sub-pages (tabs) |
| --- | --- | --- | --- |
| 01 | Monitoring | [Overview](01-overview.md) | Summary, Issues, Health Score |
| 02 | Monitoring | [Performance](02-performance.md) | CPU, Memory, Disk I/O, Network, GPU, Pressure (PSI) |
| 03 | Monitoring | [Logs](03-logs.md) | Journal, Errors, Crashes, Kernel, Boots |
| 04 | Monitoring | [Activity](04-activity.md) | Timeline, Package History, Config Changes, Device Events, DeeCure Actions |
| 05 | System | [System](05-system.md) | Overview, Boot, Updates, Kernel, Time & Locale |
| 06 | System | [Hardware](06-hardware.md) | Summary, Processor, Memory, Storage Devices, Graphics, Sensors, Peripherals, Firmware |
| 07 | System | [Power](07-power.md) | Battery, Power Profile, Sleep |
| 08 | System | [Display & Audio](08-display.md) | Displays, Audio |
| 09 | Workloads | [Processes](09-processes.md) | All Processes, Top Consumers |
| 10 | Workloads | [Services](10-services.md) | System Services, User Services, Startup Applications |
| 11 | Workloads | [Scheduled Jobs](11-jobs.md) | Calendar, Cron, Systemd Timers, Anacron, At Jobs |
| 12 | Workloads | [Containers](12-containers.md) | Containers, Images, Volumes, Virtual Machines |
| 13 | Software & Data | [Packages](13-packages.md) | Installed, Sources, Cleanup, Permissions |
| 14 | Software & Data | [Storage](14-storage.md) | Overview, Filesystems, Disk Usage, Cleanup |
| 15 | Software & Data | [Backups](15-backups.md) | Backup Jobs, System Snapshots, Restore Points |
| 16 | Network & Access | [Network](16-network.md) | Interfaces, Connections, Open Ports, Firewall, DNS & VPN, Traffic |
| 17 | Network & Access | [Accounts](17-accounts.md) | Users, Groups, Sessions, Login History, SSH Keys |
| 18 | Network & Access | [Security](18-security.md) | Overview, Vulnerabilities, Hardening, File Integrity, Malware Scan |
| 19 | Network & Access | [Privacy](19-privacy.md) | Device Access, App Permissions, Privacy Settings |
| 20 | Tools | [Diagnostics](20-diagnostics.md) | Performance Check, Network Test, Hardware Tests |
| 21 | Tools | [Assistant](21-assistant.md) | Chat |
| 22 | (footer) | [Settings](22-settings.md) | General, Access, Notifications, Assistant, Data, About |

That is 22 pages and 91 sub-pages in total.

## Every page uses the same layout

1. Breadcrumb (Group › Page › Sub-page).
2. Page title, status badge, and page actions (for example Free up space, Refresh, Export).
3. Tabs for the sub-pages, with a count badge where useful (Issues 8, Updates 12).
4. A 12-column grid of panels. Each panel has a title, an optional info tooltip, the data source (shown in Expert mode), and its own actions.

The exact spec for this layout is in [Shell](../02-shell/README.md). The order in which to build the pages is in the [roadmap](../../05-roadmap/README.md).

---

← Previous: [Behavior](../04-behavior/README.md)
