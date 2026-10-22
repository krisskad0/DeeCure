# CLI

[Docs](../README.md) › [Architecture](README.md)

```bash
sysdoc status            # health score + top issues
sysdoc slow              # "why is my PC slow?"
sysdoc show storage      # one category
sysdoc jobs              # all scheduled jobs, next run times
sysdoc history --since 7d
sysdoc fix storage.root_almost_full --preview
sysdoc report --html report.html   # full report to share
sysdoc status --json     # for scripts and servers
```

The CLI also makes SysDoc usable on servers with no desktop.
