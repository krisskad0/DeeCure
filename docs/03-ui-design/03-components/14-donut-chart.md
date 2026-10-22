# Donut chart

[Docs](../../README.md) › [UI design](../README.md) › [Components](README.md)

- 150×150 SVG. Outer radius 60, inner 42. 0.02 rad gap between segments. Starts at 12 o'clock and runs clockwise.
- Center: value (20px 600 `--ink`) at y 78, subtitle (11px muted) at y 95.
- Colors: series slots 1–5 in order, unless a segment overrides: `kw` = `--warn`, `kc` = `--crit`, `kn` = `--track`.
- Legend to the right (wraps under on narrow widths): rows of 10px rounded swatch | name | value (+ unit, or a custom display value) | percent (muted, tabular, right-aligned).
- Each segment has an SVG `<title>` tooltip.

---

← Previous: [Bar chart](13-bar-chart.md) · [Index](README.md) · Next: [Gauge](15-gauge.md) →
