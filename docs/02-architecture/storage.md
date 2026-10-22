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

## Implementation details

The code is in `crates/deecure-agent/src/store/`.

| Item | Value |
| --- | --- |
| File | `~/.local/share/deecure/deecure.db` (follows `$XDG_DATA_HOME`). Folder mode `0700`, file mode `0600`. |
| SQLite | The system library (`libsqlite3`), so Ubuntu security updates cover it. WAL mode, so the app can read while the agent writes. |
| Times | Milliseconds since 1970-01-01 UTC, the same as in the [IPC protocol](ipc-protocol.md). |
| Empty subject | Stored as `''`, so it can be part of a primary key. |
| Schema upgrades | The version lives in SQLite's `user_version`. Each migration moves it up by one, in its own transaction. A database from a newer DeeCure is refused, not damaged. |

### How metrics are stored

- Readings in the same minute are averaged into one point. The `samples` column counts the readings, so later averages stay correct.
- The cleanup job runs when the agent starts and then every hour:

| Data | Kept as is for | Then |
| --- | --- | --- |
| 1-minute points | 7 days | Rolled up into 15-minute averages |
| 15-minute points | 90 days | Rolled up into daily averages |
| Daily points | 2 years | Deleted |
| Events | 2 years | Deleted |
| Snapshots | 90 days | Deleted, except the newest of each kind |
| Findings that are no longer open | 90 days | Deleted |
| Open findings, settings | Always | — |

The retention times for events, snapshots, and closed findings were chosen during Phase 0 to keep the database under 200 MB. Change them in `store/retention.rs`, and update this table.

