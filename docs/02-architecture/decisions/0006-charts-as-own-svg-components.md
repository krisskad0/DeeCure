# 0006. Charts as our own SVG components

- Status: Accepted 2026-10-23 (closes open decision 10)
- Date: 2026-10-23

## Context

The UI spec describes every chart exactly: margins, axis rules, "nice" maximums, hover crosshairs, tooltips, forecast lines, and colors from the design tokens. The prototype already draws them as hand-made inline SVG. Our data is small: at most about 1,440 points per series per day, and fewer for longer ranges because old data is averaged.

## Decision

Draw every chart (line/area, bar, donut, gauge, heatmap, bar list, allocation bar, swim-lane, lifecycle bar) as our own React components that render SVG, following `docs/03-ui-design/03-components/`.

## Why

- They match the spec exactly, in light and dark themes, with no fighting a library's defaults.
- No chart dependency to update, audit, or bundle.
- SVG is sharp at every zoom level and easy to test.
- **Option not picked:** uPlot. It is faster for very long series, but our series are short, and matching the spec's look would take extra work.

## Consequences

- We own the chart code, so it needs tests (scales, labels, colors).
- If a future page needs tens of thousands of points, revisit this for that chart only.
