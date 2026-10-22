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
