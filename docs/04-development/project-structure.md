# Project structure

[Docs](../README.md) › [Development](README.md)

What every folder is for. Items marked *(planned)* do not exist yet.

```
sysdoc/
├── .github/                 # Issue and pull request templates; CI workflows (planned)
├── app/                     # Desktop app: Tauri v2 + React + TypeScript
│   ├── src/                 #   React UI (planned)
│   └── src-tauri/           #   Tauri Rust side, talks to the agent (planned)
├── crates/                  # Rust workspace
│   ├── sysdoc-core/         #   Shared types: findings, metrics, IPC messages, rule schema
│   ├── sysdoc-agent/        #   Collectors, rules engine, fix planner, storage, socket API
│   ├── sysdoc-helper/       #   Root helper: D-Bus service with a fixed list of actions
│   └── sysdoc-cli/          #   The `sysdoc` command
├── rules/                   # Rule and fix-action YAML files, by category
├── kb/                      # Knowledge base YAML: services, errors, ports, packages
├── packaging/               # Everything needed to ship SysDoc
│   ├── debian/              #   Debian packaging files (copied to debian/ at build time)
│   ├── desktop/             #   .desktop file, AppStream metainfo, icons
│   ├── dbus/                #   D-Bus activation and policy files for the helper
│   ├── polkit/              #   polkit action policy for the helper
│   ├── systemd/             #   systemd units for the agent and the helper
│   ├── apt-repo/            #   Config and scripts to publish the signed APT repository
│   └── install.sh           #   One-line installer (planned)
├── scripts/                 # Developer scripts: build-deb, bump-version, dev helper install
├── tests/
│   ├── fixtures/            #   Real /proc, /sys, journal and command outputs from many machines
│   └── vm/                  #   VM test matrix scripts
├── docs/                    # All documentation (start at docs/README.md)
├── Cargo.toml               # Workspace manifest (planned)
├── rust-toolchain.toml      # Pinned Rust version (planned)
├── Makefile                 # Short commands: build, test, lint, dev, deb (planned)
├── README.md  LICENSE  CONTRIBUTING.md  CODE_OF_CONDUCT.md  SECURITY.md  CHANGELOG.md
└── .editorconfig  .gitignore
```

## Where code goes

| You are changing… | Put it in… |
| --- | --- |
| How data is read from the system | `crates/sysdoc-agent/src/collectors/` |
| Distro-specific reading (Ubuntu, Fedora, Arch) | `crates/sysdoc-agent/src/adapters/` |
| A check or its severity | `rules/<category>/` (YAML) |
| A plain-English explanation | `kb/<type>/` (YAML) |
| An admin action | `crates/sysdoc-helper/` + `packaging/polkit/` (needs two reviewers) |
| A type shared by agent, CLI, and app | `crates/sysdoc-core/` |
| A screen or component | `app/src/` |
| What gets installed where | `packaging/` |

## Changes from the original plan

The plan sketched a simpler tree (`agent/`, `helper/`, `cli/` at the top). This layout follows common Rust practice instead:

- All Rust crates sit under `crates/`, in one Cargo workspace.
- A new `sysdoc-core` crate holds shared types, so the agent, CLI, helper, and app never drift apart.
- System files (D-Bus, polkit, systemd, desktop) have their own folders under `packaging/`. Every package format can reuse them.
- `debian/` lives in `packaging/debian/`, not at the root. Debian asks upstream projects not to ship a root `debian/` folder in release tarballs.
