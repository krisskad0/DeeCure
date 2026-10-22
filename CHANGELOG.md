# Changelog

All notable changes to DeeCure are listed here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed

- Renamed the app from the working name SysDoc to **DeeCure** (package and command `deecure`).

### Added

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
- Data table (search, filter chips, unit-aware sorting, paging with totals, row selection with a bulk action, Expert-only columns, empty text) and form controls (switch, select, radio cards, field rows, state tiles).
- Pinned Rust 1.99.1 (`rust-toolchain.toml`), shared lints, `Makefile` (`build`, `test`, `lint`, `fmt`), and a GitHub Actions CI workflow.

- Project documentation in `docs/`: product, architecture, UI design (foundations, shell, 24 components, 22 pages), development, roadmap, packaging, and user guide.
- Repository folder structure for the Rust crates, desktop app, rules, knowledge base, packaging, scripts, and tests.
- Community files: README, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, LICENSE (Apache-2.0, proposed).
