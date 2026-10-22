# 0003. `.deb` package and our own APT repository as the main channel

- Status: Accepted
- Date: 2026-10-22

## Context

Users must install DeeCure as easily as any other Ubuntu app, and get updates the normal way. DeeCure needs deep system access: a root helper, polkit rules, D-Bus activation, and a systemd user service.

## Decision

1. Ship a `.deb` package named `deecure`.
2. Host our own signed APT repository. Add it with a one-line install script or a few manual steps.
3. Attach the same `.deb` to every GitHub Release, for download and double-click install.
4. Add a Launchpad PPA after the public beta.
5. Long-term goal: get into the official Debian and Ubuntu archives, so `sudo apt install deecure` works with no setup.
6. Until Phase 5, ship **one** package with everything inside. Split out a headless (no GUI) package for servers later.

## Why

- `apt` is how Ubuntu users already install and update software. Updates arrive through Software Updater.
- Our own repo gives us full control: our CI builds it, it supports Debian too, and it supports many CPU types.
- One package means one file to download and double-click.
- **Options not picked for the core:** Snap and Flatpak. Their sandboxes block the system access DeeCure needs. AppImage can come later, for the UI only.

## Consequences

- We must keep a signing key safe and rotate it on time.
- We build one package per Ubuntu release (`noble`, `resolute`), because library names differ between releases.
- Servers get GUI libraries until the package split in Phase 5.
- Details: [Packaging](../../06-packaging/README.md).
