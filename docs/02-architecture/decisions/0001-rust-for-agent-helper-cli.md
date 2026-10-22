# 0001. Rust for the agent, helper, and CLI

- Status: Proposed (open decision 5: Rust vs Go)
- Date: 2026-10-22

## Context

The agent runs all the time in the background. The helper runs as root. Both must be small, fast, and safe. The plan sets a budget of under 1% idle CPU and under 60 MB RAM for the agent.

## Decision

Write `deecure-agent`, `deecure-helper`, `deecure-cli`, and the shared `deecure-core` library in Rust, in one Cargo workspace.

## Why

- No runtime to install, small memory use.
- Memory safety matters most in the root helper.
- Good Linux libraries: `zbus` (D-Bus), journal bindings, `rusqlite` (SQLite).
- Tauri (the desktop app shell) is also Rust, so the team uses one language for all backend code.
- **Option not picked:** Go. It is also a good fit and simpler to learn. Pick Go if the team knows it much better.

## Consequences

- Contributors need Rust skills for core code. Rules and the knowledge base stay in YAML, so non-Rust people can still help.
- Building from source for a Launchpad PPA needs vendored crates (see [APT repository](../../06-packaging/apt-repository.md)).
