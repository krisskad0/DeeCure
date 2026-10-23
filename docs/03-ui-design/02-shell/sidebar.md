# Sidebar

[Docs](../../README.md) › [UI design](../README.md) › [Shell](README.md)

22 pages and 91 sub-pages. Group order and page order are exactly as listed. This is Expert mode; Beginner mode shows fewer (see [Beginner mode](#beginner-mode)).

| Group | Page | ID | Icon | Initial status | Sidebar mark | Sub-pages (ID: label) |
| --- | --- | --- | --- | --- | --- | --- |
| Monitoring | Overview | `overview` | `grid` | {crit:Critical} | Count pill (9) | `summary`: Summary · `issues`: Issues (count 9, red) · `score`: Health Score |
| Monitoring | Performance | `performance` | `perf` | {ok:Healthy} | - | `cpu`: CPU · `memory`: Memory · `disk`: Disk I/O · `network`: Network · `gpu`: GPU · `pressure`: Pressure (PSI) |
| Monitoring | Logs | `logs` | `logs` | {warn:Warning} | Yellow dot | `journal`: Journal · `errors`: Errors (count 37) · `crashes`: Crashes · `kernel`: Kernel · `boots`: Boots |
| Monitoring | Activity | `activity` | `hist` | {ok:Healthy} | - | `timeline`: Timeline · `packages`: Package History · `config`: Config Changes · `devices`: Device Events · `deecure`: DeeCure Actions |
| System | System | `system` | `sys` | {info:Info} | - | `overview`: Overview · `boot`: Boot · `updates`: Updates (count 12) · `kernel`: Kernel · `time`: Time & Locale |
| System | Hardware | `hardware` | `chip` | {ok:Healthy} | - | `summary`: Summary · `processor`: Processor · `memory`: Memory · `storage`: Storage Devices · `graphics`: Graphics · `sensors`: Sensors · `peripherals`: Peripherals · `firmware`: Firmware |
| System | Power | `power` | `bat` | {warn:Warning} | Yellow dot | `battery`: Battery · `profile`: Power Profile · `sleep`: Sleep |
| System | Display & Audio | `display` | `mon` | {ok:Healthy} | - | `displays`: Displays · `audio`: Audio |
| Workloads | Processes | `processes` | `layers` | {ok:Healthy} | - | `all`: All Processes · `top`: Top Consumers |
| Workloads | Services | `services` | `cog` | {crit:Critical} | Red dot | `system`: System Services (count 1, red) · `user`: User Services · `startup`: Startup Applications |
| Workloads | Scheduled Jobs | `jobs` | `cal` | {ok:Healthy} | - | `calendar`: Calendar · `cron`: Cron · `timers`: Systemd Timers · `anacron`: Anacron · `at`: At Jobs |
| Workloads | Containers | `containers` | `cube` | {warn:Warning} | Yellow dot | `containers`: Containers · `images`: Images · `volumes`: Volumes · `vms`: Virtual Machines |
| Software & Data | Packages | `packages` | `pkg` | {warn:Warning} | Yellow dot | `installed`: Installed · `sources`: Sources · `cleanup`: Cleanup · `permissions`: Permissions |
| Software & Data | Storage | `storage` | `db` | {crit:Critical} | Red dot | `overview`: Overview · `filesystems`: Filesystems · `usage`: Disk Usage · `cleanup`: Cleanup |
| Software & Data | Backups | `backups` | `backup` | {warn:Warning} | Yellow dot | `jobs`: Backup Jobs · `snapshots`: System Snapshots · `restore`: Restore Points |
| Network & Access | Network | `network` | `wifi` | {ok:Healthy} | - | `interfaces`: Interfaces · `connections`: Connections · `ports`: Open Ports · `firewall`: Firewall · `dns`: DNS & VPN · `traffic`: Traffic |
| Network & Access | Accounts | `accounts` | `users` | {ok:Healthy} | - | `users`: Users · `groups`: Groups · `sessions`: Sessions · `logins`: Login History · `ssh`: SSH Keys |
| Network & Access | Security | `security` | `shield` | {warn:Warning} | Yellow dot | `overview`: Overview · `vulns`: Vulnerabilities (count 3) · `hardening`: Hardening · `integrity`: File Integrity · `malware`: Malware Scan |
| Network & Access | Privacy | `privacy` | `eye` | {ok:Healthy} | - | `access`: Device Access · `perms`: App Permissions · `settings`: Privacy Settings |
| Tools | Diagnostics | `diagnostics` | `gauge` | none (no badge) | - | `performance`: Performance Check · `network`: Network Test · `hardware`: Hardware Tests |
| Tools | Assistant | `assistant` | `chat` | none (no badge) | - | `chat`: Chat |
| Footer | Settings | `settings` | `sliders` | none (no badge) | - | `general`: General · `access`: Access · `notifications`: Notifications · `assistant`: Assistant · `data`: Data · `about`: About |

**Sidebar item spec**

- Group label: 10.5px 700 uppercase muted, padding 0 10 5. Groups have 12px top margin (4px for the first).
- Item: 32px tall, gap 10, padding 0 8 0 10, radius 6, `--ink-2` 500. Icon 17px. Hover: `--hover` background. Active: `--accent-soft` background, `--accent` text, 600.
- Right side: Overview shows a count pill (number of open issues; `--crit-soft` background, `--crit-ink`, 11px 700, 18px tall). Other pages show a 7px dot if their status is Warning (yellow) or Critical (red). Pages with more than one sub-page show a 14px chevron that rotates 90° when open.
- Sub-tree (open page only): margin-left 18, padding-left 10, 1px `--line` left border. Sub-items 13px, padding 4 8, radius 5. The active sub-page is `--accent` 600.
- Behavior: clicking a different page navigates to its first sub-page and opens its tree. Clicking the current page toggles its tree. Each page remembers whether its tree is open.
- Footer (pinned to the bottom with a top border): Settings item, then a "Collapse" button with a panel icon. Collapse toggles **rail mode**: 60px wide, icons only, no labels, group labels, sub-trees, chevrons or counts. Status dots move to the item's top-right corner. Groups are separated by a 1px top border. The button label changes to "Expand".

## Beginner mode

Beginner mode (the default) keeps the app light. The sidebar, the page tabs, and the search palette show only these pages and sub-pages, in the same order. Expert mode shows everything above. The list lives in one place in code: `BEGINNER` in `app/src/visible.ts`.

| Page | Sub-pages in Beginner mode |
| --- | --- |
| Overview | all: Summary, Issues, Health Score |
| Performance | CPU, Memory |
| Activity | Timeline |
| System | Overview, Updates |
| Power | all: Battery, Power Profile, Sleep |
| Storage | Overview, Cleanup |
| Security | Overview |
| Settings | all |

- **Links still work.** Links inside pages (for example Security › Overview › Hardening "View") open any page in either mode.
- **The current page stays.** When a link opens a page or sub-page that Beginner mode leaves out, it is shown in the sidebar and the tabs while the user is on it.
- **A note explains it.** Such a page starts with an info banner: "This page is part of Expert mode" with a **Switch to Expert mode** button.
- **Problems are never hidden.** Every issue still shows in Overview › Issues and the bell, whatever page it belongs to. Status dots show only on the pages in the menu.
- **Switching is instant.** The top bar, Settings › General, and onboarding step 3 share one pref (`mode`); every menu updates at once.
