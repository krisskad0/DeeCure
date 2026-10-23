# Changelog

All notable changes to DeeCure are listed here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

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
