# Pages

[Docs](../../README.md) › [UI design](../README.md)

Each insight category is its own page. Each page has sub-pages, shown as tabs under the page title and as a tree in the sidebar. The page names follow common Linux and admin-tool terms.

Each file below covers one page. Each sub-page lists its header actions (if any) and its widgets in grid order. Read [Notation](../01-foundations/notation.md) first.

Before the shell, the first launch shows [Onboarding and first scan](00-onboarding.md).

## Index

| # | Group | Page | Sub-pages (tabs) |
| --- | --- | --- | --- |
| 01 | (none, always first) | [Overview](01-overview.md) | Summary, Issues, Health Score |
| 02 | Monitor | [Performance](02-performance.md) | CPU, Memory, Disk I/O, Network, GPU, Pressure (PSI) |
| 03 | Monitor | [Logs](03-logs.md) | Journal, Errors, Crashes, Kernel |
| 04 | Monitor | [Activity](04-activity.md) | Timeline, Package History, Config Changes, Device Events, DeeCure Actions |
| 06 | Machine | [Hardware](06-hardware.md) | Summary, Processor, Memory Modules, Drives, Graphics, Sensors, Firmware |
| 08 | Machine | [Devices](08-devices.md) | Ports & Devices, Displays, Audio |
| 07 | Machine | [Power](07-power.md) | Battery, Power Profile, Sleep |
| 05 | Software | [Operating System](05-system.md) | Overview, Boot, Kernel, Time & Locale |
| 13 | Software | [Apps & Updates](13-packages.md) | Updates, Installed, Sources |
| 09 | Running | [Processes](09-processes.md) | All Processes, Top Consumers |
| 10 | Running | [Services](10-services.md) | System Services, User Services, Startup Applications |
| 11 | Running | [Scheduled Jobs](11-jobs.md) | Calendar, Cron, Systemd Timers, Anacron, At Jobs |
| 12 | Running | [Containers & VMs](12-containers.md) | Containers, Images, Volumes, Virtual Machines |
| 14 | Data | [Storage](14-storage.md) | Overview, Filesystems, Disk Usage, Cleanup |
| 15 | Data | [Backups](15-backups.md) | Backup Jobs, System Snapshots, Restore Points |
| 16 | Network & Security | [Network](16-network.md) | Interfaces, Connections, Open Ports, DNS & VPN, Traffic |
| 18 | Network & Security | [Security](18-security.md) | Overview, Vulnerabilities, Hardening, Firewall, File Integrity, Malware Scan |
| 17 | Network & Security | [Accounts](17-accounts.md) | Users, Groups, Sessions, Login History, SSH Keys |
| 19 | Network & Security | [Privacy](19-privacy.md) | Device Access, App Permissions, Privacy Settings |
| 23 | Tools | [Optimize](23-optimize.md) | Improvements (Expert level only) |
| 20 | Tools | [Diagnostics](20-diagnostics.md) | Performance Check, Network Test, Hardware Tests |
| 21 | Tools | [Assistant](21-assistant.md) | Chat |
| 22 | (footer) | [Settings](22-settings.md) | General, Access, Notifications, Assistant, Data, About |

The rows follow the sidebar order. The file numbers are older, so they are not in order. That is 23 pages and 89 sub-pages in the design. The app joins some of them into one tab, so it shows 77 ([view levels](../../01-product/view-levels.md#fewer-tabs-for-advanced-and-expert)). Old addresses of moved sub-pages still open the new place ([Sidebar › Old links](../02-shell/sidebar.md#old-links)).

## Every page uses the same layout

1. Breadcrumb (Group › Page › Sub-page).
2. Page title, status badge, and page actions (for example Free up space, Refresh).
3. Tabs for the sub-pages, with a count badge where useful (Issues 8, Updates 12).
4. A 12-column grid of panels. Each panel has a title, an optional info tooltip, the data source (shown in Expert mode), and its own actions.

The exact spec for this layout is in [Shell](../02-shell/README.md). The order in which to build the pages is in the [roadmap](../../05-roadmap/README.md).

---

← Previous: [Behavior](../04-behavior/README.md)
