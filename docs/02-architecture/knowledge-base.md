# Knowledge base

[Docs](../README.md) › [Architecture](README.md)

The knowledge base is plain YAML files, organized by type. These files are what make DeeCure friendlier than Cockpit.

```yaml
# kb/services/cups.yaml
name: cups.service
plain: "Printing service. Sends documents to printers."
safe_to_disable: "Yes, if you never print."
if_failed: "Printing will not work. Other things are not affected."
```

```yaml
# kb/errors/ext4-error.yaml
match: "EXT4-fs error"
plain: "The disk's file system reported damage."
severity: critical
next_step: "Back up important files now, then run a disk check from recovery mode."
```

```yaml
# kb/topics/storage/disk-full.yaml — a how-to article; rules link to it with
# `learn_more: topics/storage/disk-full`
title: "Free up disk space"
summary: "When the main disk is full, apps crash and updates fail."
steps:
  - "Empty the Trash."
  - "Open Storage › Cleanup to see what DeeCure can remove safely."
notes: "DeeCure never deletes personal files except the Trash."
```

## How it is used

- The agent loads `kb/` at start, from the folder next to the rules (`/usr/share/deecure/kb` when installed). Broken files are logged and skipped, and so are rule links to topics that do not exist. A test checks that every shipped rule links to a real topic.
- The app asks once per window (`inventory` kind `kb`) and looks entries up locally: issues get a folded "How to fix" article, a service's drawer says what it does and whether it is safe to turn off, open ports say what usually listens there, and grouped log errors say what they mean and what to do.
- Error `match` text is plain text; text with regular-expression characters (such as `nvme.*timeout`) is also tried as a pattern.

## What ships

| Type | Entries |
| --- | --- |
| `services/` | 59 common Ubuntu services |
| `ports/` | 26 ports |
| `errors/` | 20 log and kernel messages |
| `packages/` | 12 packages |
| `topics/` | 8 how-to articles, one for each rule |

- The first release ships with entries for the most common services, errors, and ports on Ubuntu.
- Contributors can add entries through pull requests.
- Anything not covered falls back to the AI, which labels its answer "AI explanation — may be wrong".
