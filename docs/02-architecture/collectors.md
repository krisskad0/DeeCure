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
| `containers` | `docker ps -a`, `docker stats --no-stream`, `docker system df -v` (only when Docker's socket exists), `virsh list --all` (system and user), `VBoxManage list` | Containers with state, ports, CPU, and memory; images and whether a container uses them; volumes and links; virtual machines; for running containers also network and disk bytes (`docker stats`), and the build cache entries with size, in use, and last use |
| `backups` | `gsettings org.gnome.DejaDup`, `/etc/timeshift/timeshift.json`, `/timeshift/snapshots` | Déjà Dup schedule and last backup; Timeshift setup, schedule, and snapshots |
| `cpu_detail` | `/sys/devices/system/cpu` (`cpufreq`, `cpuidle`, `thermal_throttle`, `acpi_cppc`, `cache`, `vulnerabilities`, `intel_pstate`), `/sys/class/powercap`, `/proc/interrupts`, `/proc/cpuinfo`, `/lib/firmware/intel-ucode` | Expert processor details: each CPU's speed now, allowed range, hardware maximum, speed policy, energy preference, firmware speed ranking, heat slowdowns, and sleep states; the chip's heat slowdowns, turbo, speed driver, power limits, caches, CPU flaws, every interrupt line, the running microcode and the newest installed one (Intel). Counters are totals since start, with the reading time, so the app works out rates from two readings. Read only while an Expert section is open. |
| `memory_detail` | `/proc/meminfo`, `/proc/vmstat`, `/proc/swaps`, `/sys/module/zswap`, `/sys/block/zram*`, `/sys/kernel/mm/transparent_hugepage`, `/sys/devices/system/edac/mc`, `oomctl` | Expert memory details: every `/proc/meminfo` line in bytes; page faults, hard page faults, swap-ins and swap-outs, direct reclaim, and out-of-memory kills since start; swap areas; zswap and zram; huge pages; memory error counts per controller and module (the EDAC memory type is left out because some drivers report a wrong one); the out-of-memory guard's limits and watched groups |
| `process_memory` | `/proc/<pid>/stat`, `statm`, `oom_score`, `smaps_rollup` | Every process's resident memory, hard page faults, and out-of-memory score; real memory (PSS), its own memory (USS), and swap share for the 40 biggest (reading `smaps_rollup` for all of them takes most of a second). Other users' processes have no PSS without admin rights |
| `cgroups` | `/sys/fs/cgroup` (`system.slice`, and the user's `app.slice`, `session.slice`, `background.slice`), desktop files | Each system service, user service, and app (GNOME runs each app in its own group, so its processes count once) with memory, its own memory and file cache, peak, swap, memory pressure, CPU time, tasks, and, for system services, disk bytes read and written. App names come from their desktop files |
| `storage_detail` | `/sys/block` (`queue`, partitions), `/run/udev/data` (partition table), `/sys/class/ata_link`, `/sys/class/nvme`, `/sys/module/nvme_core`, `/proc/mounts`, `statvfs`, `/sys/fs/ext4` | Expert drive and filesystem details: each disk's scheduler and the ones offered, queue size, read-ahead, block sizes, TRIM support, partition table, partitions with their 1 MiB alignment, SATA link speed and limit, NVMe firmware, transport, PCI address, and its own temperature limits; NVMe power saving; each local filesystem's mount options, inodes, and the errors ext4 recorded |
| `process_io` | `/proc/<pid>/io` | Bytes each of the user's processes read from and wrote to storage (not the cache), and how many other users' processes could not be read |
| `pci_detail` | `/sys/bus/pci/devices` (links, `aer_dev_*`, `power`), `pci.ids` | Each PCI device's class, maker and model, driver, link speed and lanes now, the most it supports, and the most its slot supports; its link errors (corrected, not fixed, fatal); its power saving setting and state, and its path under `/sys/devices` |
| `gpu_detail` | `/sys/class/drm` (i915 `gt_*` and `gt/gt0`, xe `tile0/gt0/freq0`, amdgpu `pp_dpm_sclk` and hwmon), `/proc/<pid>/fd` and `fdinfo` | Expert graphics: each card's clock now, allowed range, hardware maximum and boost, time in power saving (Intel RC6), why it is slowed down now (Intel), power drawn (AMD), whether it sleeps, whether it is the main card, and its screen connectors; and each of the user's apps' busy time per graphics engine and graphics memory (each DRM client counted once) |
| `graphics_stack` | `XDG_SESSION_TYPE` or logind, `glxinfo -B`, `dpkg-query` (`libgl1-mesa-dri`), `/sys/module/nvidia/version`, `/usr/lib/x86_64-linux-gnu/dri/*_drv_video.so`, `vainfo`, `journalctl -k -b` | The desktop session type, OpenGL renderer and whether it is software, Mesa and NVIDIA driver versions, the video-decoding drivers installed, the formats decoded in hardware (when `vainfo` is installed), and this boot's graphics firmware lines (each once) |
| `net_deep` | `nmcli`, `/proc/net/wireless`, NetworkManager `conf.d` files, `/sys/class/net/*/statistics`, `/proc/net/snmp`, `resolvectl status`, `ip -j route`, `ip -j rule` | Expert network details: the Wi-Fi network in use (channel, band, link speed, signal in percent and dBm, noise when reported, security, power saving from the network's own setting or NetworkManager's global one); errors, drops, collisions, and link changes per interface; TCP and UDP counters; DNS per link (server, DNSSEC, DNS over TLS, default route); IPv4 and IPv6 routes; and routing rules beyond the standard three |
| `tcp_traffic` | `ss -tinpH state established` | Every open TCP connection with its program (the user's own), bytes received, bytes the other side confirmed, and bytes sent again. TCP only: QUIC and other UDP traffic are not counted, and closed connections drop out |
| `power_detail` | `/sys/class/power_supply`, `/sys/class/usb_power_delivery`, `/sys/power/mem_sleep`, `/sys/module/pcie_aspm`, `/sys/power/pm_wakeup_irq`, `/etc/deecure/power.json`, `/run/deecure/power-boot.json`, `/proc/interrupts`, `/sys/class/wakeup`, `/sys/bus/usb/devices` | Expert power details: each system battery's voltage and design minimum, current, power, status, temperature, health word, manufacture date, serial, charge limits, charge modes, and charging behaviour; mains and USB chargers with their USB type and most watts; what each USB-C charger offers and what the laptop accepts (Power Delivery); suspend types; the PCIe power saving policy (`/sys/module/pcie_aspm/parameters/policy`); what closing the lid does on battery and plugged in, read the way logind reads its files (the first `logind.conf` in `/etc`, `/run`, `/usr/local/lib`, `/usr/lib`, then all `logind.conf.d` drop-ins by name, `/etc` replacing the same name), with the file each value comes from and what logind runs with now (its D-Bus properties); every wake source with its device, PCI address or USB product, counts (signalled, stopped a suspend, kept awake), and whether it may wake the computer; the interrupt that last woke it; each USB device's power saving and device path, with keyboards, mice, and sound devices marked to stay awake; every rfkill radio with soft and hard blocks; the keyboard backlight level and maximum; and the power settings kept after restart (`/etc/deecure/power.json`) with what happened to each at this start (`/run/deecure/power-boot.json`), counting a result only when it was for the same change |
| `optimize` | The Expert readers above, `systemctl list-dependencies --reverse network-online.target`, `systemctl is-enabled fstrim.timer`, `/proc/mounts`, `/etc/fstab`, `/sys/bus/usb/devices`, `/sys/class/power_supply`, and the stored `cpu.throttle_total` | Every Optimize check ([Optimize](../01-product/optimize-catalog.md)) with what it measured, offered or not: "wait for network" at boot (offered when it took over 2 s and no network mount or service started at this boot waits for the network), weekly TRIM for SSDs, old kernels, devices with power saving off (the Power › Sleep rule), heat slowdowns in the last 24 hours, PCIe links narrower (or drives slower) than both ends support, USB 3 devices at USB 2 speed, battery health under 60%, a newer kernel or microcode waiting for a restart, uncorrected memory errors; and batch 2: each cleanup part over its size limit, packages no longer needed and leftover settings, updates, the firewall, a broken package install, unused Flatpak runtimes, the log size cap and snap versions (only after a clean-up), the file-watch limit, hard disk schedulers, the battery charge limit and mode (14 days of `power.on_mains`), turbo on battery (`cpu.throttle_total` while on battery), the power mode, wake-ups with the lid closed (the system log), zswap (`memory.swap_ins_per_s` and `memory.hard_faults_per_s`), the hardware video driver, a second graphics card that stays awake, drivers Ubuntu recommends, firmware that failed to load, fwupd updates, and DKMS builds. A fix carries its fix action and subject; advice carries what to do. Read only when the page asks |
| `thermal` | `/sys/class/thermal` | Each thermal zone's temperature, policy, and the trip points in use (where it slows down, starts fans, warns, and shuts down; turned-off ones are left out), and the cooling devices grouped by kind with their highest level now and their maximum |
| `boot_detail` | `systemd-analyze critical-chain`, `systemd-analyze blame` | The boot's critical chain (each step with when it was ready and how long it took, and its depth) and every unit's start time, slowest first |
| `unit_files` | `systemctl list-units`, `systemctl list-unit-files --state=masked`, `systemctl show`, `/etc/systemd/system`, `/run/systemd/system` | Each system service's unit file, overrides made on this computer apart from the ones its package ships, what turns it on, restart policy and restart count, on/off state, and for a masked service whether it was masked here, until restart, or by its package. The systemd preset is not used: Ubuntu packages turn services on with their own scripts, so the preset is not a reliable default |
| `service_security` | `systemd-analyze security` | Each service's exposure score (0 locked down to 10) and level |
| `kernel_detail` | `sysctl.d` folders and `/etc/sysctl.conf` (in systemd's order), `/proc/sys`, `dpkg -S` and `dpkg -V` (with `/lib` and `/usr/lib` both tried, for Ubuntu's merged `/usr`), `/proc/<pid>/fdinfo` (inotify), `/proc/sys/fs`, `/proc/modules`, `/sys/module`, `dkms status`, `/lib/modules`, `/sys/kernel/security/lockdown`, `timedatectl timesync-status` | Every kernel setting from a file with its file, line, owning package, whether the file was changed from the package's version, whether a later file replaces it, and the value now (spaces normalised); file watches used by the user's programs against the limit, and the top 10 programs; open files system-wide against the limit; loaded modules with taint letters, users, and device counts; DKMS drivers and the kernels they are installed for; kernel lockdown; and time sync |
| `platform_detail` | `fwupdmgr security` (text and `--json`), `/sys/firmware/efi/efivars` (`BootOrder`, `BootCurrent`, `Boot####`), `mokutil`, `/sys/class/mei`, `/sys/class/tpm`, `/etc/crypttab`, `/sys/class/iommu`, `/sys/bus/thunderbolt`, `/sys/class/rtc`, `/proc/cpuinfo`, `/dev/kvm`, `/sys/bus/usb/devices`, `/sys/class/typec` | Expert firmware and ports: the HSI level and every fwupd check (passed, and whether a failure is in the running system); UEFI boot entries with name, file, order, and the current one; Secure Boot and enrolled MOK keys; management engine version; TPM version and driver; disk encryption and whether the TPM unlocks it; IOMMU units; Thunderbolt security and DMA protection; hardware clock drift; virtualization and KVM; each USB device's version, speed, power asked, and drivers; each USB-C port's roles, power mode, partner, and alternate modes; and Thunderbolt devices with their approval |
| `devices_detail` | `/sys/class/video4linux`, `fprintd-list`, `ethtool` (`-i`, link, `--show-eee`), `/proc/bus/input/devices`, `upower -d`, `lpstat`, `/sys/class/drm/*/edid`, `/sys/class/backlight`, `/proc/asound` | Cameras with their driver; the fingerprint reader and fingers enrolled; each network chip's driver, firmware, bus, and for Ethernet the speed, duplex, fastest supported mode, cable, and energy saving; touchpads, touch screens, and pens; batteries of mice and headsets; printers with state and waiting jobs; each connected screen from its EDID (maker code, model text with binary bytes removed, product code, year, size, native mode, refresh range, colour depth, HDR transfer functions from a CTA-861 block); backlights (the level set); and sound cards with their codecs |
| `hw_errors` | `journalctl -k --since -7d --grep`, `/sys/class/devcoredump`, `ubuntu-drivers devices` | Hardware errors of the last 7 days by kind (CPU or memory faults, firmware that failed to load or crashed, USB errors, disk link errors, PCIe errors, graphics hangs, heat events), each with count, last time, and up to three different messages; devices that left a crash dump; and the drivers Ubuntu offers per device |
| `software_health` | `apt-mark showhold`, `dpkg --audit`, `apt list --installed`, `snap list --all`, `snap refresh --time`, `flatpak list`, `flatpak info --show-metadata` | Held packages, half-installed packages, packages from no repository with their version, every snap revision with its notes (old, classic, held), the snap refresh schedule, and Flatpak runtimes with whether an app uses them or they fill an extension point of an app or its runtime (the check never runs `flatpak uninstall`) |
| `package_verify`, `package_verify_run` | `nice -n 19 ionice -c 3 dpkg --verify`, `dpkg -S` | Every installed file against its package's checksum. `package_verify` only reports; `package_verify_run` starts the check if none runs (it reads every file and takes about a minute on a laptop). Each finding is changed, missing, or unreadable (needs admin rights to check), says whether it is a configuration file, and names its package |
| `security_detail` | `/proc/<pid>/attr/apparmor/current`, `/etc/apt/apt.conf.d`, `/var/log/unattended-upgrades/unattended-upgrades.log` | Running processes grouped by AppArmor profile and mode (enforce, complain, a profile that allows everything, or none) with counts and names; the `APT::Periodic` settings (later files win); whether unattended-upgrades is installed; and the last runs from its log that say what it did (installed, nothing to do, stopped on a metered connection, errors), newest first |
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
| `memory.swap_ins_per_s` | - | count | Pages swapped in per second (`/proc/vmstat` `pswpin`), from the second reading on |
| `memory.hard_faults_per_s` | - | count | Hard page faults per second (`pgmajfault`): memory that had to be read from disk |
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
| `power.on_mains` | - | count | 1 when a mains or USB charger is online, else 0; only on computers with a system battery (Optimize: on mains most of 14 days) |
| `net.rx`, `net.tx` | interface, or none for the total | bytes per second | Data received and sent |
| `disk.read`, `disk.write` | disk, or none for the total | bytes per second | Data read from and written to disks |
| `disk.iops` | disk, or none | count per second | Reads and writes completed |
| `disk.busy` | disk, or none (busiest disk) | percent | Time the disk had work to do |
| `disk.await` | disk, or none | seconds | Average time one read or write took |
| `pressure.cpu`, `pressure.memory`, `pressure.io` | - | percent | Share of time some work waited for CPU, memory, or disk ("some", 10-second average) |
| `pressure.memory_full`, `pressure.io_full` | - | percent | Share of time all work waited at once ("full") |
| `temp` | sensor, as `<chip>/<label>` | celsius | Every temperature sensor |
| `temp.cpu` | - | celsius | The CPU temperature |
| `cpu.throttle_total` | - | count | Times heat slowed the whole chip since start (Intel `package_throttle_count`); the app charts the change per interval |
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

