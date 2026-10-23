# Debian package

[Docs](../README.md) › [Packaging](README.md)

How the `deecure` `.deb` is built, checked, and tested.

## Build and test it

```bash
make deb        # build target/deb/deecure_<version>_amd64.deb and check it with lintian
make deb-test   # install, use, and purge it in a clean Ubuntu 24.04 container (needs Docker)
```

CI does both on every push (job "Build and test the .deb") and keeps the `.deb` as a download. To get it: GitHub → **Actions** → the latest run → **Artifacts** → `deecure-deb`.

## What is inside

| Path | What |
| --- | --- |
| `/usr/bin/deecure` | The command |
| `/usr/libexec/deecure/deecure-agent` | The agent |
| `/usr/lib/systemd/user/deecure-agent.{socket,service}` | User units, turned on for every user |
| `/usr/share/deecure/rules/` | The rules |
| `/usr/share/man/man1/deecure.1.gz` | Man page |
| `/usr/share/doc/deecure/` | Copyright and changelog |

The **`deecure-desktop`** package adds the app. It depends on `deecure`, so servers can install the agent and command without any GUI libraries.

| Path | What |
| --- | --- |
| `/usr/bin/deecure-desktop` | The app, with the web UI built into it |
| `/usr/share/applications/io.github.krisskad0.DeeCure.desktop` | The menu entry |
| `/usr/share/metainfo/io.github.krisskad0.DeeCure.metainfo.xml` | AppStream data for software stores |
| `/usr/share/icons/hicolor/{scalable,32x32,128x128,256x256}/apps/` | The icon |
| `/usr/share/man/man1/deecure-desktop.1.gz` | Man page |

The desktop app, the root helper, and their system files join in Phases 1 and 2. The full list is in [Files on disk](../02-architecture/overview.md#files-on-disk).

## Where the files live

The Debian files live in `packaging/debian/`. `scripts/build-deb.sh` copies the source (every tracked file, plus new files that are not ignored) into `target/deb/build/`, adds `debian/`, and runs `dpkg-buildpackage` there. Debian asks upstream projects not to ship a root `debian/` folder in release tarballs, and building in a copy keeps build files out of your checkout.

| File in `packaging/debian/` | What it does |
| --- | --- |
| `control` | Package name, dependencies, description |
| `rules` | Build steps: `cargo build --release --locked` for the agent and command, and for the app with `--features custom-protocol` (built-in UI), then install the files. `scripts/build-deb.sh` builds the web UI (`app/dist`) first, because it needs Node. |
| `changelog` | Version history in Debian format (edit it with `dch`) |
| `copyright` | License info in Debian's machine-readable format |
| `source/format` | `3.0 (quilt)` |
| `deecure.manpages`, `deecure-desktop.manpages` | Install the man pages from `packaging/man/` |
| `deecure.lintian-overrides`, `deecure-desktop.lintian-overrides` | `initial-upload-closes-no-bugs`: only applies to uploads to the Debian archive |

| `deecure.postinst` | After install: starts the agent socket for everyone signed in now, so the app works at once (debhelper only enables it for future logins). After an upgrade it also stops the old agent, so the socket starts the new one. Uses `systemctl --user --machine=<user>@` with a 15 s limit per person, skips system accounts, and can never make the install fail. |
| `deecure.prerm` | On removal (not upgrade): stops the agent and its socket for everyone signed in, so nothing runs from deleted files |

Planned later: a debconf question (offer to add the APT repository), and `tests/control` for `autopkgtest`.

## Decisions in the package

| Decision | Why |
| --- | --- |
| Rust comes from `rustup` (pinned in `rust-toolchain.toml`), so `cargo` and `rustc` are not in `Build-Depends` | Ubuntu 24.04's Rust is older than ours. A Launchpad PPA build will need the archive's versioned `rustc-1.xx` packages instead. |
| Dependencies are found automatically (`${shlibs:Depends}`) | `deecure`: `libc6`, `libgcc-s1`, `libsqlite3-0`, `init-system-helpers`. `deecure-desktop`: adds WebKitGTK 4.1, GTK 3, and libsoup 3. |
| Two packages | The agent and command work on servers; only desktops need WebKitGTK |
| The agent asks `dpkg-query`, never reads dpkg's files | dpkg's database is private to dpkg; lintian checks this (`uses-dpkg-database-directly`) |
| Both user units are enabled for every user | Data is collected from login, which matches the "Start at login: On" default. A per-user switch comes with Settings. |
| `ConditionUser=!@system` in both units | The agent does not run for system accounts, such as the login screen's user |
| `Standards-Version: 4.6.2` | The newest version that Ubuntu 24.04's lintian knows |
| Maintainer: `DeeCure maintainers <krisskad0@users.noreply.github.com>` | A placeholder until the contact address is decided ([open decision 11](../01-product/open-decisions.md)) |
| Version `0.1.0-1`, distribution `noble` | Per-release suffixes (`~ubuntu24.04`) come with the APT repository ([release process](release-process.md)) |

Before the first public release, `copyright` must also list the licenses of the Rust crates built into the binaries (for example, generated with `cargo-about` or checked with `cargo-deny`), and, once the desktop app is packaged, of the fonts it bundles (Ubuntu Sans and Ubuntu Sans Mono).

## What `make deb-test` checks

`scripts/test-deb.sh --throwaway <deb>` runs as root on a throwaway system:

1. `apt install` works and pulls in the dependencies.
2. The user units are enabled for every user.
3. `deecure --version`, the agent, and the man page are in place.
4. As a normal user, the agent starts, loads the installed rules, keeps its database private (mode `0600`), and `deecure status` and `--json` answer.
5. `apt purge` removes everything, including the "enabled for every user" links.

## More checks before the public beta

| Check | What it proves |
| --- | --- |
| `piuparts` | Install → upgrade → remove → purge works across versions |
| `autopkgtest` | The installed package runs on the build infrastructure |
| `desktop-file-validate` | The `.desktop` file is correct (Phase 1) |
| `appstreamcli validate` | The app store listing is correct (Phase 1) |
