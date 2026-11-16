# Hardware health on Ubuntu: SSD, memory, PCIe, and USB

[Docs](../../README.md) › [User guide](../README.md) › [Problems and fixes](README.md)

Early warnings about a dying SSD, a read-only disk, memory errors, slow links, and USB problems.

Many of these details show only in the **Expert** view. Switch the view at the top of the DeeCure window.

## Is my SSD dying? Check SSD health on Ubuntu

**Signs:** none yet, or files that will not copy, or a slow disk.

**Find it in DeeCure:**

- DeeCure warns **"Drive is wearing out"** and **"A disk had read or write errors"**.
- **Hardware › Drives** shows the drive's life left (SMART), its temperature with the drive's own warning and critical temperatures, its errors, and its firmware.
- Slow checks like disk health run once a day.

**Fix it with DeeCure:** DeeCure warns early, so you have time. Back up your files now (see **Backups**), and plan to replace the drive.

**By hand:**

```bash
sudo smartctl -a /dev/nvme0        # from smartmontools
sudo nvme smart-log /dev/nvme0     # from nvme-cli
```

## Ubuntu disk became read-only, or has filesystem errors

**Signs:** "Read-only file system" when you save, after a crash or a power cut.

**Find it in DeeCure:**

- DeeCure warns **"/ became read-only"** and **"/ has filesystem errors"**, with the disk name.
- **Storage › Filesystems** shows each filesystem's mount options and the errors it has recorded.

**Fix it with DeeCure:** DeeCure warns and explains. Back up first. Then check the filesystem from the boot menu (recovery mode) with `fsck`.

**By hand:** `findmnt -o TARGET,OPTIONS /` and `sudo dmesg | grep -i -E "ext4|i/o error"`.

## Is my RAM failing? Memory errors on Ubuntu

**Find it in DeeCure:** DeeCure warns **"The memory reports errors"** and **"The processor reported hardware errors"** (machine checks), when the computer reports them.

**Fix it with DeeCure:** DeeCure warns and explains. Run a full memory test from the boot menu, and replace the module that fails.

**By hand:** `journalctl -k | grep -i -E "mce|edac"`

**Limits:** many laptops do not report memory errors at all. DeeCure shows them only when the memory controller reports them.

## Is my RAM running at full speed on Ubuntu?

**Find it in DeeCure:** **Hardware › Memory Modules** (Expert view) shows each module's slot, size, type, rated speed against the speed it really runs at, single or dual channel, and free slots.

**By hand:** `sudo dmidecode -t memory`

## My NVMe SSD or GPU is slower than it should be (PCIe link)

**Signs:** a fast SSD or a graphics card performs below its rating.

**Find it in DeeCure:**

- **Hardware › Drives** and **Hardware › Graphics** (Expert view) show the PCIe link speed and lanes now, against the most the device and the slot support.
- DeeCure warns **"A PCIe link reports errors"**.

**Fix it with DeeCure:** DeeCure shows when a device runs at a lower link speed. Move it to another slot, check the cable or riser, or update the BIOS.

**By hand:** `sudo lspci -vv | grep -E "LnkCap|LnkSta"`

## My USB 3 drive is slow on Ubuntu

**Signs:** copying to a USB 3 drive is as slow as USB 2.

**Find it in DeeCure:** **Devices › Ports & Devices** shows the USB tree, with the speed of each device, so you can see a USB 3 device running at USB 2 speed.

**Fix it with DeeCure:** use another port or cable. Front ports, hubs, and old cables are common causes.

**By hand:** `lsusb -t`

## A USB port turned off on Ubuntu

**Find it in DeeCure:** DeeCure warns **"A USB port was overloaded"** when a device drew too much power.

**Fix it with DeeCure:** DeeCure warns and explains. Use a powered hub for devices that draw a lot of power.

## What does my USB-C charger or dock support on Ubuntu?

**Find it in DeeCure:** **Devices › Ports & Devices** shows, for each USB-C port: what is plugged in, the power and data direction, the USB version, and whether it carries a display. For chargers, it shows the volts and amps the charger offers and what the laptop takes.

---

How a fix works, and how to undo it: see [Fix a problem](../using-deecure.md#3-fix-a-problem). Every change DeeCure makes is listed in **Activity**, with **Undo** where it can be undone.
