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

## What is built (Phase 0)

The code is in `crates/deecure-agent/src/collect/`. Every file collector reads through a `SystemRoot`, so tests can point it at saved copies of `/proc` and `/sys` from real machines.

| Collector | Reads | How often | Notes |
| --- | --- | --- | --- |
| CPU | `/proc/stat`, `/proc/loadavg` | Every minute | Usage needs two readings, so it appears from the second run on |
| Memory | `/proc/meminfo` | Every minute | Swap metrics only when the system has swap |
| Filesystems | `/proc/mounts`, `statvfs` | Every minute | Local types only (ext4, btrfs, xfs, vfat, ntfs…). Network shares are skipped because checking them can hang. Snap images (squashfs) are skipped. One entry per device. |
| Battery | `/sys/class/power_supply/*` | Every minute | System batteries only. Mouse and keyboard batteries (`scope` = `Device`) are skipped. Missing values are left out, never guessed. |
| Services | systemd `ListUnits` over D-Bus | Every 5 minutes | `.service` units only |

The 2-second rate while the app is open comes with the app in Phase 1. The collectors above read standard Linux files, so they work on any distro. Distro adapters arrive with the package collectors.

## Metric names

| Name | Subject | Unit | Meaning |
| --- | --- | --- | --- |
| `cpu.usage` | — | percent | Time the CPU was busy (not idle, not waiting for disk) |
| `cpu.user` | — | percent | Time in apps |
| `cpu.system` | — | percent | Time in the kernel |
| `cpu.iowait` | — | percent | Time waiting for the disk |
| `cpu.load_1m` | — | count | 1-minute load average |
| `memory.used` | — | bytes | Memory in use (total minus available) |
| `memory.used_percent` | — | percent | Same, as a share of all memory |
| `swap.used` | — | bytes | Swap in use |
| `swap.used_percent` | — | percent | Same, as a share of all swap |
| `disk.used_percent` | mount point | percent | Space used, computed like `df` |
| `disk.free` | mount point | bytes | Space a normal user can still write |
| `battery.charge` | battery, such as `BAT0` | percent | Charge level |
| `battery.health` | battery | percent | Capacity now compared with when new |
| `battery.power` | battery | watts | Power going in or out right now |
| `services.running` | — | count | Services that are running |
| `services.failed` | — | count | Services that failed |

## Inventories (snapshots)

Stored only when they change, so they take little space.

| Kind | Contents |
| --- | --- |
| `filesystems` | Mount point, device, type, and size of each local filesystem |
| `services` | Name, description, load, active, and sub state, enabled-at-boot state (`UnitFileState`), and restarts since boot (`NRestarts`) of each service |

