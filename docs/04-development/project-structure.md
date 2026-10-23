# Project structure

[Docs](../README.md) › [Development](README.md)

What every folder is for.

```
deecure/
├── .github/                 # Issue and PR templates; CI (workflows/ci.yml) and releases (workflows/release.yml)
├── app/                     # Desktop app: Tauri v2 + React + TypeScript
│   ├── src/                 #   React UI
│   │   ├── shell/           #     Top bar, sidebar, page header, palette, drawers, fix drawer, onboarding
│   │   ├── components/      #     The spec's components and SVG charts
│   │   ├── pages/<page>/    #     One folder per page; every tab shows real data
│   │   ├── spec/            #     Generated from the UI spec by scripts/gen-ui.py (do not edit)
│   │   ├── api.ts           #     Calls into the Tauri side, with the agent's types
│   │   ├── visible.ts       #     Pages hidden until they are built (the assistant)
│   │   ├── access.ts, units.ts, prefs.ts  # Access level, units and refresh, saved preferences
│   │   └── dev/             #     Component gallery (development only)
│   └── src-tauri/src/       #   Tauri side: main.rs (commands, agent client), tray.rs (tray, one copy)
├── crates/                  # Rust workspace
│   ├── deecure-core/        #   Shared types: findings, metrics, events, fix plans, reports, RPC messages, local time
│   ├── deecure-agent/src/   #   The background agent
│   │   ├── collect/         #     Every-minute collectors (CPU, memory, disks, sensors, battery, network, services…)
│   │   ├── inventory/       #     Live read-outs the app asks for (processes, hardware, logs, packages…)
│   │   ├── history/         #     What changed: package logs, snap/Flatpak, journal, boots, config files
│   │   ├── rules/           #     Rule loader, condition language, facts, disk forecast
│   │   ├── store/           #     SQLite: schema, metrics, events, findings, fixes, retention
│   │   ├── fixes.rs         #     Fix planner (plans only; nothing runs here)
│   │   ├── notify.rs        #     Notifications and the weekly summary
│   │   ├── report.rs        #     Reports for the weekly summary, the CLI, and Export
│   │   └── handler.rs, server.rs  # The JSON-RPC socket
│   ├── deecure-fix/         #   Runs fix steps in the user's session, and undo
│   ├── deecure-helper/      #   Root helper: D-Bus service with a fixed list of actions
│   └── deecure-cli/         #   The `deecure` command
├── rules/<area>/            # Rule YAML files, one folder per area
├── kb/                      # Knowledge base YAML: services, ports, errors, packages, topics
├── packaging/
│   ├── debian/              #   Debian packaging (copied to debian/ at build time)
│   ├── desktop/             #   .desktop file, AppStream metainfo, icon
│   ├── helper/              #   Helper: systemd unit, D-Bus activation and policy, polkit actions
│   ├── systemd/             #   The agent's systemd user units
│   ├── man/                 #   Man pages
│   ├── apt-repo/            #   Signed APT repository config
│   └── install.sh           #   One-line installer
├── scripts/                 # build-deb, test-deb, test-helper, test-install, build-apt-repo,
│                            # bump-version, perf-check, capture-fixture, gen-ui, dev-agent-units
├── tests/fixtures/          # Real /proc, /sys, and service lists from real machines, with expected findings
├── docs/                    # All documentation (start at docs/README.md; what is left: docs/05-roadmap/pending.md)
├── Cargo.toml               # Workspace manifest and shared lints
├── rust-toolchain.toml      # Pinned Rust version (1.99.1)
├── Makefile                 # Short commands (make help)
└── README.md  LICENSE  CONTRIBUTING.md  CODE_OF_CONDUCT.md  SECURITY.md  CHANGELOG.md
```

## Where code goes

| You are changing… | Put it in… |
| --- | --- |
| A value read every minute | `crates/deecure-agent/src/collect/` |
| Data the app reads live | `crates/deecure-agent/src/inventory/` |
| Something to keep in the "what changed" history | `crates/deecure-agent/src/history/` |
| A check or its severity | `rules/<area>/` (YAML) |
| A plain-English explanation | `kb/<type>/` (YAML) |
| A fix | Plan: `crates/deecure-agent/src/fixes.rs`; steps: `crates/deecure-fix/`; admin steps: `crates/deecure-helper/` (needs two reviewers) |
| A type shared by agent, CLI, and app | `crates/deecure-core/` |
| A screen or component | `app/src/` |
| What gets installed where | `packaging/` |

## Changes from the original plan

The plan sketched a simpler tree (`agent/`, `helper/`, `cli/` at the top). This layout follows common Rust practice instead:

- All Rust crates sit under `crates/`, in one Cargo workspace.
- A new `deecure-core` crate holds shared types, so the agent, CLI, helper, and app never drift apart.
- System files (D-Bus, polkit, systemd, desktop) have their own folders under `packaging/`. Every package format can reuse them.
- `debian/` lives in `packaging/debian/`, not at the root. Debian asks upstream projects not to ship a root `debian/` folder in release tarballs.
