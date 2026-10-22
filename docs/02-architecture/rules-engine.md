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
