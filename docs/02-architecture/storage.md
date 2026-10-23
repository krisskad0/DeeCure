# Storage (SQLite)

[Docs](../README.md) › [Architecture](README.md)

| Table | Holds |
| --- | --- |
| `snapshot` | Inventory at a point in time (apps, services, users, jobs, ports, hardware) |
| `series` | One row per metric name and subject, with its unit |
| `metric` | Time series points (CPU, RAM, temperature, disk, battery), by series number |
| `event` | Things that happened (install, update, crash, OOM, device plugged, config changed, fix) |
| `finding` | Current problems and warnings from rules, with severity and status |
| `fix_run` | Each fix applied: steps, commands, result, undo data |
| `setting` | User settings |
| `watched_file` | What each watched config file looked like last time, to show what changed |
| `notification` | Desktop notifications sent, so the same alert never repeats within 24 hours |
| `report` | Weekly summaries |

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

- Each metric name and subject (such as `disk.used_percent` for `/`) is one row in `series`. A point stores only the series number, the resolution, the time, the average, and the sample count: about 44 bytes on disk.
- Readings in the same minute are averaged into one point. The `samples` column counts the readings, so later averages stay correct.
- Container network links (`veth…`) are not recorded: each container gets a new one, and their traffic is already counted on the bridge.
- The cleanup job runs when the agent starts and then every hour:

| Data | Kept as is for | Then |
| --- | --- | --- |
| 1-minute points | 7 days | Rolled up into 15-minute averages |
| 15-minute points | 90 days | Rolled up into daily averages |
| Daily points | 2 years | Deleted |
| A series with no points left | - | Deleted (it comes back with the next reading) |
| Events | 2 years, and never more than 200,000 | Deleted, oldest first |
| Weekly reports | 2 years | Deleted |
| Snapshots | 1 day as is, then the newest of each hour; after 7 days the newest of each day | Deleted after 90 days, except the newest of each kind |
| Sent notifications | 90 days | Deleted |
| Findings that are no longer open | 90 days | Deleted |
| Open findings, settings, fix runs, watched files | Always | - |

Change these times in `store/retention.rs`, and update this table.

**Settings › Data › Keep history for** (90 days, 1 year, or 2 years; setting `data`) shortens the 2-year times for daily points, events, and weekly reports. A shorter choice cleans up at once.

**Delete history** (agent call `delete_history` with `{"confirm": true}`) deletes metrics and series, events, reports, sent notifications, every snapshot but the newest of each kind, and closed findings. It keeps fix runs (restore points), settings, open findings, and what config files look like now. The moment is saved (`history.deleted_at`); events from before it are never stored again, so package logs and the journal, which still hold old lines, cannot bring the history back.

### Size after 2 years

`make db-size` (also in CI) builds what 2 years leave behind on a big machine and measures the file. It writes in the agent's order (every series at each step), so pages fill like on a real computer. The last 7 days go through the real write path, then the real cleanup runs.

| Simulated | Size |
| --- | --- |
| 150 metric series (16 cores, 30 sensors, 6 network interfaces, 3 disks, 3 filesystems), 29,200 events with details, 515 services snapshots of 32 KB, 2,000 config file copies, 270 notifications, 104 weekly reports | **124 MB** (budget 200 MB) |

Each metric series costs about 0.85 MB at the end of 2 years (10,080 minute points, 8,640 quarter-hour points, 730 daily points). Before schema version 2, names were stored in every point and the same data took about 250 MB.

### Events

Every event has a `key`. Reading the same log twice finds the same keys, so nothing is stored twice, even after a restart. Events that grow, such as "crashed 3 times this hour", replace the stored row with the same key.

`data` holds details that depend on the kind:

| Kind | `data` fields |
| --- | --- |
| `package_installed`, `package_upgraded`, `package_removed` | `action` (install, upgrade, downgrade, remove, purge, reinstall), `from`, `to`, `format` (deb, snap, flatpak), `run` (the apt run's start, to group a run) |
| `config_changed` | `change` (added, changed, removed), `diff` (for small text files), `added`, `removed` (line counts) |
| `device_added`, `device_removed` | `bus`, `id` (`vendor:product`), `count` |
| `crash`, `out_of_memory_kill`, `service_failed` | `signal` or `result`, `count` (in that hour) |
| `boot`, `unclean_shutdown` | `boot_id`, `kernel` |
| `fix_applied`, `fix_undone` | `run_id`, `status`, `undoable` |

