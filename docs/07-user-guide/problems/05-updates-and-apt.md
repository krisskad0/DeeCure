# Update and package problems on Ubuntu (apt, dpkg, snap)

[Docs](../../README.md) › [User guide](../README.md) › [Problems and fixes](README.md)

Broken apt, a stopped upgrade, security updates, automatic updates, restarts, and changed system files.

## "dpkg was interrupted": apt is broken on Ubuntu

**Signs:** apt or the Software Updater says "dpkg was interrupted, you must manually run 'sudo dpkg --configure -a'", or it will not install anything.

**Why it happens:** an install or update stopped half way, after a power cut, a closed terminal, or a crash.

**Find it in DeeCure:** DeeCure warns **"A package install stopped half way"**, and names the packages.

**Fix it with DeeCure:**

1. Open **Overview › Issues** and click **Fix** on the warning.
2. DeeCure shows the exact command it will run (`dpkg --configure -a`). Click **Apply** and enter your password once.
3. DeeCure checks again and tells you when the warning is gone.

**By hand:**

```bash
sudo dpkg --configure -a
sudo apt --fix-broken install
sudo dpkg --audit            # anything still broken?
```

## Ubuntu upgrade to 26.04 stopped half way

**Signs:** after an upgrade from 24.04, some apps are old, the release name is wrong, or apt shows errors.

**Why it happens:** the upgrade was stopped, or a third-party source (PPA) blocked it.

**Find it in DeeCure:**

- DeeCure warns when a package install stopped half way.
- **Apps & Updates › Installed** (Expert view) shows half-installed and broken packages, held packages, and packages from no repository.
- **Apps & Updates › Sources** shows every software source and its key.

**Fix it with DeeCure:**

1. Click **Fix** on the "stopped half way" warning to finish the setup of the half-installed packages.
2. Check **Apps & Updates › Sources** for PPAs that do not support the new release.

**By hand:** `sudo dpkg --configure -a`, then `sudo apt full-upgrade`.

**Limits:** DeeCure finishes stopped installs, but it does not run the release upgrade itself.

## Security updates are waiting on Ubuntu

**Signs:** you are not sure which updates matter.

**Find it in DeeCure:**

- DeeCure warns **"Security updates are waiting"**, with how many.
- **Apps & Updates › Updates** lists every waiting update and marks the security ones.

**Fix it with DeeCure:** click **Fix** on the warning, or install them from **Apps & Updates › Updates**. Your password is asked once.

**By hand:** `sudo apt update && sudo apt upgrade`

## Ubuntu automatic security updates stopped working

**Signs:** updates pile up, even though automatic updates are on.

**Find it in DeeCure:** DeeCure warns **"Automatic security updates stopped working"**, and shows when they last ran and the result.

**Fix it with DeeCure:** DeeCure warns and explains. Turn them back on by hand.

**By hand:**

```bash
systemctl status unattended-upgrades
sudo dpkg-reconfigure -plow unattended-upgrades
less /var/log/unattended-upgrades/unattended-upgrades.log
```

## Ubuntu says "restart required" after updates

**Find it in DeeCure:** DeeCure warns **"Restart to finish installing updates"**. In Expert, it also says when a newer CPU microcode waits for a restart.

**Fix it with DeeCure:** restart when it suits you. The warning clears after the restart.

**By hand:** `cat /var/run/reboot-required.pkgs` lists the packages that need the restart.

## Were my Ubuntu system files changed or damaged?

**Signs:** something behaves strangely after a crash, a disk error, or a manual edit.

**Find it in DeeCure:** **Security › File Integrity** checks every installed file against its package checksum, when you ask, and lists files that changed.

**Fix it with DeeCure:** DeeCure shows which package each changed file belongs to. Reinstall that package to restore the file.

**By hand:** `sudo dpkg --verify`, then `sudo apt install --reinstall PACKAGE`.

## Remove unused packages and leftovers on Ubuntu

See [Clean the APT cache, trash, thumbnails, and leftover packages](02-disk-space.md#clean-the-apt-cache-trash-thumbnails-and-leftover-packages-on-ubuntu) and [Old snap versions take gigabytes](02-disk-space.md#old-snap-versions-take-gigabytes-on-ubuntu).

---

How a fix works, and how to undo it: see [Fix a problem](../using-deecure.md#3-fix-a-problem). Every change DeeCure makes is listed in **Activity**, with **Undo** where it can be undone.
