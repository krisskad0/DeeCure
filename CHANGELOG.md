# Changelog

All notable changes to DeeCure are listed here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed

- Renamed the app from the working name SysDoc to **DeeCure** (package and command `deecure`).

### Added

- Rust workspace with `deecure-core`, `deecure-agent`, and `deecure-cli` (version and help only for now).
- Shared types in `deecure-core`: categories, severity and UI status, rule and fix IDs, findings, metrics, events, snapshots, timestamps, and JSON-RPC 2.0 messages (`version`, `status`).
- Pinned Rust 1.99.1 (`rust-toolchain.toml`), shared lints, `Makefile` (`build`, `test`, `lint`, `fmt`), and a GitHub Actions CI workflow.

- Project documentation in `docs/`: product, architecture, UI design (foundations, shell, 24 components, 22 pages), development, roadmap, packaging, and user guide.
- Repository folder structure for the Rust crates, desktop app, rules, knowledge base, packaging, scripts, and tests.
- Community files: README, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, LICENSE (Apache-2.0, proposed).
