# Driver, kernel, and firmware problems on Ubuntu (NVIDIA, DKMS, fwupd)

[Docs](../../README.md) › [User guide](../README.md) › [Problems and fixes](README.md)

A black screen after a kernel update, a graphics card that is not used, kernel crashes, firmware crashes, missing drivers, and firmware updates.

Many of these details show only in the **Expert** view. Switch the view at the top of the DeeCure window.

## NVIDIA black screen or missing driver after a kernel update on Ubuntu

**Signs:** after an update and a restart, the screen stays black, the resolution is wrong, or `nvidia-smi` says it cannot talk to the driver.

**Why it happens:** the NVIDIA driver is built for each kernel by DKMS. When that build fails for the new kernel, the driver is missing.

**Find it in DeeCure:**

- **Operating System › Kernel** (Expert view) lists the drivers built locally (DKMS), and whether each one is built for every installed kernel.
- DeeCure warns **"The graphics card is not used"** when the computer falls back to basic graphics.

**Fix it with DeeCure:**

1. If the screen is black, choose the older kernel in the boot menu (Advanced options for Ubuntu) to get a working desktop.
2. Open **Optimize**. When a driver is not built for an installed kernel, it offers **Rebuild the nvidia driver**.
3. Click **Preview**, then **Apply**, and enter your password once. Restart into the new kernel.

**By hand:**

```bash
dkms status
sudo dkms autoinstall
ubuntu-drivers devices
```

**Limits:** when there is no working desktop at all, you need the older kernel first, because DeeCure is a desktop app.

## Ubuntu is not using my graphics card (software rendering)

**Signs:** the desktop feels slow, video stutters, and the graphics renderer says "llvmpipe".

**Find it in DeeCure:**

- DeeCure warns **"The graphics card is not used."**
- **Hardware › Graphics** shows the driver, the Mesa or NVIDIA version, the OpenGL and Vulkan renderer, and whether the session is Wayland or X11.

**Fix it with DeeCure:** **Optimize** offers **Install the recommended driver**, and **Install the hardware video driver** when video decoding in hardware is missing.

**By hand:** `glxinfo -B` (from `mesa-utils`), then `sudo ubuntu-drivers install`.

## Second graphics card drains the battery on Ubuntu

**Signs:** a laptop with two graphics cards runs hot and drains fast, even on light work.

**Find it in DeeCure:** **Performance › GPU** (Expert view) shows whether the second graphics card is asleep or awake, and which app keeps it awake.

**Fix it with DeeCure:** **Optimize** offers **Let the second graphics card sleep**, and names the app that keeps it awake.

## The Ubuntu kernel crashed

**Signs:** the computer froze or restarted by itself.

**Find it in DeeCure:**

- DeeCure warns **"The kernel crashed."**
- **Logs › Errors and crashes** shows the crash and kernel log errors, with the time.
- **Operating System › Kernel** shows kernel taint, which tells you if a driver from outside Ubuntu was loaded.

**Fix it with DeeCure:** DeeCure warns and shows the details. Install waiting updates, and if it repeats, report the crash with the log lines.

**By hand:** `journalctl -k -b -1 -p err` and `cat /proc/sys/kernel/tainted`.

## The screen froze for a moment: graphics card hung on Ubuntu

**Find it in DeeCure:** DeeCure warns **"The graphics card hung"**, with the time and the driver. **Logs › Errors and crashes** shows the log lines.

**Fix it with DeeCure:** DeeCure warns and explains. Install waiting updates and firmware updates. If it repeats, report it with the log lines.

**By hand:** `journalctl -k | grep -i -E "gpu hang|reset"`

## Wi-Fi or Bluetooth firmware crashed on Ubuntu

**Signs:** Wi-Fi or Bluetooth stops until you restart.

**Find it in DeeCure:**

- DeeCure warns **"Device firmware crashed."**
- **Hardware › Summary** shows driver and firmware crash reports, and firmware that failed to load.

**Fix it with DeeCure:** DeeCure warns and explains. Install waiting updates (the `linux-firmware` package), and firmware updates (below).

**By hand:** `journalctl -k | grep -i firmware`

## Firmware and BIOS updates on Ubuntu

**Find it in DeeCure:** **Hardware › Firmware** shows the firmware updates offered for your devices (through `fwupd`), the BIOS version, and the UEFI boot entries.

**Fix it with DeeCure:** **Optimize** offers **Install the firmware update** for each device that has one. Keep the laptop on the charger.

**By hand:** `fwupdmgr get-updates`, then `fwupdmgr update`.

## A device has no driver on Ubuntu

**Signs:** a webcam, fingerprint reader, or card reader does not work at all.

**Find it in DeeCure:** **Hardware › Summary** lists devices with no driver, or with a basic fallback driver instead of the real one.

**Fix it with DeeCure:** **Optimize** offers **Install missing firmware** when the firmware for a device is missing. For a missing driver, it offers **Install the recommended driver** when Ubuntu has one.

**By hand:** `lspci -k` and `lsusb`, then `ubuntu-drivers devices`.

## Remove old kernels on Ubuntu

See [/boot is full on Ubuntu, and updates fail](02-disk-space.md#boot-is-full-on-ubuntu-and-updates-fail).

---

How a fix works, and how to undo it: see [Fix a problem](../using-deecure.md#3-fix-a-problem). Every change DeeCure makes is listed in **Activity**, with **Undo** where it can be undone.
