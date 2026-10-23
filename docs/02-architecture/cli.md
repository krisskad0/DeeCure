# CLI

[Docs](../README.md) › [Architecture](README.md)

```bash
deecure status            # health score + top issues
deecure slow              # "why is my PC slow?"
deecure show storage      # one category
deecure jobs              # all scheduled jobs, next run times
deecure history --since 7d
deecure fix storage.root_almost_full --preview
deecure report --html report.html   # full report to share
deecure status --json     # for scripts and servers
```

The CLI also makes DeeCure usable on servers with no desktop.

## What is built

The code is in `crates/deecure-cli/`. Commands not listed here are still planned.

| Command | What it does |
| --- | --- |
| `deecure status` | Open problems, most serious first, in plain English, and when the agent last checked |
| `deecure status --json` | The same report as JSON, for scripts and servers |
| `deecure history` | What changed, newest first, one heading per local day. `--since` takes `30m`, `24h`, `7d` (default), `2w`, or a date like `2026-10-01`. `--kind` (repeatable): `packages`, `config`, `devices`, `system`, `services`, `crashes`, `deecure`. `--json` prints the raw `events` answer. The packages of one apt run are one line; a fix shows its number and the undo command. |
| `deecure history --fixes` | Only DeeCure's fixes, with the numbers `deecure undo` takes |
| `deecure report` | A summary of the last 7 days (`--since` as above): health and its change, problems, what changed, disks with "full in", and one tip |
| `deecure report --html FILE` | The same as one HTML file that works offline and prints well. It is written to a temporary file first, so a failed write leaves nothing half done. `-` prints it. `--json` prints the raw report. |
| `deecure fix ACTION [TARGET]` | Preview a fix and run it after a yes (`--yes` skips the question, `--list` lists them) |
| `deecure undo NUMBER` | Take back a fix |
| `deecure --socket PATH …` | Talk to an agent at another socket (for development) |
| `deecure --version`, `deecure --help` | Version and help |

Example:

```text
$ deecure status
DeeCure status · checked just now

  i Info      Battery has lost some of its capacity
              Battery BAT0 now holds 79.3% of the charge it held when new, so
              it runs out sooner. This is normal wear. If it no longer lasts
              long enough for you, think about replacing it.
              power.battery_worn · BAT0

1 problem: 0 critical, 0 warnings, 1 info.
```

- Every problem shows an icon and a word (`✗ Critical`, `! Warning`, `i Info`), so the meaning never depends on color.
- Colors appear only in a real terminal, and never when `NO_COLOR` is set.
- "No problems found" is shown only after the rules have run. Right after the agent starts, the CLI says it has not checked yet.

| Exit code | Meaning |
| --- | --- |
| `0` | It worked (whatever the findings are) |
| `1` | It could not get an answer, for example because the agent is not running |
| `2` | Wrong options |

