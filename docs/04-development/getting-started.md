# Getting started

[Docs](../README.md) › [Development](README.md)

> **Status:** Phase 0 in progress. The Rust workspace builds, and `make build`, `make test`, `make lint`, and `make fmt` work. Steps marked *(planned)* come later.

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

The repo pins the Rust version (1.99.1) in `rust-toolchain.toml`. `rustup` installs and uses it by itself.

## 4. Install Node.js

Install the current Node.js LTS release (for example with `nvm`). The desktop app's web UI needs it.

## 5. Build and run

```bash
git clone https://github.com/krisskad0/DeeCure.git
cd DeeCure

make build        # build all Rust crates
make test         # run all tests
make help         # list all commands
```

The `Makefile` gives short names to the real commands:

| Target | Runs |
| --- | --- |
| `make build` | `cargo build --workspace` (later also `npm run build` in `app/`) |
| `make test` | `cargo test --workspace` (later also `npm test` in `app/`) |
| `make lint` | `cargo fmt --check` and `cargo clippy -- -D warnings` (later also `npm run lint`) |
| `make fmt` | `cargo fmt --all` |
| `make agent` | Runs the agent in the foreground |
| `make help` | Lists all commands |
| `make dev` *(planned)* | Runs the agent, then `npm run tauri dev` in `app/` |
| `make deb` *(planned)* | Builds the `.deb` (see [Debian package](../06-packaging/debian-package.md)) |

## 6. Run the agent

Start it in one terminal:

```bash
make agent
```

Talk to it from another terminal (install `socat` with `sudo apt install socat` if needed):

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"version"}' | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/deecure/agent.sock
```

To let systemd start it on demand, the way the real package will, install the user units that point at your debug build:

```bash
scripts/dev-agent-units.sh install     # and later: scripts/dev-agent-units.sh uninstall
```

The message format is in [IPC protocol](../02-architecture/ipc-protocol.md).

## 7. Try the CLI *(planned: Phase 0, task 9)*

```bash
cargo run -p deecure-cli -- status
cargo run -p deecure-cli -- status --json
```

## 8. Work on the root helper (Phase 2 and later)

The helper needs system files for D-Bus and polkit. Install dev copies with a script, and remove them when you are done:

```bash
sudo scripts/dev-install-helper.sh
sudo scripts/dev-uninstall-helper.sh
```

Do helper work in a VM, not on your main computer.

## 9. Look at the prototype

Open `docs/03-ui-design/prototype/deecure-dashboard.html` in a browser. It shows what every screen should look like.

## Editor setup

- VS Code with the rust-analyzer, ESLint, and Prettier extensions.
- The repo's `.editorconfig` sets indentation and line endings.
