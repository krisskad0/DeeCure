# Expert coverage

[Docs](../README.md) › [Product](README.md) › [View levels](view-levels.md)

**Status: approved by the owner on 2026-10-23. Being built step by step; see [pending work](../05-roadmap/pending.md).**

What Expert mode adds, page by page. Every item is read from a named source on this computer. Nothing is estimated unless the row says so.

- **Access:** "You" works with no password. "Admin" needs Full access and goes through the admin helper. "You (own)" reads your own processes without a password; other users' processes need Admin.
- **Today:** "New" is not read yet. "Partly" is read in part; the row says what is missing. "✅ Built" is in the app (Expert level).
- Sources were checked on a real Ubuntu 24.04 laptop on 2026-10-23. Items that exist only on some hardware say so. When a source is missing, the row shows "Not reported by this computer", never a blank or a guess.

## 1. Processor (Performance › CPU, Hardware › Processor)

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Clock speed of each core now, its minimum and maximum, and whether turbo is on | Shows if the CPU can reach full speed | `/sys/devices/system/cpu/cpu*/cpufreq`, `intel_pstate/no_turbo` or `cpufreq/boost` | You | ✅ Built |
| Speed policy of each core (governor and energy preference) | Explains a slow or hot CPU; the power mode sets it | `scaling_governor`, `energy_performance_preference` | You | ✅ Built |
| Heat slowdowns: how many times and how long each core was slowed by heat | Proves heat throttling instead of guessing | `/sys/devices/system/cpu/cpu*/thermal_throttle` (Intel) | You | ✅ Built |
| CPU sleep and wake-ups | Shown with the battery, where they matter: [7b. Sleep and wake-ups](#7b-sleep-and-wake-ups) | - | - | - |
| CPU power in watts: whole chip, cores, built-in graphics, memory | Measured power, not a model | `/sys/class/powercap/intel-rapl*` (Intel, and AMD Zen) | Admin | New |
| Interrupts per device per second | Finds a device that keeps the CPU busy | `/proc/interrupts`, `/proc/softirqs` | You | ✅ Built |
| Processes waiting for the disk | High load with low CPU use comes from these | process state `D` in `/proc/<pid>/stat` | You | ✅ Built |
| Each known CPU security flaw and whether it is fixed | Real exposure, flaw by flaw | `/sys/devices/system/cpu/vulnerabilities` | You | ✅ Built |
| Microcode: version loaded, and whether a newer installed one waits for a restart | Microcode fixes CPU bugs | `/proc/cpuinfo`, `intel-microcode` or `amd64-microcode` package version | You | ✅ Built |
| Power limits: long-term and short-term watts, and how long the short boost may last | Explains a CPU that is fast for 30 seconds, then slows | `/sys/class/powercap/intel-rapl:*/constraint_*` (readable here without a password) | You | ✅ Built |
| Cache sizes per level, and which cores share them | Real facts for comparing CPUs | `/sys/devices/system/cpu/cpu*/cache/index*` | You | ✅ Built |
| Preferred cores: which cores the firmware marks as the fastest | The system puts heavy work on them first | `/sys/devices/system/cpu/cpu*/acpi_cppc/highest_perf` | You | ✅ Built |
| CPU family, model, and stepping | Exact chip revision, for known bugs and firmware | `/proc/cpuinfo` | You | ✅ Built |

## 2. Memory (Performance › Memory, Hardware › Memory)

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Full breakdown: apps, file cache, shared memory, kernel caches (can be freed or not), page tables, kernel stacks, data waiting to be written | Where RAM really goes, kernel included | `/proc/meminfo` | You | ✅ Built |
| Real memory per process: shared memory split fairly (PSS) and memory only it uses (USS) | "Resident" counts shared memory many times; PSS does not | `/proc/<pid>/smaps_rollup` | You (own) | ✅ Built |
| Memory, peak memory, swap, and memory pressure per app and per service | An app's 40 processes count as one app | cgroup `memory.current`, `memory.peak`, `memory.swap.current`, `memory.pressure` | You | ✅ Built |
| Hard page faults per second, system-wide and per process | The real sign that RAM is too small | `/proc/vmstat` `pgmajfault`, `/proc/<pid>/stat` | You | ✅ Built |
| Swap in and swap out per second | Using swap is fine; swapping all the time is slow | `/proc/vmstat` `pswpin`, `pswpout` | You | ✅ Built |
| Compressed swap (zswap, zram): on or off, size, how much it saves | Shows what RAM compression gives | `/sys/module/zswap/parameters`, `/sys/block/zram*/mm_stat`; zswap totals need Admin | You | ✅ Built (zswap totals: Admin, later) |
| Out-of-memory guard (systemd-oomd): on or off, what it watches, what it stopped | Explains apps that closed by themselves | `oomctl`, journal | You | ✅ Built |
| Which processes the system would stop first when memory runs out | Shows who is at risk | `/proc/<pid>/oom_score` | You | ✅ Built |
| Memory error counts (corrected and uncorrected), when the memory controller reports them | Uncorrected errors mean failing RAM | `/sys/devices/system/edac/mc*` | You | ✅ Built |
| Huge pages: setting and use | Matters for databases and virtual machines | `/sys/kernel/mm/transparent_hugepage`, `/proc/meminfo` | You | ✅ Built |
| Each memory module: slot, size, type, rated speed against the speed it really runs at | Finds RAM running slower than it can | `dmidecode -t 17` through the helper | Admin | Partly: type, speed, and slots |
| Free slots, and the maximum memory the board supports | Know before you buy more | `dmidecode -t 16` through the helper | Admin | New |
| Dual-channel or single-channel, from the slots in use | Single-channel halves memory speed, and slows built-in graphics | module bank locations from `dmidecode` | Admin | New |
| Soldered or removable memory | Know if it can be upgraded at all | `dmidecode` form factor | Admin | New |

## 3. Drives (Performance › Disk I/O, Hardware › Storage Devices)

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Disk reads and writes per service | Finds the service that keeps the disk busy | `/sys/fs/cgroup/system.slice/*/io.stat` | You | ✅ Built |
| Disk reads and writes per app and process (brings back "Top I/O processes", measured) | Finds the app that keeps the disk busy | `/proc/<pid>/io` (`read_bytes`, `write_bytes`) | You (own) | ✅ Built (your processes; others: Admin, later) |
| SSD wear: share of rated life used, total data written, spare blocks left | SSDs wear out; this is the drive's own number | NVMe health log, SATA SMART (`smartctl -j`) | Admin | Partly: read once when asked |
| Wear trend, and life left at today's write rate (shown with how it was worked out) | Plan a replacement early | daily stored health readings | Admin | New |
| Media errors, unsafe shutdowns, power-on hours, the drive's error log | Early signs of a failing drive | `smartctl -j` | Admin | Partly: read once when asked |
| PCIe link: speed and lanes now, against the most the device and slot support | Finds an SSD or graphics card running at half speed | `/sys/bus/pci/devices/*/{current,max}_link_{speed,width}` | You | ✅ Built |
| NVMe power states and the power-saving setting | Explains SSD wake-up lag or battery drain | `/sys/class/nvme/*`, `nvme_core` parameters | You | ✅ Built (setting; power-state list: later) |
| Scheduler, queue size, and read-ahead per disk | A wrong scheduler on a hard disk costs speed | `/sys/block/*/queue/` | You | ✅ Built |
| TRIM: automatic or not, last run, `discard` mount option | SSDs slow down without TRIM | `fstrim.timer`, journal, `/proc/mounts` | You | ✅ Built |
| Response time per disk over time | Slow answers show before a failure | `/proc/diskstats` | You | Partly: one average |
| SATA link speed now against the most the drive supports | Finds a SATA SSD on a slow port or cable | `/sys/class/ata_link/*/sata_spd`, `sata_spd_limit` | You | ✅ Built |
| NVMe details: controller and namespace, block size (512 or 4096 bytes), firmware slots, warning and critical temperatures | Real facts from the drive, not a model lookup | `/sys/class/nvme/*`, `nvme id-ctrl` through the helper | You / Admin | ✅ Built (firmware slots and namespaces: Admin, later) |
| Drive self-test results, and starting a short or long self-test | The drive checks itself | `smartctl -l selftest`, `smartctl -t` | Admin | New |

## 4. Filesystems and space (Storage)

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Mount options of each filesystem | Shows `noatime`, `discard`, compression, read-only | `/proc/mounts` | You | ✅ Built |
| Errors the filesystem has recorded | ext4 and btrfs keep error counts | `/sys/fs/ext4/*/errors_count`; btrfs device stats need Admin | You | ✅ Built (ext4; btrfs: Admin, later) |
| Inodes used and left | A disk can be "full" while space is free | `statvfs` | You | ✅ Built |
| Space reserved for the admin | ext4 keeps 5% by default | `tune2fs -l` | Admin | New |
| Partition table and partition alignment | Misaligned partitions slow SSDs | `/sys/block/*/*/start`, `alignment_offset` | You | ✅ Built |
| Duplicate files in your home folder (only when you ask) | Exact: same size, then same content hash; never by name | file walk and SHA-256 | You | New |
| Folders that grew the most in the last 7 days | Finds a runaway log or cache | stored folder scans | You | New |

## 5. Graphics (Performance › GPU, Hardware › Graphics)

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Graphics use per app | Which app uses the graphics card | `/proc/<pid>/fdinfo` (`drm-engine-*`), Intel and AMD drivers | You (own) | ✅ Built |
| Graphics clock now and maximum, power in watts, time in power saving | Real speed and power | Intel `/sys/class/drm/card*/gt*`; AMD hwmon `power1_average` and `pp_dpm_sclk`; NVIDIA `nvidia-smi` when installed | You | ✅ Built (Intel, AMD; NVIDIA through nvidia-smi: later) |
| Second graphics card asleep or awake, and which app keeps it awake | A card that never sleeps costs hours of battery | `/sys/bus/pci/devices/*/power/runtime_status`, `fdinfo` | You | ✅ Built |
| Video decoding in hardware: formats the card supports, and whether the driver for it is installed | Without it, video runs on the CPU and drains the battery | `vainfo` for the formats (not installed on this test laptop; without it DeeCure shows only whether the driver is installed), VA-API driver packages | You | ✅ Built |
| Graphics stack: Mesa or NVIDIA version, OpenGL and Vulkan renderer | Shows software rendering used by mistake | `glxinfo -B`, `vulkaninfo --summary` (when installed) | You | ✅ Built (OpenGL; Vulkan: later) |
| Desktop session type: Wayland or X11 | Some apps behave differently on each | logind session `Type` | You | ✅ Built |
| Graphics firmware loaded (Intel GuC and HuC, AMD firmware versions) | Missing firmware turns off video encoding and power saving | journal, `/sys/kernel/debug/dri` through the helper | Admin | ✅ Built (from the kernel log, no admin needed) |
| Display outputs and which graphics card drives each | On laptops with two cards, shows which card the external screen uses | `/sys/class/drm/card*-*` | You | ✅ Built |

## 6. Network

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Traffic per app, download and upload (brings back "Traffic by application") | Exact for all traffic needs Admin. Without Admin: TCP only, labelled "TCP only" (QUIC and other UDP are not counted) | Admin: eBPF counters in the helper. You: TCP socket counters (`sock_diag`) | Admin, or You (TCP only) | ✅ Built (TCP only, labelled; exact with Admin: later) |
| Wi-Fi: signal and noise in dBm, link speed, band and channel, Wi-Fi generation, power saving | Explains slow Wi-Fi | NetworkManager (D-Bus), `/proc/net/wireless`, `iw` when installed | You | ✅ Built (Wi-Fi generation needs `iw`: later) |
| Errors, dropped packets, and TCP resends | Bad cable, weak Wi-Fi, or a busy network | `/sys/class/net/*/statistics`, `/proc/net/snmp` | You | ✅ Built |
| DNS: which server answers, DNSSEC and DNS-over-TLS state, answer time | Slow DNS feels like slow internet | systemd-resolved (D-Bus) | You | ✅ Built (answer time: Diagnostics › Network Test) |
| Routes: which connection carries internet traffic, VPN routes | Explains "VPN on but traffic goes outside" | `ip -j route` | You | ✅ Built |

## 7. Power and battery (Power)

Power gets three Expert sections: **Battery**, **Sleep and wake-ups**, and **Power controls** (section 7c). Everything here is checked against the hardware first: a control the laptop does not support is not shown.

### 7a. Battery

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Charge or drain in watts, now and over time | The real drain, minute by minute | `/sys/class/power_supply/BAT*` `power_now` (or current times voltage) | You | ✅ Built |
| Drain per hour, by day, with the screen on and while suspended | Shows if sleep drains the battery | stored readings | You | ✅ Built (drain per suspend; per day: later) |
| Charge limits in use (start and stop), and the range the laptop allows | Know if the battery is kept full all the time | `charge_control_start_threshold`, `charge_control_end_threshold` | You | ✅ Built (the allowed range is not reported by the kernel) |
| Charge behaviour (normal, hold charge, discharge on mains), when the laptop offers it | Some laptops can pause charging | `charge_behaviour` | You | ✅ Built |
| Charge mode, when the laptop offers it (for example Standard, Adaptive, Fast, Trickle, Custom on Dell) | Fast charging wears the battery more; Adaptive learns your use | `charge_types` (this test laptop offers all 5 and uses Custom) | You | ✅ Built |
| Battery voltage now against its design minimum, manufacture date, serial | Low voltage under load warns of a worn battery | `voltage_now`, `voltage_min_design`, `manufacture_*`, `serial_number` | You | ✅ Built |
| Charger: connected or not, its watts, and USB-C Power Delivery details | A weak charger charges slowly or not at all under load | `/sys/class/power_supply/ucsi-source-psy-*`, `/sys/class/usb_power_delivery/*/source-capabilities` | You | ✅ Built |
| Time on mains power against time on battery, last 14 days | Decides if a charge limit makes sense | stored readings of `/sys/class/power_supply/AC*/online` | You | New |
| Measured chip power (section 1) next to battery drain | Splits CPU power from the rest of the laptop | powercap | Admin | New |
| **Not shown: watts per app.** | Linux has no exact per-app power meter. Expert shows each app's CPU time, graphics time, and wake-ups instead | - | - | - |

### 7b. Sleep and wake-ups

Two different things, shown apart:

- **Wake-ups while you work:** each one stops the CPU from resting, which costs battery.
- **Wake-ups from suspend:** a device or timer that wakes the laptop while the lid is closed.

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Whole-chip deep sleep: share of time the chip reached each sleep level | The chip, not one core, decides the battery drain; this is the number that matters | Intel `pmc_core` (`/sys/kernel/debug/pmc_core`), package C-state counters | Admin | New |
| Time each core spends in each sleep state | Shows which cores never rest | `/sys/devices/system/cpu/cpu*/cpuidle/state*` | You | ✅ Built |
| What blocks deep sleep: devices with power saving off, PCIe links with power saving off | The exact reasons, device by device | `/sys/bus/{pci,usb}/devices/*/power/*`, `/sys/bus/pci/devices/*/link/*aspm*` when the kernel exposes it | You | ✅ Built (device power saving; PCIe link power saving: later) |
| Wake-ups per second, per process | Finds the app that keeps waking the CPU | kernel tracing (`sched_wakeup`) through the helper | Admin | New |
| Interrupts per second, per device | Finds the device that keeps waking the CPU | `/proc/interrupts` | You | ✅ Built (Performance › CPU) |
| What woke the computer from suspend, with how often each source fired | Explains a laptop that wakes in the bag | `/sys/class/wakeup/*`, journal | You | ✅ Built (last wake interrupt; per-source counts) |
| Devices allowed to wake the computer from suspend | Turn off the ones that should not (7c) | `/sys/devices/*/power/wakeup`, `/proc/acpi/wakeup` | You | ✅ Built |
| Suspend type in use and the types offered (light `s2idle` or deep `deep`) | Deep sleep drains less with the lid closed, when the laptop offers it | `/sys/power/mem_sleep` | You | ✅ Built |
| Time really spent in low power during each suspend | A light sleep that never reaches low power drains the battery | `/sys/kernel/debug/pmc_core` (Intel), journal | Admin | New |
| Devices that can save power but do not | Each one is a measured fix, not a general tip | `/sys/bus/{pci,usb}/devices/*/power/{control,runtime_status}` | You | ✅ Built |

### 7c. Power controls

Direct controls, not only suggestions. Each shows its current value, applies at once, and has undo. "Survives restart" says what DeeCure does so a setting is not lost at the next boot: some laptops keep a setting in firmware, others forget it, so DeeCure checks after the next boot and sets it again with a small boot task if needed.

| Control | Changes | Shown only when | Survives restart | Access | Today |
| --- | --- | --- | --- | --- | --- |
| Battery charge limits: presets (Full 100%, Balanced 90%, Long life 80%) or your own start and stop values | `charge_control_start_threshold`, `charge_control_end_threshold` | The battery offers them (this test laptop offers both) | Yes (checked after the next boot) | Admin | New |
| Charge mode: Standard, Adaptive, Fast, Trickle, or Custom (with the limits above) | `charge_types` | The laptop offers it (this test laptop does) | Yes (kept by the firmware) | Admin | New |
| Keyboard backlight brightness | UPower `KbdBacklight` (D-Bus) | The keyboard has a backlight (this test laptop does, 2 levels) | Yes | You | New |
| Radios on or off: Wi-Fi, Bluetooth, mobile | rfkill | Always; a radio turned off by a hardware switch says so and cannot be turned on from software | Yes | You | New |
| Charge behaviour: normal, hold the charge, or discharge on mains | `charge_behaviour` | The laptop offers it | No: back to normal at restart, on purpose | Admin | New |
| Firmware power profile (for example cool, quiet, balanced, performance) | `/sys/firmware/acpi/platform_profile` | The firmware offers it (this test laptop offers 4). Changing the power mode also changes it; DeeCure says so | Set by the power mode | Admin | New |
| Turbo boost on or off, separately on battery and on mains | `intel_pstate/no_turbo` or `cpufreq/boost` | The CPU has turbo | Yes | Admin | New |
| Highest CPU speed on battery | `scaling_max_freq`, applied when the power cable is pulled and lifted when it is plugged in | Always (cpufreq present) | Yes | Admin | New |
| Stop a device from waking the computer from suspend | `power/wakeup`, `/proc/acpi/wakeup` | The device can wake the computer. Keyboard, lid, and power button stay on and are not offered | Yes | Admin | New |
| Power saving per device (PCI and USB) | `power/control` = `auto` | The device supports it and it is off. Keyboards, mice, and sound cards are not offered | Yes | Admin | New |
| Wi-Fi power saving | NetworkManager `wifi.powersave` for the current network | Wi-Fi is in use | Yes | You | New |
| PCIe power saving policy | `pcie_aspm` policy | The kernel allows it. Some hardware freezes with it, so it is applied for this session only until you choose **Keep** | Only after **Keep** | Admin | New |
| Suspend type: light or deep | `/sys/power/mem_sleep`, and the boot option `mem_sleep_default` to keep it | The laptop offers `deep` (this test laptop offers only `s2idle`, so it is hidden here). Test one suspend before keeping | Only after **Keep** | Admin | New |
| What closing the lid does, on battery and on mains | logind `HandleLidSwitch`, `HandleLidSwitchExternalPower` | Always | Yes | Admin | Partly: shown only |
| Power mode | power-profiles-daemon | Always | Yes | You | Exists |
| Power off cleanly at a low battery level | UPower | Always | Yes | Admin | Exists |

**Controlling wake-ups while you work, honestly:** Linux cannot block an app's wake-ups. What DeeCure offers for each top waker: end it, turn off its startup entry or service, or, for a device, turn on its power saving. Each uses the controls above or an existing fix.

## 8. Temperature and fans (Hardware › Sensors)

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Each thermal zone with the temperatures where the system slows down and where it shuts down | Shows how close the computer is to its limits | `/sys/class/thermal/thermal_zone*/trip_point_*` | You | ✅ Built |
| Cooling devices (fans, CPU limits) and their current level | Shows what the system does to cool down | `/sys/class/thermal/cooling_device*` | You | ✅ Built |
| Heat slowdowns on the same chart as temperature | Proves cause and effect | throttle counters and temperature history | You | ✅ Built |

## 9. Processes (detail drawer on each row)

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Process tree, and what started it (app, service, terminal) | Find where a process comes from | `/proc/<pid>/stat` parent, `/proc/<pid>/cgroup` | You | ✅ Built |
| Program file, the package it belongs to, working folder | Know what it really is | `/proc/<pid>/exe`, `cwd`, dpkg file lists | You (own) | ✅ Built |
| Priority, scheduling class, disk priority, which cores it may use | Explains why it is slow or fast | `/proc/<pid>/stat`, `sched`, `ioprio_get`, `Cpus_allowed_list` | You | ✅ Built |
| Open files against its limit, and its network connections | Finds leaks before they crash the app | `/proc/<pid>/fd`, `/proc/<pid>/limits` | You (own) | ✅ Built (connection count; the list is on Network › Connections) |
| Threads with their own CPU use | Finds the busy thread | `/proc/<pid>/task` | You | ✅ Built |
| Context switches, voluntary and forced | Too many forced ones mean CPU contention | `/proc/<pid>/status` | You | ✅ Built |
| Sandbox and security: AppArmor profile, capabilities, seccomp, container | Shows how confined it is | `/proc/<pid>/attr/apparmor/current`, `status`, `/proc/<pid>/ns` | You | ✅ Built |
| Disk, memory (PSS), graphics, and wake-ups for this process | All its costs in one place | sections 2, 3, 5, 7 | You (own) | ✅ Built (wake-ups: Admin, later) |

## 10. Services and startup (Services, System › Boot)

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| CPU time, memory, peak memory, disk I/O, tasks, and restarts per service | Cost of each service | cgroup files, systemd (D-Bus) | You | Partly: memory and restarts |
| How locked down each service is (exposure score) | Finds services with too many rights | `systemd-analyze security` | You | New |
| Unit file, local overrides, and what turns it on | Finds local changes and why a service runs | systemd `FragmentPath`, `DropInPaths`, `WantedBy` | You | New |
| What each service waits for at boot (critical chain) | Shows what really delays the desktop | `systemd-analyze critical-chain` | You | Partly: 15 slowest units |
| Boot time of every unit, and the boot time trend | See if boots get slower | systemd timestamps, stored boots | You | Partly: last boot |
| Services that restart in a loop, masked services | Hidden problems | systemd `NRestarts`, unit state | You | Partly |

## 11. Kernel and system limits (System › Kernel)

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Kernel settings changed on this computer, with the file and line that set each | Only what was changed, not 1,000 values | `/proc/sys`, `/etc/sysctl.d`, `/etc/sysctl.conf`, `/usr/lib/sysctl.d` | You | New |
| File-watch limit: used and maximum | Editors and sync apps fail when it runs out | `/proc/sys/fs/inotify/*`, `/proc/*/fdinfo` | You (own) | New |
| Open files, system-wide: used and maximum | Prevents "too many open files" | `/proc/sys/fs/file-nr` | You | New |
| Kernel modules: which device uses each, and modules from outside Ubuntu | Find what a module is for | `/sys/module/*/holders`, `/sys/bus/*/drivers`, taint | You | Partly: list of modules |
| Extra drivers built locally (DKMS) and whether each is built for every installed kernel | A missing build breaks Wi-Fi or NVIDIA after an update | `dkms status`, `/lib/modules/*/updates/dkms` | You | New |
| Kernel lockdown mode | Shows if the kernel blocks unsigned changes | `/sys/kernel/security/lockdown` | You | New |
| Time sync: server, offset, last sync | Wrong time breaks sign-ins and updates | systemd-timesyncd (D-Bus) | You | Partly: synced or not |

## 12. Hardware and firmware (Hardware)

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Firmware security level, and each check behind it | One clear level instead of many settings | fwupd `HostSecurityAttrs` (D-Bus) | You | New |
| UEFI boot entries and boot order | Find old or broken boot entries | `/sys/firmware/efi/efivars` | You | New |
| Secure Boot keys you added (MOK) | Needed for NVIDIA and DKMS drivers | `mokutil --list-enrolled` | You | Partly: Secure Boot on or off |
| USB devices: speed (USB 2 or 3), power drawn, driver | Finds a fast device on a slow port | `/sys/bus/usb/devices/*/{speed,bMaxPower}` | You | Partly: list of devices |
| Virtualization: CPU support, turned on in firmware, KVM ready | Why virtual machines are slow or do not start | `/proc/cpuinfo` (`vmx`, `svm`), `/dev/kvm` | You | New |
| Every BIOS setting with its current value and allowed values (122 on this test laptop: battery charge, sleep, camera, virtualization, boot, and more) | See firmware settings without restarting into the BIOS | `/sys/class/firmware-attributes/*` (Dell, Lenovo, HP) through the helper | Admin | New |
| Whether a BIOS admin password is set | Changing BIOS settings from Linux needs it | `/sys/class/firmware-attributes/*/authentication` | Admin | New |
| Management engine firmware version (Intel ME or AMD PSP) | Known security flaws depend on this version | `/sys/class/mei/mei*/fw_ver` | You | New |
| TPM: version, maker, firmware version, and whether disk encryption uses it | Know if the TPM is in use and up to date | `/sys/class/tpm/tpm*`, `systemd-cryptenroll` | You / Admin | Partly: TPM present |
| Protection against attacks through ports (DMA protection, IOMMU) | Thunderbolt and USB4 ports can reach memory without it | `/sys/bus/thunderbolt/devices/domain*/iommu_dma_protection`, `/sys/class/iommu` | You | New |
| Hardware clock: its time against system time, and whether it keeps local time | A drifting clock breaks dual boot and sign-ins | `/sys/class/rtc/rtc0`, `/etc/adjtime` | You | Partly: local time or UTC |

**Changing BIOS settings from Linux** is not in the first version. Reading them is safe; a wrong change can stop the computer from starting. A short list of safe changes (for example the battery charge settings) can come later, each with its own review.

## 13. Software (Packages)

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Why each package is here: installed by hand or as a dependency, by which command, and when | Clean up with confidence | `/var/lib/apt/extended_states`, apt history | You | Partly: by hand or not |
| What it needs, and what needs it | Know what breaks if it goes | `/var/lib/dpkg/status` | You | New |
| Packages from no repository, held packages, half-installed or broken packages | Hidden causes of failed updates | `apt-cache policy`, `apt-mark showhold`, `dpkg --audit` | You | New |
| Changed or damaged package files: every file checked against its package checksum | The full file check that Security › File Integrity waits for | `dpkg --verify`; files only the admin can read need Admin | You and Admin | New |
| Snap: revision, sandbox type, refresh schedule, held, disk per revision | Control snap updates and space | snapd (REST API) | You | Partly |
| Flatpak runtimes that no app uses | Free space safely | `flatpak list --runtime` | You | New |

## 14. Security

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| AppArmor state of every running process: enforced, warn-only, or not confined | See which apps run without a sandbox | `/proc/<pid>/attr/apparmor/current` | You | New |
| Automatic security updates: on or off, last run, result | Know that fixes really install | `/etc/apt/apt.conf.d/20auto-upgrades`, unattended-upgrades log | You | Partly: shown in history |
| Who may use sudo, and for what | Find too many admin rights | `/etc/sudoers`, `/etc/sudoers.d` | Admin | Partly: admin group |
| SSH server settings that weaken it | Real settings, not the config file guess | `sshd -T` | Admin | Partly: config file |

## 15. Containers and virtual machines (Containers)

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Disk and network use per container | Find the busy container | `docker stats` | You | Partly: CPU and memory |
| Build cache and unused images, with sizes | Free space safely | `docker system df -v` | You | Partly |

## 16. Ports and USB-C (Hardware › Peripherals)

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Each USB-C port: what is plugged in, power direction, data direction, USB and USB4 version | Know what each port can do | `/sys/class/typec/port*` | You | New |
| What a USB-C device or dock offers: power, display (DisplayPort mode), and data | Explains "the dock charges but the screen stays black" | `/sys/class/typec/port*-partner`, alternate modes | You | New |
| Power Delivery: what the charger offers (volts and amps, PPS) and what the laptop takes | A 45 W charger on a 65 W laptop charges slowly | `/sys/class/usb_power_delivery/*` | You | New |
| Thunderbolt and USB4: security level, connected devices, approved or not | Unapproved devices do not work, by design | `boltctl`, `/sys/bus/thunderbolt` | You | New |
| USB tree: hubs, ports, speed per device, power drawn | Finds a slow hub or a device drawing too much | `/sys/bus/usb/devices/*` | You | Partly: device list |

## 17. Built-in devices (Hardware › Peripherals, Privacy)

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Camera: model, resolutions and frame rates it supports, in use or not | Real camera facts, and whether an app has it open | `/sys/class/video4linux`, V4L2 queries | You | Partly: in use |
| Fingerprint reader: supported by Ubuntu or not, fingers enrolled for you | Know if it can be used for sign-in | fprintd (D-Bus) | You | New |
| Touchpad, touchscreen, pen: model and driver | Find the right settings and drivers | `/proc/bus/input/devices`, libinput properties | You | Partly: input list |
| Wi-Fi chip: model, driver, firmware version, bands and Wi-Fi generations it supports | Know what the chip can do, against what it uses now (section 6) | `ethtool -i`, `/sys/class/ieee80211/*`, `iw phy` when installed | You | Partly: model |
| Ethernet: negotiated speed and duplex, energy-saving Ethernet, Wake-on-LAN | Finds a gigabit port running at 100 Mb/s | `ethtool` | You | Partly: link speed |
| Bluetooth adapter: version, maker, and whether a radio switch has it off | Explains "Bluetooth not found" (it is switched off on this test laptop) | BlueZ (D-Bus), rfkill | You | Partly: adapter count and power |
| Batteries of mice, keyboards, headsets | All batteries in one place | UPower devices, BlueZ battery | You | Partly: Bluetooth only |
| Printers: queue, state, last error | Find a stuck print job | CUPS (`lpstat`) | You | New |

## 18. Display and sound hardware (Display & Audio)

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| Each screen's maker, model, manufacture year, physical size, and pixel density | Real panel facts from the screen itself | EDID (`/sys/class/drm/*/edid`) | You | Partly: name and resolution |
| Supported refresh rates, variable refresh (VRR), HDR, colour depth | Know what the screen can do, against what is set | EDID, GNOME Mutter (D-Bus) | You | Partly: refresh rate in use |
| Screen brightness: level now, and the hardware range | Real level, not the slider position | `/sys/class/backlight/*` | You | New |
| Sound chip (codec) and its driver and firmware | Explains missing speakers or microphones | `/proc/asound/card*/codec#*`, journal | You | New |
| Sample rate and format in use per output | Know if audio is resampled | PipeWire (`pw-dump`) | You | Partly: devices and volume |

## 19. Hardware errors (Hardware › Summary, one panel)

One place for every error the hardware itself reports. Each row says how many times, when last, and what it means.

| What Expert shows | Why it helps | Read from | Access | Today |
| --- | --- | --- | --- | --- |
| CPU and memory hardware errors (machine checks) | The CPU reports real hardware faults | journal `mce` lines, `/sys/devices/system/machinecheck` | You | Partly: in the error log |
| Memory errors (ECC) | See section 2 | `/sys/devices/system/edac/mc*` | You | New |
| PCIe link errors per device (corrected, non-fatal, fatal) | A bad link shows here before a crash | `/sys/bus/pci/devices/*/aer_dev_*` (readable here without a password) | You | ✅ Built (Hardware › Peripherals) |
| Firmware that failed to load | A device works badly or not at all without it | journal `Direct firmware load ... failed` | You | Partly: in the error log |
| Driver and firmware crash reports | Wi-Fi and graphics firmware crashes leave a report | `/sys/class/devcoredump`, journal | You | New |
| USB errors (device not accepting address, descriptor errors) | Bad cable, weak port, or a failing device | kernel log | You | Partly: in the kernel log |
| Disk link errors (SATA and NVMe resets, timeouts) | Bad cable or a failing drive | kernel log, NVMe error log | You / Admin | Partly: in the kernel log |
| Devices with no driver, or with a fallback driver (for example a basic display driver instead of the real one) | Hardware that runs slow or not at all | `/sys/bus/*/devices/*/driver`, `ubuntu-drivers devices` | You | Partly: devices with no driver |

## How this stays light and safe

- Expert reads run only while their page is open, except a few small daily readings for trends (drive wear, battery drain, boot time).
- The duplicate-file scan, the full package file check, and wake-up tracing run only when you ask, and can be stopped.
- New admin reads (power meters, wake-up tracing, eBPF traffic counters, sudo rules, `sshd -T`, low-power residency, BIOS settings, memory slots, NVMe controller details, graphics firmware state, drive self-tests) are added to the admin helper one by one, each with its own security review. eBPF is the biggest change to the helper and comes last.
- `make perf` keeps checking the agent's idle CPU and memory after every change.

---

[View levels](view-levels.md) · [Optimize](optimize-catalog.md)
