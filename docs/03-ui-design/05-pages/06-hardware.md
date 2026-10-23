# Hardware (`hardware`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: System · Icon: `chip` · Status badge: {ok:Healthy} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Summary, Processor, Memory, Storage Devices, Graphics, Sensors, Peripherals, Firmware

## Hardware › Summary: `hardware/summary`

1. **Key-value list**: "Machine" · span 6
- Source chip (Expert): `/sys/class/dmi/id`

| Key | Value |
| --- | --- |
| Vendor | Lenovo |
| Model | IdeaPad 5 14IAL7 |
| Chassis | Laptop |
| Serial number | `PF3K9X2M` |
| BIOS | JHCN52WW · 11 Mar 2024 |
| Secure Boot | {ok:Enabled} |

2. **Status list**: "Components" · span 6

| Icon | Label | Metric | Badge | Opens |
| --- | --- | --- | --- | --- |
| `chip` | Processor | i5-1235U · 54 °C | {ok:Healthy} | `hardware/processor` |
| `layers` | Memory | 8 GB LPDDR4X | {ok:Healthy} | `hardware/memory` |
| `db` | Storage | NVMe 256 GB · SMART passed | {ok:Healthy} | `hardware/storage` |
| `bat` | Battery | 71% of design | {warn:Degraded} | `power/battery` |
| `mon` | Graphics | Iris Xe · i915 | {ok:Healthy} | `hardware/graphics` |
| `fan` | Fans | Not reported | {off:Unknown} | `hardware/sensors` |
| `usb` | Peripherals | 11 devices · 0 missing drivers | {crit:1 unstable} | `hardware/peripherals` |
| `bolt` | Firmware | 1 update available | {info:Update} | `hardware/firmware` |

## Hardware › Processor: `hardware/processor`

1. **Gauge group**: "Live" · span 6

| Caption | Value | Max | Center text | Subtitle |
| --- | --- | --- | --- | --- |
| Load | 12 | 100 | 12% | utilization |
| Temperature | 54 | 100 | 54 °C | package |
| Frequency | 1.3 | 4.4 | 1.3 GHz | of 4.4 GHz |

2. **Key-value list**: "Specifications" · span 6
- Source chip (Expert): `lscpu`

| Key | Value |
| --- | --- |
| Model | 12th Gen Intel Core i5-1235U |
| Cores | 10 (2 performance + 8 efficient) |
| Threads | 12 |
| Base / max frequency | 1.3 / 4.4 GHz |
| L3 cache | 12 MB |
| Microcode | `0x4123` |
| Features | ‹VT-x›‹AES-NI›‹AVX2›‹SHA› |

3. **Data table**: "Cores" · span 12
- Source chip (Expert): `/sys/devices/system/cpu`

| Core | Type | Frequency [n] | Utilization [n] | Temperature [n] |
| --- | --- | --- | --- | --- |
| P0 | Performance · 2 threads | 1640 MHz | [meter 16%] | 56 °C |
| P1 | Performance · 2 threads | 1320 MHz | [meter 10%] | 55 °C |
| E0 | Efficient | 1180 MHz | [meter 22%] | 52 °C |
| E1 | Efficient | 980 MHz | [meter 11%] | 52 °C |
| E2 | Efficient | 1210 MHz | [meter 4%] | 52 °C |
| E3 | Efficient | 1050 MHz | [meter 7%] | 52 °C |
| E4 | Efficient | 1400 MHz | [meter 16%] | 53 °C |
| E5 | Efficient | 920 MHz | [meter 3%] | 53 °C |
| E6 | Efficient | 1100 MHz | [meter 8%] | 53 °C |
| E7 | Efficient | 1010 MHz | [meter 5%] | 53 °C |

## Hardware › Memory: `hardware/memory`

1. **Key-value list**: "Modules" · span 6
- Source chip (Expert): `dmidecode -t memory`

| Key | Value |
| --- | --- |
| Total | 8 GB (7.6 GB usable) |
| Type | LPDDR4X · 4266 MT/s |
| Form factor | Soldered on board |
| Slots | 0 free · not upgradeable |
| ECC | No |

2. **Checklist**: "Health" · span 6

| State | Title | Detail | Action |
| --- | --- | --- | --- |
| `pass` | No memory pressure stalls above 20% today |   |   |
| `warn` | 1 out-of-memory kill in 7 days | Mon 16:12 · chrome renderer | [View](go:logs/kernel) |
| `na` | Error correction (ECC) reporting | Not available on this memory type |   |
| `na` | Full memory test | Needs a reboot into memtest86+ | [How to run](act:toast:Restart → hold Shift → Memory test (memtest86+).) |

## Hardware › Storage Devices: `hardware/storage`

1. **Data table**: "Disks" · span 12
- Source chip (Expert): `lsblk · smartctl`

| Device | Model | Type | Size [n] | Interface | SMART | Life left [n] | Temperature [n] |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `nvme0n1` | Samsung PM991a | SSD (NVMe) | 256 GB | PCIe 3.0 x4 | {ok:Passed} | [meter:ok 88%] | 38 °C |

2. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Percentage used | 12 | % | - | of rated endurance | - | - |
| Data written | 18.4 | TB | - |   | - | - |
| Power-on hours | 6,120 | h | - |   | - | - |
| Unsafe shutdowns | 41 |  | - | +1 today | - | - |
| Media errors | 0 |  | - |   | - | - |
| Spare capacity | 100 | % | - |   | - | - |

3. **Line chart**: "Drive temperature" · span 12
- Source chip (Expert): `nvme smart-log`
- Height 180px · Y axis 20 to 70 · unit "°C"
- Series: NVMe (slot 1, gen[61, 37, 3, 2, 30, 70, 38])
- X axis: time (current range)

## Hardware › Graphics: `hardware/graphics`

1. **Key-value list**: "Adapter" · span 6
- Source chip (Expert): `lspci -k · glxinfo -B`

| Key | Value |
| --- | --- |
| GPU | Intel Iris Xe (Alder Lake-UP3 GT2) |
| Kernel driver | `i915` |
| OpenGL | Mesa 24.0.9 · 4.6 |
| Vulkan | 1.3 |
| Video decode | VA-API (iHD) |

2. **Checklist**: "Driver checks" · span 6

| State | Title | Detail | Action |
| --- | --- | --- | --- |
| `pass` | Recommended driver installed | ubuntu-drivers reports no alternatives |   |
| `pass` | Hardware acceleration active |   |   |
| `na` | GPU temperature | Not exposed by i915 |   |

## Hardware › Sensors: `hardware/sensors`

1. **Gauge group**: "Temperatures" · span 12
- Source chip (Expert): `sensors · /sys/class/hwmon`

| Caption | Value | Max | Center text | Subtitle |
| --- | --- | --- | --- | --- |
| coretemp | 54 | 100 | 54 °C | CPU package |
| coretemp | 56 | 100 | 56 °C | core max |
| nvme | 38 | 85 | 38 °C | NVMe |
| BAT0 | 31 | 60 | 31 °C | battery |
| iwlwifi | 42 | 100 | 42 °C | Wi-Fi |
| acpitz | 50 | 100 | 50 °C | ACPI zone |

2. **Line chart**: "Temperature history" · span 12
- Source chip (Expert): `sensors`
- Height 220px · Y axis 20 to 100 · unit "°C"
- Series: CPU (slot 1, gen[21, 52, 6, 3, 35, 92, 54]); NVMe (slot 2, gen[61, 37, 3, 2, 30, 70, 38]); Battery (slot 3, gen[71, 31, 2, 1, 25, 45, 31])
- X axis: time (current range)
- Threshold line at 95: "Throttle 95 °C"

3. **Alert banner**: "Fan speed not reported" · span 12
- State: `info` · Title: "Fan speed not reported" · Text: "This laptop’s embedded controller does not expose fan RPM to Linux."

## Hardware › Peripherals: `hardware/peripherals`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| USB devices | 3 |  | - |   | - | - |
| PCI devices | 18 |  | - |   | - | - |
| Bluetooth | 2 |  | - | 1 connected | - | - |
| Missing drivers | 0 |  | - |   | - | - |

2. **Data table**: "Devices" · span 12
- Source chip (Expert): `lsusb · lspci -k · bluetoothctl`
- Search: "Search devices"
- Filter chips on column 2: All / USB / PCI / Bluetooth / Input

| Device | Bus | Vendor | Driver | Status | Row action |
| --- | --- | --- | --- | --- | --- |
| USB mouse | USB | Logitech | `usbhid` | {ok:Connected} |  |
| Integrated camera | USB | Chicony | `uvcvideo` | {ok:Idle} |  |
| Bluetooth adapter | USB | Intel | `btusb` | {crit:Unstable} | `bt` |
| Wi-Fi 6 AX201 | PCI | Intel | `iwlwifi` | {ok:Active} |  |
| NVMe controller | PCI | Samsung | `nvme` | {ok:Active} |  |
| Smart Sound audio | PCI | Intel | `snd_hda_intel` | {ok:Active} |  |
| Thunderbolt 4 controller | PCI | Intel | `thunderbolt` | {ok:Active} |  |
| Wireless earbuds | Bluetooth | - | `bluez` | {warn:Reconnecting} |  |
| Phone | Bluetooth | - | `bluez` | {off:Paired} |  |
| Touchpad | Input | ELAN | `i2c_hid` | {ok:Active} |  |
| Keyboard | Input | AT Translated | `atkbd` | {ok:Active} |  |

## Hardware › Firmware: `hardware/firmware`

1. **Key-value list**: "Platform firmware" · span 5
- Source chip (Expert): `/sys/class/dmi/id · mokutil`

| Key | Value |
| --- | --- |
| BIOS vendor | LENOVO |
| Version | `JHCN52WW` |
| Release date | 11 Mar 2024 |
| Boot mode | UEFI |
| Secure Boot | {ok:Enabled} |
| TPM | 2.0 · present |

2. **Data table**: "Firmware updates" · span 7
- Source chip (Expert): `fwupdmgr get-updates`

| Device | Current | Available | Vendor | (action) |
| --- | --- | --- | --- | --- |
| Touchpad | `1.4` | `1.6` | Synaptics | [**Update**](act:toast:Firmware update scheduled for next reboot.) |
| System firmware | `JHCN52WW` | - | Lenovo | Up to date |
| NVMe SSD | `3L1QFXV7` | - | Samsung | Not on LVFS |

---

← Previous: [System](05-system.md) · [Index](README.md) · Next: [Power](07-power.md) →
