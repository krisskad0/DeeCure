# Page status rules

[Docs](../../README.md) › [UI design](../README.md) › [Behavior](README.md)

Each page has a status shown as a badge in its header and as a dot in the sidebar.

| Page | Status rule |
| --- | --- |
| Overview | Critical if any open issue is Critical, else Warning |
| Storage | Disk status: Critical if used ≥93%, Warning if ≥85%, else Healthy |
| Services | Critical, or Warning after Bluetooth is restarted |
| Security | Warning, or Healthy after security updates are installed |
| Packages | Warning, or Info after security updates are installed |
| Power, Backups, Logs, Containers | Warning |
| System | Info |
| All other pages | Healthy |

Status words: `ok` = Healthy, `warn` = Warning, `crit` = Critical, `info` = Info, `off` = Unknown.

## In the real app

The table above describes the prototype's sample data. The app works it out from real findings (`app/src/status.ts`):

| Page | Status |
| --- | --- |
| Overview | The most serious open finding anywhere. Healthy when there are none. |
| Any other page with rules | The most serious open finding whose rule starts with the page ID (`storage.…` → Storage). Healthy when there are none. |
| A page with no rules yet | **Unknown** in the page header, even with no findings, because nothing checks it yet. The agent lists the areas it checks in `status` → `checked`. In Overview › Subsystems the badge says **Not checked** instead, so it does not look like a problem. |
| Optimize, Diagnostics, Assistant, Settings, and the record and view pages (Logs, Activity, Processes, Display & Audio), which no warning covers | No badge |
| Any page while the agent is not reachable or has not checked yet | Unknown |

The sidebar shows a dot only for Warning (yellow) and Critical (red). Overview shows the number of open findings. The sub-page that shows a finding gets the same dot in the page's sub-page list, so it is clear where to look (for example Hardware › Sensors for a hot part). Each rule names its sub-page in the app (`RULE_TAB` in `app/src/status.ts`); a test fails when a rule has none. A notification opens that sub-page too.

On that sub-page, each open finding is an alert box at the top, most serious first: its title, its explanation, "Show where" (scrolls to the panel that shows it), and Fix when the rule has a fix. That panel gets a warning (or red) outline and a "Warning here" ("Problem here") badge next to its title. Each rule names its panels (`RULE_PANELS` in `app/src/status.ts`); a panel whose title goes on after " ·" (such as "Battery details · BAT0") counts too. A rule without panels is only explained at the top.

Areas with rules (October 2026): all 13 in Overview › Subsystems: System, Hardware, Power, Services, Scheduled Jobs, Packages, Containers, Storage, Backups, Network, Accounts, Security, Privacy.
