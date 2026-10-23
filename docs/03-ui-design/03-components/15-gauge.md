# Gauge

[Docs](../../README.md) › [UI design](../README.md) › [Components](README.md)

- Semicircle SVG, viewBox 200×112. Arc from (20,96) to (180,96), radius 80, stroke 14, round caps. Track `--track`. Value arc in `--s1`, or `--good`/`--warn`/`--crit` when a status is set. The value fraction is clamped to 0.001-1.
- Value text 30px 600 at (100, 86). Subtitle 11.5px muted at (100, 106). Max width 220px, centered.
- **Gauge group:** grid `repeat(auto-fit, minmax(140px, 1fr))`, gap 4 12, with a 12px `--ink-2` caption under each gauge.

---

← Previous: [Donut chart](14-donut-chart.md) · [Index](README.md) · Next: [Heatmap (day × hour)](16-heatmap.md) →
