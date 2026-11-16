# Devices (`devices`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Machine · Icon: `usb` · Status badge: {ok:Healthy} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Ports & Devices, Displays, Audio
- Old links to `hardware/peripherals`, `display/displays`, and `display/audio` open the new place.

## Devices › Ports & Devices: `devices/ports`

It was Hardware › Peripherals until 2026-11-16.

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


Expert **PCIe links** shows chips by verdict (Full speed, Nothing connected, …), a green "All good" line or "Worth a look" with each link slower than it could be or with link errors, and every link under **Show all**.

## Devices › Displays: `devices/displays`

1. **Key-value list**: "Session" · span 4
- Source chip (Expert): `$XDG_SESSION_TYPE`

| Key | Value |
| --- | --- |
| Display server | Wayland |
| Compositor | Mutter (GNOME 46) |
| Fractional scaling | Off |

2. **Data table**: "Connected displays" · span 8

| Output | Resolution | Refresh | Scale | Primary | Status |
| --- | --- | --- | --- | --- | --- |
| `eDP-1` | 1920 × 1080 | 60 Hz | 100% | ✓ | {ok:Connected} |
| `HDMI-A-1` | - | - | - | - | {off:Disconnected} |

## Devices › Audio: `devices/audio`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Output | Speakers |  | - | Volume 45% | - | - |
| Input | Internal mic |  | - | Not muted | - | - |
| Sound server | PipeWire |  | - | 1.0.5 | - | - |
| Active streams | 1 |  | - | chrome | - | - |

2. **Data table**: "Output devices" · span 6
- Source chip (Expert): `pactl list sinks`

| Device | Volume | Default |
| --- | --- | --- |
| Laptop speakers | [meter:ok 45%] | ✓ |
| Wireless earbuds (A2DP) | [meter:ok 60%] | - |
| HDMI output | [meter:ok 100%] | - |

3. **Data table**: "Input devices" · span 6
- Source chip (Expert): `pactl list sources`

| Device | Level | Default |
| --- | --- | --- |
| Internal microphone | [meter:ok 70%] | ✓ |
| Earbuds microphone (HFP) | [meter:ok 50%] | - |

---

← Previous: [Power](07-power.md) · [Index](README.md) · Next: [Processes](09-processes.md) →
