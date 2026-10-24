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
| `name` | No | A short name for Settings, such as "CPU too hot". Defaults to the title. |
| `for_each` | No | A list to go through, one finding per entry: `filesystems`, `batteries`, `services`, `sensors`, or `interfaces`. |
| `limits` | No | Values the user can change in Settings › Notifications, used in conditions as `limit.<name>`: `- name: minutes`, `label`, `default`, `unit`, `min`, `max`. The default must be inside `min`-`max`, and a condition can only use limits the rule lists. |
| `title` | Yes | One short line. May use `{fact}` placeholders. |
| `explain` | Yes | What it means, why it matters, what to do. May use `{fact}` placeholders. |
| `severity` | Yes | One condition per level: `critical`, `warning`, `info`. The most serious true one wins. |
| `fixes` | No | Fix action IDs to offer, in order: `- action: storage.clean_apt_cache`. |
| `learn_more` | Yes (a test checks it) | A knowledge-base page. |
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

### The user's choices

The `warnings` setting holds the warnings the user turned off and the limits they changed (`warning_settings` and `set_warning_settings` in the [protocol](ipc-protocol.md)). A warning that is off closes its open findings and is not checked. A user's limit is kept inside the limit's `min`-`max`; the agent stores only known warnings and limits, and only values that differ from the default. Saving runs a fresh check right away.

## Facts

Readings older than 10 minutes are ignored, so a rule never acts on stale data.

| Fact | Example | From |
| --- | --- | --- |
| Every metric without a subject, by its name | `memory.used_percent`, `cpu.usage`, `services.failed` | [Collectors](collectors.md#metric-names) |
| `root.used_percent`, `root.free_percent`, `root.free_gb`, `root.size_gb`, `root.fs_type`, `root.device`, `root.mount_point`, `root.filling`, `root.days_until_full`, `root.percent_per_day`, `root.gb_per_day` | `root.free_gb` = 13.8 | The `/` filesystem |
| `memory.errors_corrected_24h`, `pcie.errors_corrected_24h` | 3 | How much the counter rose in the last 24 hours (a restart sets it back to 0, which counts as a rise from 0) |
| `log.<kind>_24h`, `log.<kind>_7d` for `disk_io`, `gpu_hang`, `kernel_crash`, `machine_check`, `firmware_crash`, `usb_overcurrent`, `thermal_shutdown`; and `log.oom_kills_24h` | `log.disk_io_24h` = 2 | Hardware events and apps stopped for memory, from the journal history (`history/journal.rs`). Always present: 0 when nothing was logged |

Lists for `for_each`, used as `item.<field>`:

| List | One entry per | Fields |
| --- | --- | --- |
| `filesystems` | Mount point | `mount_point`, `used_percent`, `free_percent`, `free_gb`, `size_gb`, `fs_type`, `device`, `inodes_percent`, `fs_errors` (ext4), `read_only` (1 when read-only against `/etc/fstab`), and the trend: `percent_per_day`, `gb_per_day`, `filling` (true or false), `days_until_full` (only while filling) |
| `batteries` | Battery, such as `BAT0` | `name`, `charge`, `health`, `power`, `temp`, `charging`, `fault`, `empty`, `drain_on_mains_minutes` |
| `sensors` | Temperature sensor with its own limit, such as `nvme/Composite` | `name`, `part` (plain words, such as "The NVMe drive (Composite)"), `celsius`, `limit`, `critical` (1000 when the sensor reports none), `hot_minutes`, `cpu` (true for CPU chips, which have their own warning) |
| `interfaces` | Physical network interface | `name`, `error_percent` (errors over packets in the last 10 minutes) |
| `services` | `.service` unit | `name`, `description`, `load`, `active`, `sub`, `enabled`, `restarts` |

A list that is not known (no battery, for example) means "unknown", not "empty".

### Trends

The trend fields come from hourly averages of `disk.used_percent` (`rules/forecast.rs`). A disk often jumps once (an install, a download, a build) and then stays flat, and one straight line through all the history would turn that jump into "fills every day". So the rate is measured twice, as a straight line (least squares) over the last day and over the last 3 days. The disk is filling only when both rates say so, and the slower one is used. A jump that stopped shows only in the 3-day rate; a disk that keeps filling shows in both. `gb_per_day` is `percent_per_day` of the disk's size.

| Case | `filling` | `days_until_full` |
| --- | --- | --- |
| Less than 3 days of history | false (an earlier "filling" finding closes) | missing |
| 3 days or more, but readings over less than 70% of either window (the agent was off) | missing | missing (rules say "unknown", nothing changes, so a real finding does not flicker) |
| Either rate grows less than 0.05% a day, shrinks, or the date would be over a year away | false | missing |
| Filling | true | rounded, at least 1 |

The Storage page's "Full in" tile shows the same forecast (`root_forecast` in the `storage` inventory), and says how many days of readings it has until there are 3.

Write trend conditions as `item.filling and item.days_until_full <= 3`: when the disk is not filling, `item.filling` is false, so the rule is clear and an open finding closes, even though `days_until_full` is missing.

