# Health warnings

[Docs](../README.md) › [Product](README.md)

**Status: approved by the owner on 2026-10-24, and built.** Two changes were made while building, for accuracy: "battery drained too deep" became "battery ran empty", and "names do not resolve" became "internet not reachable" (both below).

DeeCure's main job is to keep the computer healthy and free of errors, and to warn early about anything that can harm it. This page lists every warning, when it fires, and why.

## Rules for every warning

1. **Use is not harm.** A CPU at 100%, RAM full of file cache, swap in use, a busy disk, or heavy network use are normal: people may use their computer to the limit. DeeCure warns about **stalls, errors, heat, wear, and risk**, not about use.
2. **The hardware's own limits come first.** CPUs, graphics cards, drives, and batteries report their own limits (critical temperature, trip points, design voltage, drive warning temperature). DeeCure uses those. A fixed number is used only where the hardware reports none, and its source is named.
3. **Sustained, not spikes.** A warning needs the problem to last (most need 5 minutes) or to repeat, so a short spike never warns.
4. **Each warning says** what was measured, why it harms, and what to do, with a fix when one exists.
5. **Levels:** **Critical** = damage or data loss is near or already happening. **Warning** = harm is likely if it goes on. **Info** = worth knowing.
6. **One problem, one warning.** A disk that is 97% full does not also say "filling up".
7. **Your limits:** every warning can be turned off, or its limits changed, in Settings › Notifications (Advanced and Expert). The default is shown next to your value, and each value stays inside a safe range.

**Data:** "✅" = built; "Admin read" = needs the password through the helper (step 6), not built yet.

## Changes to warnings that exist today

| Today | Problem | Proposed |
| --- | --- | --- |
| CPU hot at 90 °C, critical at 100 °C | Many laptop CPUs run at 90-95 °C under load by design; some are limited at 100 °C, others at 105 °C. A fixed number is wrong for many CPUs. | The CPU's own limit, below |
| Main disk warning at 85%, critical at 93% | On a 2 TB disk, 85% still leaves 300 GB. Only `/` is checked. | Free space in GB and percent together, every local disk, `/boot` on its own, below |
| Filling up | ✅ Fixed on 2026-10-24: a one-time jump no longer counts | Keep |

## CPU

| Warning | Fires when | Why it harms | Level | Data |
| --- | --- | --- | --- | --- |
| CPU too hot | Within 5 °C of the CPU's own critical temperature (`temp*_crit`, or its own lower `temp*_max`) for 5 minutes. A CPU that reports no limit (some AMD chips) uses 100 °C, critical 105 °C: above every current CPU's normal maximum | It runs at its limit and slows down hard; long stays shorten its life | Warning | ✅ |
| CPU reached its critical temperature | At or over the critical trip, or a thermal shutdown is in the log | The computer may switch off to protect itself | Critical | ✅ |
| Cooling cannot keep up | Heat slowdowns in at least 1 of 4 minutes over the last hour | Dust, a weak fan, or blocked vents | Warning | ✅ |
| CPU hardware errors | Any machine-check error in the kernel log in 7 days | The CPU or its power is failing; crashes and wrong results | Critical | ✅ |
| CPU security flaw not protected | A known CPU flaw is "Vulnerable" | Programs can read other programs' memory | Warning | ✅ |

## Memory

| Warning | Fires when | Why it harms | Level | Data |
| --- | --- | --- | --- | --- |
| Memory is short | Memory pressure "full" (all apps waiting for memory) is 10% or more for 5 minutes | The computer freezes for seconds at a time | Warning | ✅ |
| An app was killed for memory | The kernel or systemd-oomd killed an app for memory in 24 hours | Unsaved work in that app is lost | Warning | ✅ |
| Swap almost full | Swap 90% used and less than 5% of memory available | The next step is apps being killed | Warning | ✅ |
| Memory errors that were fixed | Corrected memory errors went up in 24 hours | A module is starting to fail | Warning | ✅ |
| Memory errors that could not be fixed | Any uncorrected memory error | Crashes and damaged files | Critical | ✅ |

## Disks and filesystems

| Warning | Fires when | Why it harms | Level | Data |
| --- | --- | --- | --- | --- |
| Disk low on space | Free under 10% **and** under 20 GB | Updates and apps start to fail | Warning | ✅ |
| Disk almost full | Free under 5% **or** under 5 GB | Apps crash, files can be cut short | Critical | ✅ |
| `/boot` low on space | Under 150 MB free (a kernel update needs about 100 MB); critical under 60 MB | Kernel updates fail half way | Warning / Critical | ✅ |
| EFI partition low on space | Under 10 MB free | Boot loader updates fail | Warning | ✅ |
| Disk filling up | Kept filling over the last day and the last 3 days, full within 14 days (3 days: critical) | Same as low on space, ahead of time | Warning / Critical | ✅ |
| Out of inodes | 90% of inodes used (97%: critical) | No new files, even with space left | Warning / Critical | ✅ |
| Filesystem errors recorded | The filesystem logged errors (ext4 error count above 0) | Files may be damaged; it can switch to read-only | Critical | ✅ |
| Disk became read-only | A disk is read-only now but writable in `/etc/fstab` | Nothing can be saved | Critical | ✅ |
| Disk read or write errors | I/O, ATA, or NVMe timeout errors in the kernel log in 24 hours | The drive or its cable is failing; data loss | Critical | ✅ |
| Drive too hot | Covered by "A part is too hot": an NVMe drive reports its own warning and critical temperatures | The drive slows down and wears faster | Warning / Critical | ✅ |
| Drive wearing out | NVMe life used 90% or more, or a critical warning flag; SATA reallocated or pending sectors above 0 | The drive may fail soon | Warning / Critical | Admin read |
| Everything waits for the disk | Disk pressure "full" 20% or more for 5 minutes | Freezes while the disk catches up | Warning | ✅ |

## Battery

| Warning | Fires when | Why it harms | Level | Data |
| --- | --- | --- | --- | --- |
| Battery worn | Holds under 60% of its design charge (under 80%: Info) | Much shorter time on battery | Warning / Info | ✅ |
| Battery too hot | 45 °C or more while charging; 60 °C or more at any time | Charging hot damages lithium cells; 60 °C is unsafe | Warning / Critical | ✅ |
| Battery reports a fault | The battery says Dead, Overheat, Over voltage, Cold, or Failure | It may stop working or be unsafe | Critical | ✅ |
| Battery ran empty | 2% or less and still draining. (Not by voltage: many batteries report their nominal voltage as the "design minimum", which is reached at normal charge, so a voltage rule would warn wrongly.) | Draining to empty again and again wears the cells | Warning | ✅ |
| Charger too weak | Plugged in but the battery drains, for 10 minutes ("not charging" at a charge limit is normal and does not count) | The battery runs down even on the charger | Warning | ✅ |

## Temperatures and fans

| Warning | Fires when | Why it harms | Level | Data |
| --- | --- | --- | --- | --- |
| A part is too hot | Any sensor with its own limit (drive, graphics card, Wi-Fi, chipset) at its own warning temperature (`temp*_max`, or 5 °C under `temp*_crit`) for 5 minutes; at its critical temperature: Critical. Limits outside 40-150 °C are a chip reporting nonsense and are ignored | That part slows down or switches off | Warning / Critical | ✅ |
| Fan stopped while hot | A fan reports 0 RPM while the CPU is over 75 °C for 2 minutes (fans that stop when cool are normal) | The fan may be broken or blocked | Warning | ✅ |

## Graphics

| Warning | Fires when | Why it harms | Level | Data |
| --- | --- | --- | --- | --- |
| Graphics card hung | A GPU hang or reset in the kernel log in 24 hours | Freezes and crashes of the desktop | Warning | ✅ |
| No graphics acceleration | The desktop draws in software (`llvmpipe`) | Slow, high CPU use, more heat | Warning | ✅ |

## Hardware and kernel

| Warning | Fires when | Why it harms | Level | Data |
| --- | --- | --- | --- | --- |
| Kernel crash | An oops, BUG, or panic in the log in 7 days, or the kernel is marked "died" | The system may be unstable | Critical | ✅ |
| PCIe link failing | Fatal or uncorrectable PCIe errors; corrected errors over 100 in a day | A device or its slot is failing | Critical / Warning | ✅ |
| Device firmware crashed | Wi-Fi, Bluetooth, or graphics firmware crashed in 24 hours | That device drops out | Warning | ✅ |
| USB port overloaded | USB over-current in the kernel log | The port can be damaged; the device cuts out | Warning | ✅ |

## Services and system

| Warning | Fires when | Why it harms | Level | Data |
| --- | --- | --- | --- | --- |
| Service failed | A service failed | ✅ Exists | Warning | ✅ |
| Service keeps restarting | Restarted 10 or more times since boot | It crashes in a loop and wastes power | Warning | ✅ |
| Clock not in sync | Time sync is off, or the clock is over 1 second off | Logins, websites, and logs go wrong | Warning | ✅ |
| Broken package install | `dpkg --audit` reports half-installed packages | Updates stop working | Warning | ✅ |
| Automatic security updates stopped | No successful automatic update run in 7 days (such as "stopped: metered connection") | Security fixes stop arriving | Warning | ✅ |
| Restart needed | ✅ Exists | | Info | ✅ |

## Network

| Warning | Fires when | Why it harms | Level | Data |
| --- | --- | --- | --- | --- |
| Offline | ✅ Exists | | Info | ✅ |
| Internet not reachable | Connected, but NetworkManager says the internet does not answer ("limited"), for 10 minutes. A captive portal (a sign-in page) does not count. | Websites and updates fail | Warning | ✅ |
| Network errors | Receive or send errors over 1% of packets for 10 minutes | A bad cable, port, or driver | Warning | ✅ |
| Weak Wi-Fi | Signal under -75 dBm for 10 minutes | Slow and dropping connections | Info | ✅ |

## Security

| Warning | Fires when | Level | Data |
| --- | --- | --- | --- |
| Security updates waiting, firewall off with open ports, an extra root account, a container port open to the network | ✅ Exist today | Warning / Critical | ✅ |
| AppArmor is off | AppArmor is not enabled | Warning | ✅ |
| SSH allows root login with a password | `sshd -T` says so | Warning | Admin read |
| Secure Boot is off | It is off on a computer that supports it | Info | ✅ |

## Never warned about

CPU at 100%, RAM full of cache, swap in use, a busy disk, heavy network use, many processes, a full battery on the charger (Optimize may suggest a charge limit, but it is not a problem). These are use, not harm.

---

[Optimize](optimize-catalog.md) · [Expert coverage](expert-coverage.md) · [Rules engine](../02-architecture/rules-engine.md)
