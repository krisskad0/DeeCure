# Security and privacy problems on Ubuntu

[Docs](../../README.md) › [User guide](../README.md) › [Problems and fixes](README.md)

The firewall, AppArmor, SSH, admin accounts, Secure Boot, CPU security flaws, and what your computer shares.

## Is the Ubuntu firewall on? Turn on the firewall

**Signs:** you do not know if your computer blocks connections from the network. On Ubuntu desktop, the firewall (`ufw`) is off by default.

**Find it in DeeCure:** DeeCure warns **"The firewall is off"**. **Security › Firewall** shows its state and rules.

**Fix it with DeeCure:**

1. Click **Fix** on the warning in **Overview › Issues**, or **Turn on…** in **Security › Firewall**.
2. DeeCure shows the exact command (`ufw enable`) before it runs. Enter your password once.
3. DeeCure checks again. You can undo it from **Activity**.

**By hand:** `sudo ufw status verbose`, then `sudo ufw enable`.

## AppArmor is off on Ubuntu

**Signs:** apps run without the sandbox Ubuntu normally gives them.

**Find it in DeeCure:** DeeCure warns **"AppArmor is off"**. In Expert, the **Security** page shows the AppArmor state of every running process: enforced, warn-only, or not confined.

**Fix it with DeeCure:** DeeCure warns and explains. Turn AppArmor back on by hand.

**By hand:** `sudo aa-status`, then `sudo systemctl enable --now apparmor`.

## SSH lets root sign in with a password

**Signs:** a server or desktop with the SSH server on, where `root` can sign in with a password.

**Find it in DeeCure:** DeeCure warns **"SSH lets root sign in with a password"**. The **Security** page (Expert view) lists the SSH server settings that weaken it. **Accounts › SSH Keys** lists the keys allowed to sign in.

**Fix it with DeeCure:** DeeCure warns and explains. Change the setting by hand.

**By hand:** set `PermitRootLogin prohibit-password` (or `no`) in `/etc/ssh/sshd_config`, then `sudo systemctl restart ssh`.

## Another account has admin (sudo) rights on Ubuntu

**Find it in DeeCure:** DeeCure warns **"Another account has full admin power"**. **Accounts › Users and Groups** lists who may use `sudo`. **Accounts › Sign-ins** shows the sign-ins and `sudo` use of the last 30 days.

**Fix it with DeeCure:** DeeCure warns and shows who. Remove the right by hand if it is not expected.

**By hand:** `getent group sudo`, then `sudo deluser NAME sudo`.

## Secure Boot is off on Ubuntu

**Find it in DeeCure:** DeeCure warns **"Secure Boot is off"**. In Expert, **Hardware › Firmware** shows the firmware security level and each check behind it, the Secure Boot keys you added (MOK), and the TPM.

**Fix it with DeeCure:** DeeCure warns and explains. Secure Boot is turned on in the computer's firmware (BIOS) settings.

**By hand:** `mokutil --sb-state`

## A CPU security flaw is not protected on Ubuntu

**Find it in DeeCure:** DeeCure warns **"A processor security flaw is not protected"**. **Security › Vulnerabilities** lists each known CPU flaw and whether it is fixed, and the microcode version loaded.

**Fix it with DeeCure:** install the waiting updates (microcode and kernel) in **Apps & Updates › Updates**, then restart.

**By hand:** `grep . /sys/devices/system/cpu/vulnerabilities/*`

## Which apps use my camera and microphone on Ubuntu?

**Find it in DeeCure:** **Privacy › Device Access** shows when the microphone, camera, or screen sharing is in use, and by which app. **Privacy › App Permissions** shows snap and Flatpak permissions.

## Ubuntu sends my list of installed packages every week

**Signs:** the "popularity contest" was turned on during install.

**Find it in DeeCure:** DeeCure warns **"Your list of installed packages is sent to Ubuntu every week."** **Privacy › Privacy Settings** shows what is shared.

**Fix it with DeeCure:** DeeCure warns and explains. Turn it off by hand if you do not want to share.

**By hand:** `sudo dpkg-reconfigure popularity-contest`, or `sudo apt purge popularity-contest`.

## A container is open to every network

See [A Docker container is open to the internet](09-developers-and-containers.md#a-docker-container-is-open-to-the-internet).

---

How a fix works, and how to undo it: see [Fix a problem](../using-deecure.md#3-fix-a-problem). Every change DeeCure makes is listed in **Activity**, with **Undo** where it can be undone.
