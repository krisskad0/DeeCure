# Notation used in the page map

[Docs](../../README.md) › [UI design](../README.md) › [Foundations](README.md)

| Notation | Meaning |
| --- | --- |
| `{ok:Text}` | Status badge, variant `ok` (green, check icon) |
| `{warn:Text}` | Status badge, variant `warn` (yellow, exclamation icon) |
| `{crit:Text}` | Status badge, variant `crit` (red, x icon) |
| `{info:Text}` | Status badge, variant `info` (blue, i icon) |
| `{off:Text}` | Outline badge, no icon (neutral or unknown state) |
| `‹Text›` | Tag (neutral pill) |
| `` `text` `` | Monospace text |
| `**Title** · subtitle` | Two-line cell: bold title with a 12px muted line below |
| `[meter 78% "label"]` | Inline meter ([Inline meter](../03-components/06-inline-meter.md)); `[meter:warn …]` / `[meter:ok …]` force the color |
| `✓` / `—` | Yes (green check icon) / No (muted minus icon) |
| `[toggle:on]` / `[toggle:off]` | Switch control |
| `[Label](act:xyz)` | Small button that runs action `xyz` ([Interactions](../04-behavior/interactions.md)) |
| `[**Label**](act:xyz)` | Primary small button |
| `[Label](go:page/tab)` | Button or link that navigates to a sub-page |
| `🔒` | Lock icon (needs the admin password) |
| `gen[seed, base, amp, noise, min, max, end]` | Generated time series ([Sample data generator (exact)](../04-behavior/sample-data.md)) |
| Span N | Widget width in grid columns (12 = full width) |
| Row action | Clicking the whole row runs this action |

Widgets appear in the page map in grid order (left to right, top to bottom). Values shown are the **initial state** (before the user runs any fix). [Fix definitions and state changes](../04-behavior/fixes.md) lists how values change after actions.
