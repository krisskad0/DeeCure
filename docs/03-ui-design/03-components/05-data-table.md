# Data table

[Docs](../../README.md) › [UI design](../README.md) › [Components](README.md)

- **Tools bar** (only if the table has search, filter or selection): padding 8 12, bottom border, wraps.
  - Search box: 28px, `--panel-2`, 1px border, radius 6, 14px search icon, width 180–260px, placeholder from the spec. Matches any cell's text (case-insensitive substring).
  - Filter chips: 26px pills (radius 13, 1px border, 12.5px `--ink-2`). Pressed chip: `--ink` background, `--panel` text. "All" plus options; a row matches when the chosen column's text **starts with** the option (case-insensitive).
  - Selection bar (right): "N selected" or "Select rows", plus a small button (primary when at least one row is selected).
- **Table:** 100% wide, collapsed borders, 13px.
  - Header cells: `--panel-2`, 12px 600 `--ink-2`, padding 7 12, bottom border, no wrap. Columns with a header label are sortable. Click toggles ascending/descending, with an `↑`/`↓` arrow in `--accent`.
  - Body cells: padding 7 12, bottom border (none on the last row), no wrap, vertically centered. Row hover `--hover`.
  - Column flags: `n` = numeric (right-aligned, tabular), `x` = Expert-only (hidden in Beginner mode), `w` = wrap (normal wrapping, min-width 260px).
  - Selection: 34px checkbox column, header checkbox selects all on the current page.
  - Clickable rows (pointer cursor): rows with a row action run it; tables marked "Inspect" open the row-inspect drawer ([Row-inspect drawer](../04-behavior/interactions.md#row-inspect-drawer)).
- **Sorting:** parse a leading number with units (KB, kB, K = ×0.001, MB = ×1, GB = ×1000, TB = ×1,000,000, plus ms, s, %, h, m, W). Compare numerically when both values parse, otherwise compare text.
- **Paging:** default 12 rows per page (some tables differ). Footer (top border, padding 6 12, 12px muted): "1–12 of N" and Previous/Next small buttons when there is more than one page. When the spec gives a total count, "of N" shows that total while no search or filter is active.
- **Empty:** one full-width cell, centered muted text, padding 28 14. Text comes from the spec, default "No matching rows".
- Wide tables scroll horizontally inside the panel.

---

← Previous: [KPI tile and KPI row](04-kpi-tile.md) · [Index](README.md) · Next: [Inline meter](06-inline-meter.md) →
