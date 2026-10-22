# Power (`power`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: System · Icon: `bat` · Status badge: {warn:Warning} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Battery, Power Profile, Sleep

## Power › Battery — `power/battery`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Charge | 64 | % | — | Discharging | Usage bar 64% | — |
| Health | 71 | % | {warn:Warning} | of design capacity | — | — |
| Time remaining | 3 h 10 m |  | — | Estimate | — | — |
| Power draw | 8.4 | W | — |   | Sparkline slot 2, gen[81, 8, 3, 2, 3, 25, 8.4] | — |
| Capacity | 40.5 | Wh | — | Design 57 Wh | — | — |
| Cycle count | N/A |  | — | Not reported | — | — |

2. **Line chart** — "Charge level" · span 8
- Source chip (Expert): `upower history`
- Height 220px · Y axis 0 to 100 · unit "%"
- Series: Charge (slot 1, gen[91, 60, 30, 8, 1, 100, 64])
- X axis: time (current range)
- Threshold line at 10: "Low 10%"

3. **Gauge** — "Battery health" · span 4
- Info tooltip: "How much charge the battery holds compared with when it was new."
- Value 71 of 100 · arc `warn` · center "71%" · subtitle "40.5 of 57 Wh" · caption below: "Replace below 60%"

4. **Line chart** — "Health trend" · span 6
- Subtitle: "12 months"
- Height 180px · Y axis 50 to 100 · unit "%"
- Series: Health (slot 1, data [83,82,81,80,79,78,77,76,75,74,72,71])
- X labels: Nov, Dec, Jan, Feb, Mar, Apr, May, Jun, Jul, Aug, Sep, Oct

5. **Data table** — "Top power consumers" · span 6
- Subtitle: "estimate"
- Source chip (Expert): `powertop`
- Empty text: "Requires Full access"

| Process | Power [n] | Share [n] |
| --- | --- | --- |
| chrome | 2.9 W | [meter 35%] |
| Display backlight | 1.9 W | [meter 23%] |
| gnome-shell | 1.4 W | [meter 17%] |
| code | 0.8 W | [meter 10%] |
| Wi-Fi | 0.7 W | [meter 8%] |

## Power › Power Profile — `power/profile`

1. **Radio cards ([Form controls](../03-components/21-form-controls.md))** — "Power mode" · span 6
- Source chip (Expert): `powerprofilesctl`

| Option | Description | Checked |
| --- | --- | --- |
| Power Saver | Longer battery life | No |
| Balanced | Default | Yes |
| Performance | Plugged in only | No |

Radio group name: `pp`.

2. **Key-value list** — "Battery policies" · span 6
- Source chip (Expert): `/etc/UPower/UPower.conf`

| Key | Value |
| --- | --- |
| Low battery warning | 10% |
| Critical action | {warn:Not configured} [Configure](act:fix:batt) |
| Charge threshold | Supported · Off |
| Screen blank | 5 min |
| Automatic suspend | 20 min on battery |

## Power › Sleep — `power/sleep`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Suspends | 9 |  | — | last 7 days | — | — |
| Failures | 0 |  | — |   | — | — |
| Avg resume time | 1.8 | s | — |   | — | — |
| Time asleep | 41 | h | — |   | — | — |

2. **Data table** — "Sleep events" · span 12
- Source chip (Expert): `journalctl -u systemd-suspend`

| Time | Event | Duration | Wake source | Result |
| --- | --- | --- | --- | --- |
| Mon 23:40 | Suspend | 7 h 48 m | Lid open | {ok:Success} |
| Mon 13:10 | Suspend | 42 m | Power button | {ok:Success} |
| Sun 22:55 | Suspend | 9 h 12 m | Not recorded | {ok:Success} |
| Sat 18:05 | Suspend | 1 h 30 m | USB | {ok:Success} |

---

← Previous: [Hardware](06-hardware.md) · [Index](README.md) · Next: [Display & Audio](08-display.md) →
