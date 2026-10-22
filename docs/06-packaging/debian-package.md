# Debian package

[Docs](../README.md) › [Packaging](README.md)

How the `sysdoc` `.deb` is built and checked.

## Where the files live

The Debian files live in `packaging/debian/`. The build script copies them to `debian/` at build time. Debian asks upstream projects not to ship a `debian/` folder at the root of their release tarballs.

| File in `packaging/debian/` | What it does |
| --- | --- |
| `control` | Package name, dependencies, description |
| `rules` | Build steps (a Makefile that calls `dh`) |
| `changelog` | Version history in Debian format (edit it with `dch`) |
| `copyright` | License info in Debian's machine-readable format |
| `source/format` | `3.0 (quilt)` |
| `sysdoc.install` | Which built file goes where |
| `sysdoc.postinst`, `sysdoc.postrm` | Scripts that run after install and after removal |
| `sysdoc.templates` | The debconf question: "Add the SysDoc repository for updates?" |
| `tests/control` | `autopkgtest` smoke tests |

## `control` sketch

```text
Source: sysdoc
Section: admin
Priority: optional
Maintainer: SysDoc maintainers <maintainers@example.org>
Build-Depends: debhelper-compat (= 13), cargo, rustc, nodejs, pkgconf,
 libwebkit2gtk-4.1-dev, libxdo-dev, libssl-dev, libsystemd-dev, librsvg2-dev
Standards-Version: 4.7.0
Homepage: https://github.com/<org>/sysdoc

Package: sysdoc
Architecture: amd64 arm64
Depends: ${shlibs:Depends}, ${misc:Depends}, dbus, polkitd
Recommends: smartmontools
Description: friendly health checks and safe fixes for your Linux computer
 SysDoc checks your computer, explains what it finds in plain English,
 and fixes problems safely with a preview and an undo button.
```

`${shlibs:Depends}` fills in library dependencies by itself. Replace the maintainer address and `<org>` before the first release.

## Build it

```bash
make deb
```

What `make deb` does (through `scripts/build-deb.sh`):

1. Builds the web UI: `npm ci && npm run build` in `app/`.
2. Copies `packaging/debian/` to `debian/`.
3. Runs `dpkg-buildpackage -us -uc -b` (build, no signing).
4. Runs `lintian` on the result and fails on errors and warnings.

Release builds run in a clean container for each release (`ubuntu:24.04`, `ubuntu:26.04`). Never ship a package built on a developer's own machine.

## What the install scripts do

| When | What | How |
| --- | --- | --- |
| Install | Turn on the agent socket for all users | `dh_installsystemduser` |
| Install | Load the helper's D-Bus policy | D-Bus picks it up through its package trigger |
| Install from a downloaded `.deb` | Ask to add the APT repository | debconf question, default Yes |
| Purge | Remove `/var/log/sysdoc/` and the repository files the package added | `postrm purge` |

The package **never** touches users' home folders. That is Debian policy. To delete a user's history, use Settings › Data in the app before removing the package.

## Quality checks before every release

| Check | What it proves |
| --- | --- |
| `lintian` | The package follows Debian rules |
| `piuparts` | Install → upgrade → remove → purge works and leaves nothing behind |
| `autopkgtest` | The installed package runs: `sysdoc --version`, `sysdoc status --json` |
| `desktop-file-validate` | The `.desktop` file is correct |
| `appstreamcli validate` | The app store listing is correct |
