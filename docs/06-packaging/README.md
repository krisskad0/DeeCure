# Packaging and distribution

[Docs](../README.md)

## Channels

| Format | Use | Note |
| --- | --- | --- |
| **.deb + apt repository** | Main channel for Ubuntu and Debian | Installs the agent, helper, polkit policy, and app |
| **Install script** | One-command setup | Adds the repository and installs the `.deb` |
| AppImage (UI only) | Optional, later | The agent and helper still need the `.deb` |
| Snap / Flatpak | **Not for the core** | Their sandboxes block the system access DeeCure needs |
| RPM, Arch (AUR) | Later phases | With the distro adapters |

## The goal: install like any other Ubuntu app

| Way | What the user does | Updates | Available from |
| --- | --- | --- | --- |
| **APT repository** (recommended) | Runs one command, or 4 manual steps | Yes, through Software Updater and `apt upgrade` | Phase 2 (public beta) |
| **Download the `.deb`** | Double-clicks it, or runs `sudo apt install ./deecure_….deb` | Yes. The package offers to add the APT repository (like VS Code and Chrome). | Phase 2 |
| **Launchpad PPA** | `sudo add-apt-repository ppa:deecure/stable` then `sudo apt install deecure` | Yes | After the beta |
| **Ubuntu / Debian archive** | `sudo apt install deecure`, nothing else | Yes | Long-term goal |

User-facing steps: [Install guide](../07-user-guide/install.md).

## Supported systems

| System | Codename | CPU | When |
| --- | --- | --- | --- |
| Ubuntu 24.04 LTS | `noble` | amd64, then arm64 | First |
| Ubuntu 26.04 LTS | `resolute` | amd64, then arm64 | First |
| Debian 13 | `trixie` | amd64 | After the Ubuntu builds are stable |
| Ubuntu interim releases | - | amd64 | Best effort |

We build one package per release, because library package names differ between releases.

## One package for now

Until Phase 5, everything ships in **one** package: `deecure`. One file is easy to download and double-click. A headless package for servers comes later. See [ADR 0003](../02-architecture/decisions/0003-deb-and-apt-repository.md).

## Names and IDs (provisional)

| Thing | Value |
| --- | --- |
| Debian package | `deecure` |
| App ID (`.desktop`, AppStream, icon) | `io.github.krisskad0.DeeCure` |
| Helper D-Bus name | `io.github.krisskad0.DeeCure.Helper1` |
| polkit action prefix | `io.github.krisskad0.deecure.helper.` |
| CLI command | `deecure` |
| Desktop app command | `deecure-desktop` |

Where each file is installed: [Files on disk](../02-architecture/overview.md#files-on-disk).

## Files in this section

| File | What it covers |
| --- | --- |
| [debian-package.md](debian-package.md) | How the `.deb` is built and checked |
| [apt-repository.md](apt-repository.md) | The signed APT repository, the install script, and the PPA |
| [release-process.md](release-process.md) | Version numbers and the release checklist |
