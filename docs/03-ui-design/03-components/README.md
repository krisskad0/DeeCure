# Components

[Docs](../../README.md) › [UI design](../README.md)

24 reusable components. Build each one once, as a component that takes a config object. Then every page is just a list of components with data.

## Rules for every component

- Use only the [design tokens](../01-foundations/design-tokens.md). No hard-coded colors.
- Work in light and dark themes.
- Numbers that line up in columns use tabular figures.
- Respect reduced motion.
- Never show status by color alone. Always add an icon and a word.

## Index

| # | Component | Used for | Example |
| --- | --- | --- | --- |
| 01 | [Panel](01-panel.md) | The frame around every widget: title, info tooltip, source chip, actions | Every panel on every page |
| 02 | [Status badge](02-status-badge.md) | Healthy / Warning / Critical / Info / Unknown, plus neutral tags | Page status, table cells |
| 03 | [Buttons](03-buttons.md) | Default, primary, small, link, danger, icon | Resolve, Refresh, Export |
| 04 | [KPI tile and KPI row](04-kpi-tile.md) | One headline number + sparkline or bar | Memory 5.9 GB, 78% of 7.6 GB |
| 05 | [Data table](05-data-table.md) | Lists of records | Services, processes, packages, ports |
| 06 | [Inline meter](06-inline-meter.md) | A small usage bar inside a table cell | Disk use per mount |
| 07 | [Key-value list](07-key-value-list.md) | Facts about one thing | OS version, kernel, BIOS |
| 08 | [Status list](08-status-list.md) | Status of several areas, each with a link | Overview › Subsystems |
| 09 | [Checklist](09-checklist.md) | Pass / warn / fail checks | Security posture, filesystem checks |
| 10 | [Activity timeline](10-activity-timeline.md) | Changes in order | Package installs, config edits |
| 11 | [Alert banner](11-alert-banner.md) | One important problem with an action | "Port 5432 bypasses the UFW firewall" |
| 12 | [Line / area chart](12-line-area-chart.md) | Values over time; stacked parts of a total over time | CPU and memory utilization; CPU by User / System / I/O wait |
| 13 | [Bar chart](13-bar-chart.md) | Counts per period or item | Boot time for the last 10 boots |
| 14 | [Donut chart](14-donut-chart.md) | Share of a whole | Disk usage by type |
| 15 | [Gauge](15-gauge.md) | One value against a limit | Health score, battery health, temperatures |
| 16 | [Heatmap (day × hour)](16-heatmap.md) | When things happen | Errors by hour, job runs by hour |
| 17 | [Horizontal bar list](17-horizontal-bar-list.md) | Ranking | Largest folders, slowest boot units |
| 18 | [Allocation bar](18-allocation-bar.md) | Space split on one disk | Home / Apps / System / Reclaimable / Free |
| 19 | [Swim-lane schedule](19-swim-lane-schedule.md) | Upcoming scheduled runs | Next 24 hours of cron and timer runs |
| 20 | [Lifecycle bar](20-lifecycle-bar.md) | Support window | Ubuntu 24.04 released → support ends |
| 21 | [Form controls](21-form-controls.md) | Settings: switch, select, radio cards, field rows, state tiles | Power profile, access level |
| 22 | [Code and log blocks](22-code-and-log-blocks.md) | Commands and log lines | Fix commands, journal lines |
| 23 | [Diagnostics runner](23-diagnostics-runner.md) | A step-by-step check with a result | Diagnostics › Performance Check |
| 24 | [Chat](24-chat.md) | Questions and answers with sources | Assistant › Chat |

**Data tables** all support the same features: sort by any column, search, filter chips, paging, row selection with a bulk action, and click-to-inspect.

## Shell-level components

These live in the shell, not inside pages. Their spec is in [Overlays](../02-shell/overlays.md).

| Component | Used for | Example |
| --- | --- | --- |
| Detail drawer | Inspect a row or run a fix | bluetooth.service crash details |
| Command palette (Ctrl K) | Jump to any page, service, package or job | Type "docker" |
| Notifications popover | Open issues from anywhere | Bell icon in the top bar |
| Toast | A short message after an action | "bluetooth.service restarted" |

---

← Previous: [Shell](../02-shell/README.md) · Next: [Behavior](../04-behavior/README.md) →
