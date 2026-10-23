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
| Diagnostics, Assistant, Settings | No badge |
| Any page while the agent is not reachable or has not checked yet | Unknown |

The sidebar shows a dot only for Warning (yellow) and Critical (red). Overview shows the number of open findings.

Areas with rules (October 2026): System, Hardware, Power, Services, Packages, Storage, Backups, Network, Accounts, Security. Not checked yet: Scheduled Jobs, Containers, Privacy (and the pages with no status of their own).
