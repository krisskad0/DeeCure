# Writing rules and KB entries

[Docs](../README.md) › [Development](README.md)

This is the easiest way to help DeeCure. You need YAML and Linux knowledge, not Rust.

- A **rule** turns system data into a finding, like "Main disk is almost full". Format, conditions, and facts: [Rules engine](../02-architecture/rules-engine.md#rule-file-reference).
- A **fix action** is a safe step that solves a finding. It points to one allowed helper call.
- A **KB entry** explains one thing in plain English: a service, an error, a port, or a package. Format: [Knowledge base](../02-architecture/knowledge-base.md).

## Where files go

| Type | Path | Example |
| --- | --- | --- |
| Rule | `rules/<category>/<name>.yaml` | `rules/storage/root_almost_full.yaml` (id `storage.root_almost_full`) |
| Fix action | `rules/fixes/<category>/<name>.yaml` | `rules/fixes/storage/remove_old_snap_revisions.yaml` |
| KB: service | `kb/services/<unit>.yaml` | `kb/services/cups.yaml` |
| KB: error | `kb/errors/<name>.yaml` | `kb/errors/ext4-error.yaml` |
| KB: port | `kb/ports/<number>-<proto>.yaml` | `kb/ports/5432-tcp.yaml` |
| KB: package | `kb/packages/<name>.yaml` | `kb/packages/snapd.yaml` |
| KB: topic | `kb/topics/<area>/<name>.yaml` | `kb/topics/storage/disk-full.yaml` (a rule links to it with `learn_more: topics/storage/disk-full`) |

The file name must match the last part of the `id`.

`cargo test -p deecure-agent kb` checks every KB file, so CI tells you before a reviewer does:

- A service's `name` is the file name plus `.service`, and a package's `name` is the file name.
- A port's file name is `<port>-<proto>.yaml`, `proto` is `tcp` or `udp`, and `risk_if_open` is `low`, `medium`, or `high`.
- An error's `severity` is `info`, `warning`, or `critical`, and no two errors have the same `match`.
- No long dashes, and every `plain`, `safe_to_disable`, `if_failed`, and `next_step` ends with a full stop.

## Writing style

- Say three things: what it means, why it matters, and what to do.
- Short sentences. Common words. No blame ("you did X wrong").
- If a technical word is needed, explain it in the same sentence.
- Be honest. If DeeCure cannot check something, say so. Never guess a value.
- Use the user's numbers: "only 13 GB left", not "low disk space".
- No long dashes (the em dash or the en dash): use a colon, a comma, or a full stop. Every sentence ends with a full stop.
- Warn about harm, not use ([Health warnings](../01-product/health-warnings.md)): a busy CPU or full RAM cache is never a problem. Use the hardware's own limits when it reports them, and make the problem last (a `*_minutes` reading) so a spike never warns.
- Give every number a user could disagree with a `limits` entry (`limit.<name>` in the condition), with a safe `min` and `max`, and a short `name` for Settings. Every rule needs a `learn_more` page (a test checks it).

## Choosing severity

| Severity | Use it when | Example |
| --- | --- | --- |
| `critical` | Data loss, the PC may not boot, or a security hole is open now | Disk under 5% or 5 GB free, SSD failing |
| `warning` | It will cause trouble soon, or it is a weak setting | No backup in 21 days |
| `info` | Good to know, no action needed | Reboot pending after an update |

## Steps to add a rule

1. Pick an `id`: `<category>.<short_name>`.
2. Write the YAML file. Fill in `title`, `explain`, `severity`, and `fixes`.
3. Add a test in `crates/deecure-agent/tests/rules.rs` with a healthy case and a problem case. If the rule changes what a [fixture](testing.md#fixtures) machine reports, update its `expected-findings.yaml`.
4. Run the rule tests: `cargo test -p deecure-agent --test rules --test fixtures`. A broken rule file fails them.
5. Try it live: `make agent` loads the rules from `rules/`.
6. Open a pull request using the checklist below.

## Fix actions: safety rules

- A fix action can only call a helper action from the fixed list. You cannot add a shell command in YAML.
- Always fill in `preview_command`, `undo`, and `undo_note`. Users see them before they approve.
- `undo: none` needs a clear warning in `undo_note`.
- A new helper action needs Rust code in `crates/deecure-helper/`, a polkit entry, and two reviewers. See [Helper](../02-architecture/helper.md).

## Pull request checklist

- [ ] The `id` matches the file name and category.
- [ ] The text follows the writing style above.
- [ ] Severity follows the table above.
- [ ] A fixture test covers the rule (healthy case and problem case).
- [ ] Each fix has a preview command, an undo type, and an undo note.
