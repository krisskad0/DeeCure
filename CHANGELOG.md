# Changelog

All notable changes to DeeCure are listed here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Three view levels: **Beginner** (the default: the sidebar, page tabs, and search show only 8 pages and 22 sub-pages), **Advanced** (every page), and **Expert** (also data sources, commands, raw columns, and deep details). Switch in the top bar or Settings › General. Links inside pages still open other pages from Beginner, with a note and a **Switch to Advanced** button. The first-run screens ask "Are you new to Linux?" to pick the level. A saved choice is kept.
- Expert processor details: each core's speed now, allowed range, speed policy and energy preference, preferred cores, and heat slowdowns (with a per-second rate); the chip's power limits, caches, revision, running and installed microcode, and every known CPU flaw with its state; busiest interrupts per second; processes waiting for the disk; and the share of time in each CPU sleep state (Power › Sleep).
- Expert memory details: where every byte goes (apps, file cache, shared memory, kernel caches, page tables), hard page faults and swapping per second, memory per app and service with the app's own memory apart from file cache, real memory (PSS and USS) of the biggest processes, out-of-memory scores, swap and compressed swap, huge pages, the out-of-memory guard's limits, and memory error counts per module (Hardware › Memory).
- Expert drive details: disk use per service and per process (per second and since start); each drive's scheduler, queue, block sizes, TRIM support, partition table and alignment, SATA link, NVMe firmware and temperature limits, and PCIe link against the most both the drive and its slot support; weekly TRIM state; each filesystem's mount options, inode use, and recorded errors (Storage › Filesystems); and every PCIe link with its link errors (Hardware › Peripherals).
- Expert graphics details: each card's clock (or "resting"), allowed range, time in power saving, why it is slowed down, power, whether a second card sleeps and which app keeps it awake, and its screens; graphics use per app (render, video, other engines, memory); and the graphics software: session type, renderer (with a warning for software rendering), Mesa and NVIDIA versions, the fitting video-decoding driver, formats decoded in hardware, and this boot's graphics firmware (Hardware › Graphics).
- Expert network details: the Wi-Fi link (signal in dBm with a plain verdict, band, channel, speed, security, power saving); TCP data sent again (now and since start), failed and reset connections, and UDP drops; errors, drops, and link changes per interface; DNS per connection with DNSSEC and DNS over TLS; routes and VPN routing rules (DNS & VPN); and traffic per app from open TCP connections, labelled "TCP only" (Traffic).
- Expert power details: the battery's voltage against its design minimum, current, power, temperature, health, manufacture date, charge limits, and charge modes; chargers and what a USB-C charger offers against what the laptop accepts; battery used per hour during each recent suspend (Power › Battery); the suspend type and what last woke the computer, every device allowed to wake it with plain names, and devices that could save power but do not (Power › Sleep).
- Expert temperature details (Hardware › Sensors): every thermal zone with where the system slows down, starts fans, and shuts down, and the room left before it slows; cooling devices in plain words (CPU speed limit, lowered heat limit, fans); and heat slowdowns over time next to the CPU temperature, from a new stored counter.
- Expert process drawer (Processes › All Processes, click a row): where the process comes from (program, package, parents, its own processes, the app or service it belongs to), how it is scheduled (CPU time, priority, scheduling, disk priority, CPUs it may use, its busiest threads), its open files against its limit, its sandbox and rights (AppArmor, capabilities, seccomp, namespaces), and what it costs (real memory, swap, hard faults, disk, graphics time, out-of-memory score). Other users' processes say plainly what is hidden.
- Expert services and boot: the cost of each service (CPU now and in total, own memory, peak, disk, tasks); each service's setup (state in plain words, what turns it on, overrides made on this computer, who blocked it: this computer or its package, restarts with the policy); how locked down each service is (systemd's exposure score); what the desktop really waited for at boot (the critical chain, with the slowest step marked); and every unit's start time (System › Boot).
- Expert kernel details (System › Kernel): every kernel setting from the sysctl files with its value now, the package it comes from (or "This computer"), whether its file was changed, and whether a later file replaces it; file watches used against the limit and the programs using most; open files; loaded modules with taint letters and the devices they drive; extra drivers (DKMS) and kernels they are missing for; kernel lockdown; and time sync (server, offset, delay).
- Expert firmware and ports: the firmware security level (HSI) with what it means and every check behind it, Secure Boot and your enrolled keys, management engine version, TPM and whether it unlocks the disk, memory protection from devices, Thunderbolt security, hardware clock, virtualization and KVM, and the UEFI boot entries in boot order (Hardware › Firmware); every USB device's real speed, with USB 3 devices held to USB 2 speed pointed out, USB-C ports with their roles and modes, and Thunderbolt devices with their approval (Hardware › Peripherals).
- Expert devices and hardware errors: cameras, fingerprint reader, touchpads, batteries of mice and headsets, printers, and network chips with Ethernet speed against what the chip supports (Hardware › Peripherals); every screen from its own EDID (maker, model, year, size and pixel density, refresh range, colour depth, HDR) and the backlight (Display & Audio › Displays); sound cards with their codecs (Audio); and one hardware errors panel with the last 7 days of kernel-logged faults by kind, PCIe and memory error counters, driver crash dumps, and drivers Ubuntu offers (Hardware › Summary).
- Expert software: a package drawer with the newest version offered, every source, what it needs and what needs it, and its changed files (Packages › Installed); package health with held, half-installed, and no-repository packages, every snap version (old versions and classic snaps marked), the snap refresh schedule, and Flatpak runtimes no app uses (Packages › Sources); and the full check of every installed file against its package, started only with its button, at low priority (Security › File Integrity, replacing the "comes later" note).
- Expert security and containers: whether automatic security updates are on and what their last runs really did, such as "stopped: the connection is marked as metered" (Security › Overview); every running process grouped by AppArmor profile and mode; network and disk bytes per running container; and the Docker build cache with what can be removed (Containers › Images and Volumes).
- Expert power controls, each with a preview and undo: battery charge limits (presets or your own start and stop), charge mode, CPU turbo, keyboard backlight, radios, and Wi-Fi power saving (Power › Power Profile); stop a device waking the computer, and try power saving for devices that have it off (Power › Sleep). The admin helper writes only to the exact battery, turbo, and device power files, checks each against the live system, and never stops the lid, power buttons, or keyboards from waking the computer or gives power saving to keyboards, mice, or sound devices.
- Power settings are kept after restart: charge limits, charge mode, turbo, and device wake-up and power saving are set again at each start by a small boot service, after the same checks, and skipped for a device that is not the same one. Power › Power Profile lists what is kept and whether it was set at this start. Setting a value back to what it was stops keeping it.
- More power controls (Power › Power Profile): the highest CPU speed on battery (80, 60, or 40% of the top speed; full speed on mains, set again at each plug in and unplug), the PCIe power saving policy, and the suspend type. PCIe saving and deep suspend can misbehave on some hardware, so they are tried for this session first and kept after restart only when you press **Keep**.
- Optimize page (Tools › Optimize, Expert level): every improvement a measurement on this computer shows is needed, with the numbers, and nothing "just in case". Fixes open the usual preview: stop "wait for network" from delaying boot (only when nothing that starts at boot needs the network), turn on the weekly TRIM for SSDs, remove old kernels, and let devices with power saving off sleep when idle. Advice for what software cannot change: heat slowdowns in the last 24 hours, narrow or slow PCIe links, USB 3 devices at USB 2 speed, a worn battery, a restart waiting for a new kernel or microcode, and memory errors. Every check that found nothing to do is listed with what it measured.
- Optimize, batch 2: each cleanup part over its size limit, packages no longer needed and leftover settings, updates, the firewall, and the power mode; a size cap for the system log and fewer snap versions (only when they grew back after a clean-up), unused Flatpak runtimes, the file-watch limit, a better scheduler for hard disks, compressed swap (zswap) when swapping stays high, and finishing a broken package install; a charge limit of 80% or a gentler charge mode when the laptop was on mains most of 14 days, turbo off on battery when heat slows it on battery, and wake-ups with the lid closed; the hardware video driver, a second graphics card that stays awake, drivers Ubuntu recommends, firmware that failed to load, firmware updates, and extra drivers (DKMS) missing for a kernel. Checks that need history say how much they have so far.
- New stored readings: on mains or battery, swap-ins and hard page faults per second.
- Health warnings (docs/01-product/health-warnings.md): about 50 warnings, up from 16, about harm and never about use. They use each part's own limits (CPU, drives, graphics, Wi-Fi), need a problem to last (most 5 minutes), and cover heat and cooling, fans, CPU and memory errors, memory and disk stalls, apps stopped for memory, disk errors, read-only disks, `/boot` and EFI space, file slots, battery heat, faults, running empty, and a weak charger, graphics hangs and no acceleration, kernel crashes, PCIe and firmware errors, USB over-current, thermal shutdowns, services restarting in a loop, the clock, broken package installs, stopped automatic updates, internet not reachable, network errors, weak Wi-Fi, AppArmor, Secure Boot, and CPU security flaws. Each has a help page.
- Settings › Notifications › Warnings and limits (Advanced and Expert): turn any warning off, or change its limits; the default is shown next to each value, and values stay inside a safe range.
- Hardware problems the kernel logs now appear on Activity › Timeline and in the weekly report.
- What closing the lid does, separately on battery and plugged in (suspend, lock, do nothing, or power off). It applies after the next restart, and the app says so until then; it also names another settings file that overrides it.
- Fewer tabs: Logs, Activity, Scheduled Jobs, Containers, Backups, and Accounts merge tabs that showed the same kind of list (89 tabs become 77). Old links open the merged tab at the right part.
- Performance › Disk I/O shows each drive's type (SSD, hard disk, USB drive) and size, so you can tell the drives apart.

### Changed

- DeeCure describes itself as what it now is, deep insight into every layer of the computer: the app menu entry (more search words, such as battery, temperature, fan, and optimize), the app store listing, the package descriptions, and the README.
- "CPU very hot" uses the CPU's own limit and needs 5 minutes near it, instead of a fixed 90 °C (many laptop CPUs run at 90-95 °C under load by design). "Main disk almost full" looks at free space in GB and percent together (under 10% and 20 GB: warning; under 5% or 5 GB: critical) instead of 85% and 93% used, so a large disk with hundreds of GB free no longer warns; other disks, `/boot`, and the EFI partition have their own warnings. The Storage page shows the same warning.

- The package details panel points to Packages › Cleanup for removing packages no longer needed, instead of saying it "comes with the fix engine".
- Faster install: `install.sh` skips optional "recommended" packages (about 30 MB and 57 packages less on a new system; the tray library is still installed), skips refreshing package lists that are less than 6 hours old (and refreshes and retries if an install then fails), and says what it is doing while it downloads.

### Removed

- The greyed-out **Export** button on every page, which did nothing. Settings › Data › **Export report** exports the whole report.
- Three panels that said "Requires Full access" but could not show anything even with Full access: Disk I/O › Top I/O processes, Power › Top power consumers, and Network › Traffic by application. They come back when DeeCure can measure use per app.

### Fixed

- "Disk will be full in about N days" no longer fires after a one-time jump, such as a big install or build followed by flat use: the disk must keep filling over both the last day and the last 3 days, with 3 days of readings, and the slower rate is used. The text now says GB a day. The Storage page's "Full in" tile uses the same forecast, and says how many days of readings it has so far.
- Power › Power Profile "Closing the lid" read only `/etc`; it now reads every logind settings file the way logind does, so a vendor or `/run` setting is no longer missed.
- One part of a page that fails to draw no longer blanks the page: it shows the reason and "Try again", and the rest keeps working.
- Rows were dropped or doubled where two had the same name: two untagged Docker images, two wake sources, two jobs with one name, and the "System › System" breadcrumb.
- Stopping the agent while a folder-size scan ran waited for the scan; it now stops within 5 seconds.

- `install.sh` on a bare system with only `python3-minimal` (no `json` module) now installs `python3`, and says "could not read the release list" instead of "no published release" when it cannot read GitHub's answer.

## [0.1.0-alpha.3] - 2026-10-23

### Fixed

- The first health score is no longer delayed by the one-time history import: the import waits until the first check of the computer is done.

- Right after installing, the app said the agent was unreachable until the next login: the package only enabled the agent for future logins. It now starts the agent for everyone signed in (and restarts it after an upgrade, and stops it on removal), and the app and `deecure` start it themselves if it is still not running.

### Added

- `make clean-install-test`: the real one-command install from GitHub on clean Ubuntu 24.04 (curl) and 26.04 (wget) containers, run again to update, then uninstall.

## [0.1.0-alpha.2] - 2026-10-23

First published pre-release, for testing (0.1.0-alpha.1 did not finish building). Install it with one command: see the [install guide](docs/07-user-guide/install.md).

### Changed

- Features that are not built yet are hidden instead of shown as placeholders: the Assistant page, Settings › Assistant, and "Add a machine".
- Renamed the app from the working name SysDoc to **DeeCure** (package and command `deecure`).

### Added

- **Phase 3: history and trends.**
- History: the agent saves what changed as events, every 5 minutes: apt and dpkg package changes (also from the newest rotated log), snap and Flatpak apps, USB devices, crashes, memory kills, and failed services from the journal (grouped per hour; failures around a shutdown or logout are left out), boots with kernel changes and unclean shutdowns, config files in `/etc`, autostart, systemd user units, and the crontab (with diffs, without inotify), and DeeCure's own fixes.
- Activity pages read the saved history: the timeline groups apt runs and config files, shows a "new kernel" tip with how to start the previous one, and has Undo for fixes and View for config diffs. Package History adds snap and Flatpak.
- New agent calls: `events`, `event`, `notify_settings`, `set_notify_settings`, `notify_test`, `notifications`, `report`, and `weekly_reports`.
- Disk forecast in the agent: `filling`, `days_until_full`, and `percent_per_day` for every filesystem, and the rule `storage.filling_up`.
- Desktop notifications from the agent: critical problems right away, warnings when asked, quiet hours, no repeats within 24 hours, bursts as one summary. Settings › Notifications saves to the agent and can send a test.
- Weekly summary on Sunday 18:00 (saved, and shown as a notification), listed on Overview › Health Score.
- `deecure history` (what changed, `--since`, `--kind`, `--json`, `--fixes`) and `deecure report` (text, `--html FILE`, `--json`); Export report in Settings › Data.
- Optional tray icon (off by default), with start at login, and one running copy of the app.
- Storage schema version 2: metric names stored once, so 2 years of history on a big machine take 124 MB instead of about 250 MB (`make db-size`, in CI). Old snapshots are thinned, and events are capped.
- KB topic: start once with the previous kernel.
- Every area is checked: new rules `hardware.cpu_hot`, `network.offline`, `packages.updates_available`, `accounts.extra_root`, `jobs.job_failed` (failed scheduled jobs, no longer also reported as failed services), `containers.port_exposed`, and `privacy.popcon`, with KB articles. Overview › Subsystems shows a real number for every row, and "Not checked" instead of "Unknown" for areas without rules.
- New metrics: `net.default_route`, `accounts.people`, `accounts.uid0_extra`, `privacy.popcon`, `containers.exposed`.
- Access level is a real choice (onboarding and Settings › Access): Basic locks admin read-outs and admin fix steps; Full needs the admin helper.
- Settings › Data: keep history for 90 days, 1 year, or 2 years, and Delete history (asks first; restore points and settings stay; deleted history cannot come back from old logs).
- Settings › General: refresh interval (2 s, 5 s, 15 s, Manual; the Refresh button reloads every panel) and temperature unit (°C or °F).
- The app says "Agent needs a restart" when the running agent is older than the app.
- User guide: [Using DeeCure](docs/07-user-guide/using-deecure.md).

- Rust workspace with `deecure-core`, `deecure-agent`, and `deecure-cli` (version and help only for now).
- Shared types in `deecure-core`: categories, severity and UI status, rule and fix IDs, findings, metrics, events, snapshots, timestamps, and JSON-RPC 2.0 messages (`version`, `status`).
- Agent skeleton: listens on a private Unix socket (mode 0600, same user only), supports systemd socket activation, answers `version` and `status`, and cleans up on stop. systemd user units in `packaging/systemd/`, a dev script to install them, and `make agent`.
- Storage: SQLite database at `~/.local/share/deecure/deecure.db` (private, WAL mode) with the six tables, schema migrations, minute-averaged metrics, and an hourly cleanup that rolls old points up into 15-minute and daily averages. `status` now returns open findings from the database. New agent option `--db PATH`.
- Collectors: CPU, memory, filesystems, battery (every minute), and systemd services over D-Bus (every 5 minutes). Inventories are saved only when they change. New agent calls `metrics` and `snapshot`.
- Rules engine: YAML rules checked on load, a small condition language (`and`, `or`, `not`, comparisons), facts from the newest data, and findings that open, update, and resolve on their own. Missing facts never change findings. First rules: `storage.root_almost_full`, `services.unit_failed`, `power.battery_worn`. New agent option `--rules DIR`.
- CLI: `deecure status` shows open problems in plain English with icon, word, and color (honours `NO_COLOR`), and `--json` prints the raw report. The status report now says when the rules last ran (`checked_at`). New `make status`.
- Fixture tests: `scripts/capture-fixture.py` saves a machine's data with private data removed, and a test replays every machine in `tests/fixtures/` through the collectors and rules, checking the exact findings. Three fixtures: a real Dell Latitude 3450, the prototype's "Riya" laptop, and a desktop with no battery.
- First `.deb` package: `make deb` builds it and checks it with lintian; `make deb-test` installs, uses, and purges it in a clean Ubuntu 24.04 container. It ships the agent, the `deecure` command with a man page, the rules, and systemd user units enabled for every user (not for system accounts). CI builds and tests the package on every push and keeps it as a download. CI is also sturdier: retries, time limits, and older runs are cancelled by newer pushes.
- Performance check: `make perf` measures time to first result, idle CPU, peak RAM, and database size against the budget, and CI fails the build when the agent is over it.
- Desktop app (Phase 1 start): Tauri v2 + React + TypeScript in `app/`, with the prototype's design tokens, an app icon, and a first live screen that lists the agent's findings with severity badges and plain-English text. The CLI and the app share one agent client in `deecure-core`. `make app` runs it with live reload; CI type-checks and builds the web UI and lints the Tauri side.
- UI foundations: bundled Ubuntu Sans and Ubuntu Sans Mono fonts (works offline), spec typography, an `Icon` component, and `scripts/gen-ui.py`, which generates the icon set and the 22-page list from the UI spec (checked in CI).
- App shell: top bar, sidebar with all 22 pages and their sub-pages (rail mode and a drawer on narrow windows), sticky page header with breadcrumb, status badge, actions, and tabs, and a 12-column grid. Page status comes from real findings and says "Unknown" for areas DeeCure does not check yet; the agent's `status` now lists the areas it checks. Overview › Summary and Issues show live findings; other pages say they are not built yet and link to their spec.
- Overlays: Ctrl K command palette (every page and sub-page, plus the services the agent knows; arrow keys, Enter, Esc), the notifications list under the bell, the machines list, and toast messages. New Tauri command `snapshot`.
- UI components: panel (info tooltip, Expert-only source chip), buttons, KPI tiles with sparklines and usage bars, inline meter, key-value list, status list, checklist, activity timeline, alert banner, and code and log blocks. A development-only component gallery shows each one with the prototype's sample data. UI tests run with Vitest in `make web` and CI. Charts will be our own SVG components (ADR 0006).
- Charts, drawn as our own SVG components: line/area (stacked areas, threshold, forecast, hover crosshair and tooltip, legend), bar, donut, gauge and gauge group, day × hour heatmap, horizontal bar list, disk allocation bar, 24-hour swim lanes, and lifecycle bar. All appear in the component gallery with the spec's data, with tests for scales, labels, and colours.
- More collectors: network traffic, disk reads and writes, pressure (PSI), temperatures and fans, per-core CPU, memory breakdown, and battery time left.
- Health score: `status` now returns a 0-100 score and the points each area lost (critical −15, warning −5, at most 15 per area). The agent saves it after every check, and `deecure status` shows it.
- New agent calls: `latest` (newest value of every metric), `inventory` (live `system` facts and `processes`), and `max_points` on `metrics`, which averages long ranges so they always fit in one message.
- Overview pages with real data: Summary (KPI row, CPU and memory chart over the chosen time range, subsystems, active issues, health gauge, disk donut, top processes, system facts), Issues (counts by severity), and Health Score (30-day trend and breakdown).
- Performance pages with real data: CPU (load, clock speed, context switches, stacked usage by type, per-core bars, top consumers), Memory (usage over time, composition that adds up to the total, top consumers with swap), Disk I/O (throughput, IOPS, busy time, wait time, devices), Network (throughput, data today, connections, interfaces), GPU (model from the PCI list, driver, honest "not reported" values), and Pressure (some and full, peak in range). New collectors values and the live `network` and `gpu` inventories feed them.
- System pages with real data: Overview (release, desktop, virtualization, Ubuntu Pro, release lifecycle bar from distro-info), Boot (phase times, slowest units, history of boots), Updates (pending packages from a read-only `apt list --upgradable` every hour, apt history), Kernel (installed kernels, modules, taint), and Time & Locale.
- Processes pages with real data: all processes with state counts, user filter, start time, and a row-inspect side drawer; top consumers by CPU and memory.
- New rules: `security.updates_pending` (warning) and `system.reboot_required` (info).
- Hardware pages with real data: machine and firmware (Secure Boot, TPM, BIOS), processor with performance and efficient cores, memory, disks, graphics driver checks, all sensors and fans, PCI/USB/input/Bluetooth devices with drivers, and firmware updates from fwupd.
- Power pages with real data: battery charge, health, capacity, cycles, charge limit, and trends; power mode and battery policies; sleep history from the journal.
- Display & Audio pages with real data: every screen with resolution, refresh, scale, and primary (from GNOME); sound devices, volumes, defaults, and streams (from PipeWire).
- The agent asks D-Bus services only when they already run, and never starts one.
- Storage pages with real data: used, free, inodes, encryption, a 30-day trend with a "full in" forecast, mount points, fstab checks, a background folder-size scan (largest folders and files), and cleanup candidates (Trash, thumbnails, apt cache, old snap revisions, journal, unused Docker images and build cache). Nothing is deleted yet.
- Services pages: memory per system service, user services, and startup applications.
- Scheduled Jobs pages: next-24-hours swim lanes, runs by hour for 7 days, cron (plain-English schedules and next run in local time), systemd timers with results, anacron, and at jobs.
- Fix: the .deb test counts the shipped rules instead of expecting 3.
- Logs pages with real data from the journal: entries of the last 24 hours with hourly volume, errors grouped and classified with a day × hour heatmap, crashes (kernel, systemd, apport), the kernel log, and boot history with unclean shutdowns.
- Activity pages: a 7-day change timeline (packages, configuration, devices, boots, crashes), package history with versions, files changed in /etc, and USB device events. Overview now shows recent activity and errors by hour.
- Network pages with real data: interfaces with addresses and traffic, Wi-Fi signal, connectivity, connections and listening ports with who can reach them, the firewall, DNS, VPNs, and traffic over time.
- Accounts pages: users and admins, groups and what they allow, sessions, sign-ins and sudo use, and SSH keys and server settings.
- Security pages: a posture checklist, packages with security fixes, kernel hardening settings, SUID programs and their packages, and malware signs. New rule `security.firewall_off` (a warning only when ports are open to every network).
- Privacy pages: apps using the microphone, camera, or screen now, snap and Flatpak permissions, and desktop privacy settings.
- Packages pages: every package from APT, snap, Flatpak, and AppImages with size and who installed it, software sources with signature checks, and what could be removed (from a simulation).
- Containers pages: Docker containers with ports open to every network flagged, images, volumes (never offered for deletion), and virtual machines.
- Backups pages: Déjà Dup and Timeshift status and snapshots. New rules `backups.none` and `backups.overdue`.
- Diagnostics: a performance check that names the main cause of slowness, a network test (gateway, packet loss, DNS, internet, IPv6), and hardware tests. New diagnostics runner component.
- Settings: Expert mode, default time range, and narrow sidebar (shared across the app at once), access levels, notification choices, data retention and database size, and About with the agent's version, uptime, memory, and rules. The Assistant page explains what is planned.
- Every one of the 22 pages and 91 tabs now shows real data.
- New package `deecure-desktop`: the app with a menu entry, icons, AppStream data, and a man page. `deecure` stays GUI-free for servers. The container test installs both, checks every library is found, and purges them.
- The agent reads installed packages with `dpkg-query` instead of dpkg's private files.
- Knowledge base: 125 plain-English entries for services, ports, log errors, packages, and how-to topics, shipped in `/usr/share/deecure/kb`. Issues now have a "How to fix" article; services, ports, errors, and packages are explained where they appear.
- Onboarding: three short first-launch screens (welcome, privacy, access level) and a live first check where each area is a real request to the agent. Settings › General can show them again.
- Insights catalog: every insight and whether it needs no password, the admin helper, or is not possible.
- Admin helper (`deecure-helper`): a root D-Bus service with 12 fixed operations and 6 read-outs. polkit asks for the password per operation; inputs are checked against strict patterns and the live system; programs run without a shell; every call is audited; it exits after 2 idle minutes. Installed but never enabled or started by the package.
- Fix engine: 22 fixes with a preview (untick steps, undo and admin badges), live progress, verification, history, and undo. Fix buttons across the app; `deecure fix`, `deecure history`, and `deecure undo` in the terminal.
- Full-access read-outs: SMART disk health, firewall rules, memory modules, and the serial number.
- `make helper-test` (also in CI) tests the helper end to end with a real system D-Bus and polkit in a container.
- Distribution, ready but not published: `packaging/install.sh` (with `--server`, `--beta`, `--dry-run`, `--uninstall`), a signed APT repository builder, a release workflow for `v*` tags, and `scripts/bump-version.sh`. `make install-test` checks the whole path against a repository signed with a throwaway key.
- Install guide rewritten to match what is built.
- Data table (search, filter chips, unit-aware sorting, paging with totals, row selection with a bulk action, Expert-only columns, empty text) and form controls (switch, select, radio cards, field rows, state tiles).
- First real page: Services › System Services, with a KPI row, an alert per failed service, and a searchable, filterable units table. The services collector now also records each service's enabled-at-boot state and restarts since boot.
- Pinned Rust 1.99.1 (`rust-toolchain.toml`), shared lints, `Makefile` (`build`, `test`, `lint`, `fmt`), and a GitHub Actions CI workflow.

- Project documentation in `docs/`: product, architecture, UI design (foundations, shell, 24 components, 22 pages), development, roadmap, packaging, and user guide.
- Repository folder structure for the Rust crates, desktop app, rules, knowledge base, packaging, scripts, and tests.
- Community files: README, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, LICENSE (Apache-2.0, proposed).

### Fixed

- The agent keeps answering the app and the CLI while it imports history on its first start.
- Key-value lists in flush panels (24 of them, such as Settings › About) now line up with tables and setting rows.
- Network totals no longer count container traffic twice, and container links (`veth…`) no longer fill the history.
