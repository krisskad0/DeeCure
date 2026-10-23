# Top bar (left to right)

[Docs](../../README.md) › [UI design](../README.md) › [Shell](README.md)

| # | Element | Spec |
| --- | --- | --- |
| 1 | Menu button | Only at ≤900px. 32px icon button (hamburger). Opens the sidebar drawer. |
| 2 | Brand | 28px square logo, radius 7, `--accent` background, pulse/heartbeat icon in `--accent-ink` (path `M2.5 13h4l2.2-6 3.6 11 3-8 1.5 3H21.5`, 18px, stroke 2.2). Wordmark "DeeCure". |
| 3 | Machine picker | Button 32px tall, 1px `--line` border, radius 7, margin-left 10. 8px green dot, "riya-laptop", small muted "Ubuntu 24.04.1", down chevron. Opens the Machines popover. |
| 4 | Prototype badge | "Prototype · sample data", 11.5px, `--warn-ink` on `--warn-soft`, radius 4. Hidden at ≤1240px. |
| 5 | Search trigger | Flex 1, max-width 420, margin-left auto, 32px, `--panel-2` background, border, radius 7. Search icon, muted text "Search pages, services, packages…", `kbd` "Ctrl K". Opens the command palette. |
| 6 | Time range | Segmented control: `1h`, `24h` (default), `7d`, `30d`. Hidden at ≤900px. |
| 7 | Live indicator | "Live · 21:34" with a 7px green dot and a 3px `--good-soft` ring. 12px muted. Hidden at ≤1240px. |
| 8 | View level | Segmented control: `Beginner` (default), `Advanced`, `Expert` ([View levels](../../01-product/view-levels.md)). Beginner shows fewer pages and sub-pages ([Beginner mode](sidebar.md#beginner-mode)); Advanced shows every page; Expert also shows data sources, commands, raw columns, and deep details. Hidden at ≤560px (still in Settings › General). |
| 9 | Notifications | 32px icon button (bell). Red count badge (16px, `--crit` background, white 10.5px 700) = number of open Critical and Warning issues. Opens the Notifications popover. |
| 10 | Avatar | 28px circle, `--accent-soft` background, `--accent` text, "R". |

**Segmented control:** 1px `--line` border, radius 7, padding 2, `--panel-2` background. Buttons are 26px tall, padding 0 10, radius 5, 12.5px 500 `--ink-2`. The pressed button has a `--panel` background, `--ink` text, and an inset 1px `--line-2` ring. Uses `aria-pressed`.
