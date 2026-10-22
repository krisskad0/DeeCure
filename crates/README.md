# crates

The Rust workspace. One folder per crate.

| Crate | Builds | Runs as | Spec |
| --- | --- | --- | --- |
| `sysdoc-core` | Library | — | Shared types: findings, metrics, events, messages, rule schema |
| `sysdoc-agent` | `sysdoc-agent` | Normal user (systemd user service) | [Architecture overview](../docs/02-architecture/overview.md) |
| `sysdoc-helper` | `sysdoc-helper` | Root, started by D-Bus on demand | [Helper](../docs/02-architecture/helper.md) |
| `sysdoc-cli` | `sysdoc` | Normal user | [CLI](../docs/02-architecture/cli.md) |

Changes to `sysdoc-helper` need two reviewers. See [CONTRIBUTING.md](../CONTRIBUTING.md).
