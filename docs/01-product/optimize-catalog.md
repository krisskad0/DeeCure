# Optimize

[Docs](../README.md) › [Product](README.md) › [View levels](view-levels.md)

**Status: approved by the owner on 2026-10-23. Being built step by step; see [pending work](../05-roadmap/pending.md).**

The Optimize page (Tools › Optimize, in the menus at the Expert level, as [View levels](view-levels.md) sets) lists every improvement that a measurement on this computer shows is needed. Nothing is offered "just in case".

Every item follows the fix engine's rules: a preview of each step, your OK first, the password only for admin steps, a check afterwards, and undo when undo is possible. Items that cannot be undone say so before you apply them.

- **Access:** "You" needs no password; "Admin" needs Full access.
- **Today:** "✅ On Optimize" is checked and offered on the page now; "Exists" is a fix the app already has (on Issues and its own page) that Optimize does not list yet; "New" is not built yet.
- Every check is listed on the page, also when nothing is needed, with what it measured.

## Offered when measured

| Improvement | Offered only when | What it does | Undo | Access | Today |
| --- | --- | --- | --- | --- | --- |
| Free up space: Trash, thumbnails, package cache, old snap versions, system log, Docker images and build cache | Each part is over its size limit | Cleans only that part | No; caches rebuild themselves | You / Admin | ✅ On Optimize (one item per part over its limit: Trash, old snap versions 1 GB; thumbnails, package cache, system log 500 MB; Docker images, build cache 2 GB) |
| Remove packages no longer needed, and leftover settings | apt's own simulation lists them | `apt autoremove`, purge leftovers | Reinstall from history | Admin | ✅ On Optimize |
| Remove old kernels | More than the running and one fallback kernel are installed | Removes the old ones | Reinstall | Admin | ✅ On Optimize |
| Keep fewer old snap versions from now on | Old snap versions came back over 1 GB after a clean-up | Sets snap to keep 2 versions instead of 3 | Yes | Admin | ✅ On Optimize (a snap keeps 3 or more versions, old ones use over 1 GB, and they were cleaned before; undo puts back the old count) |
| Remove unused Flatpak runtimes | A runtime no installed app uses | `flatpak uninstall --unused` | Reinstall when an app needs it | You / Admin | ✅ On Optimize (user installs need no password; system installs need it) |
| Cap the system log size from now on | The journal grew over 1 GB again after it was shrunk | Adds a size limit for the journal | Yes (removes the limit) | Admin | ✅ On Optimize (cap 500 MB; applies to new logs after the next restart, shrunk now) |
| Remove duplicate files | The duplicate scan found exact copies (same content hash) | Moves the copies you pick to the Trash | Yes (restore from Trash) | You | New (needs a duplicate scan first) |
| Turn on automatic TRIM | The computer has an SSD and `fstrim.timer` is off | Turns on the weekly TRIM | Yes | Admin | ✅ On Optimize |
| Use a better scheduler for a hard disk | A spinning disk uses the `none` scheduler | Sets `mq-deadline` for that disk | Yes | Admin | ✅ On Optimize (`mq-deadline`, else `bfq`; USB sticks are left out because many say they spin; kept after restart, skipped for a different disk) |
| Turn on compressed swap (zswap) | Hard page faults and swap-ins stay high for 10 minutes or more | Turns on zswap | Yes | Admin | ✅ On Optimize (high = over 50 pages swapped in and 100 hard faults a second, 10 minutes in a row in the last 24 hours; kept after restart) |
| Raise the file-watch limit | More than 80% of file watches are in use | Raises the limit with one settings file | Yes (removes the file) | Admin | ✅ On Optimize (to twice the limit, at least 524288) |
| Let a device save power | The device supports power saving but it is off, and it is not a keyboard, mouse, or sound device | Turns on power saving for that one device | Yes | Admin | ✅ On Optimize (same rule as Power › Sleep) |
| Stop a device from waking the computer | It woke the computer from suspend at least 3 times in 7 days while the lid was closed | Turns off waking for that one device | Yes | Admin | ✅ On Optimize as advice: the log shows wake-ups with the lid closed (3 or more in 7 days) but not which device, so it names the last one and points to Power › Sleep |
| Turn off turbo on battery | The CPU often slows down from heat while on battery | Turns turbo off on battery only | Yes | Admin | ✅ On Optimize ("often" = heat slowdowns in at least 1 of 10 minutes on battery, with 60 minutes or more on battery in 7 days) |
| Let the second graphics card sleep | It is awake while no app uses it | Turns on its power saving, or names the app that holds it | Yes | Admin | ✅ On Optimize (a fix when its power saving is off; advice that names the app, or says a screen is connected, otherwise) |
| Install the hardware video driver | The graphics card can decode video but its driver package is missing | Installs `intel-media-va-driver` or `mesa-va-drivers` | Yes (remove the package) | Admin | ✅ On Optimize |
| Stop "wait for network" from delaying boot | It delays boot by more than 2 seconds, and nothing on this computer needs the network before sign-in (no network mounts, no service that wants `network-online.target`) | Turns off `NetworkManager-wait-online` | Yes | Admin | ✅ On Optimize (a service counts when it wants `network-online.target` and started at this boot) |
| Turn off a service that slows boot | It is on the boot's critical chain, and it is not needed (for example Bluetooth on a computer without Bluetooth) | Turns the service off | Yes | Admin | Partly: turn off any service exists; the "not needed" check is new |
| Turn off a startup app that slows sign-in | It used much CPU in the first minute after sign-in | Turns the startup app off | Yes | You | Partly: turning off exists; measuring CPU in the first minute after sign-in is new |
| Limit battery charge to 80% | The battery supports a limit, and the laptop was on mains power most of the last 14 days | Sets the charge limits (start 75%, stop 80%); any other values are in Power › Power controls | Yes | Admin | ✅ On Optimize ("most" = 70% of 14 days; until 14 days of readings exist, the page says how many days it has) |
| Change the power mode | Power saver on mains power, or Performance on battery for a long time | Changes the mode | Yes | You | ✅ On Optimize ("a long time" = the last hour, all on mains or all on battery) |
| Install security updates, or all updates | Updates are waiting | Installs them | No | Admin | ✅ On Optimize |
| Turn on the firewall | The firewall is installed but off | Turns it on | Yes | Admin | ✅ On Optimize |
| Install firmware updates | fwupd offers an update | Installs it | Some firmware cannot go back: said before you apply | Admin | ✅ On Optimize (fwupd asks for the password itself) |
| Rebuild a missing driver (DKMS) | A local driver is not built for an installed kernel | Builds it | Not needed (only adds) | Admin | ✅ On Optimize (only kernels that can start) |
| Install the recommended driver | A device uses a fallback driver or none, and `ubuntu-drivers` recommends one | Installs that driver package | Yes (remove the package) | Admin | ✅ On Optimize (when Ubuntu's pick is not installed) |
| Install missing firmware | The kernel log shows firmware that failed to load, and a newer `linux-firmware` has it | Updates `linux-firmware` | Yes (older version from the archive) | Admin | ✅ On Optimize (offered only when a newer linux-firmware is waiting) |
| Use a gentler charge mode | Fast charging is on, and the laptop was on mains power most of the last 14 days | Sets the charge mode to Adaptive or Standard | Yes | Admin | ✅ On Optimize (Adaptive, else Standard; same 14 days rule) |
| Finish a broken package install | `dpkg --audit` reports half-installed packages | Finishes the pending step | No | Admin | ✅ On Optimize (keeps your own settings files) |

## Shown as advice only (no automatic change)

These cannot be fixed by software, so DeeCure shows the measurement and what to do:

| Advice | Shown when | Today |
| --- | --- | --- |
| Clean the fans, or check the cooling | The CPU slowed down from heat in the last 24 hours (the stored counter, added up across restarts) | ✅ On Optimize |
| Move the SSD or graphics card, or check a firmware setting | It runs with fewer lanes than both sides support, or a drive runs at a lower PCIe speed than both support (graphics and other devices slow their link when idle, so their speed alone does not count; links that are down and built-in devices without a real link are skipped) | ✅ On Optimize |
| Plug the device into a faster USB port | A USB 3 device works at USB 2 speed, and the computer has USB 3 ports | ✅ On Optimize |
| Turn on the memory's rated speed in the BIOS | A memory module runs slower than its rated speed | Needs the memory module read (step 6) |
| Add a second memory module | Memory runs in single-channel mode and a slot is free | Needs the memory module read (step 6) |
| Use a stronger charger | The charger offers fewer watts than the laptop draws under load | New |
| Check or reseat the device and its cable | PCIe, SATA, or USB link errors keep rising | New (needs stored counts to see them rise) |
| Use a faster SATA port or cable | A SATA drive links below the speed it supports | Needs the drive read (step 6): Linux reports only the controller's limit, not the drive's |
| Replace the drive soon | Drive wear or media errors cross the drive's own warning level | New |
| Replace the battery | Battery health is under 60% | ✅ On Optimize |
| Restart to load new microcode or a new kernel | A newer one is installed but not running | ✅ On Optimize |
| Replace a memory module | Uncorrected memory errors are reported | ✅ On Optimize |

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
