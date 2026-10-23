# Rules engine

[Docs](../README.md) › [Architecture](README.md)

Every check is a small YAML rule. That makes it easy to add checks and to review them.

```yaml
id: storage.root_almost_full
category: storage
severity:
  warning: "root.used_percent >= 85"
  critical: "root.used_percent >= 93"
title: "Main disk is almost full"
explain: >
  Your main disk has only {root.free_gb} GB left. When it is full,
  apps can crash and updates can fail.
cause_hint: storage.cleanable_breakdown   # shows what is using space
fixes:
  - action: storage.clean_apt_cache
  - action: storage.remove_old_snap_revisions
  - action: storage.empty_trash
learn_more: kb/storage/disk-full.md
```

A fix action looks like this:

```yaml
id: storage.remove_old_snap_revisions
needs_admin: true
helper_call: RemoveSnapRevision
preview_command: "snap remove {name} --revision={rev}"
undo: none
undo_note: "These are old copies. Your current apps are not touched."
verify: "snap list --all shows no disabled revisions"
```

## Rule file reference

The code is in `crates/deecure-agent/src/rules/`. Rule files live in `rules/<category>/<name>.yaml` in the repo, and in `/usr/share/deecure/rules/` when installed.

| Field | Required | Meaning |
| --- | --- | --- |
| `id` | Yes | `<category>.<name>`. Must match the file path: `storage/root_almost_full.yaml`. |
| `category` | Yes | Must match the first part of `id`. |
| `for_each` | No | A list to go through, one finding per entry: `filesystems`, `batteries`, or `services`. |
| `title` | Yes | One short line. May use `{fact}` placeholders. |
| `explain` | Yes | What it means, why it matters, what to do. May use `{fact}` placeholders. |
| `severity` | Yes | One condition per level: `critical`, `warning`, `info`. The most serious true one wins. |
| `fixes` | No | Fix action IDs to offer, in order: `- action: storage.clean_apt_cache`. |
| `learn_more` | No | A knowledge-base page. |
| `cause_hint` | No | Reserved for Phase 2. |

Unknown fields are refused, so a typo cannot hide.

### Conditions

```text
root.used_percent >= 93
item.active == 'failed'
item.health < 60 and not (cpu.usage > 95)
```

- Values: numbers, text in quotes, `true`, `false`, and fact names.
- Comparisons: `<`, `<=`, `>`, `>=`, `==`, `!=`. Text and true/false only allow `==` and `!=`.
- Logic: `and`, `or`, `not`, and brackets. `and` binds tighter than `or`.

### When a fact is missing

If a condition needs a fact that is not known right now, the rule says "unknown" and **nothing changes**: no new finding, and no finding is closed. This keeps DeeCure honest on computers without a battery or a sensor. A side that is surely false still decides an `and` (and surely true decides an `or`).

### How findings open and close

| The rule… | Then |
| --- | --- |
| fires | The finding opens, or is updated. `first_seen` stays from when it first opened. A finding the user dismissed stays dismissed. |
| is clear | An open finding is resolved. |
| is unknown | Nothing changes. |
| (`for_each`) entry is gone from the list | Its open finding is resolved. |
| has a mistake, such as comparing text with a number | Nothing changes; the agent logs it once. |

Rules run every minute, right after the collectors save new data.

## Facts

Readings older than 10 minutes are ignored, so a rule never acts on stale data.

| Fact | Example | From |
| --- | --- | --- |
| Every metric without a subject, by its name | `memory.used_percent`, `cpu.usage`, `services.failed` | [Collectors](collectors.md#metric-names) |
| `root.used_percent`, `root.free_gb`, `root.size_gb`, `root.fs_type`, `root.device`, `root.mount_point`, `root.filling`, `root.days_until_full`, `root.percent_per_day` | `root.free_gb` = 13.8 | The `/` filesystem |

Lists for `for_each`, used as `item.<field>`:

| List | One entry per | Fields |
| --- | --- | --- |
| `filesystems` | Mount point | `mount_point`, `used_percent`, `free_gb`, `size_gb`, `fs_type`, `device`, and the trend: `percent_per_day`, `filling` (true or false), `days_until_full` (only while filling) |
| `batteries` | Battery, such as `BAT0` | `name`, `charge`, `health`, `power` |
| `services` | `.service` unit | `name`, `description`, `load`, `active`, `sub`, `enabled`, `restarts` |

A list that is not known (no battery, for example) means "unknown", not "empty".

### Trends

The trend fields come from a straight line (least squares) through the last 7 days of hourly averages of `disk.used_percent` (`rules/forecast.rs`).

| Case | `filling` | `days_until_full` |
| --- | --- | --- |
| Less than a day of history, or fewer than 12 hours with data | missing | missing (rules say "unknown", nothing changes) |
| Grows less than 0.05% a day, shrinks, or would take over a year | false | missing |
| Filling | true | rounded, at least 1 |

Write trend conditions as `item.filling and item.days_until_full <= 3`: when the disk is not filling, `item.filling` is false, so the rule is clear and an open finding closes, even though `days_until_full` is missing.

