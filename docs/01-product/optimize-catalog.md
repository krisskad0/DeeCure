# Optimize

[Docs](../README.md) › [Product](README.md) › [View levels](view-levels.md)

**Status: approved by the owner on 2026-10-23. Being built step by step; see [pending work](../05-roadmap/pending.md).**

The Optimize page (Advanced and Expert) lists every improvement that a measurement on this computer shows is needed. Nothing is offered "just in case".

Every item follows the fix engine's rules: a preview of each step, your OK first, the password only for admin steps, a check afterwards, and undo when undo is possible. Items that cannot be undone say so before you apply them.

- **Access:** "You" needs no password; "Admin" needs Full access.
- **Today:** "Exists" is one of the 22 fixes today; "New" is not built yet.

## Offered when measured

| Improvement | Offered only when | What it does | Undo | Access | Today |
| --- | --- | --- | --- | --- | --- |
| Free up space: Trash, thumbnails, package cache, old snap versions, system log, Docker images and build cache | Each part is over its size limit | Cleans only that part | No; caches rebuild themselves | You / Admin | Exists |
| Remove packages no longer needed, and leftover settings | apt's own simulation lists them | `apt autoremove`, purge leftovers | Reinstall from history | Admin | Exists |
| Remove old kernels | More than the running and one fallback kernel are installed | Removes the old ones | Reinstall | Admin | Exists |
| Keep fewer old snap versions from now on | Old snap versions came back over 1 GB after a clean-up | Sets snap to keep 2 versions instead of 3 | Yes | Admin | New (removing old versions exists) |
| Remove unused Flatpak runtimes | A runtime no installed app uses | `flatpak uninstall --unused` | Reinstall when an app needs it | You / Admin | New |
| Cap the system log size from now on | The journal grew over 1 GB again after it was shrunk | Adds a size limit for the journal | Yes (removes the limit) | Admin | New (shrinking it once exists) |
| Remove duplicate files | The duplicate scan found exact copies (same content hash) | Moves the copies you pick to the Trash | Yes (restore from Trash) | You | New |
| Turn on automatic TRIM | The computer has an SSD and `fstrim.timer` is off | Turns on the weekly TRIM | Yes | Admin | New |
| Use a better scheduler for a hard disk | A spinning disk uses the `none` scheduler | Sets `mq-deadline` for that disk | Yes | Admin | New |
| Turn on compressed swap (zswap) | Hard page faults and swap-ins stay high for 10 minutes or more | Turns on zswap | Yes | Admin | New |
| Raise the file-watch limit | More than 80% of file watches are in use | Raises the limit with one settings file | Yes (removes the file) | Admin | New |
| Let a device save power | The device supports power saving but it is off, and it is not a keyboard, mouse, or sound device | Turns on power saving for that one device | Yes | Admin | New |
| Stop a device from waking the computer | It woke the computer from suspend at least 3 times in 7 days while the lid was closed | Turns off waking for that one device | Yes | Admin | New |
| Turn off turbo on battery | The CPU often slows down from heat while on battery | Turns turbo off on battery only | Yes | Admin | New |
| Let the second graphics card sleep | It is awake while no app uses it | Turns on its power saving, or names the app that holds it | Yes | Admin | New |
| Install the hardware video driver | The graphics card can decode video but its driver package is missing | Installs `intel-media-va-driver` or `mesa-va-drivers` | Yes (remove the package) | Admin | New |
| Stop "wait for network" from delaying boot | It delays boot by more than 2 seconds, and nothing on this computer needs the network before sign-in (no network mounts, no service that wants `network-online.target`) | Turns off `NetworkManager-wait-online` | Yes | Admin | New |
| Turn off a service that slows boot | It is on the boot's critical chain, and it is not needed (for example Bluetooth on a computer without Bluetooth) | Turns the service off | Yes | Admin | Partly: turn off any service exists |
| Turn off a startup app that slows sign-in | It used much CPU in the first minute after sign-in | Turns the startup app off | Yes | You | Partly: turning off exists |
| Limit battery charge to 80% | The battery supports a limit, and the laptop was on mains power most of the last 14 days | Sets the charge limits (start 75%, stop 80%); any other values are in Power › Power controls | Yes | Admin | New |
| Change the power mode | Power saver on mains power, or Performance on battery for a long time | Changes the mode | Yes | You | Exists |
| Install security updates, or all updates | Updates are waiting | Installs them | No | Admin | Exists |
| Turn on the firewall | The firewall is installed but off | Turns it on | Yes | Admin | Exists |
| Install firmware updates | fwupd offers an update | Installs it | Some firmware cannot go back: said before you apply | Admin | New |
| Rebuild a missing driver (DKMS) | A local driver is not built for an installed kernel | Builds it | Not needed (only adds) | Admin | New |
| Install the recommended driver | A device uses a fallback driver or none, and `ubuntu-drivers` recommends one | Installs that driver package | Yes (remove the package) | Admin | New |
| Install missing firmware | The kernel log shows firmware that failed to load, and a newer `linux-firmware` has it | Updates `linux-firmware` | Yes (older version from the archive) | Admin | New |
| Use a gentler charge mode | Fast charging is on, and the laptop was on mains power most of the last 14 days | Sets the charge mode to Adaptive or Standard | Yes | Admin | New |
| Finish a broken package install | `dpkg --audit` reports half-installed packages | Finishes the pending step | No | Admin | New |

## Shown as advice only (no automatic change)

These cannot be fixed by software, so DeeCure shows the measurement and what to do:

| Advice | Shown when |
| --- | --- |
| Clean the fans, or check the cooling | The CPU slows down because of heat again and again |
| Move the SSD or graphics card, or check a firmware setting | It runs at a lower PCIe speed or fewer lanes than both sides support |
| Plug the device into a faster USB port | A USB 3 device works at USB 2 speed |
| Turn on the memory's rated speed in the BIOS | A memory module runs slower than its rated speed |
| Add a second memory module | Memory runs in single-channel mode and a slot is free |
| Use a stronger charger | The charger offers fewer watts than the laptop draws under load |
| Check or reseat the device and its cable | PCIe, SATA, or USB link errors keep rising |
| Use a faster SATA port or cable | A SATA drive links below the speed it supports |
| Replace the drive soon | Drive wear or media errors cross the drive's own warning level |
| Replace the battery | Battery health is under 60% |
| Restart to load new microcode or a new kernel | A newer one is installed but not running |
| Replace a memory module | Uncorrected memory errors are reported |

## Not offered, and why

Common "speed-up tips" that DeeCure will not offer:

| Tip | Why not |
| --- | --- |
| Clear the RAM cache | File cache is memory Linux gives back at once when an app needs it. Clearing it makes the computer slower. |
| Set swappiness to 10 | It helps only in some cases. DeeCure measures swapping instead and offers compressed swap when it really happens. |
| Turn off CPU security fixes (`mitigations=off`) | A little faster, but it removes protection against real attacks. |
| "RAM boosters", `preload`, system "cleaners" | No measured benefit on current Ubuntu. |
| Turn off logging | You lose the history needed to find problems. |
| `noatime` on every disk | Ubuntu already uses `relatime`; the difference is very small. |
| Remove snapd | Firefox and other Ubuntu apps are snaps; they would stop working. |
| Turn off the firewall or AppArmor for speed | No real speed gain, and a real loss of safety. |
| Overclock or undervolt | Risk to the hardware; undervolting is locked on most current Intel CPUs. |
| Change the disk scheduler on an NVMe SSD | `none`, Ubuntu's default for NVMe, is already the best choice. |

---

[View levels](view-levels.md) · [Expert coverage](expert-coverage.md)
