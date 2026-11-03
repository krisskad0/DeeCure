# Power (`power`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: System · Icon: `bat` · Status badge: {warn:Warning} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Battery, Power Profile, Sleep

## Power › Battery: `power/battery`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Charge | 64 | % | - | Discharging | Usage bar 64% | - |
| Health | 71 | % | {warn:Warning} | of design capacity | - | - |
| Time remaining | 3 h 10 m |  | - | Estimate | - | - |
| Power draw | 8.4 | W | - |   | Sparkline slot 2, gen[81, 8, 3, 2, 3, 25, 8.4] | - |
| Capacity | 40.5 | Wh | - | Design 57 Wh | - | - |
| Cycle count | N/A |  | - | Not reported | - | - |

2. **Line chart**: "Charge level" · span 8
- Source chip (Expert): `upower history`
- Height 220px · Y axis 0 to 100 · unit "%"
- Series: Charge (slot 1, gen[91, 60, 30, 8, 1, 100, 64])
- X axis: time (current range)
- Threshold line at 10: "Low 10%"

3. **Gauge**: "Battery health" · span 4
- Info tooltip: "How much charge the battery holds compared with when it was new."
- Value 71 of 100 · arc `warn` · center "71%" · subtitle "40.5 of 57 Wh" · caption below: "Replace below 60%"

4. **Line chart**: "Health trend" · span 6
- Subtitle: "12 months"
- Height 180px · Y axis 50 to 100 · unit "%"
- Series: Health (slot 1, data [83,82,81,80,79,78,77,76,75,74,72,71])
- X labels: Nov, Dec, Jan, Feb, Mar, Apr, May, Jun, Jul, Aug, Sep, Oct

5. **Bar list**: "Battery use by app" · span 12, above the Health trend (side by side, the shorter panel was stretched with empty space), at every level
- Subtitle: "estimate" · Info tooltip: how the estimate is made (the usual low drain goes to the screen and the system at rest; the drain above it is split by each app's CPU and graphics time)
- Source chip (Expert): `cgroup cpu.stat · DRM fdinfo · /sys/class/power_supply`
- Dropdown in the header: Today (default), Yesterday, Last 7 days
- First line: "On battery today: used 30% · 15.0 Wh of a full charge."
- Then the 8 biggest as bars, each with its kind as a hint (App, System service, Background service, "Screen, Wi-Fi, and the computer idle", "Many small ones together") and its share of a full charge with the watt hours; every one is in a table under "Show all" (App or part, Kind, Used, Share)
- Empty text: "Nothing counted today. DeeCure counts battery use each minute while the laptop runs on battery, not while it charges."

| App or part | Used |
| --- | --- |
| Google Chrome | 18% · 9.0 Wh |
| Screen and system at rest | 12% · 6.0 Wh |
| Visual Studio Code | 4% · 2.1 Wh |
| Other apps and services | <1% · 0.30 Wh |

Expert: "Battery used while suspended" starts with one line on the usual loss per hour while asleep (normal at 2% or less, a warning above), or says no charge readings exist yet; then bars of how long each recent suspend lasted, with the charge before and after as a hint. The table is under "Show all".

## Power › Power Profile: `power/profile`

1. **Radio cards ([Form controls](../03-components/21-form-controls.md))**: "Power mode" · span 6
- Source chip (Expert): `powerprofilesctl`

| Option | Description | Checked |
| --- | --- | --- |
| Power Saver | Longer battery life | No |
| Balanced | Default | Yes |
| Performance | Plugged in only | No |

Radio group name: `pp`.

2. **Key-value list**: "Battery policies" · span 6
- Source chip (Expert): `/etc/UPower/UPower.conf`

| Key | Value |
| --- | --- |
| Low battery warning | 10% |
| Critical action | {warn:Not configured} [Configure](act:fix:batt) |
| Charge threshold | Supported · Off |
| Screen blank | 5 min |
| Automatic suspend | 20 min on battery |

## Power › Sleep: `power/sleep`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Suspends | 9 |  | - | last 7 days | - | - |
| Failures | 0 |  | - |   | - | - |
| Avg resume time | 1.8 | s | - |   | - | - |
| Time asleep | 41 | h | - |   | - | - |

2. **Data table**: "Sleep events" · span 12
- Source chip (Expert): `journalctl -u systemd-suspend`

| Time | Event | Duration | Wake source | Result |
| --- | --- | --- | --- | --- |
| Mon 23:40 | Suspend | 7 h 48 m | Lid open | {ok:Success} |
| Mon 13:10 | Suspend | 42 m | Power button | {ok:Success} |
| Sun 22:55 | Suspend | 9 h 12 m | Not recorded | {ok:Success} |
| Sat 18:05 | Suspend | 1 h 30 m | USB | {ok:Success} |

---

← Previous: [Hardware](06-hardware.md) · [Index](README.md) · Next: [Display & Audio](08-display.md) →

**Sleep events** starts with chips (suspends, woke fine, no resume logged, failed, longest), then the 8 most recent as bars of how long the computer slept (with the result under the date); every event is under **Show all**. Expert: **May wake the computer** shows chips (may wake it, can be stopped, always kept) and only the devices that signalled since start, each with **Stop waking…**, or a green "Quiet" line; the full table is under **Show all**. **CPU sleep states** are bars of the share of time in each state now (since start until the second reading), with the wake-up time and the share since start on the second line. Whole-chip low power and Wake-ups per process sit side by side.
