# Install DeeCure

[Docs](../README.md) › [User guide](README.md)

> **Not released yet.** The steps below work once the first release is published. The addresses (`krisskad0.github.io/DeeCure`) are provisional.

## What you need

- Ubuntu 24.04 LTS or Ubuntu 26.04 LTS.
- A 64-bit Intel or AMD computer (ARM comes later).
- Your password, to approve the install.

## Option 1: the install script (recommended)

Open the Terminal (`Ctrl` + `Alt` + `T`). Download the script, read it, then run it:

```bash
curl -fsSL https://krisskad0.github.io/DeeCure/install.sh -o install.sh
less install.sh
sudo sh install.sh
```

It adds DeeCure's signing key and software source, then installs DeeCure. It changes nothing else, and it is safe to run twice. Options:

| Option | What it does |
| --- | --- |
| `--server` | Install only the background agent and the `deecure` command, without the desktop app |
| `--beta` | Use pre-releases |
| `--dry-run` | Print the steps without changing anything |
| `--uninstall` | Remove DeeCure, its key, and its source |

To check the script first: `sha256sum install.sh` must match `install.sh.sha256` on the same site.

## Option 2: download the files

1. From the [Releases page](https://github.com/krisskad0/DeeCure/releases), download `deecure_<version>_amd64.deb` and `deecure-desktop_<version>_amd64.deb`.
2. Check them (optional): `sha256sum --check --ignore-missing SHA256SUMS` should say `OK` for both.
3. Install both, from your Downloads folder:

   ```bash
   sudo apt install ./deecure_*_amd64.deb ./deecure-desktop_*_amd64.deb
   ```

Files installed this way do not update by themselves. Use Option 1 to get updates with Ubuntu's normal updates.

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
| "Agent not running" in the app | Run `systemctl --user start deecure-agent.socket` |
| "Requires Full access" | The admin helper is part of the `deecure` package; if Settings › Access says "Not installed", reinstall `deecure`. |
| Something else | Open an issue on GitHub with the output of `deecure --version` and `lsb_release -a`. |
