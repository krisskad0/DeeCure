# Getting started

[Docs](../README.md) › [Development](README.md)

> **Status:** Phases 0-3 and the Expert build are built. Every command below works. What is left: [Pending work](../05-roadmap/pending.md).

## 1. What you need

- Ubuntu 24.04 LTS or newer (a VM is fine).
- About 10 GB of free disk space for build files.

## 2. Install system packages

```bash
sudo apt update
sudo apt install -y build-essential curl wget file git pkg-config \
  libwebkit2gtk-4.1-dev libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev \
  libsystemd-dev libsqlite3-dev socat \
  debhelper devscripts lintian desktop-file-utils appstream
```

| Packages | Why |
| --- | --- |
| `build-essential`, `pkg-config` | C compiler and build tools |
| `libwebkit2gtk-4.1-dev` … `librsvg2-dev` | Needed by Tauri v2 on Linux |
| `libsystemd-dev` | Read the systemd journal |
| `libsqlite3-dev` | The agent's database |
| `socat` | Talk to the agent's socket by hand |
| `debhelper`, `devscripts`, `lintian` | Build and check `.deb` packages |
| `desktop-file-utils`, `appstream` | Check the `.desktop` file and the app store listing |

## 3. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

The repo pins the Rust version (1.99.1) in `rust-toolchain.toml`. `rustup` installs and uses it by itself.

## 4. Install Node.js

Install Node.js 22 or newer, for example with `nvm install 22`. The desktop app's web UI needs it.

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
| `make build` | `cargo build --workspace` |
| `make test` | `cargo test --workspace` (Rust unit, integration, and real-machine fixture tests) |
| `make lint` | `cargo fmt --check` and `cargo clippy -- -D warnings` |
| `make fmt` | `cargo fmt --all` |
| `make web` | UI tests (Vitest), type-check, and build of the web UI |
| `make agent` | Runs the agent in the foreground, with the rules and knowledge base from the repo |
| `make status` | Runs `deecure status` against the running agent |
| `make app` | Runs the desktop app with live reload (start `make agent` first) |
| `make perf` | Checks the agent against the [performance budget](../02-architecture/performance-budget.md) (about 80 s) |
| `make db-size` | Simulates 2 years of history and checks the database stays under 200 MB (about 1 min) |
| `make deb` | Builds both `.deb` packages into `target/deb/` and checks them with lintian ([Debian package](../06-packaging/debian-package.md)) |
| `make deb-test` | Installs, uses, and purges the packages in a clean Ubuntu container (needs Docker) |
| `make helper-test` | Tests the root helper with real D-Bus and polkit in a container (needs Docker) |
| `make install-test` | Tests `install.sh` against a signed throwaway APT repository in a container (needs Docker) |
| `make clean-install-test` | The real one-command install from the newest GitHub release on clean Ubuntu 24.04 (curl) and 26.04 (wget), then update and uninstall (needs Docker and internet) |
| `make help` | Lists all commands |

**Long tests take minutes** (`make deb`, the container tests, `make perf`). Run them in a second terminal or in the background. Build `make deb` once; each container test target builds it again first. Never rebuild while a container test is running.

**GitHub from the terminal:** with `gh auth login` done, `gh run list`, `gh run view --log-failed`, and `gh release view` show CI and releases.

**Never test write paths on your own computer.** Fixes, the helper, and package installs are tested in containers or fixtures only. Reading the system (running the agent) is fine.

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

## 7. Run the desktop app

With the agent running:

```bash
make app
```

A DeeCure window opens. Changes to `app/src/` show up in it right away. See [app/README.md](../../app/README.md).

## 8. Try the CLI

With the agent running (step 6):

```bash
make status                                 # same as: cargo run -p deecure-cli -- status
cargo run -p deecure-cli -- status --json
cargo run -p deecure-cli -- history --since 1d
cargo run -p deecure-cli -- report --html /tmp/report.html
```

## 9. Work on the root helper

The helper runs as root, so it is never installed on your own computer during development. Test it in a throwaway container with a real system D-Bus and polkit:

```bash
make helper-test
```

The app shows the access level as Basic and the helper as "Not installed" in a development build; that is expected.

To try the agent against a throwaway database (for example, to see the first-run history), give it its own paths. Socket paths must be short (under about 100 characters):

```bash
./target/debug/deecure-agent --rules rules --db /tmp/dc/test.db --socket /tmp/dc/agent.sock
./target/debug/deecure --socket /tmp/dc/agent.sock status
```

## 10. Look at the prototype

Open `docs/03-ui-design/prototype/deecure-dashboard.html` in a browser. It shows what every screen should look like.

## Editor setup

- VS Code with the rust-analyzer, ESLint, and Prettier extensions.
- The repo's `.editorconfig` sets indentation and line endings.
