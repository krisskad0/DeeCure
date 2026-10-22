# Bar chart

[Docs](../../README.md) › [UI design](../README.md) › [Components](README.md)

- Margins left 42, right 8, top 18, bottom 22. Y max = given, or the nice ceiling of max × 1.1. 5 gridlines with labels (integer or 1 decimal, plus unit).
- Bars: width min(24px, slot − 2), centered in their slot, rounded top corners (radius ≤4), square base. Color `--s1`. Highlighted indices use `--warn` and show their value as a bold label 5px above the bar. A bar can override its color (`kn` = `--track` grey for empty or skipped values).
- X labels thinned to fit (every ceil(n / max(4, floor(width / 46)))), always including the last.
- Hover/tap anywhere in a bar's slot: tooltip with the bar's long label (or short label) and value + unit.

---

← Previous: [Line / area chart](12-line-area-chart.md) · [Index](README.md) · Next: [Donut chart](14-donut-chart.md) →
