# Panel

[Docs](../../README.md) › [UI design](../README.md) › [Components](README.md)

- `--panel` background, 1px `--line` border, radius 8, vertical flex.
- **Header** (optional, min-height 42, padding 0 12 0 14, bottom border): title (13.5px 600), optional subtitle (12px muted, e.g. "last 7 days"), optional info icon (16px circled "i", muted; hover/focus shows a tooltip: `--ink` background, `--panel` text, 12px, max-width 260px, radius 6, below-left). Right side: **source chip** (mono 11.5px, `--panel-2`, 1px border, radius 4, max-width 260 with ellipsis; **visible only in Expert mode**), then small action buttons.
- **Body:** padding 12 14 14. Tables, lists, checklists, timelines and forms use a flush body (padding 0).
- **Warning here:** when an open warning on this sub-page is shown by this panel ([Page status](../04-behavior/page-status.md)), the panel gets a 1px outline in the warning color (`--warn`, or `--crit` for critical) around its border, and a "Warning here" ("Problem here") badge after the title. A panel whose title goes on after " ·" (such as "Battery details · BAT0") matches too.

---

[Index](README.md) · Next: [Status badge](02-status-badge.md) →
