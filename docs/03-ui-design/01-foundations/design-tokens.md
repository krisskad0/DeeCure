# Design tokens

[Docs](../../README.md) › [UI design](../README.md) › [Foundations](README.md)

## Colors

| Token | Light | Dark | Used for |
| --- | --- | --- | --- |
| `--bg` | `#f3f4f6` | `#0b0d11` | Page background |
| `--panel` | `#ffffff` | `#14171c` | Panel, top bar, sidebar, drawer background |
| `--panel-2` | `#f7f8fa` | `#191d23` | Secondary surface: table header, inputs, chips, footers |
| `--hover` | `#eef1f5` | `#1f242b` | Row and button hover |
| `--track` | `#e8ebf0` | `#262c35` | Empty part of bars, meters and gauges |
| `--line` | `#e2e5ea` | `#252a32` | Hairline borders and dividers |
| `--line-2` | `#cfd4dc` | `#343b46` | Stronger borders: buttons, inputs, outline badges |
| `--ink` | `#111620` | `#e8ebef` | Primary text |
| `--ink-2` | `#465061` | `#aab2bf` | Secondary text: labels, table headers |
| `--muted` | `#768092` | `#7d8695` | Muted text: subtexts, axis labels |
| `--accent` | `#2563c9` | `#5f9df0` | Primary action, active nav item, links, focus ring |
| `--accent-ink` | `#ffffff` | `#0b0d11` | Text on accent |
| `--accent-soft` | `#e8f0fc` | `#16243a` | Active nav background, selected card |
| `--s1` | `#2a78d6` | `#3987e5` | Series slot 1 (blue) |
| `--s2` | `#eb6834` | `#d95926` | Series slot 2 (orange) |
| `--s3` | `#1baf7a` | `#199e70` | Series slot 3 (aqua) |
| `--s4` | `#eda100` | `#c98500` | Series slot 4 (yellow) |
| `--s5` | `#e87ba4` | `#d55181` | Series slot 5 (magenta) |
| `--good` | `#0ca30c` | `#0ca30c` | Healthy mark (dot, gauge) |
| `--good-ink` | `#14742b` | `#5fcf6a` | Healthy text |
| `--good-soft` | `#e4f4e6` | `#12261a` | Healthy badge background |
| `--warn` | `#fab219` | `#fab219` | Warning mark |
| `--warn-ink` | `#8a5900` | `#f6c457` | Warning text |
| `--warn-soft` | `#fdf2da` | `#2c2411` | Warning badge background |
| `--crit` | `#d03b3b` | `#d03b3b` | Critical mark |
| `--crit-ink` | `#b42525` | `#f08585` | Critical text |
| `--crit-soft` | `#fbe7e7` | `#331919` | Critical badge background |
| `--info-ink` | `#2457ad` | `#8fb8f6` | Info text |
| `--info-soft` | `#e8f0fc` | `#16243a` | Info badge background |
| `--neutral-soft` | `#eceef2` | `#222831` | Neutral tag and pill background |
| `--grid` | `#eceef2` | `#1f242b` | Chart gridlines |
| `--axis` | `#c9ced6` | `#384049` | Chart baseline |
| `--h1` | `#e3edfb` | `#17202d` | Heatmap level 1 (lowest) |
| `--h2` | `#b7d3f6` | `#1c3a63` | Heatmap level 2 |
| `--h3` | `#6da7ec` | `#255da0` | Heatmap level 3 |
| `--h4` | `#2a78d6` | `#3987e5` | Heatmap level 4 |
| `--h5` | `#184f95` | `#86b6ef` | Heatmap level 5 (highest) |
| `--scrim` | `rgba(17,22,32,.42)` | `rgba(0,0,0,.6)` | Overlay backdrop |
| `--shadow` | `0 12px 32px -8px rgba(17,22,32,.25), 0 2px 6px rgba(17,22,32,.08)` | `0 12px 32px -8px rgba(0,0,0,.7), 0 2px 6px rgba(0,0,0,.4)` | Overlay shadow |

Dark values apply under `@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) { … } }` and again under `:root[data-theme="dark"]`, both with `color-scheme: dark`. `body` sets `background: var(--bg); color: var(--ink)`.

**Color rules**

- Series colors `--s1`…`--s5` are used in fixed order for chart series (slot 1 = first series). They were checked for color-blind separation.
- Status colors (`--good`, `--warn`, `--crit`, and their `-ink`/`-soft` pairs) are only used for state, never for data series.
- Text never uses series colors. Labels use `--ink`, `--ink-2`, or `--muted`.
- Heatmap uses the sequential ramp `--h1` (lowest) to `--h5` (highest).

## Typography

| Role | Font | Size | Weight | Other |
| --- | --- | --- | --- | --- |
| Base body | Ubuntu Sans | 13.5px | 400 | line-height 1.45, antialiased |
| Page title (H1) | Ubuntu Sans | 20px | 700 | letter-spacing −0.01em |
| Drawer title | Ubuntu Sans | 16px | 700 | |
| Brand wordmark | Ubuntu Sans | 16px | 700 | letter-spacing −0.01em |
| Panel title | Ubuntu Sans | 13.5px | 600 | ellipsis |
| KPI value | Ubuntu Sans | 24px | 600 | letter-spacing −0.015em |
| KPI unit | Ubuntu Sans | 13px | 500 | muted, 3px left margin |
| Gauge value | Ubuntu Sans | 30px | 600 | SVG text |
| Donut center value | Ubuntu Sans | 20px | 600 | SVG text |
| Table body | Ubuntu Sans | 13px | 400 | |
| Table header | Ubuntu Sans | 12px | 600 | `--ink-2` |
| Labels, subtexts | Ubuntu Sans | 12px | 400–500 | `--muted` |
| Sidebar group label | Ubuntu Sans | 10.5px | 700 | uppercase, letter-spacing 0.08em |
| Drawer section label | Ubuntu Sans | 11px | 700 | uppercase, letter-spacing 0.07em |
| Badge | Ubuntu Sans | 11.5px | 600 | line-height 20px |
| Chart axis text | Ubuntu Sans | 11px | 400 | `--muted` |
| Code, paths, units, IDs | Ubuntu Sans Mono | 12–12.5px | 400 | |

Font stacks: `"Ubuntu Sans", system-ui, -apple-system, "Segoe UI", Roboto, sans-serif` and `"Ubuntu Sans Mono", ui-monospace, "DejaVu Sans Mono", Menlo, Consolas, monospace`.

## Shape and spacing

| Token | Value |
| --- | --- |
| Panel / tile radius | 8px |
| Button radius | 6px |
| Badge radius | 4px |
| Grid gap | 12px |
| Icon size | 16px default (sidebar 17px), stroke 1.8, round caps and joins, no fill |
| Focus ring | 2px solid `--accent`, offset 1px |
| Shadow (overlays) | `--shadow` |
