# Architecture

[Docs](../README.md)

How SysDoc is built. Start with the overview, then read the part you work on.

## Big picture

| File | What it covers |
| --- | --- |
| [overview.md](overview.md) | Components, tech stack, how the parts talk, files on disk |
| [security-model.md](security-model.md) | How SysDoc keeps itself safe, trust boundaries |
| [performance-budget.md](performance-budget.md) | CPU, RAM, and speed limits the app must meet |

## Parts (one file each)

| File | Part | Runs in |
| --- | --- | --- |
| [collectors.md](collectors.md) | Collectors: read data from the system | Agent |
| [storage.md](storage.md) | SQLite tables and data retention | Agent |
| [rules-engine.md](rules-engine.md) | YAML rules that turn data into findings | Agent |
| [knowledge-base.md](knowledge-base.md) | Plain-English meanings of services, errors, ports | Agent |
| [health-score.md](health-score.md) | How the 0–100 score is calculated | Agent |
| [fix-engine.md](fix-engine.md) | Plan → preview → protect → apply → verify, and undo | Agent + helper |
| [helper.md](helper.md) | The small root helper with a fixed list of actions | Helper |
| [ai-assistant.md](ai-assistant.md) | The optional local AI with read-only tools | Agent |
| [notifications.md](notifications.md) | When and how SysDoc alerts the user | Agent |
| [cli.md](cli.md) | The `sysdoc` command | CLI |

## Decisions

[decisions/](decisions/README.md) holds short records of big technical choices: what we chose and why.
