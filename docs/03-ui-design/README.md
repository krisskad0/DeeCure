# UI design

[Docs](../README.md)

The full spec for the DeeCure desktop UI, split into small files. With these files, a developer can rebuild every screen exactly.

- **Source of truth:** the clickable prototype [prototype/deecure-dashboard.html](prototype/deecure-dashboard.html). Open it in a browser.
- The page specs were generated from the prototype's code, so their data matches the HTML exactly.

## Build order

Follow the folders in number order.

| Step | Folder | What you build |
| --- | --- | --- |
| 1 | [01-foundations](01-foundations/README.md) | Design tokens (colors, type, spacing), icons, and the notation used in page specs |
| 2 | [02-shell](02-shell/README.md) | Top bar, sidebar, page header, tabs, 12-column grid, overlays |
| 3 | [03-components](03-components/README.md) | 24 reusable components. Build each one once, as a component that takes a config object. |
| 4 | [04-behavior](04-behavior/README.md) | State model, page status rules, fixes, sample data, interactions |
| 5 | [05-pages](05-pages/README.md) | 22 pages and 91 sub-pages. Each widget lists its type, grid width, title, and full data. |

## Spec to code

Some parts of this spec are turned into code by `scripts/gen-ui.py`, so the app cannot drift from the docs:

| Spec | Generated file |
| --- | --- |
| [icons.md](01-foundations/icons.md) | `app/src/spec/icons.ts` |
| [sidebar.md](02-shell/sidebar.md) (pages, groups, icons, sub-pages) | `app/src/spec/pages.ts` |

Change the spec, run `scripts/gen-ui.py`, and commit both. CI runs `scripts/gen-ui.py --check`.

## Screens list

These are the screen types the UI team builds. The full page tree is in [05-pages](05-pages/README.md).

| # | Screen type | Purpose | Where in the prototype |
| --- | --- | --- | --- |
| 1 | Onboarding (3 steps) | Welcome, privacy, access level | First launch only (not in prototype) |
| 2 | First scan | Live progress by category | First launch only (not in prototype) |
| 3 | Overview dashboard | Stat tiles, charts, issues, subsystems | Overview › Summary |
| 4 | Category page (x20) | Page header, tabs, panel grid | Every sidebar page |
| 5 | Data table page | Searchable, sortable records | Processes, Services, Packages, Logs › Journal |
| 6 | Detail drawer | Inspect one row: facts, logs, actions | Click any row in Processes or Services |
| 7 | Fix drawer | Steps, commands, undo type, password, progress, result | Storage › Free up space |
| 8 | Diagnostics runner | Step-by-step check with a result | Diagnostics › Performance Check |
| 9 | Assistant chat | Questions with sources shown | Assistant › Chat |
| 10 | Activity timeline | Changes in order, with Undo | Activity › Timeline, DeeCure Actions |
| 11 | Scheduled jobs calendar | Next 24 hours lanes + run heatmap | Scheduled Jobs › Calendar |
| 12 | Settings forms | Toggles, selects, radio cards | Settings |
| 13 | Command palette | Search across everything | Ctrl K |

Screens 1 and 2 still need a design. The user journey describes them: [First launch](../01-product/user-journey.md#first-launch-3-short-screens) and [First scan](../01-product/user-journey.md#first-scan-about-3060-seconds).

## Prototype rules vs the real app

The prototype was built with these rules:

- One self-contained HTML file. Vanilla JavaScript, no framework. All charts are hand-drawn inline SVG. No chart library.
- External resource: Google Fonts only (`Ubuntu Sans` 400-700 and `Ubuntu Sans Mono` 400/500), with system fallbacks.
- Render by building HTML strings per page from a `PAGES` definition, then mounting charts after insertion. Re-render the whole main area on state change, keeping the scroll position.
- Light and dark themes through CSS custom properties ([Design tokens](01-foundations/design-tokens.md)). Dark applies through `prefers-color-scheme: dark` unless `data-theme="light"` is set on `<html>`, and also when `data-theme="dark"` is set.
- All numbers that line up in columns use `font-variant-numeric: tabular-nums`.
- Respect `prefers-reduced-motion` (disable animations and transitions).

**The real app** uses Tauri v2 + React + TypeScript ([ADR 0002](../02-architecture/decisions/0002-tauri-react-desktop-ui.md)). What changes:

- Each component becomes a React component. The look, sizes, and tokens stay exactly the same.
- The theme, tabular numbers, and reduced-motion rules still apply.
- Ship the fonts inside the app. The app must work offline, so it cannot load Google Fonts.
- Real data from the agent replaces the [sample data](04-behavior/sample-data.md). Keep the sample data for tests and demos.
- Charts are our own SVG components ([ADR 0006](../02-architecture/decisions/0006-charts-as-own-svg-components.md)).
