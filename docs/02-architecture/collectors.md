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
| Services | systemd `ListUnits` over D-Bus | Every 5 minutes | `.service` units only. Each is marked `timer` when a `.timer` unit with the same name exists (a scheduled job). |
| Network | `/proc/net/dev` | Every minute | Rates need two readings. The loopback interface (`lo`) is left out. |
| Disk I/O | `/proc/diskstats` | Every minute | Whole disks only: partitions, loop devices, and RAM disks are left out. For all disks together, `disk.busy` is the busiest disk. |
| Pressure | `/proc/pressure/{cpu,memory,io}` | Every minute | The "some" 10-second average. Missing on kernels without PSI. |
| Boot | systemd over D-Bus (`Manager` and unit timestamps) | Once per boot, after it finished | Phase times and the 15 slowest units, like `systemd-analyze`. Saves `boot.time` at the moment of boot, so the history shows one point per boot. |
| Updates | `apt list --upgradable` | Every hour; the last answer is repeated every minute | Read-only: no admin rights, no lock, changes nothing. Also reads `/etc/apt/apt.conf.d/20auto-upgrades` and `/run/reboot-required`. |
| Backups | Déjà Dup settings (`gsettings`), `/etc/timeshift/timeshift.json`, Timeshift snapshot folders | Every hour; repeated every minute | Whether backups run on a schedule, and the age of the newest backup or snapshot |
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
| `net_details` | `ip -j addr`, `/sys/class/net/*/statistics`, `/proc/net/route`, `/proc/net/wireless`, NetworkManager (system D-Bus), `/etc/resolv.conf`, `/run/systemd/resolve/resolv.conf`, `/etc/ufw/ufw.conf`, `/etc/default/ufw` | Addresses and traffic per interface, default gateway, Wi-Fi name, signal, band, and speed, internet connectivity, DNS resolver and servers, VPNs, and the firewall state and default policies (rules need root) |
| `sockets` | `/proc/net/{tcp,tcp6,udp,udp6}`, `/proc/<pid>/fd` | Every socket with local and remote address, state, who can reach it (this computer, one network, every network), and the owning program when it is the user's |
| `accounts` | `/etc/passwd`, `/etc/group`, logind (system D-Bus), `last`, journal (sudo), `~/.ssh`, `ssh-keygen -lf`, `/etc/ssh/sshd_config` | People who can sign in with groups and admin rights, groups with members and what they allow, open sessions, sign-ins and sudo use (30 days), SSH server settings and keys. `/etc/shadow` is never read. |
| `security` | `/proc/sys`, `/sys/module/apparmor`, SUID files and `dpkg -S`, scanner programs, ClamAV signature files, autostart and cron files, `/sys/module/*/taint`, `gsettings` | Kernel hardening settings, AppArmor, SUID programs and their packages, malware scanners, simple malware signs, and the screen lock |
| `privacy` | `pw-dump`, `snap connections`, `flatpak info --show-permissions`, `gsettings`, `/etc/popularity-contest.conf` | Apps using the microphone, camera, or screen now; what snap and Flatpak apps may use; desktop privacy settings |
| `packages` | `/var/lib/dpkg/status`, `/var/lib/apt/extended_states`, `snap list --all`, `flatpak list`, AppImages in home folders, apt sources (`.list` and deb822 `.sources`), `apt-get -s autoremove` (a simulation) | Every installed package with source, version, size, and whether a person asked for it; software sources with type, signature checking, and own key; packages no longer needed, disabled snap revisions, and leftover settings |
| `containers` | `docker ps -a`, `docker stats --no-stream`, `docker system df -v` (only when Docker's socket exists), `virsh list --all` (system and user), `VBoxManage list` | Containers with state, ports, CPU, and memory; images and whether a container uses them; volumes and links; virtual machines |
| `backups` | `gsettings org.gnome.DejaDup`, `/etc/timeshift/timeshift.json`, `/timeshift/snapshots` | Déjà Dup schedule and last backup; Timeshift setup, schedule, and snapshots |
| `cpu_detail` | `/sys/devices/system/cpu` (`cpufreq`, `cpuidle`, `thermal_throttle`, `acpi_cppc`, `cache`, `vulnerabilities`, `intel_pstate`), `/sys/class/powercap`, `/proc/interrupts`, `/proc/cpuinfo`, `/lib/firmware/intel-ucode` | Expert processor details: each CPU's speed now, allowed range, hardware maximum, speed policy, energy preference, firmware speed ranking, heat slowdowns, and sleep states; the chip's heat slowdowns, turbo, speed driver, power limits, caches, CPU flaws, every interrupt line, the running microcode and the newest installed one (Intel). Counters are totals since start, with the reading time, so the app works out rates from two readings. Read only while an Expert section is open. |
| `memory_detail` | `/proc/meminfo`, `/proc/vmstat`, `/proc/swaps`, `/sys/module/zswap`, `/sys/block/zram*`, `/sys/kernel/mm/transparent_hugepage`, `/sys/devices/system/edac/mc`, `oomctl` | Expert memory details: every `/proc/meminfo` line in bytes; page faults, hard page faults, swap-ins and swap-outs, direct reclaim, and out-of-memory kills since start; swap areas; zswap and zram; huge pages; memory error counts per controller and module (the EDAC memory type is left out because some drivers report a wrong one); the out-of-memory guard's limits and watched groups |
| `process_memory` | `/proc/<pid>/stat`, `statm`, `oom_score`, `smaps_rollup` | Every process's resident memory, hard page faults, and out-of-memory score; real memory (PSS), its own memory (USS), and swap share for the 40 biggest (reading `smaps_rollup` for all of them takes most of a second). Other users' processes have no PSS without admin rights |
| `cgroups` | `/sys/fs/cgroup` (`system.slice`, and the user's `app.slice`, `session.slice`, `background.slice`), desktop files | Each system service, user service, and app (GNOME runs each app in its own group, so its processes count once) with memory, its own memory and file cache, peak, swap, memory pressure, CPU time, tasks, and, for system services, disk bytes read and written. App names come from their desktop files |
| `gpu` | `/sys/class/drm/card*`, `pci.ids` | Each graphics card's maker and model, PCI ID, driver, whether it is the main card, and load, video memory, and temperature when the driver reports them without admin rights |

The 2-second rate while the app is open comes with the app in Phase 1. The collectors above read standard Linux files, so they work on any distro. Distro adapters arrive with the package collectors.

## History events

The code is in `crates/deecure-agent/src/history/`. A separate task reads these sources at start and then every 5 minutes, and saves what is new in the `event` table ([Storage › Events](storage.md#events)). Each event has a key, so reading a source twice stores nothing twice. A failing source is logged once and skipped; the others still run.

| Source | Reads | Events | Notes |
| --- | --- | --- | --- |
| apt | `/var/log/apt/history.log`, plus the newest rotated copy (`.1` or `.1.gz`) | One per package installed, upgraded, downgraded, removed, or purged, with versions and who started the run | Read only when a log's inode, size, or time changed. A run with no `End-Date` (power loss) counts as one hour long. |
| dpkg | `/var/log/dpkg.log` and its newest rotated copy | Packages changed with `dpkg -i` or `dpkg -r`, outside every apt run | apt runs are already counted. A purge right after a remove is one change. |
| snap, Flatpak | `snap list` (only when `/run/snapd.socket` exists), `flatpak list --app` | Apps installed, updated, or removed since the last list | The first list is only remembered. A failing command changes nothing, so it never looks like every app was removed. |
| Journal | `journalctl --grep` for USB plug lines, segfaults, out-of-memory kills, core dumps, and `Failed with result` | Devices connected and disconnected, crashes, memory kills, failed services | Grouped per hour ("crashed 14 times"). Each read starts at the hour of the last one, so counts are complete. The first read goes back 30 days, at most 20,000 entries. |
| Boots | `journalctl --list-boots` (at start, then every 6 hours) | Each start, "Started with a new kernel X (was Y)", and unclean shutdowns | |
| Config files | `/etc`, `~/.config/autostart`, `~/.config/systemd/user`, `crontab -l` | Files added, changed (with a diff for small text files), or removed | See below. |
| Fixes | The `fix_record` and `fix_undone` calls | Fixes applied and undone, with the run number for undo | Fixes from before history existed are added once at start. |

### Config files

- **No inotify.** Watching all of `/etc` would use hundreds of the watches that editors and IDEs need. Instead, every 5 minutes the folders are listed and each file's time and size are compared with the last scan (about 3,600 entries, a few milliseconds). This also finds changes made while the agent was not running.
- Only files that look different are read. A file whose content is the same (only touched) gives no event.
- Text files up to 64 KB are kept (`watched_file`), so the next change shows a diff of up to 8 KB. Bigger or binary files are compared by a fingerprint (FNV-1a) up to 8 MB, and by time and size above that. Files the user cannot read are compared by time and size.
- **The first scan** only remembers, except files changed in the last 30 days, which are shown as changes without a diff.
- **When:** the file's own time when it is newer than the last scan. Package managers keep the package's old file times, so then the folder's time is used, and if that is old too, the scan time.
- **Who:** "apt" (or "unattended-upgrades", "Software Updater") when it happened during an apt run, "DeeCure" during a fix, otherwise the file's owner.
- **Removed** only when the folder could be listed (or is gone too). A folder that becomes unreadable does not make its files look removed. A scan that hits 20,000 entries reports no removals.
- **Limits:** changes to one file on one day are one event that counts them. At most 200 events per scan; the rest become one "N more configuration files changed" event.
- **Left out:** `/etc/ssl/certs`, `/etc/ld.so.cache`, `/etc/apparmor.d/cache`, `/etc/mtab`, `/etc/.pwd.lock`, and CUPS files that change on every print job.

On a real Ubuntu 24.04 laptop the first run found 1,163 events from the last 30 days and kept 2,908 files (3.6 MB).

## Metric names

| Name | Subject | Unit | Meaning |
| --- | --- | --- | --- |
| `cpu.usage` | - | percent | Time the CPU was busy (not idle, not waiting for disk) |
| `cpu.user` | - | percent | Time in apps |
| `cpu.system` | - | percent | Time in the kernel |
| `cpu.iowait` | - | percent | Time waiting for the disk |
| `cpu.core` | core number, such as `0` | percent | Time one core was busy |
| `cpu.load_1m`, `cpu.load_5m`, `cpu.load_15m` | - | count | Load averages |
| `cpu.frequency` | - | hertz | Average current clock speed of all CPUs |
| `cpu.context_switches`, `cpu.interrupts` | - | count per second | Task switches and hardware interrupts |
| `memory.used` | - | bytes | Memory in use (total minus available) |
| `memory.used_percent` | - | percent | Same, as a share of all memory |
| `memory.total` | - | bytes | All memory |
| `memory.cache` | - | bytes | Page cache (given back when apps need it) |
| `memory.buffers` | - | bytes | Kernel buffers |
| `memory.free` | - | bytes | Memory not used for anything |
| `swap.used` | - | bytes | Swap in use |
| `swap.used_percent` | - | percent | Same, as a share of all swap |
| `swap.total` | - | bytes | All swap |
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
| `pressure.cpu`, `pressure.memory`, `pressure.io` | - | percent | Share of time some work waited for CPU, memory, or disk ("some", 10-second average) |
| `pressure.memory_full`, `pressure.io_full` | - | percent | Share of time all work waited at once ("full") |
| `temp` | sensor, as `<chip>/<label>` | celsius | Every temperature sensor |
| `temp.cpu` | - | celsius | The CPU temperature |
| `fan` | fan, as `<chip>/<label>` | count | Fan speed in RPM |
| `boot.time` | - | seconds | How long a boot took, stored at the time of that boot |
| `updates.pending`, `updates.security` | - | count | Packages with a newer version, and how many of those are security updates |
| `system.reboot_required` | - | count | 1 when an update asks for a restart, else 0 |
| `net.listening_all` | - | count | TCP ports listening on every network |
| `net.default_route` | - | count | 1 when an IPv4 or IPv6 default route is up (the loopback's reject routes do not count), else 0; missing without `/proc/net/route` |
| `accounts.people` | - | count | Accounts a person can sign in with: UID 1000 and up, a real shell, not `nobody` (from `/etc/passwd`) |
| `accounts.uid0_extra` | - | count | Accounts other than `root` with UID 0 |
| `privacy.popcon` | - | count | 1 when `/etc/popularity-contest.conf` says `PARTICIPATE="yes"`, else 0 (also when it is not installed) |
| `containers.exposed` | - | count | Running Docker containers with a port on every address (`0.0.0.0`, `::`), every 5 minutes; 0 without Docker; missing when this user may not talk to Docker |
| `firewall.enabled` | - | count | 1 when ufw is on, 0 when off; missing when ufw is not installed |
| `backup.configured` | - | count | 1 when Déjà Dup or Timeshift runs on a schedule |
| `backup.age_days` | - | count | Days since the newest backup or snapshot |
| `health.score` | - | count | The [health score](health-score.md), saved after every check |
| `services.running` | - | count | Services that are running |
| `services.failed` | - | count | Services that failed |

## Inventories (snapshots)

Stored only when they change, so they take little space.

| Kind | Contents |
| --- | --- |
| `filesystems` | Mount point, device, type, and size of each local filesystem |
| `boot` | Boot ID, boot time, systemd version, phase times, and the slowest units |
| `updates` | Pending updates (name, installed and new version, origin, security or not), when apt last refreshed, and whether automatic security updates are on |
| `services` | Name, description, load, active, and sub state, enabled-at-boot state (`UnitFileState`), and restarts since boot (`NRestarts`) of each service |

