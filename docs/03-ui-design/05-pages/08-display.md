# Display & Audio (`display`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: System · Icon: `mon` · Status badge: {ok:Healthy} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Displays, Audio

## Display & Audio › Displays — `display/displays`

1. **Key-value list** — "Session" · span 4
- Source chip (Expert): `$XDG_SESSION_TYPE`

| Key | Value |
| --- | --- |
| Display server | Wayland |
| Compositor | Mutter (GNOME 46) |
| Fractional scaling | Off |

2. **Data table** — "Connected displays" · span 8

| Output | Resolution | Refresh | Scale | Primary | Status |
| --- | --- | --- | --- | --- | --- |
| `eDP-1` | 1920 × 1080 | 60 Hz | 100% | ✓ | {ok:Connected} |
| `HDMI-A-1` | — | — | — | — | {off:Disconnected} |

## Display & Audio › Audio — `display/audio`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Output | Speakers |  | — | Volume 45% | — | — |
| Input | Internal mic |  | — | Not muted | — | — |
| Sound server | PipeWire |  | — | 1.0.5 | — | — |
| Active streams | 1 |  | — | chrome | — | — |

2. **Data table** — "Output devices" · span 6
- Source chip (Expert): `pactl list sinks`

| Device | Volume | Default |
| --- | --- | --- |
| Laptop speakers | [meter:ok 45%] | ✓ |
| Wireless earbuds (A2DP) | [meter:ok 60%] | — |
| HDMI output | [meter:ok 100%] | — |

3. **Data table** — "Input devices" · span 6
- Source chip (Expert): `pactl list sources`

| Device | Level | Default |
| --- | --- | --- |
| Internal microphone | [meter:ok 70%] | ✓ |
| Earbuds microphone (HFP) | [meter:ok 50%] | — |

---

← Previous: [Power](07-power.md) · [Index](README.md) · Next: [Processes](09-processes.md) →
