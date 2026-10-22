# crates

The Rust workspace. One folder per crate.

| Crate | Builds | Runs as | Spec |
| --- | --- | --- | --- |
| `deecure-core` | Library | — | Shared types: findings, metrics, events, messages, rule schema |
| `deecure-agent` | `deecure-agent` | Normal user (systemd user service) | [Architecture overview](../docs/02-architecture/overview.md) |
| `deecure-helper` | `deecure-helper` | Root, started by D-Bus on demand | [Helper](../docs/02-architecture/helper.md) |
| `deecure-cli` | `deecure` | Normal user | [CLI](../docs/02-architecture/cli.md) |

`deecure-helper` joins the workspace in Phase 2. Until then its folder is empty.

Changes to `deecure-helper` need two reviewers. See [CONTRIBUTING.md](../CONTRIBUTING.md).
