# Getting started

[Docs](../README.md) › [Development](README.md)

> **Status:** there is no code yet. This page is the target setup. Update it when Phase 0 lands.

## 1. What you need

- Ubuntu 24.04 LTS or newer (a VM is fine).
- About 10 GB of free disk space for build files.

## 2. Install system packages

```bash
sudo apt update
sudo apt install -y build-essential curl wget file git pkg-config \
  libwebkit2gtk-4.1-dev libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev \
  libsystemd-dev \
  debhelper devscripts lintian desktop-file-utils appstream
```

| Packages | Why |
| --- | --- |
| `build-essential`, `pkg-config` | C compiler and build tools |
| `libwebkit2gtk-4.1-dev` … `librsvg2-dev` | Needed by Tauri v2 on Linux |
| `libsystemd-dev` | Read the systemd journal |
| `debhelper`, `devscripts`, `lintian` | Build and check `.deb` packages |
| `desktop-file-utils`, `appstream` | Check the `.desktop` file and the app store listing |

## 3. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

The repo will pin the Rust version in `rust-toolchain.toml`. `rustup` picks it up by itself.

## 4. Install Node.js

Install the current Node.js LTS release (for example with `nvm`). The desktop app's web UI needs it.

## 5. Build and run

```bash
git clone https://github.com/<org>/sysdoc.git
cd sysdoc

make build        # build all Rust crates and the web UI
make test         # run all tests
make dev          # start the agent and the desktop app in dev mode
```

The `Makefile` gives short names to the real commands:

| Target | Runs |
| --- | --- |
| `make build` | `cargo build --workspace` and `npm run build` in `app/` |
| `make test` | `cargo test --workspace` and `npm test` in `app/` |
| `make lint` | `cargo fmt --check`, `cargo clippy -- -D warnings`, `npm run lint` |
| `make dev` | Runs the agent, then `npm run tauri dev` in `app/` |
| `make deb` | Builds the `.deb` (see [Debian package](../06-packaging/debian-package.md)) |

## 6. Try the CLI

```bash
cargo run -p sysdoc-cli -- status
cargo run -p sysdoc-cli -- status --json
```

## 7. Work on the root helper (Phase 2 and later)

The helper needs system files for D-Bus and polkit. Install dev copies with a script, and remove them when you are done:

```bash
sudo scripts/dev-install-helper.sh
sudo scripts/dev-uninstall-helper.sh
```

Do helper work in a VM, not on your main computer.

## 8. Look at the prototype

Open `docs/03-ui-design/prototype/sysdoc-dashboard.html` in a browser. It shows what every screen should look like.

## Editor setup

- VS Code with the rust-analyzer, ESLint, and Prettier extensions.
- The repo's `.editorconfig` sets indentation and line endings.
