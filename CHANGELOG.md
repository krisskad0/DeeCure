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
- Pinned Rust 1.99.1 (`rust-toolchain.toml`), shared lints, `Makefile` (`build`, `test`, `lint`, `fmt`), and a GitHub Actions CI workflow.

- Project documentation in `docs/`: product, architecture, UI design (foundations, shell, 24 components, 22 pages), development, roadmap, packaging, and user guide.
- Repository folder structure for the Rust crates, desktop app, rules, knowledge base, packaging, scripts, and tests.
- Community files: README, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, LICENSE (Apache-2.0, proposed).
