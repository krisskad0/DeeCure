# Buttons

[Docs](../../README.md) › [UI design](../README.md) › [Components](README.md)

| Variant | Spec |
| --- | --- |
| Default | 30px tall, padding 0 12, radius 7, 1px `--line-2` border, `--panel` background, 13px 500, `--ink`, a soft shadow (`--btn-shadow`). Hover: `--hover` background and a `--muted` border. Pressed: moves down 1px without the shadow. Keyboard focus: a 3px `--accent-soft` ring with a 1px `--accent` edge. Changes fade in 0.12 s. |
| Primary | `--accent` background and border, `--accent-ink` text. Hover: the accent mixed 12% toward `--ink` (slightly darker in light mode, lighter in dark mode). |
| Small | 26px tall, padding 0 10, radius 6, 12.5px. Used inside panels and tables. |
| Link | No border, background, or shadow; `--accent` text, padding 0 8. Hover: `--accent-soft` background (the same hover area as a button, not an underline). Used for Open, View, and similar. |
| Danger | `--crit-ink` text; hover `--crit-soft` background and `--crit` border. |
| Disabled | Opacity 0.45, not-allowed cursor, no shadow. |

Every clickable thing shows the hand cursor, whatever its class: buttons, selects, `summary` (the "Show all" and "How to fix" folds), links, and elements with `role="button"` or `role="tab"` (`styles/base.css`). Chips and the top bar's segmented control get a `--hover` background on hover.
| Icon button | 32px square, no border, radius 7, `--ink-2`; hover `--hover` background and `--ink`. |

---

← Previous: [Status badge](02-status-badge.md) · [Index](README.md) · Next: [KPI tile and KPI row](04-kpi-tile.md) →
