# Heatmap (day × hour)

[Docs](../../README.md) › [UI design](../README.md) › [Components](README.md)

- CSS grid: a label column + 24 hour columns (min 5px each), gap 2px.
- Header row: hour labels "00", "03", "06" … (every 3rd), 11px muted, centered.
- Cells 18px tall, radius 3. Level: value 0 → `--h1`, otherwise `2 + floor(value / max × 3.99)`, capped at 5 → `--h2`…`--h5`. Tooltip `"<Day> <HH>:00 · <value> <unit>"`.
- Legend under the grid, right-aligned: "Less" + five 14×10 swatches (h1-h5) + "More".

---

← Previous: [Gauge](15-gauge.md) · [Index](README.md) · Next: [Horizontal bar list](17-horizontal-bar-list.md) →
