# Knowledge base

[Docs](../README.md) › [Architecture](README.md)

The knowledge base is plain YAML files, organized by type. These files are what make SysDoc friendlier than Cockpit.

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

- The first release ships with entries for the most common services, errors, and ports on Ubuntu.
- Contributors can add entries through pull requests.
- Anything not covered falls back to the AI, which labels its answer "AI explanation — may be wrong".
