# KPI tile and KPI row

[Docs](../../README.md) › [UI design](../README.md) › [Components](README.md)

- **Row:** full width, grid `repeat(auto-fit, minmax(168px, 1fr))`, gap 12. 2 columns at ≤560px.
- **Tile:** `--panel`, 1px border, radius 8, padding 11 14 10, vertical stack:
  1. Label row (min-height 20): label 12px 500 `--ink-2` (ellipsis) + status badge on the right if the status is not `ok`.
  2. Value 24px 600 + unit (13px 500 muted).
  3. Subtext 12px muted (ellipsis), or a non-breaking space when empty.
  4. Visual, one of: **sparkline** (SVG 100% × 28px, `preserveAspectRatio="none"`, min-max normalized, line 1.6px in the series color with non-scaling stroke, area at 12% opacity), or **usage bar** (6px, `--track`, radius 3, fill `--s1`, or `--warn`/`--crit` when the tile status is warn/crit; 10px top margin, 8px bottom).
- Tiles with a link are `<button>`s that navigate. Hover: `--line-2` border, `--panel-2` background.
- Sparklines use the same generator as charts ([Sample data generator (exact)](../04-behavior/sample-data.md)) with the current time range.

---

← Previous: [Buttons](03-buttons.md) · [Index](README.md) · Next: [Data table](05-data-table.md) →
