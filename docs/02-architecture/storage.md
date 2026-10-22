# Storage (SQLite)

[Docs](../README.md) › [Architecture](README.md)

| Table | Holds |
| --- | --- |
| `snapshot` | Inventory at a point in time (apps, services, users, jobs, ports, hardware) |
| `metric` | Time series (CPU, RAM, temperature, disk, battery) |
| `event` | Things that happened (install, update, crash, OOM, login fail, device plugged) |
| `finding` | Current problems and warnings from rules, with severity and status |
| `fix_run` | Each fix applied: steps, commands, result, undo data |
| `setting` | User settings |

**Retention:** 1-minute data is kept 7 days, 15-minute averages 90 days, and daily averages 2 years. The target database size is under 200 MB.
