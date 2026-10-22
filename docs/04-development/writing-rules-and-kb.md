# Writing rules and KB entries

[Docs](../README.md) › [Development](README.md)

This is the easiest way to help SysDoc. You need YAML and Linux knowledge, not Rust.

- A **rule** turns system data into a finding, like "Main disk is almost full". Format: [Rules engine](../02-architecture/rules-engine.md).
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

The file name must match the last part of the `id`.

## Writing style

- Say three things: what it means, why it matters, and what to do.
- Short sentences. Common words. No blame ("you did X wrong").
- If a technical word is needed, explain it in the same sentence.
- Be honest. If SysDoc cannot check something, say so. Never guess a value.
- Use the user's numbers: "only 13 GB left", not "low disk space".

## Choosing severity

| Severity | Use it when | Example |
| --- | --- | --- |
| `critical` | Data loss, the PC may not boot, or a security hole is open now | Disk 94% full, SSD failing |
| `warning` | It will cause trouble soon, or it is a weak setting | No backup in 21 days |
| `info` | Good to know, no action needed | Reboot pending after an update |

## Steps to add a rule

1. Pick an `id`: `<category>.<short_name>`.
2. Write the YAML file. Fill in `title`, `explain`, `severity`, and `fixes`.
3. Add a test case in `tests/fixtures/`: the input data and the finding you expect.
4. Run the rule tests: `cargo test -p sysdoc-agent rules`.
5. Open a pull request using the checklist below.

## Fix actions: safety rules

- A fix action can only call a helper action from the fixed list. You cannot add a shell command in YAML.
- Always fill in `preview_command`, `undo`, and `undo_note`. Users see them before they approve.
- `undo: none` needs a clear warning in `undo_note`.
- A new helper action needs Rust code in `crates/sysdoc-helper/`, a polkit entry, and two reviewers. See [Helper](../02-architecture/helper.md).

## Pull request checklist

- [ ] The `id` matches the file name and category.
- [ ] The text follows the writing style above.
- [ ] Severity follows the table above.
- [ ] A fixture test covers the rule (healthy case and problem case).
- [ ] Each fix has a preview command, an undo type, and an undo note.
