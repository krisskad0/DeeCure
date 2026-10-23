# Install DeeCure

[Docs](../README.md) › [User guide](README.md)

> **Pre-release for testing:** [v0.1.0-alpha.2](https://github.com/krisskad0/DeeCure/releases) is out. Option 1 installs it. DeeCure's signed APT repository comes later; until then Option 1 installs from GitHub Releases.

## What you need

- Ubuntu 24.04 LTS or Ubuntu 26.04 LTS.
- A 64-bit Intel or AMD computer (ARM comes later).
- Your password, to approve the install.

## Option 1: one command (recommended)

Open the Terminal (`Ctrl` + `Alt` + `T`) and paste:

```bash
curl -fsSL https://raw.githubusercontent.com/krisskad0/DeeCure/main/packaging/install.sh | sudo sh
```

No `curl`? Ubuntu Desktop always has `wget`:

```bash
wget -qO- https://raw.githubusercontent.com/krisskad0/DeeCure/main/packaging/install.sh | sudo sh
```

Prefer to read it first (a good habit for any install script):

```bash
curl -fsSL https://raw.githubusercontent.com/krisskad0/DeeCure/main/packaging/install.sh -o install.sh
less install.sh
sudo sh install.sh
```

Then open **DeeCure** from the app menu, or run `deecure status` in the Terminal.

**What it does:**

- When DeeCure's signed APT repository is online, it adds DeeCure's signing key and software source, and installs DeeCure from there. Updates then come through Software Updater.
- Until then, it installs the newest release from [GitHub Releases](https://github.com/krisskad0/DeeCure/releases): it downloads the two packages, checks each one against the release's `SHA256SUMS` (a package that does not match is never installed), and installs them with apt, which also installs the libraries they need from Ubuntu. **Run the same command again to update.**
- While there is no stable release yet, it uses the newest pre-release and says so.
- It changes nothing else, and it is safe to run twice.

| Option | What it does |
| --- | --- |
| `--server` | Install only the background agent and the `deecure` command, without the desktop app |
| `--beta` | Use pre-releases even when a stable release exists |
| `--from-github` | Always install from GitHub Releases |
| `--dry-run` | Check and download, but change nothing |
| `--uninstall` | Remove DeeCure, its key, and its source |

Options go after `sh`, for example `… | sudo sh -s -- --server`.

**Check where a package came from (optional):** every release file has signed build provenance. With the GitHub command (`sudo apt install gh`), `gh attestation verify deecure_<version>_amd64.deb --repo krisskad0/DeeCure` confirms it was built by DeeCure's release workflow from the tagged source.

## Option 2: download the files

1. From the [Releases page](https://github.com/krisskad0/DeeCure/releases), download `deecure_<version>_amd64.deb` and `deecure-desktop_<version>_amd64.deb`.
2. Check them (optional): `sha256sum --check --ignore-missing SHA256SUMS` should say `OK` for both.
3. Install both, from your Downloads folder:

   ```bash
   sudo apt install ./deecure_*_amd64.deb ./deecure-desktop_*_amd64.deb
   ```

Files installed this way do not update by themselves. Run Option 1 again, or use it once the APT repository is online, to get updates.

## Option 3: step by step

The same as Option 1, by hand:

```bash
sudo install -d -m 0755 /etc/apt/keyrings
curl -fsSL https://krisskad0.github.io/DeeCure/apt/deecure-archive-keyring.asc \
  | sudo tee /etc/apt/keyrings/deecure.asc > /dev/null

sudo tee /etc/apt/sources.list.d/deecure.sources > /dev/null <<EOF
Types: deb
URIs: https://krisskad0.github.io/DeeCure/apt
Suites: $(. /etc/os-release && echo "$VERSION_CODENAME")
Components: main
Signed-By: /etc/apt/keyrings/deecure.asc
EOF

sudo apt update
sudo apt install deecure deecure-desktop
```

`Signed-By` means Ubuntu trusts this key only for DeeCure's packages.

## Start DeeCure

Open the app menu and search for **DeeCure**. The first time, three short screens explain what DeeCure does and run a first check. In the terminal, run `deecure status`.

**Access:** DeeCure reads everything your account can read without a password. Some details (disk health, firewall rules) and fixes that change the system need the admin helper, which comes with the package. When you use one of them, Ubuntu asks for your password, each time. Nothing runs as admin without your OK.

## Update

Updates come with your normal Ubuntu updates, in **Software Updater**. Or run:

```bash
sudo apt update && sudo apt upgrade
```

## Uninstall

```bash
sudo sh install.sh --uninstall
```

Or by hand:

```bash
sudo apt purge deecure-desktop deecure
sudo rm -f /etc/apt/sources.list.d/deecure.sources /etc/apt/keyrings/deecure.asc
sudo apt update
```

Your history stays in your home folder. To delete it too: `rm -rf ~/.local/share/deecure`

## Problems?

| Problem | Try this |
| --- | --- |
| `apt update` shows a key error | Run Option 3's first two commands again to refresh the key. |
| DeeCure is not in the app menu | Install `deecure-desktop`, then log out and back in. |
| "Agent not running" in the app | It starts by itself right after the install (and the app starts it if needed). If it stays: `systemctl --user daemon-reload && systemctl --user start deecure-agent.socket`, or sign out and in again |
| "Requires Full access" | The admin helper is part of the `deecure` package; if Settings › Access says "Not installed", reinstall `deecure`. |
| Something else | Open an issue on GitHub with the output of `deecure --version` and `lsb_release -a`. |
