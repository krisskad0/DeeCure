# Collector (deecure-agent)

[Docs](../README.md) › [Architecture](README.md)

The agent reads data on a schedule. Heavy work runs only when the PC is idle, with low priority (`nice` and `ionice`).

| What | How often | Source examples |
| --- | --- | --- |
| Live metrics (CPU, RAM, pressure, temperature, network) | Every 2s while the UI is open; every 60s in the background | /proc, /sys, hwmon |
| Services and timers state | On change (D-Bus signals) plus every 5 min | systemd over D-Bus |
| Logs (errors, crashes, OOM kills) | Live follow | systemd journal |
| Apps and packages | On change (watch dpkg log, snap and flatpak) plus daily | dpkg, snap, flatpak |
| Config and jobs changes | On change (inotify on /etc, cron folders, autostart) | File watcher |
| Hardware inventory | At start and on device plug events | udev, /sys, lspci, lsusb |
| Disk health (SMART) | Daily (through the helper) | smartctl, nvme |
| Full disk-usage scan | Weekly or on demand, when idle | Directory walk with cache |
| Security checks | Daily | Rules engine |

Every collector has an **adapter** interface. Ubuntu/Debian adapters come first; Fedora, Arch, and others plug in later.

## What is built

The code is in `crates/deecure-agent/src/collect/`. Every file collector reads through a `SystemRoot`, so tests can point it at saved copies of `/proc` and `/sys` from real machines.

| Collector | Reads | How often | Notes |
| --- | --- | --- | --- |
| CPU | `/proc/stat`, `/proc/loadavg`, `/sys/devices/system/cpu/cpu*/cpufreq` | Every minute | Usage and rates need two readings, so they appear from the second run on |
| Memory | `/proc/meminfo` | Every minute | Swap metrics only when the system has swap |
| Filesystems | `/proc/mounts`, `statvfs` | Every minute | Local types only (ext4, btrfs, xfs, vfat, ntfs…). Network shares are skipped because checking them can hang. Snap images (squashfs) are skipped. One entry per device. |
| Battery | `/sys/class/power_supply/*` | Every minute | System batteries only. Mouse and keyboard batteries (`scope` = `Device`) are skipped. Missing values are left out, never guessed. |
| Services | systemd `ListUnits` over D-Bus | Every 5 minutes | `.service` units only |
| Network | `/proc/net/dev` | Every minute | Rates need two readings. The loopback interface (`lo`) is left out. |
| Disk I/O | `/proc/diskstats` | Every minute | Whole disks only: partitions, loop devices, and RAM disks are left out. For all disks together, `disk.busy` is the busiest disk. |
| Pressure | `/proc/pressure/{cpu,memory,io}` | Every minute | The "some" 10-second average. Missing on kernels without PSI. |
| Boot | systemd over D-Bus (`Manager` and unit timestamps) | Once per boot, after it finished | Phase times and the 15 slowest units, like `systemd-analyze`. Saves `boot.time` at the moment of boot, so the history shows one point per boot. |
| Updates | `apt list --upgradable` | Every hour; the last answer is repeated every minute | Read-only: no admin rights, no lock, changes nothing. Also reads `/etc/apt/apt.conf.d/20auto-upgrades` and `/run/reboot-required`. |
| Sensors | `/sys/class/hwmon/*` | Every minute | Every temperature and fan the kernel exposes. `temp.cpu` comes from the best CPU sensor (`coretemp` package, `k10temp`, …). |

### Live inventories

Some data changes too fast to store, or is cheap to read again. The agent reads it when asked (the `inventory` call), and keeps nothing.

Safety rules for live reads:

- Read only. Helper programs (`pw-dump`, `journalctl`, `gsettings`) only read, and run with a time limit.
- D-Bus services are asked only when they are already running (`NameHasOwner`). DeeCure never starts a service by asking it (no D-Bus activation).
- Values only root can read are left out, and the app says they need Full access.

| Kind | Reads | Contents |
| --- | --- | --- |
| `system` | `/etc/os-release`, `/proc`, `/sys/class/dmi/id`, `/run/reboot-required` | Host name, OS, kernel, architecture, maker and model, CPU model and threads, memory, uptime, and whether an update asks for a restart |
| `processes` | `/proc/<pid>/stat`, `status`, `cmdline`, `/etc/passwd` | PID, name, user, state, CPU %, memory, swap, threads, and command line of every process. CPU % is measured between two calls; the first call gives the average since each process started. Names cut at 15 characters by Linux are completed from the command line. |
| `network` | `/sys/class/net`, `/proc/net/{tcp,tcp6,udp,udp6}` | Each interface's kind (Wi-Fi, Ethernet, USB Ethernet, bridge, loopback, mobile, VPN, virtual), state, MAC, link speed, and MTU; counts of open TCP connections, listening TCP ports, and UDP sockets |
| `os` | `/etc/os-release`, `/usr/share/distro-info/ubuntu.csv`, `/proc/cpuinfo`, `/sys/class/dmi/id`, `/etc/machine-id`, `/run/systemd/users`, `/var/lib/ubuntu-advantage/status.json`, `/etc/timezone`, `/run/systemd/timesync/synchronized`, `/etc/adjtime`, `/etc/default/{locale,keyboard}` | Release name and support dates, virtualization, machine ID (shortened), people signed in, Ubuntu Pro and Livepatch, time zone, NTP sync, time server, RTC mode, locale, and keyboard layout |
| `kernel` | `/proc/modules`, `/proc/cmdline`, `/proc/sys/kernel/tainted`, `/lib/modules`, `/boot` | Running kernel, taint flags, boot options, installed kernels (running, fallback, old) with size and install date, and loaded modules |
| `update_history` | `/var/log/apt/history.log` | The last 100 apt runs: when, who started them, packages changed, and whether apt logged an error. Rotated logs are not read yet. |
| `hardware` | `/sys/class/dmi/id`, `/sys/firmware`, `/sys/class/tpm`, `/proc/cpuinfo`, `/sys/devices/system/cpu`, `/sys/devices/cpu_{core,atom}`, `/sys/block`, `/sys/bus/{pci,usb}`, `/proc/bus/input/devices`, `pci.ids` | Machine, chassis, BIOS, boot mode, Secure Boot, TPM; processor model, cores (performance and efficient), threads, speeds, L3 cache, microcode, features, and each logical CPU; memory total and ECC reporting; whole disks; PCI, USB, keyboard, and pointer devices with their drivers |
| `power` | `/sys/class/power_supply`, power-profiles-daemon (D-Bus), `/etc/UPower/UPower.conf`, `gsettings`, logind config, `/sys/power/mem_sleep` | Batteries (status, capacity, design, cycles, charge limit), mains power, power mode, battery policy, screen blank and suspend timeouts, lid action, sleep mode |
| `display` | `/sys/class/drm` (EDID), GNOME Mutter `GetCurrentState` (session D-Bus), `gsettings` | Every connector, connected screens with name, resolution, refresh rate, scale, and primary; GNOME Shell version; fractional scaling |
| `audio` | `pw-dump` | Sound server, output and input devices with volume, mute, and default; apps playing sound |
| `bluetooth` | BlueZ (system D-Bus) | Adapters, whether one is on, and paired or connected devices with battery level |
| `firmware` | fwupd (system D-Bus) | Devices with firmware, their version, and updates fwupd's local metadata offers |
| `sleep` | `journalctl` (systemd-sleep) | Suspends and hibernates of the last 30 days, how long each lasted, and failures |
| `storage` | `/proc/mounts`, `/sys/class/block` (device-mapper layers), `statvfs`, `/etc/fstab`, Trash, thumbnail cache, `/var/cache/apt/archives`, `snap list --all`, `journalctl --disk-usage`, `docker system df` (only when Docker runs) | Root encryption (LUKS anywhere below `/`), the root-only reserve, `/etc/fstab` problems, and cleanup candidates with size, undo note, and whether the admin password is needed. Docker volumes are never offered: they hold data. |
| `disk_usage` | A walk of home and `/` (one filesystem), like `du` | Top folders of home and the system disk, and the 25 biggest files in home. Runs on a background thread only when asked, at most once an hour; while it runs the answer says `scanning`. Stops after 3 million entries. |
| `user_services` | user systemd (session D-Bus) | The user's services with state, enabled state, memory, and restarts |
| `service_memory` | systemd (system D-Bus) `MemoryCurrent` | Memory of each running system service |
| `startup` | `~/.config/autostart`, `/etc/xdg/autostart` | Apps started at a GNOME login, user files overriding system ones, with on/off state. GNOME's own settings daemons are left out. |
| `jobs` | `crontab -l`, `/etc/crontab`, `/etc/cron.d`, systemd timers (system and user D-Bus), `/etc/anacrontab`, `/var/spool/anacron`, `atq`, journal (CRON and unit start/fail messages, 7 days) | Cron lines, timers with schedule, last and next run, and result, anacron jobs with last run, at jobs, and every job run of the last 7 days |
| `journal` | `journalctl --since -24h` | The newest 500 entries of the last 24 hours (time, priority, unit, message) and entries per hour |
| `errors` | `journalctl -p err --since -7d` | Errors grouped by message with numbers taken out, each marked "needs attention", "worth a look", or "harmless", and every error time for the heatmap |
| `kernel_log` | `journalctl -k --since -7d` | The newest 500 kernel messages with their subsystem |
| `crashes` | journal (segfaults, out-of-memory kills, services that dumped core), `/var/crash` (apport) | Crashes of the last 7 days, grouped by program and signal |
| `boots` | `journalctl --list-boots`, then each boot's first kernel line and last lines | The last 10 boots with start, end, kernel, and whether the shutdown was clean |
| `device_events` | kernel log (`usb …` lines) | USB devices plugged in and out in the last 30 days |
| `config_changes` | `/etc` (file times) | Files in `/etc` changed in the last 30 days, with their owner |
| `package_history` | `/var/log/apt/history.log` | Every package installed, upgraded, or removed, with versions and who started it |
| `gpu` | `/sys/class/drm/card*`, `pci.ids` | Each graphics card's maker and model, PCI ID, driver, whether it is the main card, and load, video memory, and temperature when the driver reports them without admin rights |

The 2-second rate while the app is open comes with the app in Phase 1. The collectors above read standard Linux files, so they work on any distro. Distro adapters arrive with the package collectors.

## Metric names

| Name | Subject | Unit | Meaning |
| --- | --- | --- | --- |
| `cpu.usage` | — | percent | Time the CPU was busy (not idle, not waiting for disk) |
| `cpu.user` | — | percent | Time in apps |
| `cpu.system` | — | percent | Time in the kernel |
| `cpu.iowait` | — | percent | Time waiting for the disk |
| `cpu.core` | core number, such as `0` | percent | Time one core was busy |
| `cpu.load_1m`, `cpu.load_5m`, `cpu.load_15m` | — | count | Load averages |
| `cpu.frequency` | — | hertz | Average current clock speed of all CPUs |
| `cpu.context_switches`, `cpu.interrupts` | — | count per second | Task switches and hardware interrupts |
| `memory.used` | — | bytes | Memory in use (total minus available) |
| `memory.used_percent` | — | percent | Same, as a share of all memory |
| `memory.total` | — | bytes | All memory |
| `memory.cache` | — | bytes | Page cache (given back when apps need it) |
| `memory.buffers` | — | bytes | Kernel buffers |
| `memory.free` | — | bytes | Memory not used for anything |
| `swap.used` | — | bytes | Swap in use |
| `swap.used_percent` | — | percent | Same, as a share of all swap |
| `swap.total` | — | bytes | All swap |
| `disk.used_percent` | mount point | percent | Space used, computed like `df` |
| `disk.free` | mount point | bytes | Space a normal user can still write |
| `disk.used` | mount point | bytes | Space in use |
| `disk.inodes_percent` | mount point | percent | File slots in use; missing for filesystems without a fixed count (btrfs) |
| `battery.charge` | battery, such as `BAT0` | percent | Charge level |
| `battery.health` | battery | percent | Capacity now compared with when new |
| `battery.power` | battery | watts | Power going in or out right now |
| `battery.time_left` | battery | seconds | Time until empty, only while discharging |
| `net.rx`, `net.tx` | interface, or none for the total | bytes per second | Data received and sent |
| `disk.read`, `disk.write` | disk, or none for the total | bytes per second | Data read from and written to disks |
| `disk.iops` | disk, or none | count per second | Reads and writes completed |
| `disk.busy` | disk, or none (busiest disk) | percent | Time the disk had work to do |
| `disk.await` | disk, or none | seconds | Average time one read or write took |
| `pressure.cpu`, `pressure.memory`, `pressure.io` | — | percent | Share of time some work waited for CPU, memory, or disk ("some", 10-second average) |
| `pressure.memory_full`, `pressure.io_full` | — | percent | Share of time all work waited at once ("full") |
| `temp` | sensor, as `<chip>/<label>` | celsius | Every temperature sensor |
| `temp.cpu` | — | celsius | The CPU temperature |
| `fan` | fan, as `<chip>/<label>` | count | Fan speed in RPM |
| `boot.time` | — | seconds | How long a boot took, stored at the time of that boot |
| `updates.pending`, `updates.security` | — | count | Packages with a newer version, and how many of those are security updates |
| `system.reboot_required` | — | count | 1 when an update asks for a restart, else 0 |
| `health.score` | — | count | The [health score](health-score.md), saved after every check |
| `services.running` | — | count | Services that are running |
| `services.failed` | — | count | Services that failed |

## Inventories (snapshots)

Stored only when they change, so they take little space.

| Kind | Contents |
| --- | --- |
| `filesystems` | Mount point, device, type, and size of each local filesystem |
| `boot` | Boot ID, boot time, systemd version, phase times, and the slowest units |
| `updates` | Pending updates (name, installed and new version, origin, security or not), when apt last refreshed, and whether automatic security updates are on |
| `services` | Name, description, load, active, and sub state, enabled-at-boot state (`UnitFileState`), and restarts since boot (`NRestarts`) of each service |

