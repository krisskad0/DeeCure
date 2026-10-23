# Page header (sticky)

[Docs](../../README.md) › [UI design](../README.md) › [Shell](README.md)

Sticky at top 0 inside the main scroll area, `--bg` background, padding 14 0 0 20.

1. **Breadcrumb:** 12px muted: `Group › Page › Sub-page` with 12px chevrons. The page name is a button that opens the first sub-page. Pages with one sub-page show `Group › Page`. Settings has no group, so it shows `Settings › General` and so on.
2. **Title row** (margin-top 6, wraps): 32px icon tile (`--panel` background, 1px border, radius 8, page icon in `--ink-2`), H1 page title, page status badge ([Status badge](../03-components/02-status-badge.md)). Tools pages (Diagnostics, Assistant) and Settings have no status badge.
3. **Actions** (right side): page or sub-page specific buttons first (see the page map), then `Refresh` (refresh icon + label) and `Export` (download icon + label). Export is hidden on Tools pages and Settings. At ≤900px only the icons show.
4. **Tabs:** one per sub-page, margin-top 10, 1px `--line` bottom border, horizontally scrollable. Tab: padding 8 12 9, 500 `--ink-2`. The active tab is `--ink` 600 with a 2px `--accent` bottom border. Optional count pill (11px 600; neutral `--neutral-soft`/`--ink-2`, or `--crit-soft`/`--crit-ink` when marked critical). Pages with one sub-page show no tab bar, only a 10px spacer. Beginner mode shows only its sub-pages, plus the open one ([Beginner mode](sidebar.md#beginner-mode)).
