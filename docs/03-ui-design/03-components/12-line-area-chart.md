# Line / area chart

[Docs](../../README.md) › [UI design](../README.md) › [Components](README.md)

- SVG sized to the container width × the specified height (default 200). Margins: left 46, right 12 (74 with a forecast), top 8, bottom 22.
- Y axis: 5 evenly spaced horizontal lines from min to max. The bottom line uses `--axis`, the others `--grid`. Labels 11px muted, right-aligned at x = left − 7. Format: rounded when ≥10 or an integer, else 1 decimal, plus the unit. Max `auto` = the "nice" ceiling of max(data) × 1.15 (1, 2, 2.5, 5, 10 × 10ⁿ).
- X axis labels: for time series, 5 labels at 0, 25, 50, 75 and 100% of the points, the last one "Now". Format by range: 1h → `HH:MM`, 24h → `HH:MM`, 7d → weekday, 30d → `D Mon`. For label arrays, show every ceil(n/6)-th label plus the last. Labels within 24px of an edge are anchored start or end.
- Series: 2px lines with round caps and joins in `--sN`. Area under each line at 10% opacity (20% when stacked). Stacked series add their values; each area fills down to the series below it. Series are drawn from top to bottom so the first series sits on top.
- Threshold (optional): full-width dashed line (4 4, 1px muted) with a right-aligned label 5px above.
- Forecast (optional, `[daysAhead, value, label]`): the x-domain extends by `daysAhead` points. Dashed `--crit` line (5 4, 2px) from the last point to the forecast point, a 4.5px `--crit` dot with a 2px `--panel` ring, and a bold `--ink-2` label to its right.
- End markers: a 4px dot with a 2px `--panel` ring at the last point of each series.
- Hover/tap: the nearest point index snaps; a vertical crosshair (`--line-2`) and a ring dot on each series appear. Tooltip (`--ink` background, `--panel` text, 12px, radius 7, padding 7 9, shadow) sits centered above the highest point: a header (time label, e.g. `Wed 21:15`, or the label array value) and one row per series (8px color square, "Name: value" or just the value for a single series). Hidden on pointer leave.
- Legend: shown above the chart only when there are 2 or more series: 12×3px color line + name, 12px `--ink-2`, gap 14.

---

← Previous: [Alert banner](11-alert-banner.md) · [Index](README.md) · Next: [Bar chart](13-bar-chart.md) →
