# Install SysDoc

[Docs](../README.md) › [User guide](README.md)

> **Not released yet.** These steps work from the public beta onward. Addresses marked "provisional" may change.

## What you need

- Ubuntu 24.04 LTS or Ubuntu 26.04 LTS.
- A 64-bit Intel or AMD computer (ARM support comes later).
- Your password, to approve the install.

## Option 1: one command (recommended)

Open the Terminal (press `Ctrl` + `Alt` + `T`) and paste:

```bash
curl -fsSL https://sysdoc.github.io/install.sh | sudo bash
```

This adds the SysDoc software source to your computer and installs SysDoc. You can [read the script](https://sysdoc.github.io/install.sh) before you run it.

## Option 2: download the file

1. Download `sysdoc_<version>_amd64.deb` from the [Releases page](https://github.com/sysdoc/sysdoc/releases) (provisional address).
2. Double-click the file. It opens in App Center. Click **Install**.

   Or, in the Terminal, from your Downloads folder:

   ```bash
   sudo apt install ./sysdoc_*_amd64.deb
   ```

3. When asked "Add the SysDoc repository for updates?", choose **Yes** to get updates automatically.

## Option 3: step by step

Use this if you want to see every step. It does the same thing as Option 1.

```bash
sudo install -d -m 0755 /etc/apt/keyrings
curl -fsSL https://sysdoc.github.io/apt/sysdoc-archive-keyring.asc \
  | sudo tee /etc/apt/keyrings/sysdoc.asc > /dev/null

sudo tee /etc/apt/sources.list.d/sysdoc.sources > /dev/null <<EOF
Types: deb
URIs: https://sysdoc.github.io/apt
Suites: $(. /etc/os-release && echo "$VERSION_CODENAME")
Components: main
Signed-By: /etc/apt/keyrings/sysdoc.asc
EOF

sudo apt update
sudo apt install sysdoc
```

## Check your download (optional)

Each release has a `SHA256SUMS` file and its signature. In the folder with your download:

```bash
sha256sum --check --ignore-missing SHA256SUMS
```

You should see `OK`.

## Start SysDoc

Open the app menu and search for **SysDoc**. On first launch, SysDoc asks you to choose an access level:

- **Basic:** no password. About 80% of the checks.
- **Full (recommended):** approve the SysDoc helper once with your password. All checks.

You can change this later in Settings › Access.

## Update

SysDoc updates come with your normal Ubuntu updates, through **Software Updater**. Or run:

```bash
sudo apt update && sudo apt upgrade
```

## Uninstall

1. **Optional:** to delete your SysDoc history, open SysDoc › Settings › Data › **Delete history** first. Removing the package does not touch your home folder.
2. Remove the app:

   ```bash
   sudo apt purge sysdoc
   ```

3. Remove the software source (only if you added it):

   ```bash
   sudo rm /etc/apt/sources.list.d/sysdoc.sources /etc/apt/keyrings/sysdoc.asc
   sudo apt update
   ```

## Problems?

| Problem | Try this |
| --- | --- |
| `apt update` shows a key error | Run Option 3's first two commands again to refresh the key. |
| SysDoc is not in the app menu | Log out and log back in. |
| "Requires Full access" appears | Settings › Access › choose **Full**. |
| Something else | Open an issue on GitHub and include the output of `sysdoc --version` and `lsb_release -a`. |
