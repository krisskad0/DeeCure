# Sidebar

[Docs](../../README.md) › [UI design](../README.md) › [Shell](README.md)

23 pages and 78 sub-pages. Group order and page order are exactly as listed. Advanced and Expert show all of them; Beginner shows fewer (see [Beginner mode](#beginner-mode)).

Each group answers one plain question:

- **Monitor:** What is happening now, and what happened before?
- **Machine:** What is inside my computer, and is it healthy?
- **Software:** Which operating system and apps do I have, and are they up to date?
- **Running:** What runs on my computer, now or on a timer?
- **Data:** Do I have space, and is my data safe?
- **Network & Security:** Who and what can reach my computer?
- **Tools:** Things I start on purpose: fixes, tests, and the assistant.

| Group | Page | ID | Icon | Initial status | Sidebar mark | Sub-pages (ID: label) |
| --- | --- | --- | --- | --- | --- | --- |
|  | Overview | `overview` | `grid` | {crit:Critical} | Count pill (9) | `summary`: Summary · `issues`: Issues (count 9, red) · `score`: Health Score |
| Monitor | Performance | `performance` | `perf` | {ok:Healthy} | - | `cpu`: CPU · `memory`: Memory · `disk`: Disk I/O · `network`: Network · `gpu`: GPU · `pressure`: Pressure (PSI) |
| Monitor | Logs | `logs` | `logs` | {warn:Warning} | Yellow dot | `journal`: Journal · `errors`: Errors and crashes (count 37) |
| Monitor | Activity | `activity` | `hist` | {ok:Healthy} | - | `timeline`: Timeline · `config`: Config Changes |
| Machine | Hardware | `hardware` | `chip` | {ok:Healthy} | - | `summary`: Summary · `parts`: All Parts · `processor`: Processor · `memory`: Memory Modules · `storage`: Drives · `graphics`: Graphics · `sensors`: Sensors · `firmware`: Firmware |
| Machine | Devices | `devices` | `usb` | {ok:Healthy} | - | `ports`: Ports & Devices · `displays`: Displays · `audio`: Audio |
| Machine | Power | `power` | `bat` | {warn:Warning} | Yellow dot | `battery`: Battery · `profile`: Power Profile · `sleep`: Sleep |
| Software | Operating System | `system` | `sys` | {info:Info} | - | `overview`: Overview · `boot`: Boot · `kernel`: Kernel · `time`: Time & Locale |
| Software | Apps & Updates | `packages` | `pkg` | {warn:Warning} | Yellow dot | `updates`: Updates (count 12) · `installed`: Installed · `sources`: Sources |
| Running | Processes | `processes` | `layers` | {ok:Healthy} | - | `all`: All Processes · `top`: Top Consumers |
| Running | Services | `services` | `cog` | {crit:Critical} | Red dot | `system`: System Services (count 1, red) · `user`: User Services · `startup`: Startup Applications |
| Running | Scheduled Jobs | `jobs` | `cal` | {ok:Healthy} | - | `calendar`: Calendar · `all`: All Jobs |
| Running | Containers & VMs | `containers` | `cube` | {warn:Warning} | Yellow dot | `containers`: Containers · `images`: Images and Volumes · `vms`: Virtual Machines |
| Data | Storage | `storage` | `db` | {crit:Critical} | Red dot | `overview`: Overview · `filesystems`: Filesystems · `usage`: Disk Usage · `cleanup`: Cleanup |
| Data | Backups | `backups` | `backup` | {warn:Warning} | Yellow dot | `jobs`: Backups · `restore`: Restore Points |
| Network & Security | Network | `network` | `wifi` | {ok:Healthy} | - | `interfaces`: Interfaces · `connections`: Connections · `ports`: Open Ports · `dns`: DNS & VPN · `traffic`: Traffic |
| Network & Security | Security | `security` | `shield` | {warn:Warning} | Yellow dot | `overview`: Overview · `vulns`: Vulnerabilities (count 3) · `hardening`: Hardening · `firewall`: Firewall · `integrity`: File Integrity · `malware`: Malware Scan |
| Network & Security | Accounts | `accounts` | `users` | {ok:Healthy} | - | `users`: Users and Groups · `sessions`: Sign-ins · `ssh`: SSH Keys |
| Network & Security | Privacy | `privacy` | `eye` | {ok:Healthy} | - | `access`: Device Access · `perms`: App Permissions · `settings`: Privacy Settings |
| Tools | Optimize | `optimize` | `bolt` | none (no badge) | - | `all`: Improvements |
| Tools | Diagnostics | `diagnostics` | `gauge` | none (no badge) | - | `performance`: Performance Check · `network`: Network Test · `hardware`: Hardware Tests |
| Tools | Assistant | `assistant` | `chat` | none (no badge) | - | `chat`: Chat |
| Footer | Settings | `settings` | `sliders` | none (no badge) | - | `general`: General · `access`: Access · `notifications`: Notifications · `assistant`: Assistant · `data`: Data · `about`: About |

**Sidebar item spec**

- Group label: 10.5px 700 uppercase muted, padding 0 10 5. Groups have 12px top margin (4px for the first). Overview has no group, so it shows no group label and always comes first.
- Item: 32px tall, gap 10, padding 0 8 0 10, radius 6, `--ink-2` 500. Icon 17px. Hover: `--hover` background. Active: `--accent-soft` background, `--accent` text, 600.
- Right side: Overview shows a count pill (number of open issues; `--crit-soft` background, `--crit-ink`, 11px 700, 18px tall). Other pages show a 7px dot if their status is Warning (yellow) or Critical (red). Pages with more than one sub-page show a 14px chevron that rotates 90° when open.
- Sub-tree (open page only): margin-left 18, padding-left 10, 1px `--line` left border. Sub-items 13px, padding 4 8, radius 5. The active sub-page is `--accent` 600.
- Behavior: clicking a different page navigates to its first sub-page and opens its tree. Clicking the current page toggles its tree. Each page remembers whether its tree is open.
- Footer (pinned to the bottom with a top border): Settings item, then a "Collapse" button with a panel icon. Collapse toggles **rail mode**: 60px wide, icons only, no labels, group labels, sub-trees, chevrons or counts. Status dots move to the item's top-right corner. Groups are separated by a 1px top border. The button label changes to "Expand".

## Old links

Some sub-pages moved to another page on 2026-11-16. An old link opens the new place, through `MOVED` in `app/src/route.ts`:

| Old address | Opens |
| --- | --- |
| `hardware/peripherals` | `devices/ports` |
| `display/displays` | `devices/displays` |
| `display/audio` | `devices/audio` |
| `display` (no sub-page) | `devices/displays` |
| `system/updates` | `packages/updates` |
| `network/firewall` | `security/firewall` |
| `logs/boots` | `system/boot` (the Boot history table moved there) |
| `packages/cleanup` | `storage/cleanup` (the Reclaimable panel moved there) |
| `packages/permissions` | `privacy/perms` (it was the same table) |

## Beginner mode

Beginner (the default [view level](../../01-product/view-levels.md)) keeps the app light. The sidebar, the page tabs, and the search palette show only these pages and sub-pages, in the same order. Advanced shows everything above except Optimize, which only Expert shows. The list lives in one place in code: `BEGINNER` in `app/src/visible.ts`.

| Page | Sub-pages in Beginner mode |
| --- | --- |
| Overview | all: Summary, Issues, Health Score |
| Performance | CPU, Memory, Disk I/O, Network, GPU (not Pressure (PSI), a kernel stall measure) |
| Activity | Timeline |
| Power | all: Battery, Power Profile, Sleep |
| Operating System | Overview |
| Apps & Updates | Updates |
| Storage | Overview, Cleanup |
| Security | Overview |
| Settings | all |

- **Links still work.** Links inside pages (for example Security › Overview › Hardening "View") open any page in either mode.
- **The current page stays.** When a link opens a page or sub-page that Beginner mode leaves out, it is shown in the sidebar and the tabs while the user is on it.
- **A note explains it.** Such a page starts with an info banner: "This page is part of the Advanced view" with a **Switch to Advanced** button.
- **Problems are never hidden.** Every issue still shows in Overview › Issues and the bell, whatever page it belongs to. Status dots show only on the pages in the menu.
- **Switching is instant.** The top bar, Settings › General, and onboarding step 3 share one pref (`mode`: `beginner`, `advanced`, or `expert`); every menu updates at once.
