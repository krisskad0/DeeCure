# App shell

[Docs](../../README.md) › [UI design](../README.md)

```
┌──────────────────────────────── Top bar (52px) ─────────────────────────────────┐
├──────────────┬──────────────────────────────────────────────────────────────────┤
│ Sidebar      │ Page header (sticky): breadcrumb · title + status · actions      │
│ 240px        │ Tabs (sub-pages)                                                  │
│ (rail 60px)  ├──────────────────────────────────────────────────────────────────┤
│              │ Content: 12-column panel grid, gap 12                            │
│              │                                                                   │
│ Footer:      │                                                                   │
│ Settings     │                                                                   │
│ Collapse     │                                                                   │
└──────────────┴──────────────────────────────────────────────────────────────────┘
```

- `.app` is a CSS grid filling 100% height: columns `240px | 1fr` (60px in rail mode), rows `52px | 1fr`.
- Body background `--bg`. Top bar and sidebar background `--panel` with 1px `--line` borders.
- Main area scrolls on its own (`overflow:auto`), padding-right 20px.

## Content grid

- Padding 16 0 28 20. CSS grid with 12 equal columns, gap 12.
- Widgets declare a span: 3, 4, 5, 6, 7, 8, 9 or 12 columns (default 12). KPI rows and alert banners always span all 12.

## Responsive behavior

| Width | Changes |
| --- | --- |
| ≤1240px | Span 3 and 4 become 6. Spans 5, 7, 8 and 9 become 12. Prototype badge and Live indicator are hidden. |
| ≤900px | One column. Sidebar becomes an off-canvas drawer (fixed, left, width min(290px, 86vw), slides in, z-index 44), opened by the menu button and closed by clicking outside. Rail mode is ignored. Search trigger shrinks to its icon. Machine picker subtitle, time range and Collapse are hidden. Main padding is 16 on both sides. Spans 3, 4 and 6 become 12. Header action buttons show icons only. |
| ≤560px | Machine picker, avatar and wordmark are hidden. KPI rows use 2 columns. Drawer result grid uses 1 column. |

Every edge respects `env(safe-area-inset-*)`. The page body never scrolls horizontally; wide tables scroll inside their own container.

## Parts (build in this order)

| File | What it covers |
| --- | --- |
| [top-bar.md](top-bar.md) | Brand, machine picker, search, time range, mode switch, notifications |
| [sidebar.md](sidebar.md) | All 22 pages and sub-pages, item styles, rail mode |
| [page-header.md](page-header.md) | Breadcrumb, title, status badge, actions, tabs |
| [overlays.md](overlays.md) | Drawer, popovers, command palette, toast |

---

← Previous: [Foundations](../01-foundations/README.md) · Next: [Components](../03-components/README.md) →
