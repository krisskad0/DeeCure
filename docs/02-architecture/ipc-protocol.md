# IPC protocol

[Docs](../README.md) › [Architecture](README.md)

How the CLI and the desktop app talk to the agent. IPC means "inter-process communication": programs on the same computer sending messages to each other.

The Rust types live in `crates/deecure-core/src/rpc.rs`. Their tests pin the exact JSON shown here.

## Transport

| Item | Value |
| --- | --- |
| Socket | Unix socket at `$XDG_RUNTIME_DIR/deecure/agent.sock`, mode `0600` |
| Format | JSON-RPC 2.0 |
| Framing | One JSON message per line, UTF-8, ending with `\n` |
| Who may connect | Only the same user. The agent checks the caller's user ID. |
| Protocol version | `1` (`PROTOCOL_VERSION`). Raised only for breaking changes. |

One message per line keeps it simple to debug by hand:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"version"}' | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/deecure/agent.sock
```

## Request and response

```json
{"jsonrpc": "2.0", "id": 1, "method": "status"}
{"jsonrpc": "2.0", "id": 1, "result": {"generated_at": 1761200000000, "findings": []}}
```

- `id` is a number or a text. The response always carries the same `id`.
- A response has `result` **or** `error`, never both.
- If the agent cannot read the request at all, the response `id` is `null`.

## Methods

| Method | Params | Result | Since |
| --- | --- | --- | --- |
| `version` | none | `{"agent": "0.1.0", "protocol": 1}` | Phase 0 |
| `status` | none | `{"generated_at": <ms>, "checked_at": <ms>?, "checked": [<category>, …]?, "score": <0-100>?, "deductions": [{"category": …, "points": …}, …]?, "findings": [<Finding>, …]}`: open findings, most serious first. `checked_at` is when the rules last ran; it is missing until they have run once. `checked` lists the areas that have rules (such as `["power", "services", "storage"]`). `score` and `deductions` are the [health score](health-score.md); both are missing before the first check. | Phase 0 |
| `metrics` | `{"name": "cpu.usage", "subject": "/"?, "since": <ms>, "until": <ms>, "max_points": 160?}` | `[<Metric>, …]`, oldest first. Points are averaged into time buckets so about `max_points` remain (at most 2000, the default). Buckets start at whole multiples of their length, so different metrics line up. | Phase 0 |
| `latest` | none | `[<Metric>, …]`: the newest point of every metric and subject from the last 10 minutes | Phase 1 |
| `snapshot` | `{"kind": "filesystems"}` (also `services`, `boot`, `updates`) | The newest `<Snapshot>` of that kind, or `null` | Phase 0 |
| `inventory` | `{"kind": "system"}` | Live data read now: `system`, `processes`, `network`, `gpu`, `cpu_detail`, `memory_detail`, `process_memory`, `cgroups`, `storage_detail`, `process_io`, `pci_detail`, `gpu_detail`, `graphics_stack`, `net_deep`, `tcp_traffic`, `power_detail`, `thermal`, `boot_detail`, `unit_files`, `service_security`, `kernel_detail`, `platform_detail`, `os`, `kernel`, `update_history`, `hardware`, `power`, `display`, `audio`, `bluetooth`, `firmware`, `sleep`, `storage`, `disk_usage`, `user_services`, `service_memory`, `startup`, `jobs`, `journal`, `errors`, `kernel_log`, `crashes`, `boots`, `device_events`, `config_changes`, `package_history`, `net_details`, `sockets`, `accounts`, `security`, `privacy`, `packages`, `containers`, `backups`, `diagnose_performance` and `diagnose_network` (on-demand checks that take a second or two), or `agent` (DeeCure's own facts) ([Collectors](collectors.md#live-inventories)). An unknown kind is a `-32602` error whose message starts with "unknown inventory"; the desktop app shows it as "Agent needs a restart", because a newer app may ask an agent started before an update. | Phase 1 |
| `process_detail` | `{"pid": 1234}` | Everything about one process for the Expert drawer: name, state, user, parents and children, command line, program file, working folder, package, control group, start time, CPU time, nice, scheduling policy, disk priority, allowed CPUs, threads and the busiest 10, context switches, open files and connections, limits, AppArmor label, capabilities, seccomp, no-new-privileges, namespaces it does not share, resident and real memory, swap, hard faults, out-of-memory score, disk bytes, and graphics time. Other users' processes leave out what the kernel hides. A process that is gone is a `-32602` error "process N is not running". An agent older than the app answers `-32601`, which the app shows as "Agent needs a restart". | Expert |
| `events` | `{"since": <ms>, "until": <ms>?, "kinds": ["boot", …]?, "limit": 500?}` | `{"events": [<Event>, …], "truncated": false}`: what changed, newest first ([Storage › Events](storage.md#events)). At most 2000 (the default), and never more than fits in three quarters of one message; `truncated` says more matched. Config diffs are left out. | Phase 3 |
| `event` | `{"id": 42}` | One `<Event>` with every detail, such as a config file's `diff`, or `null` | Phase 3 |
| `notify_settings` | none | `{"critical": true, "warnings": false, "weekly": true, "quiet_hours": "22-07"}` ([Notifications](notifications.md#settings)) | Phase 3 |
| `set_notify_settings` | `{"settings": {…}}` (missing fields take their defaults) | The saved settings. Unknown fields or values are a `-32602` error. | Phase 3 |
| `notify_test` | none | `true` when a test notification was shown; otherwise an error saying why (such as no desktop session) | Phase 3 |
| `notifications` | none | `[{"at": <ms>, "title": …, "body": …}, …]`: the last 30 shown, newest first | Phase 3 |
| `report` | `{"since": <ms>?, "until": <ms>?}` (default: the last 7 days) | A report: `created_at`, `since`, `until`, `host`, `os`, `score`?, `score_trend` (`[[<ms>, <score>], …]`, daily), `findings`, `changes` (counts by type), `highlights` (at most 12 events), `disks` (with `days_until_full` while filling), and `tip` | Phase 3 |
| `weekly_reports` | none | The last 8 weekly reports, newest first | Phase 3 |
| `data_settings` | none | `{"keep_history": "90d" \| "1y" \| "2y"}` (default `"2y"`) | Phase 3 |
| `set_data_settings` | `{"settings": {"keep_history": …}}` | The saved settings; cleanup runs at once | Phase 3 |
| `delete_history` | `{"confirm": true}` (anything else is a `-32602` error) | `{"rows": …, "bytes_freed": …}` ([Storage](storage.md)) | Phase 3 |

`?` marks an optional field. Unknown param fields are rejected. Metric names are listed in [Collectors](collectors.md#metric-names).

New methods are added to this table and to `rpc::method` in the same pull request.

## Errors

```json
{"jsonrpc": "2.0", "id": 1, "error": {"code": -32601, "message": "unknown method 'reboot'"}}
```

| Code | Meaning |
| --- | --- |
| `-32700` | The message is not valid JSON |
| `-32600` | The JSON is not a valid request |
| `-32601` | No such method |
| `-32602` | Wrong params for this method |
| `-32603` | Something went wrong inside the agent |

## Shared values

| Value | JSON | Example |
| --- | --- | --- |
| Time | Milliseconds since 1970-01-01 UTC, as a number | `1761200000000` |
| Severity | `"info"`, `"warning"`, `"critical"` | `"critical"` |
| UI status | `"ok"`, `"info"`, `"warn"`, `"crit"`, `"off"` | `"warn"` |
| Category | Page ID | `"storage"`, `"jobs"` |
| Rule or fix ID | `<category>.<name>`, lowercase, `a-z 0-9 _` | `"storage.root_almost_full"` |

A finding looks like this:

```json
{
  "rule": "storage.root_almost_full",
  "subject": "/",
  "severity": "critical",
  "state": "open",
  "title": "Main disk is almost full",
  "explain": "Your main disk has only 13.8 GB left.",
  "first_seen": 1761200000000,
  "last_seen": 1761203600000,
  "fixes": ["storage.clean_apt_cache"]
}
```

`subject` and `fixes` are left out when empty.
