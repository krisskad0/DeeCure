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
| `status` | none | `{"generated_at": <ms>, "checked_at": <ms>?, "findings": [<Finding>, …]}`: open findings, most serious first. `checked_at` is when the rules last ran; it is missing until they have run once. | Phase 0 |
| `metrics` | `{"name": "cpu.usage", "subject": "/"?, "since": <ms>, "until": <ms>}` | `[<Metric>, …]`, oldest first | Phase 0 |
| `snapshot` | `{"kind": "filesystems"}` | The newest `<Snapshot>` of that kind, or `null` | Phase 0 |

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
