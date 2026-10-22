# Architecture decision records

[Docs](../../README.md) › [Architecture](../README.md)

An architecture decision record (ADR) is a short note about one big technical choice. It says what we chose, why, and what it costs. We never edit an old decision. We write a new one that replaces it.

## Index

| # | Decision | Status |
| --- | --- | --- |
| [0001](0001-rust-for-agent-helper-cli.md) | Rust for the agent, helper, and CLI | Proposed |
| [0002](0002-tauri-react-desktop-ui.md) | Tauri v2 + React + TypeScript for the desktop app | Accepted |
| [0003](0003-deb-and-apt-repository.md) | `.deb` package and our own APT repository as the main channel | Accepted |
| [0004](0004-license-apache-2.0.md) | Apache-2.0 license | Proposed |
| [0005](0005-root-helper-dbus-polkit.md) | Separate root helper on D-Bus, guarded by polkit | Accepted |

**Status words:** Proposed (waiting for a team decision) · Accepted · Replaced by #NNNN.

## Template

Copy this into a new file named `NNNN-short-title.md`.

```markdown
# NNNN. Title

- Status: Proposed
- Date: YYYY-MM-DD

## Context
What problem do we face? What limits us?

## Decision
What we chose, in one or two sentences.

## Why
The main reasons. Name the options we did not pick.

## Consequences
What gets easier, what gets harder, and what we must do next.
```
