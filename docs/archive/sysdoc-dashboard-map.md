# SysDoc Dashboard: Complete UI Map (clone spec)

As of Oct 22, 2026 · Source of truth: `sysdoc-dashboard.html` · Companions: `sysdoc-implementation-plan.md`, `linux-system-insights.md`

This document describes the SysDoc dashboard prototype completely: design tokens, layout, every component, every page and sub-page, every widget with its exact data, and every interaction. An AI or developer can rebuild the prototype from this file alone. The page map in section 12 is generated directly from the prototype's code, so its data matches the HTML exactly.

---

## 0. How to use this document

1. Build the **shell** (sections 2-4): top bar, sidebar, page header, tabs, 12-column grid.
2. Build the **components** (section 5) once, as reusable functions that take a config object.
3. Add the **state model and logic** (sections 7-9).
4. Fill in **every page** from the page map (section 12). Each widget lists its type, grid width, title, and full data.
5. Wire the **interactions** (section 10).

**Build rules**

- One self-contained HTML file. Vanilla JavaScript, no framework. All charts are hand-drawn inline SVG. No chart library.
- External resource: Google Fonts only (`Ubuntu Sans` 400-700 and `Ubuntu Sans Mono` 400/500), with system fallbacks.
- Render by building HTML strings per page from a `PAGES` definition, then mounting charts after insertion. Re-render the whole main area on state change, keeping the scroll position.
- Light and dark themes through CSS custom properties (section 1). Dark applies through `prefers-color-scheme: dark` unless `data-theme="light"` is set on `<html>`, and also when `data-theme="dark"` is set.
- All numbers that line up in columns use `font-variant-numeric: tabular-nums`.
- Respect `prefers-reduced-motion` (disable animations and transitions).

---

## 1. Design tokens

### 1.1 Colors

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

### 1.2 Typography

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
| Labels, subtexts | Ubuntu Sans | 12px | 400-500 | `--muted` |
| Sidebar group label | Ubuntu Sans | 10.5px | 700 | uppercase, letter-spacing 0.08em |
| Drawer section label | Ubuntu Sans | 11px | 700 | uppercase, letter-spacing 0.07em |
| Badge | Ubuntu Sans | 11.5px | 600 | line-height 20px |
| Chart axis text | Ubuntu Sans | 11px | 400 | `--muted` |
| Code, paths, units, IDs | Ubuntu Sans Mono | 12-12.5px | 400 | |

Font stacks: `"Ubuntu Sans", system-ui, -apple-system, "Segoe UI", Roboto, sans-serif` and `"Ubuntu Sans Mono", ui-monospace, "DejaVu Sans Mono", Menlo, Consolas, monospace`.

### 1.3 Shape and spacing

| Token | Value |
| --- | --- |
| Panel / tile radius | 8px |
| Button radius | 6px |
| Badge radius | 4px |
| Grid gap | 12px |
| Icon size | 16px default (sidebar 17px), stroke 1.8, round caps and joins, no fill |
| Focus ring | 2px solid `--accent`, offset 1px |
| Shadow (overlays) | `--shadow` |

---

## 2. App shell

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

### 2.1 Top bar (left to right)

| # | Element | Spec |
| --- | --- | --- |
| 1 | Menu button | Only at ≤900px. 32px icon button (hamburger). Opens the sidebar drawer. |
| 2 | Brand | 28px square logo, radius 7, `--accent` background, pulse/heartbeat icon in `--accent-ink` (path `M2.5 13h4l2.2-6 3.6 11 3-8 1.5 3H21.5`, 18px, stroke 2.2). Wordmark "SysDoc". |
| 3 | Machine picker | Button 32px tall, 1px `--line` border, radius 7, margin-left 10. 8px green dot, "riya-laptop", small muted "Ubuntu 24.04.1", down chevron. Opens the Machines popover. |
| 4 | Prototype badge | "Prototype · sample data", 11.5px, `--warn-ink` on `--warn-soft`, radius 4. Hidden at ≤1240px. |
| 5 | Search trigger | Flex 1, max-width 420, margin-left auto, 32px, `--panel-2` background, border, radius 7. Search icon, muted text "Search pages, services, packages…", `kbd` "Ctrl K". Opens the command palette. |
| 6 | Time range | Segmented control: `1h`, `24h` (default), `7d`, `30d`. Hidden at ≤900px. |
| 7 | Live indicator | "Live · 21:34" with a 7px green dot and a 3px `--good-soft` ring. 12px muted. Hidden at ≤1240px. |
| 8 | Mode | Segmented control: `Beginner` (default), `Expert`. |
| 9 | Notifications | 32px icon button (bell). Red count badge (16px, `--crit` background, white 10.5px 700) = number of open Critical and Warning issues. Opens the Notifications popover. |
| 10 | Avatar | 28px circle, `--accent-soft` background, `--accent` text, "R". |

**Segmented control:** 1px `--line` border, radius 7, padding 2, `--panel-2` background. Buttons are 26px tall, padding 0 10, radius 5, 12.5px 500 `--ink-2`. The pressed button has a `--panel` background, `--ink` text, and an inset 1px `--line-2` ring. Uses `aria-pressed`.

### 2.2 Sidebar

22 pages and 91 sub-pages. Group order and page order are exactly as listed.

| Group | Page | ID | Icon | Initial status | Sidebar mark | Sub-pages (ID: label) |
| --- | --- | --- | --- | --- | --- | --- |
| Monitoring | Overview | `overview` | `grid` | {crit:Critical} | Count pill (9) | `summary`: Summary · `issues`: Issues (count 9, red) · `score`: Health Score |
| Monitoring | Performance | `performance` | `perf` | {ok:Healthy} | - | `cpu`: CPU · `memory`: Memory · `disk`: Disk I/O · `network`: Network · `gpu`: GPU · `pressure`: Pressure (PSI) |
| Monitoring | Logs | `logs` | `logs` | {warn:Warning} | Yellow dot | `journal`: Journal · `errors`: Errors (count 37) · `crashes`: Crashes · `kernel`: Kernel · `boots`: Boots |
| Monitoring | Activity | `activity` | `hist` | {ok:Healthy} | - | `timeline`: Timeline · `packages`: Package History · `config`: Config Changes · `devices`: Device Events · `sysdoc`: SysDoc Actions |
| System | System | `system` | `sys` | {info:Info} | - | `overview`: Overview · `boot`: Boot · `updates`: Updates (count 12) · `kernel`: Kernel · `time`: Time & Locale |
| System | Hardware | `hardware` | `chip` | {ok:Healthy} | - | `summary`: Summary · `processor`: Processor · `memory`: Memory · `storage`: Storage Devices · `graphics`: Graphics · `sensors`: Sensors · `peripherals`: Peripherals · `firmware`: Firmware |
| System | Power | `power` | `bat` | {warn:Warning} | Yellow dot | `battery`: Battery · `profile`: Power Profile · `sleep`: Sleep |
| System | Display & Audio | `display` | `mon` | {ok:Healthy} | - | `displays`: Displays · `audio`: Audio |
| Workloads | Processes | `processes` | `layers` | {ok:Healthy} | - | `all`: All Processes · `top`: Top Consumers |
| Workloads | Services | `services` | `cog` | {crit:Critical} | Red dot | `system`: System Services (count 1, red) · `user`: User Services · `startup`: Startup Applications |
| Workloads | Scheduled Jobs | `jobs` | `cal` | {ok:Healthy} | - | `calendar`: Calendar · `cron`: Cron · `timers`: Systemd Timers · `anacron`: Anacron · `at`: At Jobs |
| Workloads | Containers | `containers` | `cube` | {warn:Warning} | Yellow dot | `containers`: Containers · `images`: Images · `volumes`: Volumes · `vms`: Virtual Machines |
| Software & Data | Packages | `packages` | `pkg` | {warn:Warning} | Yellow dot | `installed`: Installed · `sources`: Sources · `cleanup`: Cleanup · `permissions`: Permissions |
| Software & Data | Storage | `storage` | `db` | {crit:Critical} | Red dot | `overview`: Overview · `filesystems`: Filesystems · `usage`: Disk Usage · `cleanup`: Cleanup |
| Software & Data | Backups | `backups` | `backup` | {warn:Warning} | Yellow dot | `jobs`: Backup Jobs · `snapshots`: System Snapshots · `restore`: Restore Points |
| Network & Access | Network | `network` | `wifi` | {ok:Healthy} | - | `interfaces`: Interfaces · `connections`: Connections · `ports`: Open Ports · `firewall`: Firewall · `dns`: DNS & VPN · `traffic`: Traffic |
| Network & Access | Accounts | `accounts` | `users` | {ok:Healthy} | - | `users`: Users · `groups`: Groups · `sessions`: Sessions · `logins`: Login History · `ssh`: SSH Keys |
| Network & Access | Security | `security` | `shield` | {warn:Warning} | Yellow dot | `overview`: Overview · `vulns`: Vulnerabilities (count 3) · `hardening`: Hardening · `integrity`: File Integrity · `malware`: Malware Scan |
| Network & Access | Privacy | `privacy` | `eye` | {ok:Healthy} | - | `access`: Device Access · `perms`: App Permissions · `settings`: Privacy Settings |
| Tools | Diagnostics | `diagnostics` | `gauge` | none (no badge) | - | `performance`: Performance Check · `network`: Network Test · `hardware`: Hardware Tests |
| Tools | Assistant | `assistant` | `chat` | none (no badge) | - | `chat`: Chat |
| Footer | Settings | `settings` | `sliders` | none (no badge) | - | `general`: General · `access`: Access · `notifications`: Notifications · `assistant`: Assistant · `data`: Data · `about`: About |


**Sidebar item spec**

- Group label: 10.5px 700 uppercase muted, padding 0 10 5. Groups have 12px top margin (4px for the first).
- Item: 32px tall, gap 10, padding 0 8 0 10, radius 6, `--ink-2` 500. Icon 17px. Hover: `--hover` background. Active: `--accent-soft` background, `--accent` text, 600.
- Right side: Overview shows a count pill (number of open issues; `--crit-soft` background, `--crit-ink`, 11px 700, 18px tall). Other pages show a 7px dot if their status is Warning (yellow) or Critical (red). Pages with more than one sub-page show a 14px chevron that rotates 90° when open.
- Sub-tree (open page only): margin-left 18, padding-left 10, 1px `--line` left border. Sub-items 13px, padding 4 8, radius 5. The active sub-page is `--accent` 600.
- Behavior: clicking a different page navigates to its first sub-page and opens its tree. Clicking the current page toggles its tree. Each page remembers whether its tree is open.
- Footer (pinned to the bottom with a top border): Settings item, then a "Collapse" button with a panel icon. Collapse toggles **rail mode**: 60px wide, icons only, no labels, group labels, sub-trees, chevrons or counts. Status dots move to the item's top-right corner. Groups are separated by a 1px top border. The button label changes to "Expand".

### 2.3 Page header (sticky)

Sticky at top 0 inside the main scroll area, `--bg` background, padding 14 0 0 20.

1. **Breadcrumb:** 12px muted: `Group › Page › Sub-page` with 12px chevrons. The page name is a button that opens the first sub-page. Pages with one sub-page show `Group › Page`. Settings has no group, so it shows `Settings › General` and so on.
2. **Title row** (margin-top 6, wraps): 32px icon tile (`--panel` background, 1px border, radius 8, page icon in `--ink-2`), H1 page title, page status badge (section 5.2). Tools pages (Diagnostics, Assistant) and Settings have no status badge.
3. **Actions** (right side): page or sub-page specific buttons first (see the page map), then `Refresh` (refresh icon + label) and `Export` (download icon + label). Export is hidden on Tools pages and Settings. At ≤900px only the icons show.
4. **Tabs:** one per sub-page, margin-top 10, 1px `--line` bottom border, horizontally scrollable. Tab: padding 8 12 9, 500 `--ink-2`. The active tab is `--ink` 600 with a 2px `--accent` bottom border. Optional count pill (11px 600; neutral `--neutral-soft`/`--ink-2`, or `--crit-soft`/`--crit-ink` when marked critical). Pages with one sub-page show no tab bar, only a 10px spacer.

### 2.4 Content grid

- Padding 16 0 28 20. CSS grid with 12 equal columns, gap 12.
- Widgets declare a span: 3, 4, 5, 6, 7, 8, 9 or 12 columns (default 12). KPI rows and alert banners always span all 12.

### 2.5 Responsive behavior

| Width | Changes |
| --- | --- |
| ≤1240px | Span 3 and 4 become 6. Spans 5, 7, 8 and 9 become 12. Prototype badge and Live indicator are hidden. |
| ≤900px | One column. Sidebar becomes an off-canvas drawer (fixed, left, width min(290px, 86vw), slides in, z-index 44), opened by the menu button and closed by clicking outside. Rail mode is ignored. Search trigger shrinks to its icon. Machine picker subtitle, time range and Collapse are hidden. Main padding is 16 on both sides. Spans 3, 4 and 6 become 12. Header action buttons show icons only. |
| ≤560px | Machine picker, avatar and wordmark are hidden. KPI rows use 2 columns. Drawer result grid uses 1 column. |

Every edge respects `env(safe-area-inset-*)`. The page body never scrolls horizontally; wide tables scroll inside their own container.

---

## 3. Page status rules

Each page has a status shown as a badge in its header and as a dot in the sidebar.

| Page | Status rule |
| --- | --- |
| Overview | Critical if any open issue is Critical, else Warning |
| Storage | Disk status: Critical if used ≥93%, Warning if ≥85%, else Healthy |
| Services | Critical, or Warning after Bluetooth is restarted |
| Security | Warning, or Healthy after security updates are installed |
| Packages | Warning, or Info after security updates are installed |
| Power, Backups, Logs, Containers | Warning |
| System | Info |
| All other pages | Healthy |

Status words: `ok` = Healthy, `warn` = Warning, `crit` = Critical, `info` = Info, `off` = Unknown.

---

## 4. Overlays

| Overlay | Trigger | Spec |
| --- | --- | --- |
| Scrim | Behind drawers and the palette | Fixed full screen, `--scrim`, z-index 40. Clicking it closes (except while a fix is running). |
| Drawer | Fix actions, Inspect, row click | Fixed right, full height, width min(480px, 100%), `--panel`, 1px left border, shadow, z-index 41. Header (padding 14 16, bottom border): title 16px 700 (wraps), subtitle 12.5px muted, close icon button on the right. Body scrolls (padding 14 16, gap 14). Footer (`--panel-2`, top border, padding 12 16): note text on the left (12px muted), buttons on the right. |
| Popover | Bell, machine picker | Fixed under the trigger (right-aligned for the bell, left-aligned for the machine picker), width min(360px, 100vw − 24px), radius 9, shadow, z-index 30. A transparent full-screen layer behind it closes it on click. |
| Command palette | Search trigger, Ctrl/Cmd + K | Fixed, top 12vh, centered, width min(560px, 100% − 32px), radius 12, shadow, z-index 45, over the scrim. Input row 48px with a search icon, 15px input and an `Esc` kbd. Results list max-height min(380px, 60vh). |
| Toast | Many actions | Fixed bottom center (20px + safe area), `--ink` background, `--panel` text, 13px, radius 8, padding 9 14, z-index 60. Auto-hides after 3.2 s. One at a time. |

**Notifications popover:** header "Notifications" + muted "N open". The first 6 issues: a colored dot by severity, bold title, small "Category · First seen". Clicking an issue runs its action. Footer link "View all issues" opens Overview › Issues.

**Machines popover:** header "Machines". Item 1: green dot, **riya-laptop**, "This computer · Ubuntu 24.04.1 LTS" (closes). Item 2: external-link icon, **Add a machine**, "Planned · small office view" → toast "Multi-machine view is planned for a later phase."

**Command palette results** (case-insensitive substring match, max 40, right-aligned kind label):

- Every page and sub-page as `Page › Sub-page` (or just `Page` when it has one sub-page), kind "Page".
- Services: bluetooth.service, docker.service, NetworkManager.service, snapd.service, cups-browsed.service → Services › System Services.
- Packages: the first 14 package names from Packages › Installed → Packages › Installed.
- Jobs: sync-notes.sh, backup-photos.sh → Scheduled Jobs › Cron; logrotate.timer, e2scrub_all.timer → Scheduled Jobs › Systemd Timers.
- Keyboard: ↑/↓ moves the highlight (wraps, scrolls into view), Enter opens, Esc closes. The highlighted row has an `--accent-soft` background.

---

## 5. Components

### 5.1 Panel

- `--panel` background, 1px `--line` border, radius 8, vertical flex.
- **Header** (optional, min-height 42, padding 0 12 0 14, bottom border): title (13.5px 600), optional subtitle (12px muted, e.g. "last 7 days"), optional info icon (16px circled "i", muted; hover/focus shows a tooltip: `--ink` background, `--panel` text, 12px, max-width 260px, radius 6, below-left). Right side: **source chip** (mono 11.5px, `--panel-2`, 1px border, radius 4, max-width 260 with ellipsis; **visible only in Expert mode**), then small action buttons.
- **Body:** padding 12 14 14. Tables, lists, checklists, timelines and forms use a flush body (padding 0).

### 5.2 Status badge

Inline-flex, gap 4, padding 0 7 0 5, radius 4, 11.5px 600, line-height 20px, with a 12px icon (stroke 2.6).

| Variant | Background | Text | Icon |
| --- | --- | --- | --- |
| `ok` | `--good-soft` | `--good-ink` | check |
| `warn` | `--warn-soft` | `--warn-ink` | exclamation |
| `crit` | `--crit-soft` | `--crit-ink` | x |
| `info` | `--info-soft` | `--info-ink` | i |
| `off` | transparent + inset 1px `--line-2` ring | `--muted` | none |

**Tag:** padding 0 6, radius 4, 11.5px, line-height 19px, `--neutral-soft` / `--ink-2`. Used for group names and features.

### 5.3 Buttons

| Variant | Spec |
| --- | --- |
| Default | 30px tall, padding 0 12, radius 6, 1px `--line-2` border, `--panel` background, 13px 500, `--ink`. Hover `--hover` background. |
| Primary | `--accent` background and border, `--accent-ink` text. Hover brightness 1.07. |
| Small | 26px tall, padding 0 9, 12.5px. Used inside panels and tables. |
| Link | No border or background, `--accent` text, padding 0 6, underline on hover. |
| Danger | `--crit-ink` text. |
| Disabled | Opacity 0.45, not-allowed cursor. |
| Icon button | 32px square, no border, radius 7, `--ink-2`; hover `--hover` background and `--ink`. |

### 5.4 KPI tile and KPI row

- **Row:** full width, grid `repeat(auto-fit, minmax(168px, 1fr))`, gap 12. 2 columns at ≤560px.
- **Tile:** `--panel`, 1px border, radius 8, padding 11 14 10, vertical stack:
  1. Label row (min-height 20): label 12px 500 `--ink-2` (ellipsis) + status badge on the right if the status is not `ok`.
  2. Value 24px 600 + unit (13px 500 muted).
  3. Subtext 12px muted (ellipsis), or a non-breaking space when empty.
  4. Visual, one of: **sparkline** (SVG 100% × 28px, `preserveAspectRatio="none"`, min-max normalized, line 1.6px in the series color with non-scaling stroke, area at 12% opacity), or **usage bar** (6px, `--track`, radius 3, fill `--s1`, or `--warn`/`--crit` when the tile status is warn/crit; 10px top margin, 8px bottom).
- Tiles with a link are `<button>`s that navigate. Hover: `--line-2` border, `--panel-2` background.
- Sparklines use the same generator as charts (section 9) with the current time range.

### 5.5 Data table

- **Tools bar** (only if the table has search, filter or selection): padding 8 12, bottom border, wraps.
  - Search box: 28px, `--panel-2`, 1px border, radius 6, 14px search icon, width 180-260px, placeholder from the spec. Matches any cell's text (case-insensitive substring).
  - Filter chips: 26px pills (radius 13, 1px border, 12.5px `--ink-2`). Pressed chip: `--ink` background, `--panel` text. "All" plus options; a row matches when the chosen column's text **starts with** the option (case-insensitive).
  - Selection bar (right): "N selected" or "Select rows", plus a small button (primary when at least one row is selected).
- **Table:** 100% wide, collapsed borders, 13px.
  - Header cells: `--panel-2`, 12px 600 `--ink-2`, padding 7 12, bottom border, no wrap. Columns with a header label are sortable. Click toggles ascending/descending, with an `↑`/`↓` arrow in `--accent`.
  - Body cells: padding 7 12, bottom border (none on the last row), no wrap, vertically centered. Row hover `--hover`.
  - Column flags: `n` = numeric (right-aligned, tabular), `x` = Expert-only (hidden in Beginner mode), `w` = wrap (normal wrapping, min-width 260px).
  - Selection: 34px checkbox column, header checkbox selects all on the current page.
  - Clickable rows (pointer cursor): rows with a row action run it; tables marked "Inspect" open the row-inspect drawer (section 10.6).
- **Sorting:** parse a leading number with units (KB, kB, K = ×0.001, MB = ×1, GB = ×1000, TB = ×1,000,000, plus ms, s, %, h, m, W). Compare numerically when both values parse, otherwise compare text.
- **Paging:** default 12 rows per page (some tables differ). Footer (top border, padding 6 12, 12px muted): "1-12 of N" and Previous/Next small buttons when there is more than one page. When the spec gives a total count, "of N" shows that total while no search or filter is active.
- **Empty:** one full-width cell, centered muted text, padding 28 14. Text comes from the spec, default "No matching rows".
- Wide tables scroll horizontally inside the panel.

### 5.6 Inline meter

Inline-flex, gap 8, min-width 130. Track 6px tall, flex 1, min-width 56, radius 3, `--track`. Fill `--s1`; `--warn` when ≥80%, `--crit` when ≥90%, unless a variant is set explicitly. Label on the right, min-width 38, right-aligned, tabular. Width = value %, capped at 100.

### 5.7 Key-value list

Two-column grid: `minmax(120px, 40%) | 1fr`. Rows padding 7 0, bottom border except the last row. Key `--muted`, value `--ink` (wraps anywhere).

### 5.8 Status list

Rows: grid `1fr | auto | auto`, gap 10, padding 6 14, bottom border. Label: 16px muted icon + 500 text. Metric: 12.5px muted. Status badge. Rows that link are full-width buttons with a hover background.

### 5.9 Checklist

Rows: grid `22px | 1fr | auto`, gap 10, padding 8 14, bottom border. 20px circle icon (12px glyph, stroke 2.8): `pass` = check on `--good-soft`/`--good-ink`, `fail` = x on crit colors, `warn` = exclamation on warn colors, `na` = minus on `--neutral-soft`/`--muted`. Title + optional 12px muted second line. Optional small button on the right.

### 5.10 Activity timeline

List padding 4 14 8. Each row: grid `62px | 14px | 1fr`, gap 8, padding 7 0. Time 12px muted tabular. A 9px dot (top margin 5, 2px `--panel` ring) colored `--crit`, `--warn`, `--good`, `--s1` (info) or `--line-2` (default). A 1px `--line` connector runs down from the dot to the next row (none on the last). Title 600 + 12px muted second line.

### 5.11 Alert banner

Full width. Flex, gap 12, padding 10 14, radius 8, 1px border, `--panel`. 28px circle icon tinted by state (`*-soft` background, `*-ink` icon). Title 600 + 12.5px muted text. Small buttons on the right.

### 5.12 Line / area chart

- SVG sized to the container width × the specified height (default 200). Margins: left 46, right 12 (74 with a forecast), top 8, bottom 22.
- Y axis: 5 evenly spaced horizontal lines from min to max. The bottom line uses `--axis`, the others `--grid`. Labels 11px muted, right-aligned at x = left − 7. Format: rounded when ≥10 or an integer, else 1 decimal, plus the unit. Max `auto` = the "nice" ceiling of max(data) × 1.15 (1, 2, 2.5, 5, 10 × 10ⁿ).
- X axis labels: for time series, 5 labels at 0, 25, 50, 75 and 100% of the points, the last one "Now". Format by range: 1h → `HH:MM`, 24h → `HH:MM`, 7d → weekday, 30d → `D Mon`. For label arrays, show every ceil(n/6)-th label plus the last. Labels within 24px of an edge are anchored start or end.
- Series: 2px lines with round caps and joins in `--sN`. Area under each line at 10% opacity (20% when stacked). Stacked series add their values; each area fills down to the series below it. Series are drawn from top to bottom so the first series sits on top.
- Threshold (optional): full-width dashed line (4 4, 1px muted) with a right-aligned label 5px above.
- Forecast (optional, `[daysAhead, value, label]`): the x-domain extends by `daysAhead` points. Dashed `--crit` line (5 4, 2px) from the last point to the forecast point, a 4.5px `--crit` dot with a 2px `--panel` ring, and a bold `--ink-2` label to its right.
- End markers: a 4px dot with a 2px `--panel` ring at the last point of each series.
- Hover/tap: the nearest point index snaps; a vertical crosshair (`--line-2`) and a ring dot on each series appear. Tooltip (`--ink` background, `--panel` text, 12px, radius 7, padding 7 9, shadow) sits centered above the highest point: a header (time label, e.g. `Wed 21:15`, or the label array value) and one row per series (8px color square, "Name: value" or just the value for a single series). Hidden on pointer leave.
- Legend: shown above the chart only when there are 2 or more series: 12×3px color line + name, 12px `--ink-2`, gap 14.

### 5.13 Bar chart

- Margins left 42, right 8, top 18, bottom 22. Y max = given, or the nice ceiling of max × 1.1. 5 gridlines with labels (integer or 1 decimal, plus unit).
- Bars: width min(24px, slot − 2), centered in their slot, rounded top corners (radius ≤4), square base. Color `--s1`. Highlighted indices use `--warn` and show their value as a bold label 5px above the bar. A bar can override its color (`kn` = `--track` grey for empty or skipped values).
- X labels thinned to fit (every ceil(n / max(4, floor(width / 46)))), always including the last.
- Hover/tap anywhere in a bar's slot: tooltip with the bar's long label (or short label) and value + unit.

### 5.14 Donut chart

- 150×150 SVG. Outer radius 60, inner 42. 0.02 rad gap between segments. Starts at 12 o'clock and runs clockwise.
- Center: value (20px 600 `--ink`) at y 78, subtitle (11px muted) at y 95.
- Colors: series slots 1-5 in order, unless a segment overrides: `kw` = `--warn`, `kc` = `--crit`, `kn` = `--track`.
- Legend to the right (wraps under on narrow widths): rows of 10px rounded swatch | name | value (+ unit, or a custom display value) | percent (muted, tabular, right-aligned).
- Each segment has an SVG `<title>` tooltip.

### 5.15 Gauge

- Semicircle SVG, viewBox 200×112. Arc from (20,96) to (180,96), radius 80, stroke 14, round caps. Track `--track`. Value arc in `--s1`, or `--good`/`--warn`/`--crit` when a status is set. The value fraction is clamped to 0.001-1.
- Value text 30px 600 at (100, 86). Subtitle 11.5px muted at (100, 106). Max width 220px, centered.
- **Gauge group:** grid `repeat(auto-fit, minmax(140px, 1fr))`, gap 4 12, with a 12px `--ink-2` caption under each gauge.

### 5.16 Heatmap (day × hour)

- CSS grid: a label column + 24 hour columns (min 5px each), gap 2px.
- Header row: hour labels "00", "03", "06" … (every 3rd), 11px muted, centered.
- Cells 18px tall, radius 3. Level: value 0 → `--h1`, otherwise `2 + floor(value / max × 3.99)`, capped at 5 → `--h2`…`--h5`. Tooltip `"<Day> <HH>:00 · <value> <unit>"`.
- Legend under the grid, right-aligned: "Less" + five 14×10 swatches (h1-h5) + "More".

### 5.17 Horizontal bar list

Rows: grid `minmax(90px, 38%) | 1fr | 70px`, gap 10, padding 5 0. Label (ellipsis, may be monospace). Bar 10px track with right corners rounded 4, fill `--s1` (or `--warn`), width = value / max. Value right-aligned, tabular, `--ink-2`.

### 5.18 Allocation bar (disk)

26px tall, radius 6, segments with 2px gaps, width = GB / total GB. Colors: Home `--s1`, Apps `--s2`, System `--s3`, Containers `--s4`, Reclaimable = 135° stripes (`--crit` 4px / `--crit-soft` 4px), Free = `--track`. Legend below: 10px swatches + name + bold GB value, wraps, gap 6 18.

### 5.19 Swim-lane schedule (next 24 hours)

Scroll container with a min-width 620 inner. Rows 28px: grid `160px | 1fr`, gap 10. Name in mono 12.5px `--ink-2`. Track: 1px left border, 1px `--grid` horizontal center line. Each run is a 10px `--s1` circle with a 2px `--panel` ring at `left = hoursFromNow / 24`. Very frequent jobs use 6px dots without a ring. Axis row: labels at hour offsets 0 "Now 21:34", 2.43 "00:00", 8.43 "06:00", 14.43 "12:00", 20.43 "18:00".

### 5.20 Lifecycle bar

10px track, radius 5, top margin 26. Fill `--s1` to 50.4%. 2px `--ink` marker at 50.4% (extends 6px above and below) with the label "Today" (11.5px 600) above it. Axis row below: left "Released · Apr 2024", right "Standard support ends · Apr 2029" (12px muted).

### 5.21 Form controls

- **Switch:** 36×20, track `--line-2` (off) or `--accent` (on), 16px white knob with a light shadow, 16px travel, 0.15s transition. A real checkbox overlays it.
- **Select:** 30px tall, 1px `--line-2` border, radius 6, `--panel` background, padding 0 8.
- **Radio cards:** grid `repeat(auto-fit, minmax(200px, 1fr))`, gap 10, padding 12 14. Card: 1px border, radius 7, padding 10 12, radio + bold title + 12px muted description. Checked: `--accent` border, `--accent-soft` background.
- **Field row:** grid `1fr | auto`, gap 16, padding 10 14, bottom border: bold label + 12px muted description | control.
- **State tiles:** grid `repeat(auto-fit, minmax(150px, 1fr))`, gap 10. Tile: 1px border, radius 7, padding 10 12, 32px icon square (`--neutral-soft`) + bold title + 12px muted state.

### 5.22 Code and log blocks

- **Command block:** mono 12px, `--panel-2`, 1px border, radius 5, padding 4 8, preformatted, scrolls horizontally, prefixed with a muted `$ `.
- **Log line:** mono 12px, `--panel-2`, 1px border, radius 5, padding 6 9, preformatted, scrolls horizontally.
- Inline monospace values (paths, versions, unit names, commands) use the mono font at 12.5px.

### 5.23 Diagnostics runner

- **Idle:** step grid (`repeat(auto-fit, minmax(170px, 1fr))`, gap 8). Each step: 1px border, radius 6, padding 8 10, a grey 8px dot, muted name. Below it, a primary button with a play icon.
- **Running:** a 6px progress bar (`--accent` fill, 0.35s width transition) above the step grid. Finished steps show a 20px green check circle and `--ink` text; the current step a 14px spinner (2px ring, `--accent` top, 0.8s rotation); later steps a grey dot. Steps advance every 450ms.
- **Done:** the panel shows the result as an inline alert (no border or padding), adds a "Run again" action to the header, and extra widgets appear below (see the page map).

### 5.24 Chat

- Message column: gap 12, padding 14, min-height 360, max-height 520, scrolls, and auto-scrolls to the bottom after render.
- User bubble: right-aligned, `--accent` background, `--accent-ink` text, padding 8 12, radius 12 12 3 12, max-width 78%.
- Assistant bubble: left-aligned, `--panel-2`, 1px border, radius 12 12 12 3, padding 10 12, max-width 92%, gap 8. Contains the answer paragraphs, an optional **Sources** block (dashed top border, 11px 700 uppercase muted label "Sources", log lines) and an optional small primary action button.
- Waiting bubble: assistant style with muted "Reading logs…", replaced after 700ms.
- Composer (top border, padding 10 14, gap 8): input 34px (radius 7, placeholder "Ask about this computer…") + primary "Send".

---

## 6. Notation used in the page map

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
| `[meter 78% "label"]` | Inline meter (section 5.6); `[meter:warn …]` / `[meter:ok …]` force the color |
| `✓` / `-` | Yes (green check icon) / No (muted minus icon) |
| `[toggle:on]` / `[toggle:off]` | Switch control |
| `[Label](act:xyz)` | Small button that runs action `xyz` (section 10) |
| `[**Label**](act:xyz)` | Primary small button |
| `[Label](go:page/tab)` | Button or link that navigates to a sub-page |
| `🔒` | Lock icon (needs the admin password) |
| `gen[seed, base, amp, noise, min, max, end]` | Generated time series (section 9) |
| Span N | Widget width in grid columns (12 = full width) |
| Row action | Clicking the whole row runs this action |

Widgets appear in the page map in grid order (left to right, top to bottom). Values shown are the **initial state** (before the user runs any fix). Section 8 lists how values change after actions.

---

## 7. State model

```js
S = {
  page: 'overview', tab: 'summary',   // current page / sub-page
  range: '24h',                       // '1h' | '24h' | '7d' | '30d'
  expert: false,                      // Beginner / Expert mode
  rail: false,                        // sidebar collapsed
  access: 'full',                     // 'full' | 'basic'
  ai: true, aiCloud: false,           // assistant on, cloud fallback off
  used: 224.2, total: 238,            // root filesystem GB
  cleaned: {},                        // storage cleanup steps done: snap, docker, trash, apt
  secFixed: false,                    // security updates installed
  battFixed: false, battTime: '',     // low-battery action configured
  btRestarted: false, btTime: '',     // bluetooth restarted
  cupsUndone: false,                  // cups-browsed change undone
  tracked: {},                        // cron jobs wrapped for result tracking: sync, backup
  open: {},                           // sidebar tree open per page
  drawer: null, pop: null, cmdk: null,// overlays
  TS: {},                             // per-table UI state {q, f, sort, dir, pg, sel}
  diag: {phase:'idle', n:0}, net: {phase:'idle', n:0},
  clock: 34                           // minutes after 21:00; +1 for each timestamped action
}
NOW = 2026-10-21 21:34 (local)
```

**Derived values**

```js
pct       = used / total * 100                    // 94.2 at start
dPct      = round(pct)                            // 94
diskSt    = pct >= 93 ? 'crit' : pct >= 85 ? 'warn' : 'ok'
daysFull  = max(1, round((100 - pct) / 0.69))     // 8 at start
cleanable = sum of sizes of storage steps not yet cleaned   // 15.2 GB at start
deductions = [
  ['Storage',  pct >= 93 ? 8 : pct >= 85 ? 2 : 0],
  ['Services', 5],
  ['Security', secFixed ? 0 : 3],
  ['Power',    1],
  ['Backups',  1]]
score     = 100 - sum(deductions)                 // 82 at start
scoreSt   = score >= 90 ? 'ok' : score >= 75 ? 'warn' : 'crit'
clockTick = () => { clock++; return `${21 + floor(clock/60)}:${pad2(clock % 60)}` }
```

### 7.1 Issues list (`issues()`, in this order)

| # | Severity | Title | Detail | Category | First seen | Count | Action | Shown when |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | {crit:Critical} | Root filesystem almost full | / at 94% · 15.2 GB reclaimable | Storage | 14 Oct | - | [Resolve](act:fix:storage) | used ≥ 93% (if 85-93%: Warning "Root filesystem above 85%", detail "/ at N% · largest: ~/Videos 52 GB", action View → go:storage/usage) |
| 2 | {crit:Critical} | bluetooth.service crash loop | 14 restarts today · SIGSEGV | Services | Today 09:12 | 14 | [Inspect](act:bt) | always (detail after restart: "Restarted at HH:MM · monitoring") |
| 3 | {warn:Warning} | Security updates pending | 4 packages · 2 high-severity CVEs | Security | Today 06:21 | 4 | [Install](act:fix:sec) | until updates installed |
| 4 | {warn:Warning} | Battery health degraded | 71% of design capacity | Power | 02 Sep | - | [View](go:power/battery) | always |
| 5 | {warn:Warning} | Backup overdue | Last successful backup 21 days ago | Backups | 07 Oct | - | [View](go:backups/jobs) | always |
| 6 | {warn:Warning} | Out-of-memory kill | chrome renderer · Mon 16:12 | Logs | Mon 16:12 | 1 | [View](go:logs/kernel) | always |
| 7 | {info:Info} | Reboot required | libc6, openssl, sudo updated | System | Today 10:40 | - | [Details](go:system/updates) | always |
| 8 | {info:Info} | Low-battery action not set | Unclean shutdown at 02:14 today | Power | Today 02:14 | 1 | [Configure](act:fix:batt) | until low-battery action configured |
| 9 | {info:Info} | Missed timer run | e2scrub_all.timer · Sun 03:10 | Scheduled Jobs | Sun 03:10 | 1 | [View](go:jobs/timers) | always |

In the Overview "Active issues" table and the Issues table, the action button is primary when the issue is Critical.

The notification badge count = number of issues with severity Critical or Warning.

---

## 8. Fix definitions and state changes

### 8.1 Fix definitions

**`fix:storage`: Free up disk space** (subtitle "Root filesystem ( / )")

| Step ID | Title | Description | Size | Admin | Undo | Checked by default | Command |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `snap` | Remove disabled snap revisions | 6 old revisions kept after updates | 8.7 GB | Yes | `none` | Yes | `snap remove firefox --revision=4793 # +5 more` |
| `docker` | Prune unused Docker images | 7 images not used by any container | 4.0 GB | No | `regen` | Yes | `docker image prune -a` |
| `trash` | Empty Trash | 312 files | 2.1 GB | No | `none` | Yes | `gio trash --empty` |
| `apt` | Clean APT package cache | Downloaded .deb files | 0.4 GB | Yes | `regen` | Yes | `sudo apt-get clean` |

**`fix:sec`: Install updates** (subtitle "APT · noble-security")

| Step ID | Title | Description | Size | Admin | Undo | Checked by default | Command |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `secu` | Install 4 security updates | openssl, libssl3t64, sudo, linux-firmware | - | Yes | `none` | Yes | `sudo apt-get install --only-upgrade openssl libssl3t64 sudo linux-firmware` |
| `other` | Install 8 regular updates | google-chrome-stable, git and 6 more | - | Yes | `none` | No | `sudo apt-get upgrade` |

**`fix:batt`: Configure low-battery action** (subtitle "UPower · /etc/UPower/UPower.conf")

| Step ID | Title | Description | Size | Admin | Undo | Checked by default | Command |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `up` | Warn at 10%, power off cleanly at 5% | Prevents unclean shutdowns like today at 02:14 | - | Yes | `full` | Yes | `sudo sed -i 's/^PercentageAction=.*/PercentageAction=5/' /etc/UPower/UPower.conf` |



Undo badge mapping: `full` → `{ok:Reversible}`, `regen` → `{info:Regenerates}`, `none` → `{warn:Irreversible}`. Admin steps also show `{off:Admin}`.

### 8.2 What changes after each action

| Action | State change | Visible effects |
| --- | --- | --- |
| Free up disk space (chosen steps) | `cleaned[step]=true`; `used -= step sizes` | Disk %, free GB, reclaimable, "full in" days, donut, allocation bar, forecast and mount meters update. With all steps: 94% → 88%, the Storage issue becomes Warning "Root filesystem above 85%" (action View → Storage › Disk Usage), score 82 → 88, the Storage page turns Warning. Cleaned rows leave Storage › Cleanup. If the Docker step ran: unused images leave Containers › Images and the Prune button is hidden. If the snap step ran: Packages › Cleanup shows 0 snap revisions. Adds a SysDoc log entry (not reversible). |
| Install updates | `secFixed=true` | Security issue removed; score +3. Updates: 12 → 8 pending, security 0, the security rows leave the pending table, the update history gains a "SysDoc" row. Security › Overview shows all checks passing (donut 7/10) and an empty Vulnerabilities table. Security page Healthy; Packages page Info. Adds a SysDoc log entry. |
| Configure low-battery action | `battFixed=true`, `battTime=now` | Issue "Low-battery action not set" removed. Power › Power Profile shows `{ok:Power off at 5%}`. Activity › Config Changes gains an UPower.conf row. Adds a SysDoc log entry **with Undo** (also listed in Backups › Restore Points). |
| Restart Bluetooth | `btRestarted=true`, `btTime=now` | The service row becomes `{ok:active}` running 6 MB. The crash-loop alert is hidden. KPIs show Running 143, Failed 0 `ok`. The issue text becomes "Restarted at HH:MM · monitoring". The Services tab count is hidden and the Services page turns Warning. Toast "bluetooth.service restarted · monitoring for new crashes." |
| Undo cups-browsed | `cupsUndone=true` | The cups-browsed row becomes active/running/enabled, 5 MB. Restore points show "Restored"; SysDoc Actions shows "Undone". Toast "Undone · cups-browsed.service enabled and started." |
| Undo low-battery change | `battFixed=false` | Reverts the battery change and marks the log entry undone. Toast "Undone · UPower.conf restored from the restore point." |
| Track cron job | `tracked[id]=true` | The cron result becomes `{info:Tracking}`. Toast "Job wrapped · results are recorded from the next run." |
| Access = Basic | `access='basic'` | Admin-only values show `{off:Requires Full access}` or `{off:Locked}`; admin-only tables are empty with "Requires Full access". Toast "Basic access · admin-only checks are locked." (Full: "Full access · admin helper authorized.") |
| Expert mode | `expert=true` | Panel source chips appear; columns marked `x` appear. |
| Time range | `range` | All generated series and sparklines regenerate with that range's point count and labels. |

---

## 9. Sample data generator (exact)

All live charts and sparklines use this deterministic generator, so the clone matches the prototype point for point.

```js
const NOW = new Date(2026, 9, 21, 21, 34);
const RANGES = { '1h':{n:60, step:1/60}, '24h':{n:96, step:0.25}, '7d':{n:168, step:1}, '30d':{n:120, step:6} }; // step in hours
function rng(seed){ let s = (seed * 9301 + 49297) % 2147483647; if (s <= 0) s += 2147483646;
  return () => (s = s * 16807 % 2147483647) / 2147483647; }
const tAt = (i, n, step) => new Date(NOW - (n - 1 - i) * step * 3600e3);
function gen([seed, base, amp, noise, lo = 0, hi = 100, end]){
  const R = RANGES[S.range], r = rng(seed * 31 + R.n), out = []; let w = 0;
  for (let i = 0; i < R.n; i++){
    const t = tAt(i, R.n, R.step), hr = t.getHours() + t.getMinutes() / 60;
    w = w * 0.72 + (r() - 0.5) * noise;
    out.push(Math.max(lo, Math.min(hi, base + amp * Math.sin((hr - 9) / 24 * 2 * Math.PI) + w)));
  }
  if (end != null){ out[R.n - 1] = end; out[R.n - 2] = (out[R.n - 2] + end) / 2; }
  return out;
}
function tLab(i, n, tooltip){ // axis label (tooltip=false) or tooltip header (true)
  const d = tAt(i, n, RANGES[S.range].step), hh = pad2(d.getHours()) + ':' + pad2(d.getMinutes());
  const wd = d.toLocaleDateString('en-GB', {weekday:'short'}), dm = d.toLocaleDateString('en-GB', {day:'numeric', month:'short'});
  if (S.range === '1h') return hh;
  if (S.range === '24h') return tooltip ? `${wd} ${hh}` : hh;
  if (S.range === '7d') return tooltip ? `${wd} ${hh}` : wd;
  return tooltip ? `${dm} ${hh}` : dm;
}
// Heatmaps: heatData(seed, rows, cols, base, spikes): cell = max(0, round((rng()-0.55)*base)), then spikes [row, col, value] override.
function heatData(seed, rows, cols, base, spikes = []){ const r = rng(seed);
  return Array.from({length:rows}, (_, ri) => Array.from({length:cols}, (_, ci) => {
    let v = Math.max(0, Math.round((r() - 0.55) * base)); spikes.forEach(s => { if (s[0] === ri && s[1] === ci) v = s[2]; }); return v; })); }
// 30-day disk history: r = rng(11); for i in 0..29: 74 + i*0.69 + (r()-0.5)*1.1; last value = 224.2/238*100.
// 30-day labels: dates 22 Sep … 21 Oct formatted 'D Mon' (en-GB).
```

The heatmap matrices in the page map are the computed outputs, so you can paste them directly.

---

## 10. Interactions

### 10.1 Navigation

- `go(page/tab)`: sets the page and sub-page, opens that page's sidebar tree, closes popovers and the mobile drawer, re-renders, and scrolls the main area to the top.
- Deep link: on load, `#page.tab` (for example `#storage.cleanup`) opens that sub-page.
- Re-render on state change keeps the scroll position.

### 10.2 Actions (`act:` names used in the page map)

| Action | Behavior |
| --- | --- |
| `go:page/tab` | Navigate |
| `toast:Message` | Show a toast with that message (prototype stand-in for a real operation) |
| `fix:storage`, `fix:sec`, `fix:batt` | Open the fix drawer (10.3). If already done, show a toast instead: "Nothing left to clean." / "Security updates are already installed." / "Low-battery action is already configured." |
| `bt` | Open the Bluetooth service drawer (10.4) |
| `btrestart` | Restart Bluetooth (section 8.2) and close the drawer |
| `askgo:bt` | Close the drawer, open Assistant › Chat, ask the Bluetooth question |
| `ask:<key>` | Ask a suggested question (10.5) |
| `undo:cups`, `undo:batt` | Undo (section 8.2) |
| `track:sync`, `track:backup` | Track a cron job (section 8.2) |
| `diag:run`, `net:run` | Start the diagnostics runner (section 5.23) |
| `range:X`, `mode:b`, `mode:e` | Time range; Beginner / Expert |
| `rail`, `navtoggle`, `pop:notes`, `pop:host`, `cmdk` | Collapse sidebar, mobile menu, popovers, command palette |
| `refresh` | Re-render; toast "Refreshed · 21:34" |
| `export` | Toast "Export is disabled in the prototype." |
| `close`, `close-go:page/tab` | Close overlays (and navigate) |

### 10.3 Fix drawer flow

1. **Preview.** Title and subtitle from the fix definition. Section label "Steps". Each step is a checkbox row: bold title, muted description, badges (undo type, plus `{off:Admin}` for admin steps), command block, and the size in GB on the right. Steps marked "off by default" start unchecked. Storage cleanup already done is not listed. When opened from Storage › Cleanup with rows selected, only those rows start checked.
   - Footer note: "Password required once" if any chosen step is admin, "No password needed" if none, "Select at least one step" if nothing is chosen.
   - Buttons: Cancel, and primary Apply. Its label is "Free X.X GB" for storage, "Apply N steps" for more than one step, otherwise "Apply". Disabled with no steps chosen. The label and note update live as boxes change.
2. **Authenticate** (only if a chosen step needs admin). A centered card on `--panel-2`: 40px lock circle, bold "Authentication required", muted "sysdoc-helper wants to run N privileged action(s)", a fake password field (`••••••••`), small note "Simulated polkit prompt. In the real app Ubuntu shows this window and SysDoc never sees the password." Buttons: Back, primary Authenticate.
3. **Run.** An ordered list of chosen steps. Finished steps show a green check circle and their command; the current step a spinner; later steps a dot. One step every 700ms. Footer note "Stops on the first error". Scrim clicks and Esc are ignored.
4. **Done.** Apply the state change (section 8.2). Inline success alert: bold result title and "Logged in Activity › SysDoc Actions at HH:MM". A 2-column grid of result cells (12px muted label, 16px 600 value). An optional info note. Buttons: View log (→ Activity › SysDoc Actions), primary Done.
   - Storage result: "X.X GB freed"; cells Root filesystem `94% → 88%`, Health score `82 → 88`, Free space, Full in `~N days`. Note when still ≥85%: "Still above 85%. ~/Videos uses 52 GB; moving it to external storage brings / to about N%."
   - Updates result: "4 updates installed" (12 when both steps); cells Security updates pending `4 → 0`, Health score, Known CVEs `3 → 0`, Reboot `Required`. Note "Restart when convenient to load the updated libraries."
   - Battery result: "Low-battery action configured"; cells At 10% `Warning`, At 5% `Clean power-off`. Note "Reversible from Activity › SysDoc Actions or Backups › Restore Points."

### 10.4 Bluetooth service drawer (`bt`)

- Title `bluetooth.service`, subtitle "Bluetooth service · crash loop".
- 2×2 result cells: Active state `failed` (or `active` after restart), Restarts today `14`, Exit status `11 (SIGSEGV)`, First crash `Today 09:12`.
- Section "Correlated change": info alert "Kernel upgraded 6.8.0-45 → 6.8.0-47" / "Tue 18:40 · crashes started at next boot".
- Section "Log excerpt": two log lines: `09:12:44 kernel: bluetoothd[1342]: segfault at 0 ip 000055d1c2a4 error 4` and `systemd: bluetooth.service: Main process exited, code=dumped, status=11/SEGV`.
- Section "Remediation", a bordered checklist with 3 rows:
  1. Refresh icon (pass style) · "Restart service" / `sudo systemctl restart bluetooth` · [**Restart**](act:btrestart)
  2. Terminal icon (na) · "Boot previous kernel once" / "Boot menu › Advanced options › 6.8.0-45" · [Guide](toast:Hold Shift at startup → Advanced options for Ubuntu → 6.8.0-45-generic.)
  3. Chat icon (na) · "Ask the assistant" / "Explain the crash in plain words" · [Ask](act:askgo:bt)
- Footer: [Crash reports](go:logs/crashes), Close.

### 10.5 Assistant

Starts with one exchange already shown: the user question for `restart` and its answer. Suggested questions append a user bubble, then a waiting bubble, then the answer after 700ms. Free text in the composer matches keywords: restart / shut / power off / turned off → `restart`; bluetooth / earbud → `bt`; ssd / nvme / drive health → `ssd`; disk / space / full / storage → `disk`. Anything else gets the fallback: "This prototype answers the suggested questions only. The real assistant reads this computer's logs and settings to answer anything."

| Key | Question | Answer | Sources | Action button |
| --- | --- | --- | --- | --- |
| `restart` | Why did my laptop restart last night? | It did not restart. **The battery ran out at 02:14**, so the laptop powered off without a clean shutdown. You started it again at 08:58. No low-battery action is configured, so this can happen again. | `02:09:12 upowerd: battery BAT0 is critically low (2%)`<br>`02:13:58 upowerd: battery BAT0 at 1%, critical action not configured`<br>`-- boot ended without clean shutdown --` | [**Configure low-battery action**](act:fix:batt) |
| `bt` | Why is Bluetooth crashing? | **bluetoothd crashes with SIGSEGV each time the earbuds connect.** It started at 09:12 today, after yesterday’s kernel upgrade 6.8.0-45 → 6.8.0-47. Restart the service first. If it keeps crashing, boot once with 6.8.0-45 to confirm the regression. | `09:12:44 kernel: bluetoothd[1342]: segfault at 0 … error 4`<br>`systemd: bluetooth.service: Main process exited, status=11/SEGV`<br>`Tue 18:40 apt: upgrade linux-image-6.8.0-47-generic` | [**Inspect bluetooth.service**](act:bt) |
| `ssd` | Is my SSD healthy? | **Yes.** SMART passed, 88% life left, 0 media errors. At your write rate that is many years. | `SMART overall-health self-assessment: PASSED`<br>`Percentage Used: 12%`<br>`Media and Data Integrity Errors: 0` | - |
| `disk` | What is filling my disk? | **~/Videos (52 GB) and ~/Documents (41 GB)** are the largest folders. System leftovers worth 15.2 GB can also be removed safely. | `du -h --max-depth=1 ~`<br>`snap list --all \| grep disabled` | [**Free up space**](act:fix:storage) |


### 10.6 Row-inspect drawer

Tables marked **Inspect** open a drawer when a row is clicked: title = the first cell's text, subtitle = the table title, body = a key-value list of every column header and that row's cell. Footer = the table's row actions (listed in the page map) + primary Close. The prototype stand-ins: "End process" → toast "SIGTERM sent to <name> (prototype)."; service actions → toast "<Action> requested for <unit> (prototype)."; other row actions run as listed.

### 10.7 Keyboard

- Ctrl/Cmd + K opens the command palette.
- Esc closes the palette, popovers, the mobile sidebar and drawers (not while a fix is running).
- All controls are buttons or inputs, reachable by Tab, with a visible focus ring.

### 10.8 Form changes

- Any switch or select not listed below shows the toast "Saved."
- Power profile radios → toast "Power profile set to <name>."
- Settings › Assistant "Enable assistant" → toggles the assistant (toast "Assistant enabled." / "Assistant disabled."); "Cloud fallback" → toast "Cloud fallback on · data redacted before sending." / "Cloud fallback off."
- Settings › General "Expert mode" toggles Expert mode; "Default time range" sets the range.

### 10.9 Charts

- Charts redraw when the window or main area resizes (debounced 80-120ms) and after every render.

---

## 11. Icons

All icons are 24×24 viewBox, one `<path>` each, `fill:none; stroke:currentColor; stroke-width:1.8; stroke-linecap:round; stroke-linejoin:round`.

| Name | Path `d` |
| --- | --- |
| `grid` | `M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z` |
| `perf` | `M3 12h4l3-7 4 14 3-7h4` |
| `logs` | `M5 6h14M5 10h14M5 14h10M5 18h7` |
| `hist` | `M12 8v4l3 2M3.5 12a8.5 8.5 0 1 0 2.4-5.9M3 3.5v4h4` |
| `sys` | `M3.5 5h17v14h-17zM7.5 10l2.5 2-2.5 2M12 14h4.5` |
| `chip` | `M7 7h10v10H7zM10 3v4M14 3v4M10 17v4M14 17v4M3 10h4M3 14h4M17 10h4M17 14h4` |
| `bat` | `M3 8h15v8H3zM21 10.5v3M6 11v2M9 11v2` |
| `mon` | `M3 4.5h18v11.5H3zM8 20h8M12 16v4` |
| `layers` | `M12 3l9 5-9 5-9-5zM3 13l9 5 9-5` |
| `cog` | `M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7zM12 2.5v3M12 18.5v3M2.5 12h3M18.5 12h3M5.3 5.3l2.1 2.1M16.6 16.6l2.1 2.1M5.3 18.7l2.1-2.1M16.6 7.4l2.1-2.1` |
| `cal` | `M4 5.5h16V20H4zM4 10h16M8.5 3v4M15.5 3v4` |
| `cube` | `M12 3l8 4.5v9L12 21l-8-4.5v-9zM12 12l8-4.5M12 12v9M12 12L4 7.5` |
| `pkg` | `M3.5 4h17v4h-17zM5 8v12h14V8M10 12h4` |
| `db` | `M4 6c0-1.7 3.6-3 8-3s8 1.3 8 3-3.6 3-8 3-8-1.3-8-3zM4 6v12c0 1.7 3.6 3 8 3s8-1.3 8-3V6M4 12c0 1.7 3.6 3 8 3s8-1.3 8-3` |
| `backup` | `M7 18.5a4.5 4.5 0 0 1-.6-9A6 6 0 0 1 18 9a4.5 4.5 0 0 1-.5 9.5zM12 11.5v6M9.5 14l2.5-2.5 2.5 2.5` |
| `wifi` | `M2 9a15 15 0 0 1 20 0M5 12.5a10 10 0 0 1 14 0M8.5 16a5 5 0 0 1 7 0M12 19.5h.01` |
| `users` | `M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM2 21v-1a6 6 0 0 1 12 0v1M16 3.5a4 4 0 0 1 0 7.5M22 21v-1a6 6 0 0 0-4-5.6` |
| `shield` | `M12 3l8 3v6c0 4.5-3.4 8.4-8 9-4.6-.6-8-4.5-8-9V6zM9 12l2 2 4-4` |
| `eye` | `M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12zM12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z` |
| `gauge` | `M4 17a8 8 0 1 1 16 0M12 17l4-5` |
| `chat` | `M4 5h16v11H9l-5 4z` |
| `sliders` | `M4 6h9M17 6h3M4 12h3M11 12h9M4 18h11M19 18h1M15 4v4M9 10v4M17 16v4` |
| `chev` | `M9 6l6 6-6 6` |
| `search` | `M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14zM20 20l-3.5-3.5` |
| `x` | `M6 6l12 12M18 6L6 18` |
| `check` | `M5 12.5l4.2 4L19 7` |
| `excl` | `M12 6.5v7M12 17.5h.01` |
| `info` | `M12 11v6M12 7h.01` |
| `minus` | `M6 12h12` |
| `i` | `M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20zM12 11v5M12 8h.01` |
| `play` | `M7 5l12 7-12 7z` |
| `dl` | `M12 4v11M7 10l5 5 5-5M5 20h14` |
| `ref` | `M20 11a8 8 0 0 0-14.7-4M4 3.5V8h4.5M4 13a8 8 0 0 0 14.7 4M20 20.5V16h-4.5` |
| `rail` | `M4 4h16v16H4zM9 4v16` |
| `ext` | `M14 4h6v6M20 4l-9 9M18 14v6H4V6h6` |
| `lock` | `M6 11h12v9H6zM8.5 11V8a3.5 3.5 0 0 1 7 0v3` |
| `mic` | `M12 3a3 3 0 0 1 3 3v6a3 3 0 0 1-6 0V6a3 3 0 0 1 3-3zM5 11a7 7 0 0 0 14 0M12 18v3` |
| `cam` | `M3 7h12v10H3zM15 10.5l6-3.5v10l-6-3.5` |
| `screen` | `M3 4.5h18v12H3zM8 20h8` |
| `pin` | `M12 21s-7-6.2-7-11.5a7 7 0 0 1 14 0C19 14.8 12 21 12 21zM12 12a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z` |
| `fan` | `M12 14a2 2 0 1 0 0-4 2 2 0 0 0 0 4zM12 10c0-4 1-7 4-7 2 0 3 2 1 4M14 12c4 0 7 1 7 4 0 2-2 3-4 1M12 14c0 4-1 7-4 7-2 0-3-2-1-4M10 12c-4 0-7-1-7-4 0-2 2-3 4-1` |
| `bolt` | `M13 3L5 13.5h6L10 21l8-10.5h-6z` |
| `usb` | `M12 3v14M9 6l3-3 3 3M7 10v3l5 3 5-3v-3M12 17a2 2 0 1 0 0 4 2 2 0 0 0 0-4z` |

Page icons: Overview = `grid`, Performance = `perf`, Logs = `logs`, Activity = `hist`, System = `sys`, Hardware = `chip`, Power = `bat`, Display & Audio = `mon`, Processes = `layers`, Services = `cog`, Scheduled Jobs = `cal`, Containers = `cube`, Packages = `pkg`, Storage = `db`, Backups = `backup`, Network = `wifi`, Accounts = `users`, Security = `shield`, Privacy = `eye`, Diagnostics = `gauge`, Assistant = `chat`, Settings = `sliders`. Hamburger menu: `M4 6h16M4 12h16M4 18h16`. Bell: `M6 16v-5a6 6 0 0 1 12 0v5l1.5 2h-15zM10 20.5a2 2 0 0 0 4 0`. Chevron down (machine picker): `M6 9l6 6 6-6`.

---

## 12. Page map

Each sub-page lists its header actions (if any) and its widgets in grid order.

### 12.1 Overview (`overview`)

- Group: Monitoring · Icon: `grid` · Status badge: {crit:Critical} (rule in section 3)
- Sub-pages: Summary, Issues, Health Score

#### 12.1.1 Overview › Summary: `overview/summary`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Health score | 82 | /100 | {warn:Warning} | 9 open issues | Usage bar 82% | `overview/score` |
| CPU | 12 | % | - | Load 0.84 · 12 threads | Sparkline slot 1, gen[3, 14, 8, 6, 2, 60, 12] | `performance/cpu` |
| Memory | 5.9 | GB | - | 78% of 7.6 GB | Sparkline slot 2, gen[5, 70, 8, 3, 40, 95, 78] | `performance/memory` |
| Disk ( / ) | 94 | % | {crit:Critical} | 13.8 GB free of 238 GB | Usage bar 94.2% | `storage/overview` |
| Network | 1.2 | MB/s | - | ↑ 0.3 MB/s · Wi-Fi | Sparkline slot 3, gen[9, 0.8, 0.6, 0.8, 0, 6, 1.2] | `performance/network` |
| Battery | 64 | % | {warn:Warning} | Health 71% · 3 h 10 m left | Usage bar 64% | `power/battery` |

2. **Line chart**: "CPU and memory utilization" · span 8
- Info tooltip: "How busy the processor and memory were. Change the time range at the top to zoom out."
- Source chip (Expert): `/proc/stat · /proc/meminfo`
- Height 300px · Y axis 0 to 100 · unit "%"
- Series: CPU (slot 1, gen[3, 14, 8, 6, 2, 60, 12]); Memory (slot 2, gen[5, 70, 8, 3, 40, 95, 78])
- X axis: time (current range)

3. **Status list**: "Subsystems" · span 4
- Info tooltip: "Status of every area SysDoc checks. Select a row to open it."

| Icon | Label | Metric | Badge | Opens |
| --- | --- | --- | --- | --- |
| `chip` | Hardware | CPU 54 °C | {ok:Healthy} | `hardware/summary` |
| `bat` | Power | Health 71% | {warn:Warning} | `power/battery` |
| `sys` | System | Reboot required | {info:Info} | `system/overview` |
| `cog` | Services | 1 failed | {crit:Critical} | `services/system` |
| `cal` | Scheduled Jobs | 10 jobs | {ok:Healthy} | `jobs/calendar` |
| `pkg` | Packages | 12 updates | {warn:Warning} | `packages/installed` |
| `cube` | Containers | 1 exposed port | {warn:Warning} | `containers/containers` |
| `db` | Storage | 94% used | {crit:Critical} | `storage/overview` |
| `backup` | Backups | 21 days ago | {warn:Warning} | `backups/jobs` |
| `wifi` | Network | Wi-Fi −52 dBm | {ok:Healthy} | `network/interfaces` |
| `users` | Accounts | 2 users | {ok:Healthy} | `accounts/users` |
| `shield` | Security | 4 updates | {warn:Warning} | `security/overview` |
| `eye` | Privacy | No active access | {ok:Healthy} | `privacy/access` |

4. **Data table**: "Active issues" · span 8
- Subtitle: "9 open"
- Header actions: [View all](go:overview/issues) (link style)

| Severity | Issue | Category | Detected | (action) |
| --- | --- | --- | --- | --- |
| {crit:Critical} | **Root filesystem almost full** · / at 94% · 15.2 GB reclaimable | Storage | 14 Oct | [**Resolve**](act:fix:storage) |
| {crit:Critical} | **bluetooth.service crash loop** · 14 restarts today · SIGSEGV | Services | Today 09:12 | [**Inspect**](act:bt) |
| {warn:Warning} | **Security updates pending** · 4 packages · 2 high-severity CVEs | Security | Today 06:21 | [Install](act:fix:sec) |
| {warn:Warning} | **Battery health degraded** · 71% of design capacity | Power | 02 Sep | [View](go:power/battery) |
| {warn:Warning} | **Backup overdue** · Last successful backup 21 days ago | Backups | 07 Oct | [View](go:backups/jobs) |
| {warn:Warning} | **Out-of-memory kill** · chrome renderer · Mon 16:12 | Logs | Mon 16:12 | [View](go:logs/kernel) |

5. **Gauge + horizontal bars**: "Health score" · span 4
- Info tooltip: "Starts at 100. Each open issue removes points from its area."
- Header actions: [Details](go:overview/score) (link style)

Gauge: value **82**, subtitle "out of 100", arc color `warn` (by score: ≥90 ok, ≥75 warn, else crit). Below it, horizontal bars (max 10, warn color): Storage −8, Services −5, Security −3, Power −1, Backups −1 (only categories with a deduction).

6. **Donut chart**: "Disk usage ( / )" · span 4
- Subtitle: "238 GB"
- Source chip (Expert): `df -h / · du`
- Header actions: [Open](go:storage/overview) (link style)
- Center: "94%" / "used" · unit "GB"

| Segment | Value | Color | Display value |
| --- | --- | --- | --- |
| Home | 128 | slot 1 | - |
| Apps | 46 | slot 2 | - |
| System | 22 | slot 3 | - |
| Containers | 13 | slot 4 | - |
| Reclaimable | 15.2 | kc | - |
| Free | 13.8 | kn | - |

7. **Data table**: "Top processes" · span 4
- Subtitle: "by memory"
- Source chip (Expert): `ps -eo comm,%cpu,rss`
- Header actions: [View all](go:processes/all) (link style)

| Process | CPU [n] | Memory [n] |
| --- | --- | --- |
| chrome | 4.2% | 842 MB |
| chrome (renderer) | 2.6% | 612 MB |
| gnome-shell | 3.8% | 410 MB |
| code | 1.9% | 486 MB |
| telegram-desktop | 0.4% | 280 MB |
| dropbox | 0.3% | 210 MB |

8. **Key-value list**: "System" · span 4
- Source chip (Expert): `hostnamectl`

| Key | Value |
| --- | --- |
| Hostname | riya-laptop |
| OS | Ubuntu 24.04.1 LTS |
| Kernel | `6.8.0-47-generic` |
| Uptime | 12 h 36 m |
| Last boot | Today 08:58 |
| Reboot required | {warn:Yes} |

9. **Activity timeline**: "Recent activity" · span 6
- Header actions: [View all](go:activity/timeline) (link style)

| Time | Title | Detail | Dot |
| --- | --- | --- | --- |
| 10:40 | Unattended upgrade: libc6, openssl, sudo | Reboot required | info |
| 09:12 | bluetooth.service entered crash loop | SIGSEGV · 14 restarts | crit |
| 08:58 | Boot completed in 41.2 s | Kernel 6.8.0-47-generic | ok |
| 02:14 | Unclean shutdown | Battery depleted | warn |
| Tue 18:41 | linux-firmware upgraded | …0ubuntu2.5 | default |
| Tue 18:40 | Kernel upgraded 6.8.0-45 → 6.8.0-47 | Software Updater | default |

10. **Heatmap (day × hour)**: "Errors and warnings by hour" · span 6
- Subtitle: "last 7 days"
- Source chip (Expert): `journalctl -p warning`
- Rows: Thu, Fri, Sat, Sun, Mon, Tue, Wed · Columns: hours 00-23 · unit "events"
- Values (row by row, 24 per row):

```
Thu  1 0 0 1 1 0 0 0 1 1 0 0 0 0 0 0 0 1 1 1 2 0 0 1
Fri  0 1 0 0 0 1 0 1 0 1 1 0 1 0 1 0 0 0 1 0 0 0 0 0
Sat  1 0 0 0 1 1 1 0 0 0 0 1 0 1 1 0 0 0 1 0 0 0 0 1
Sun  1 0 1 2 1 0 1 0 0 0 0 0 0 0 0 1 0 0 0 0 0 1 1 0
Mon  0 0 0 0 0 0 0 0 2 1 1 0 0 0 0 0 7 1 0 0 0 0 0 0
Tue  1 0 0 1 1 0 0 0 1 0 0 1 0 0 0 0 0 1 0 0 2 1 0 0
Wed  1 1 3 0 0 2 0 1 1 9 0 0 1 0 2 0 0 1 0 0 0 6 0 2
```

#### 12.1.2 Overview › Issues: `overview/issues`

Tab count pill: 9 (red).

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Critical | 2 |  | {crit:Critical} |   | - | - |
| Warning | 4 |  | {warn:Warning} |   | - | - |
| Info | 3 |  | - |   | - | - |
| Resolved | 11 |  | - | last 30 days | - | - |
| Mean time to resolve | 1.8 | days | - |   | - | - |

2. **Data table**: "All issues" · span 12
- Search: "Search issues"
- Filter chips on column 1: All / Critical / Warning / Info
- Table ID: `issues-all`

| Severity | Issue | Category | First seen | Count [n] | Status | (action) |
| --- | --- | --- | --- | --- | --- | --- |
| {crit:Critical} | **Root filesystem almost full** · / at 94% · 15.2 GB reclaimable | Storage | 14 Oct | - | {off:Open} | [**Resolve**](act:fix:storage) |
| {crit:Critical} | **bluetooth.service crash loop** · 14 restarts today · SIGSEGV | Services | Today 09:12 | 14 | {off:Open} | [**Inspect**](act:bt) |
| {warn:Warning} | **Security updates pending** · 4 packages · 2 high-severity CVEs | Security | Today 06:21 | 4 | {off:Open} | [Install](act:fix:sec) |
| {warn:Warning} | **Battery health degraded** · 71% of design capacity | Power | 02 Sep | - | {off:Open} | [View](go:power/battery) |
| {warn:Warning} | **Backup overdue** · Last successful backup 21 days ago | Backups | 07 Oct | - | {off:Open} | [View](go:backups/jobs) |
| {warn:Warning} | **Out-of-memory kill** · chrome renderer · Mon 16:12 | Logs | Mon 16:12 | 1 | {off:Open} | [View](go:logs/kernel) |
| {info:Info} | **Reboot required** · libc6, openssl, sudo updated | System | Today 10:40 | - | {off:Open} | [Details](go:system/updates) |
| {info:Info} | **Low-battery action not set** · Unclean shutdown at 02:14 today | Power | Today 02:14 | 1 | {off:Open} | [Configure](act:fix:batt) |
| {info:Info} | **Missed timer run** · e2scrub_all.timer · Sun 03:10 | Scheduled Jobs | Sun 03:10 | 1 | {off:Open} | [View](go:jobs/timers) |

#### 12.1.3 Overview › Health Score: `overview/score`

1. **Line chart**: "Health score trend" · span 8
- Subtitle: "30 days"
- Height 240px · Y axis 60 to 100
- Series: Score (slot 1, data [96,96,95,95,94,94,93,93,92,92,92,91,91,90,90,89,89,88,88,87,87,87,86,82,81,81,80,80,79,82])
- X labels: 30 labels: 22 Sept, 23 Sept, 24 Sept … 20 Oct, 21 Oct

2. **Gauge + horizontal bars**: "Current score" · span 4

Gauge: value **82**, subtitle "out of 100", arc color `warn` (by score: ≥90 ok, ≥75 warn, else crit). Caption under the gauge (muted, centered): "90+ Healthy · 75-89 Fair · below 75 At risk".

3. **Data table**: "Score breakdown" · span 12

| Category | Weight [n] | Deduction [n] | Open findings | Status |
| --- | --- | --- | --- | --- |
| Storage | 15 | −8 | Root filesystem almost full | {crit:Critical} |
| Services | 15 | −5 | bluetooth.service crash loop | {crit:Critical} |
| Security | 15 | −3 | 4 security updates pending | {warn:Warning} |
| Power | 10 | −1 | Battery health 71% | {warn:Warning} |
| Backups | 10 | −1 | Last backup 21 days ago | {warn:Warning} |
| Hardware | 10 | 0 | - | {ok:Healthy} |
| Network | 10 | 0 | - | {ok:Healthy} |
| Accounts | 5 | 0 | - | {ok:Healthy} |
| Privacy | 5 | 0 | - | {ok:Healthy} |
| Packages | 5 | 0 | - | {ok:Healthy} |

### 12.2 Performance (`performance`)

- Group: Monitoring · Icon: `perf` · Status badge: {ok:Healthy} (rule in section 3)
- Sub-pages: CPU, Memory, Disk I/O, Network, GPU, Pressure (PSI)

#### 12.2.1 Performance › CPU: `performance/cpu`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Utilization | 12 | % | - |   | Sparkline slot 1, gen[3, 14, 8, 6, 2, 60, 12] | - |
| Load average | 0.84 |  | - | 5 min 0.92 · 15 min 1.10 | - | - |
| Frequency | 1.3 | GHz | - | Max 4.4 GHz | - | - |
| Temperature | 54 | °C | - |   | Sparkline slot 2, gen[21, 52, 6, 3, 35, 90, 54] | - |
| Processes | 312 |  | - | 1,204 threads | - | - |
| Context switches | 6.2K | /s | - | Interrupts 3.1K/s | - | - |

2. **Line chart**: "CPU utilization by type" · span 12
- Info tooltip: "User = your apps. System = the kernel. I/O wait = waiting for the disk."
- Source chip (Expert): `/proc/stat`
- Height 240px · Y axis 0 to 50 · unit "%" · **stacked**
- Series: User (slot 1, gen[3, 8, 5, 4, 0, 60]); System (slot 2, gen[13, 3, 2, 2, 0, 30]); I/O wait (slot 3, gen[17, 0.8, 0.6, 1, 0, 15])
- X axis: time (current range)

3. **Bar chart**: "Per-core utilization" · span 6
- Subtitle: "now"
- Source chip (Expert): `mpstat -P ALL`
- Height 200px · Y max 100 · unit "%"
- Bars (label: value · tooltip label): P0: 18 · "Thread P0"; P1: 9 · "Thread P1"; P2: 14 · "Thread P2"; P3: 6 · "Thread P3"; E0: 22 · "Thread E0"; E1: 11 · "Thread E1"; E2: 4 · "Thread E2"; E3: 7 · "Thread E3"; E4: 16 · "Thread E4"; E5: 3 · "Thread E5"; E6: 8 · "Thread E6"; E7: 5 · "Thread E7"

4. **Data table**: "Top CPU consumers" · span 6
- Source chip (Expert): `ps --sort=-%cpu`

| Process | PID [n] | User | CPU [n] |
| --- | --- | --- | --- |
| chrome | 4121 | riya | [meter 21% "4.2%"] |
| gnome-shell | 1874 | riya | [meter 19% "3.8%"] |
| chrome (renderer) | 4388 | riya | [meter 13% "2.6%"] |
| code | 5102 | riya | [meter 10% "1.9%"] |
| Xwayland | 1990 | riya | [meter 3% "0.6%"] |
| pipewire | 1702 | riya | [meter 3% "0.5%"] |
| telegram-desktop | 2230 | riya | [meter 2% "0.4%"] |

#### 12.2.2 Performance › Memory: `performance/memory`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Used | 5.9 | GB | - | 78% of 7.6 GB | Sparkline slot 2, gen[5, 70, 8, 3, 40, 95, 78] | - |
| Available | 1.7 | GB | - |   | - | - |
| Cache and buffers | 1.6 | GB | - |   | - | - |
| Swap | 1.2 | GB | - | of 2.0 GB | Usage bar 60% | - |
| Pressure | 0.4 | % | - | some avg10 | - | - |
| OOM kills | 1 |  | {warn:Warning} | last 7 days | - | - |

2. **Line chart**: "Memory usage" · span 8
- Source chip (Expert): `/proc/meminfo`
- Height 240px · Y axis 0 to 8 · unit "GB" · **stacked**
- Series: Used (slot 1, gen[5, 4.6, 0.9, 0.5, 2.5, 6.4, 5.9]); Cache (slot 2, gen[7, 1.4, 0.3, 0.2, 0.5, 2.5, 1.6])
- X axis: time (current range)
- Threshold line at 7.6: "Total 7.6 GB"

3. **Donut chart**: "Composition" · span 4
- Subtitle: "now"
- Center: "7.6" / "GB total" · unit "GB"

| Segment | Value | Color | Display value |
| --- | --- | --- | --- |
| Used | 5.9 | slot 1 | - |
| Cache | 1.6 | slot 2 | - |
| Free | 0.1 | kn | 0.1 GB |

4. **Data table**: "Top memory consumers" · span 12
- Source chip (Expert): `ps --sort=-rss`

| Process | PID [n] | User | Resident [n] | Share of total [n] | Swap [n, ,x] |
| --- | --- | --- | --- | --- | --- |
| chrome | 4121 | riya | 842 MB | [meter 44% "11.1%"] | 120 MB |
| chrome (renderer) | 4388 | riya | 612 MB | [meter 32% "8.1%"] | 84 MB |
| gnome-shell | 1874 | riya | 410 MB | [meter 22% "5.4%"] | 0 |
| code | 5102 | riya | 486 MB | [meter 26% "6.4%"] | 62 MB |
| telegram-desktop | 2230 | riya | 280 MB | [meter 15% "3.7%"] | 0 |
| dropbox | 2301 | riya | 210 MB | [meter 11% "2.8%"] | 0 |
| postgres | 2811 | 999 | 142 MB | [meter 7% "1.9%"] | 0 |
| dockerd | 1320 | root | 96 MB | [meter 5% "1.3%"] | 0 |

#### 12.2.3 Performance › Disk I/O: `performance/disk`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Read | 2.4 | MB/s | - |   | Sparkline slot 1, gen[31, 2, 2, 3, 0, 40, 2.4] | - |
| Write | 0.8 | MB/s | - |   | Sparkline slot 2, gen[32, 1, 1, 1.5, 0, 30, 0.8] | - |
| IOPS | 142 |  | - |   | - | - |
| Utilization | 6 | % | - |   | - | - |
| Avg latency | 0.4 | ms | - |   | - | - |
| Queue depth | 0.02 |  | - |   | - | - |

2. **Line chart**: "Throughput" · span 12
- Source chip (Expert): `/proc/diskstats`
- Height 220px · Y axis 0 to auto · unit "MB/s"
- Series: Read (slot 1, gen[31, 2, 2, 3, 0, 40, 2.4]); Write (slot 2, gen[32, 1, 1, 1.5, 0, 30, 0.8])
- X axis: time (current range)

3. **Data table**: "Devices" · span 6
- Source chip (Expert): `iostat -x`

| Device | Read/s [n] | Write/s [n] | Util [n] | Await [n] |
| --- | --- | --- | --- | --- |
| `nvme0n1` | 2.4 MB | 0.8 MB | 6% | 0.4 ms |
| `loop0-loop13` | 0 | 0 | 0% | 0 ms |

4. **Data table**: "Top I/O processes" · span 6
- Source chip (Expert): `iotop -b -n1`
- Empty text: "Requires Full access · Settings › Access"

| Process | Read [n] | Write [n] |
| --- | --- | --- |
| chrome | 1.1 MB/s | 0.4 MB/s |
| tracker-miner-fs-3 | 0.8 MB/s | 0.1 MB/s |
| code | 0.2 MB/s | 0.1 MB/s |
| postgres | 0.1 MB/s | 0.1 MB/s |

#### 12.2.4 Performance › Network: `performance/network`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Download | 1.2 | MB/s | - |   | Sparkline slot 1, gen[9, 0.8, 0.6, 0.8, 0, 6, 1.2] | - |
| Upload | 0.3 | MB/s | - |   | Sparkline slot 2, gen[10, 0.2, 0.2, 0.3, 0, 3, 0.3] | - |
| Data today | 4.2 | GB | - |   | - | - |
| Connections | 68 |  | - | 5 listening | - | - |
| Packet loss | 0 | % | - |   | - | - |
| Latency | 18 | ms | - | to 1.1.1.1 | - | - |

2. **Line chart**: "Throughput" · span 12
- Source chip (Expert): `/proc/net/dev`
- Height 220px · Y axis 0 to auto · unit "MB/s"
- Series: Download (slot 1, gen[9, 0.8, 0.6, 0.8, 0, 6, 1.2]); Upload (slot 2, gen[10, 0.2, 0.2, 0.3, 0, 3, 0.3])
- X axis: time (current range)

3. **Bar chart**: "Daily data usage" · span 6
- Subtitle: "14 days"
- Source chip (Expert): `vnstat -d`
- Height 200px · Y max auto · unit "GB"
- Bars (label: value · tooltip label): 8: 3.1 · "8 Oct"; 9: 4.4 · "9 Oct"; 10: 2.2 · "10 Oct"; 11: 1.9 · "11 Oct"; 12: 5.8 · "12 Oct"; 13: 3.6 · "13 Oct"; 14: 4.1 · "14 Oct"; 15: 3.3 · "15 Oct"; 16: 6.2 · "16 Oct"; 17: 2.8 · "17 Oct"; 18: 1.5 · "18 Oct"; 19: 4.9 · "19 Oct"; 20: 5.1 · "20 Oct"; 21: 4.2 · "21 Oct"

4. **Data table**: "Interfaces" · span 6
- Source chip (Expert): `ip -s link`

| Interface | Type | State | RX/s [n] | TX/s [n] |
| --- | --- | --- | --- | --- |
| `wlp0s20f3` | Wi-Fi | {ok:up} | 1.2 MB | 0.3 MB |
| `docker0` | Bridge | {ok:up} | 12 KB | 9 KB |
| `lo` | Loopback | {ok:up} | 4 KB | 4 KB |
| `enx00e04c68` | Ethernet (USB) | {off:down} | 0 | 0 |

#### 12.2.5 Performance › GPU: `performance/gpu`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Device | Iris Xe |  | - | Intel · Alder Lake-UP3 GT2 | - | - |
| Utilization | 8 | % | - |   | Sparkline slot 1, gen[41, 7, 5, 4, 0, 80, 8] | - |
| Memory | Shared |  | - | Uses system RAM | - | - |
| Temperature | N/A |  | - | Not reported by i915 | - | - |
| Driver | i915 |  | - | Mesa 24.0.9 | - | - |

2. **Line chart**: "GPU utilization" · span 12
- Source chip (Expert): `intel_gpu_top (admin)`
- Height 220px · Y axis 0 to 100 · unit "%"
- Series: Render (slot 1, gen[41, 7, 5, 4, 0, 80, 8])
- X axis: time (current range)

3. **Alert banner**: "GPU temperature is not exposed" · span 12
- State: `info` · Title: "GPU temperature is not exposed" · Text: "The Intel i915 driver shares the CPU package sensor. See Hardware › Sensors." · Buttons: [Open sensors](go:hardware/sensors)

#### 12.2.6 Performance › Pressure (PSI): `performance/pressure`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| CPU some | 1.2 | % | - |   | - | - |
| Memory some | 0.4 | % | - |   | - | - |
| Memory full | 0.1 | % | - |   | - | - |
| I/O some | 0.8 | % | - |   | - | - |
| I/O full | 0.2 | % | - |   | - | - |

2. **Line chart**: "Pressure stall information" · span 12
- Info tooltip: "Share of time tasks waited for CPU, memory or disk. The most reliable sign of a slow system."
- Source chip (Expert): `/proc/pressure/{cpu,memory,io}`
- Height 240px · Y axis 0 to auto · unit "%"
- Series: CPU (slot 1, gen[51, 1.4, 1, 1.5, 0, 40, 1.2]); Memory (slot 2, gen[52, 0.6, 0.5, 1.2, 0, 40, 0.4]); I/O (slot 3, gen[53, 0.9, 0.6, 1, 0, 40, 0.8])
- X axis: time (current range)

3. **Key-value list**: "Thresholds" · span 12

| Key | Value |
| --- | --- |
| Healthy | below 5% (avg10) |
| Noticeable slowdown | 5-20% |
| Severe | above 20% |
| Peak in range | {warn:Memory 23.4% · Mon 16:12} |

### 12.3 Logs (`logs`)

- Group: Monitoring · Icon: `logs` · Status badge: {warn:Warning} (rule in section 3)
- Sub-pages: Journal, Errors, Crashes, Kernel, Boots

#### 12.3.1 Logs › Journal: `logs/journal`

1. **Bar chart**: "Log volume" · span 12
- Subtitle: "events per hour, last 24 h"
- Source chip (Expert): `journalctl --since -24h`
- Height 150px · Y max auto · highlighted (warn + value label): index 8
- Bars (label: value · tooltip label): 00: 14 · "00:00"; 01: 9 · "01:00"; 02: 41 · "02:00"; 03: 22 · "03:00"; 04: 6 · "04:00"; 05: 4 · "05:00"; 06: 5 · "06:00"; 07: 6 · "07:00"; 08: 88 · "08:00"; 09: 64 · "09:00"; 10: 31 · "10:00"; 11: 18 · "11:00"; 12: 22 · "12:00"; 13: 19 · "13:00"; 14: 16 · "14:00"; 15: 34 · "15:00"; 16: 21 · "16:00"; 17: 14 · "17:00"; 18: 26 · "18:00"; 19: 19 · "19:00"; 20: 31 · "20:00"; 21: 46 · "21:00"; 22: 12 · "22:00"; 23: 8 · "23:00"

2. **Data table**: "Journal entries" · span 12
- Search: "Filter messages"
- Filter chips on column 2: All / error / warning / info
- Page size 12
- Table ID: `journal`

| Time | Priority | Unit | Message [w] |
| --- | --- | --- | --- |
| `21:31:02` | {off:info} | `NetworkManager` | `device (wlp0s20f3): state change: activated` |
| `21:20:15` | {off:info} | `systemd` | `bluetooth.service: Scheduled restart job, restart counter is at 14.` |
| `21:20:14` | {warn:warning} | `systemd` | `bluetooth.service: Main process exited, code=dumped, status=11/SEGV` |
| `21:20:14` | {crit:error} | `kernel` | `bluetoothd[7702]: segfault at 0 ip 000055d1c2a4 sp 00007ffd error 4` |
| `21:05:44` | {off:info} | `gnome-shell` | `Overwriting existing binding of keysym 31 with keysym 31` |
| `20:12:03` | {off:info} | `systemd` | `Started gnome-terminal-server.service` |
| `18:52:10` | {off:info} | `systemd` | `Starting apt-daily.service - Daily apt download activities...` |
| `15:22:31` | {off:info} | `snapd` | `Installed "code" snap` |
| `10:40:02` | {off:info} | `unattended-upgrades` | `Packages that were upgraded: libc6 openssl sudo` |
| `09:12:44` | {crit:error} | `kernel` | `bluetoothd[1342]: segfault at 0 ip 000055d1c2a4 error 4` |
| `09:00:17` | {crit:error} | `kernel` | `ACPI BIOS Error (bug): Could not resolve symbol [\_SB.PC00.LPCB.EC0], AE_NOT_FOUND` |
| `08:58:41` | {off:info} | `systemd` | `Startup finished in 8.1s (firmware) + 2.4s (loader) + 3.9s (kernel) + 26.8s (userspace) = 41.2s.` |
| `08:58:05` | {off:info} | `kernel` | `Linux version 6.8.0-47-generic (buildd@lcy02-amd64-061)` |
| `02:13:58` | {warn:warning} | `upowerd` | `battery BAT0 at 1%, critical action not configured` |
| `02:09:12` | {warn:warning} | `upowerd` | `battery BAT0 is critically low (2%)` |
| `Mon 16:12:31` | {crit:error} | `kernel` | `Out of memory: Killed process 30412 (chrome) anon-rss:1843212kB` |
| `Mon 16:12:30` | {warn:warning} | `systemd-oomd` | `Memory pressure for user@1000.service is high` |
| `Mon 10:02:11` | {off:info} | `kernel` | `usb 3-2: new SuperSpeed USB device number 4 using xhci_hcd` |
| `Mon 09:44:09` | {off:info} | `kernel` | `usb 3-2: USB disconnect, device number 3` |
| `Sun 03:10:00` | {off:notice} | `systemd` | `e2scrub_all.timer: skipped, system suspended` |

#### 12.3.2 Logs › Errors: `logs/errors`

Tab count pill: 37.

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Errors | 37 |  | {warn:Warning} | last 7 days | - | - |
| Unique messages | 6 |  | - |   | - | - |
| Needs attention | 2 |  | {crit:Critical} |   | - | - |
| Known harmless | 29 |  | - | ACPI, gnome-shell | - | - |

2. **Data table**: "Grouped errors" · span 12

| Message | Unit | Count [n] | Last seen | Classification | Row action |
| --- | --- | --- | --- | --- | --- |
| **Segmentation fault (SIGSEGV)** · bluetoothd | `bluetooth.service` | 14 | Today 21:20 | {crit:Needs attention} | `bt` |
| **Out of memory: Killed process** · chrome renderer | `kernel` | 1 | Mon 16:12 | {warn:Needs attention} |  |
| **ACPI BIOS Error (AE_NOT_FOUND)** · Firmware table bug | `kernel` | 18 | Today 09:00 | {off:Harmless} |  |
| **Overwriting existing binding of keysym** · Keyboard shortcut | `gnome-shell` | 11 | Today 21:05 | {off:Harmless} |  |
| **USB disconnect** · usb 3-2 | `kernel` | 2 | Mon 09:44 | {info:Review} |  |
| **Failed to start Network Time Synchronization** · Retried successfully | `systemd` | 1 | Sat 08:12 | {off:Harmless} |  |

3. **Heatmap (day × hour)**: "Errors by day and hour" · span 12
- Source chip (Expert): `journalctl -p err`
- Rows: Thu, Fri, Sat, Sun, Mon, Tue, Wed · Columns: hours 00-23 · unit "errors"
- Values (row by row, 24 per row):

```
Thu  1 1 0 0 1 0 0 1 0 0 0 0 0 1 0 0 0 0 0 1 0 0 0 0
Fri  0 1 0 0 0 1 0 0 1 0 0 0 0 1 0 0 1 0 0 1 0 0 0 0
Sat  0 0 0 1 1 0 1 1 0 0 0 0 0 0 1 1 0 0 0 0 0 0 1 0
Sun  0 1 0 0 1 0 0 0 0 0 1 0 0 0 0 0 1 1 0 0 0 0 0 0
Mon  0 0 1 0 0 1 0 0 1 0 0 1 0 0 1 0 3 0 1 0 0 0 0 0
Tue  1 1 0 0 1 0 0 0 1 0 0 1 0 1 0 0 0 0 1 1 0 1 0 0
Wed  2 0 1 1 0 0 1 0 0 9 1 1 1 0 0 0 0 1 0 0 0 5 1 0
```

#### 12.3.3 Logs › Crashes: `logs/crashes`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Crashes | 16 |  | - | last 7 days | - | - |
| Applications | 3 |  | - |   | - | - |
| Core dumps stored | 420 | MB | - |   | - | - |

2. **Data table**: "Crash reports" · span 12
- Source chip (Expert): `coredumpctl list · /var/crash`

| Time | Application | Signal | Source | Count [n] | Status | Row action |
| --- | --- | --- | --- | --- | --- | --- |
| Today 21:20 | `bluetoothd` | SIGSEGV | systemd-coredump | 14 | {crit:Recurring} | `bt` |
| Mon 16:12 | `chrome (renderer)` | SIGKILL (OOM) | kernel | 1 | {warn:Resolved} |  |
| Mon 11:47 | `gnome-calculator` | SIGABRT | apport | 1 | {off:Reported} |  |

#### 12.3.4 Logs › Kernel: `logs/kernel`

1. **Data table**: "Kernel messages" · span 12
- Source chip (Expert): `journalctl -k`
- Search: "Filter kernel log"

| Time | Level | Subsystem | Message [w] |
| --- | --- | --- | --- |
| 21:20:14 | {crit:err} | traps | `bluetoothd[7702]: segfault at 0 ip 000055d1c2a4 error 4` |
| Mon 16:12 | {crit:err} | oom | `Out of memory: Killed process 30412 (chrome)` |
| 09:00:17 | {crit:err} | ACPI | `ACPI BIOS Error (bug): AE_NOT_FOUND` |
| 08:58:07 | {off:info} | iwlwifi | `loaded firmware version 89.202a2f7b.0 so-a0-gf-a0-89.ucode` |
| 08:58:06 | {off:info} | nvme | `nvme0: 8/0/0 default/read/poll queues` |
| Mon 10:02 | {off:info} | usb | `usb 3-2: new SuperSpeed USB device number 4` |
| Mon 09:44 | {warn:warn} | usb | `usb 3-2: USB disconnect, device number 3` |

#### 12.3.5 Logs › Boots: `logs/boots`

1. **Bar chart**: "Boot duration" · span 6
- Subtitle: "last 10 boots"
- Source chip (Expert): `systemd-analyze`
- Height 200px · Y max auto · unit "s" · highlighted (warn + value label): index 9
- Bars (label: value): 8 Oct: 29; 10 Oct: 28; 12 Oct: 31; 13 Oct: 27; 14 Oct: 28; 15 Oct: 30; 16 Oct: 29; 17 Oct: 28; 19 Oct: 28; Today: 41.2

2. **Data table**: "Boot history" · span 6
- Source chip (Expert): `journalctl --list-boots`

| Started | Ended | Duration | Shutdown | Kernel |
| --- | --- | --- | --- | --- |
| Today 08:58 | - | 12 h 36 m | {info:Running} | `6.8.0-47` |
| Mon 08:41 | Today 02:14 | 1 d 17 h | {crit:Unclean} | `6.8.0-45` |
| Sat 10:12 | Mon 08:40 | 1 d 22 h | {ok:Clean} | `6.8.0-45` |
| Thu 09:01 | Sat 10:11 | 2 d 1 h | {ok:Clean} | `6.8.0-45` |

### 12.4 Activity (`activity`)

- Group: Monitoring · Icon: `hist` · Status badge: {ok:Healthy} (rule in section 3)
- Sub-pages: Timeline, Package History, Config Changes, Device Events, SysDoc Actions

#### 12.4.1 Activity › Timeline: `activity/timeline`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Changes | 18 |  | - | last 7 days | - | - |
| Package changes | 9 |  | - |   | - | - |
| Configuration changes | 3 |  | - |   | - | - |
| Sign-ins | 14 |  | - |   | - | - |
| SysDoc actions | 1 |  | - |   | - | - |

2. **Activity timeline**: "Change timeline" · span 8

| Time | Title | Detail | Dot |
| --- | --- | --- | --- |
| 10:40 | Unattended upgrade: libc6, openssl, sudo | System · Reboot required | info |
| 09:12 | bluetooth.service entered crash loop | Services · SIGSEGV · 14 restarts | crit |
| 08:58 | Boot completed in 41.2 s | System · Kernel 6.8.0-47-generic | ok |
| 02:14 | Unclean shutdown | Power · Battery depleted | warn |
| Tue 18:41 | linux-firmware upgraded | Packages · …0ubuntu2.5 | default |
| Tue 18:40 | Kernel upgraded 6.8.0-45 → 6.8.0-47 | Packages · Software Updater | default |
| Tue 15:22 | Installed snap: code 1.94.2 | Packages · 340 MB | default |
| Tue 11:05 | Disabled cups-browsed.service | SysDoc · Reversible | info |
| Mon 21:30 | crontab changed (riya): +backup-photos.sh | Jobs · 0 21 * * * | default |
| Mon 16:12 | OOM kill: chrome renderer (pid 30412) | Logs · RSS 1.8 GB | warn |
| Mon 10:02 | USB storage connected: PHOTOS (64 GB) | Devices · usb 3-2 | default |

3. **Donut chart**: "Changes by type" · span 4
- Subtitle: "7 days"
- Center: "18" / "changes"

| Segment | Value | Color | Display value |
| --- | --- | --- | --- |
| Packages | 9 | slot 1 | - |
| Configuration | 3 | slot 2 | - |
| Services | 2 | slot 3 | - |
| Devices | 3 | slot 4 | - |
| Jobs | 1 | slot 5 | - |

#### 12.4.2 Activity › Package History: `activity/packages`

1. **Data table**: "Package history" · span 12
- Source chip (Expert): `/var/log/apt/history.log · snap changes`
- Search: "Search packages"
- Filter chips on column 2: All / Install / Upgrade / Remove

| Date | Action | Package | From | To | Source |
| --- | --- | --- | --- | --- | --- |
| Today 10:40 | {info:Upgrade} | libc6 | `2.39-0ubuntu8.2` | `2.39-0ubuntu8.3` | unattended-upgrades |
| Today 10:40 | {info:Upgrade} | openssl | `3.0.13-0ubuntu3.3` | `3.0.13-0ubuntu3.4` | unattended-upgrades |
| Today 10:40 | {info:Upgrade} | sudo | `1.9.15p5-3ubuntu5` | `1.9.15p5-3ubuntu5.1` | unattended-upgrades |
| Tue 18:40 | {info:Upgrade} | linux-image-generic | `6.8.0-45` | `6.8.0-47` | Software Updater |
| Tue 18:41 | {info:Upgrade} | linux-firmware | `…0ubuntu2.4` | `…0ubuntu2.5` | Software Updater |
| Tue 15:22 | {ok:Install} | code (snap) | - | `1.94.2` | snap |
| Sat 19:03 | {ok:Install} | com.spotify.Client | - | `1.2.45` | flatpak |
| Fri 12:20 | {crit:Remove} | thunderbird (snap) | `128.3.1` | - | snap |
| Thu 09:30 | {ok:Install} | httpie | - | `3.2.2` | pip |

#### 12.4.3 Activity › Config Changes: `activity/config`

1. **Data table**: "Configuration changes" · span 12
- Source chip (Expert): `inotify watcher · /etc`

| Time | File | Change | Changed by | (action) |
| --- | --- | --- | --- | --- |
| Tue 11:05 | `/etc/systemd/system/multi-user.target.wants/cups-browsed.service` | Removed (symlink) | SysDoc | [View diff](act:toast:Symlink removed by systemctl disable cups-browsed) |
| Tue 15:20 | `/etc/apt/sources.list.d/vscode.list` | Added | apt | [View diff](act:toast:+ deb [arch=amd64] https://packages.microsoft.com/repos/code stable main) |
| Mon 21:30 | `/var/spool/cron/crontabs/riya` | ✓ | riya | [View diff](act:toast:+ 0 21 * * * ~/bin/backup-photos.sh) |

#### 12.4.4 Activity › Device Events: `activity/devices`

1. **Data table**: "Device events" · span 12
- Source chip (Expert): `udev · journal`

| Time | Event | Device | Bus | ID |
| --- | --- | --- | --- | --- |
| Today 21:20 | {warn:Disconnected} | Wireless earbuds | Bluetooth | `A4:C1:38:..:5E` |
| Today 21:20 | {ok:Connected} | Wireless earbuds | Bluetooth | `A4:C1:38:..:5E` |
| Mon 10:02 | {ok:Connected} | USB storage PHOTOS (64 GB) | USB 3.2 | `0781:5583` |
| Mon 09:44 | {warn:Disconnected} | USB storage PHOTOS (64 GB) | USB 3.2 | `0781:5583` |
| Sat 18:30 | {ok:Connected} | USB mouse | USB 2.0 | `046d:c077` |

#### 12.4.5 Activity › SysDoc Actions: `activity/sysdoc`

1. **Data table**: "Remediation log" · span 12
- Info tooltip: "Every change SysDoc made, with undo where possible."

| Time | Action | Target | Result | Undo |
| --- | --- | --- | --- | --- |
| Tue 11:05 | Disabled service | `cups-browsed.service` | {ok:Succeeded} | [Undo](act:undo:cups) |

### 12.5 System (`system`)

- Group: System · Icon: `sys` · Status badge: {info:Info} (rule in section 3)
- Sub-pages: Overview, Boot, Updates, Kernel, Time & Locale

#### 12.5.1 System › Overview: `system/overview`

1. **Key-value list**: "Operating system" · span 6
- Source chip (Expert): `/etc/os-release · uname -a`

| Key | Value |
| --- | --- |
| Distribution | Ubuntu 24.04.1 LTS (Noble Numbat) |
| Kernel | `6.8.0-47-generic` |
| Architecture | x86_64 |
| Desktop | GNOME 46 · Wayland |
| Init system | systemd 255 |
| Hostname | riya-laptop |
| Virtualization | None (bare metal) |
| Machine ID | `4f1c9d…a72e` |

2. **Key-value list**: "Status" · span 6

| Key | Value |
| --- | --- |
| Uptime | 12 h 36 m |
| Last boot | Today 08:58 · 41.2 s |
| Reboot required | {warn:Yes · libc6, openssl, sudo} |
| Load average | `0.84 0.92 1.10` |
| Users logged in | 1 |
| Running services | 142 |
| Ubuntu Pro | {off:Not attached} |
| Livepatch | {off:Disabled} |

3. **Lifecycle bar (section 5.20)**: "Release lifecycle" · span 12
- Subtitle: "Ubuntu 24.04 LTS"
- Info tooltip: "How long this Ubuntu version gets security updates."

Fill and "Today" marker at 50.4%. Left label "Released · Apr 2024", right label "Standard support ends · Apr 2029".

#### 12.5.2 System › Boot: `system/boot`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Last boot | 41.2 | s | {warn:Warning} | Average 28.6 s | - | - |
| Firmware | 8.1 | s | - |   | - | - |
| Loader | 2.4 | s | - |   | - | - |
| Kernel | 3.9 | s | - |   | - | - |
| Userspace | 26.8 | s | - |   | - | - |

2. **Horizontal bar list**: "Slowest units" · span 6
- Source chip (Expert): `systemd-analyze blame`

| Label | Value | Shown as |
| --- | --- | --- |
| `snapd.service` | 6.2 | 6.2 s |
| `NetworkManager-wait-online.service` | 5.1 | 5.1 s |
| `plymouth-quit-wait.service` | 4.4 | 4.4 s |
| `docker.service` | 3.1 | 3.1 s |
| `dev-nvme0n1p2.device` | 2.2 | 2.2 s |
| `apt-daily.service` | 1.7 | 1.7 s |
| `systemd-udev-settle.service` | 1.2 | 1.2 s |

3. **Bar chart**: "Boot time history" · span 6
- Subtitle: "last 10 boots"
- Height 220px · Y max auto · unit "s" · highlighted (warn + value label): index 9
- Bars (label: value): 8 Oct: 29; 10 Oct: 28; 12 Oct: 31; 13 Oct: 27; 14 Oct: 28; 15 Oct: 30; 16 Oct: 29; 17 Oct: 28; 19 Oct: 28; Today: 41.2

#### 12.5.3 System › Updates: `system/updates`

Header actions: [**Install security updates**](act:fix:sec) (hidden once done), then Refresh and Export.

Tab count pill: 12.

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Pending | 12 |  | {warn:Warning} |   | - | - |
| Security | 4 |  | {warn:Warning} |   | - | - |
| Last check | 2 | h ago | - |   | - | - |
| Automatic security updates | On |  | - |   | - | - |
| Reboot required | Yes |  | {info:Info} |   | - | - |

2. **Data table**: "Pending updates" · span 12
- Source chip (Expert): `apt list --upgradable`
- Row selection · bulk button [Install selected](act:fix:sec)
- Total count shown: 12
- Table ID: `pending`

| Package | Installed [x] | Candidate | Priority | Size [n] |
| --- | --- | --- | --- | --- |
| openssl | `3.0.13-0ubuntu3.4` | `3.0.13-0ubuntu3.5` | {warn:Security} | 1.2 MB |
| libssl3t64 | `3.0.13-0ubuntu3.4` | `3.0.13-0ubuntu3.5` | {warn:Security} | 1.9 MB |
| sudo | `1.9.15p5-3ubuntu5.1` | `1.9.15p5-3ubuntu5.2` | {warn:Security} | 0.9 MB |
| linux-firmware | `…0ubuntu2.5` | `…0ubuntu2.6` | {warn:Security} | 298 MB |
| google-chrome-stable | `130.0.6723.58` | `130.0.6723.69` | {off:Regular} | 112 MB |
| git | `1:2.43.0-1ubuntu7.1` | `1:2.43.0-1ubuntu7.2` | {off:Regular} | 3.7 MB |
| docker-ce | `5:27.3.1` | `5:27.3.2` | {off:Regular} | 25 MB |
| code (snap) | `1.94.2` | `1.95.0` | {off:Regular} | 106 MB |

3. **Data table**: "Update history" · span 12

| Date | Packages [n] | Source | Result |
| --- | --- | --- | --- |
| Today 10:40 | 3 | unattended-upgrades | {ok:Success} |
| Tue 18:40 | 6 | Software Updater | {ok:Success} |
| Thu 06:24 | 2 | unattended-upgrades | {ok:Success} |

#### 12.5.4 System › Kernel: `system/kernel`

1. **Key-value list**: "Running kernel" · span 5
- Source chip (Expert): `uname -r · /proc/cmdline`

| Key | Value |
| --- | --- |
| Version | `6.8.0-47-generic` |
| Tainted | {ok:No (0)} |
| Modules loaded | 187 |
| Command line | `BOOT_IMAGE=/vmlinuz-6.8.0-47-generic root=UUID=… ro quiet splash` |

2. **Data table**: "Installed kernels" · span 7

| Version | Status | Size [n] | Installed | (action) |
| --- | --- | --- | --- | --- |
| `6.8.0-47-generic` | {ok:Running} | 312 MB | Tue 18:40 |   |
| `6.8.0-45-generic` | {info:Fallback} | 310 MB | 12 Sep |   |
| `6.8.0-41-generic` | {off:Old} | 320 MB | 14 Aug | [Remove](act:toast:Old kernel queued for removal.) |

3. **Data table**: "Kernel modules" · span 12
- Source chip (Expert): `lsmod`
- Search: "Search modules"
- Page size 8
- Total count shown: 187

| Module | Size [n] | Used by |
| --- | --- | --- |
| `i915` | 4.2 MB | 34 |
| `iwlmvm` | 704 KB | 1 |
| `iwlwifi` | 598 KB | iwlmvm |
| `btusb` | 77 KB | 0 |
| `bluetooth` | 1.0 MB | btusb, btrtl, btintel |
| `nvme` | 61 KB | 2 |
| `snd_hda_intel` | 61 KB | 3 |
| `kvm_intel` | 487 KB | 0 |
| `overlay` | 212 KB | 2 |
| `xt_conntrack` | 12 KB | 2 |

#### 12.5.5 System › Time & Locale: `system/time`

1. **Key-value list**: "Date and time" · span 6
- Source chip (Expert): `timedatectl`

| Key | Value |
| --- | --- |
| Local time | Wed 21 Oct 2026 21:34 IST |
| Time zone | Asia/Kolkata (IST, +0530) |
| NTP synchronized | {ok:Yes} |
| Time server | ntp.ubuntu.com |
| RTC in local time | No |

2. **Key-value list**: "Locale" · span 6
- Source chip (Expert): `localectl`

| Key | Value |
| --- | --- |
| System locale | `LANG=en_IN.UTF-8` |
| Keyboard layout | us |
| Input method | IBus |
| Units | Metric · °C |

### 12.6 Hardware (`hardware`)

- Group: System · Icon: `chip` · Status badge: {ok:Healthy} (rule in section 3)
- Sub-pages: Summary, Processor, Memory, Storage Devices, Graphics, Sensors, Peripherals, Firmware

#### 12.6.1 Hardware › Summary: `hardware/summary`

1. **Key-value list**: "Machine" · span 6
- Source chip (Expert): `/sys/class/dmi/id`

| Key | Value |
| --- | --- |
| Vendor | Lenovo |
| Model | IdeaPad 5 14IAL7 |
| Chassis | Laptop |
| Serial number | `PF3K9X2M` |
| BIOS | JHCN52WW · 11 Mar 2024 |
| Secure Boot | {ok:Enabled} |

2. **Status list**: "Components" · span 6

| Icon | Label | Metric | Badge | Opens |
| --- | --- | --- | --- | --- |
| `chip` | Processor | i5-1235U · 54 °C | {ok:Healthy} | `hardware/processor` |
| `layers` | Memory | 8 GB LPDDR4X | {ok:Healthy} | `hardware/memory` |
| `db` | Storage | NVMe 256 GB · SMART passed | {ok:Healthy} | `hardware/storage` |
| `bat` | Battery | 71% of design | {warn:Degraded} | `power/battery` |
| `mon` | Graphics | Iris Xe · i915 | {ok:Healthy} | `hardware/graphics` |
| `fan` | Fans | Not reported | {off:Unknown} | `hardware/sensors` |
| `usb` | Peripherals | 11 devices · 0 missing drivers | {crit:1 unstable} | `hardware/peripherals` |
| `bolt` | Firmware | 1 update available | {info:Update} | `hardware/firmware` |

#### 12.6.2 Hardware › Processor: `hardware/processor`

1. **Gauge group**: "Live" · span 6

| Caption | Value | Max | Center text | Subtitle |
| --- | --- | --- | --- | --- |
| Load | 12 | 100 | 12% | utilization |
| Temperature | 54 | 100 | 54 °C | package |
| Frequency | 1.3 | 4.4 | 1.3 GHz | of 4.4 GHz |

2. **Key-value list**: "Specifications" · span 6
- Source chip (Expert): `lscpu`

| Key | Value |
| --- | --- |
| Model | 12th Gen Intel Core i5-1235U |
| Cores | 10 (2 performance + 8 efficient) |
| Threads | 12 |
| Base / max frequency | 1.3 / 4.4 GHz |
| L3 cache | 12 MB |
| Microcode | `0x4123` |
| Features | ‹VT-x›‹AES-NI›‹AVX2›‹SHA› |

3. **Data table**: "Cores" · span 12
- Source chip (Expert): `/sys/devices/system/cpu`

| Core | Type | Frequency [n] | Utilization [n] | Temperature [n] |
| --- | --- | --- | --- | --- |
| P0 | Performance · 2 threads | 1640 MHz | [meter 16%] | 56 °C |
| P1 | Performance · 2 threads | 1320 MHz | [meter 10%] | 55 °C |
| E0 | Efficient | 1180 MHz | [meter 22%] | 52 °C |
| E1 | Efficient | 980 MHz | [meter 11%] | 52 °C |
| E2 | Efficient | 1210 MHz | [meter 4%] | 52 °C |
| E3 | Efficient | 1050 MHz | [meter 7%] | 52 °C |
| E4 | Efficient | 1400 MHz | [meter 16%] | 53 °C |
| E5 | Efficient | 920 MHz | [meter 3%] | 53 °C |
| E6 | Efficient | 1100 MHz | [meter 8%] | 53 °C |
| E7 | Efficient | 1010 MHz | [meter 5%] | 53 °C |

#### 12.6.3 Hardware › Memory: `hardware/memory`

1. **Key-value list**: "Modules" · span 6
- Source chip (Expert): `dmidecode -t memory`

| Key | Value |
| --- | --- |
| Total | 8 GB (7.6 GB usable) |
| Type | LPDDR4X · 4266 MT/s |
| Form factor | Soldered on board |
| Slots | 0 free · not upgradeable |
| ECC | No |

2. **Checklist**: "Health" · span 6

| State | Title | Detail | Action |
| --- | --- | --- | --- |
| `pass` | No memory pressure stalls above 20% today |   |   |
| `warn` | 1 out-of-memory kill in 7 days | Mon 16:12 · chrome renderer | [View](go:logs/kernel) |
| `na` | Error correction (ECC) reporting | Not available on this memory type |   |
| `na` | Full memory test | Needs a reboot into memtest86+ | [How to run](act:toast:Restart → hold Shift → Memory test (memtest86+).) |

#### 12.6.4 Hardware › Storage Devices: `hardware/storage`

1. **Data table**: "Disks" · span 12
- Source chip (Expert): `lsblk · smartctl`

| Device | Model | Type | Size [n] | Interface | SMART | Life left [n] | Temperature [n] |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `nvme0n1` | Samsung PM991a | SSD (NVMe) | 256 GB | PCIe 3.0 x4 | {ok:Passed} | [meter:ok 88%] | 38 °C |

2. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Percentage used | 12 | % | - | of rated endurance | - | - |
| Data written | 18.4 | TB | - |   | - | - |
| Power-on hours | 6,120 | h | - |   | - | - |
| Unsafe shutdowns | 41 |  | - | +1 today | - | - |
| Media errors | 0 |  | - |   | - | - |
| Spare capacity | 100 | % | - |   | - | - |

3. **Line chart**: "Drive temperature" · span 12
- Source chip (Expert): `nvme smart-log`
- Height 180px · Y axis 20 to 70 · unit "°C"
- Series: NVMe (slot 1, gen[61, 37, 3, 2, 30, 70, 38])
- X axis: time (current range)

#### 12.6.5 Hardware › Graphics: `hardware/graphics`

1. **Key-value list**: "Adapter" · span 6
- Source chip (Expert): `lspci -k · glxinfo -B`

| Key | Value |
| --- | --- |
| GPU | Intel Iris Xe (Alder Lake-UP3 GT2) |
| Kernel driver | `i915` |
| OpenGL | Mesa 24.0.9 · 4.6 |
| Vulkan | 1.3 |
| Video decode | VA-API (iHD) |

2. **Checklist**: "Driver checks" · span 6

| State | Title | Detail | Action |
| --- | --- | --- | --- |
| `pass` | Recommended driver installed | ubuntu-drivers reports no alternatives |   |
| `pass` | Hardware acceleration active |   |   |
| `na` | GPU temperature | Not exposed by i915 |   |

#### 12.6.6 Hardware › Sensors: `hardware/sensors`

1. **Gauge group**: "Temperatures" · span 12
- Source chip (Expert): `sensors · /sys/class/hwmon`

| Caption | Value | Max | Center text | Subtitle |
| --- | --- | --- | --- | --- |
| coretemp | 54 | 100 | 54 °C | CPU package |
| coretemp | 56 | 100 | 56 °C | core max |
| nvme | 38 | 85 | 38 °C | NVMe |
| BAT0 | 31 | 60 | 31 °C | battery |
| iwlwifi | 42 | 100 | 42 °C | Wi-Fi |
| acpitz | 50 | 100 | 50 °C | ACPI zone |

2. **Line chart**: "Temperature history" · span 12
- Source chip (Expert): `sensors`
- Height 220px · Y axis 20 to 100 · unit "°C"
- Series: CPU (slot 1, gen[21, 52, 6, 3, 35, 92, 54]); NVMe (slot 2, gen[61, 37, 3, 2, 30, 70, 38]); Battery (slot 3, gen[71, 31, 2, 1, 25, 45, 31])
- X axis: time (current range)
- Threshold line at 95: "Throttle 95 °C"

3. **Alert banner**: "Fan speed not reported" · span 12
- State: `info` · Title: "Fan speed not reported" · Text: "This laptop’s embedded controller does not expose fan RPM to Linux."

#### 12.6.7 Hardware › Peripherals: `hardware/peripherals`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| USB devices | 3 |  | - |   | - | - |
| PCI devices | 18 |  | - |   | - | - |
| Bluetooth | 2 |  | - | 1 connected | - | - |
| Missing drivers | 0 |  | - |   | - | - |

2. **Data table**: "Devices" · span 12
- Source chip (Expert): `lsusb · lspci -k · bluetoothctl`
- Search: "Search devices"
- Filter chips on column 2: All / USB / PCI / Bluetooth / Input

| Device | Bus | Vendor | Driver | Status | Row action |
| --- | --- | --- | --- | --- | --- |
| USB mouse | USB | Logitech | `usbhid` | {ok:Connected} |  |
| Integrated camera | USB | Chicony | `uvcvideo` | {ok:Idle} |  |
| Bluetooth adapter | USB | Intel | `btusb` | {crit:Unstable} | `bt` |
| Wi-Fi 6 AX201 | PCI | Intel | `iwlwifi` | {ok:Active} |  |
| NVMe controller | PCI | Samsung | `nvme` | {ok:Active} |  |
| Smart Sound audio | PCI | Intel | `snd_hda_intel` | {ok:Active} |  |
| Thunderbolt 4 controller | PCI | Intel | `thunderbolt` | {ok:Active} |  |
| Wireless earbuds | Bluetooth | - | `bluez` | {warn:Reconnecting} |  |
| Phone | Bluetooth | - | `bluez` | {off:Paired} |  |
| Touchpad | Input | ELAN | `i2c_hid` | {ok:Active} |  |
| Keyboard | Input | AT Translated | `atkbd` | {ok:Active} |  |

#### 12.6.8 Hardware › Firmware: `hardware/firmware`

1. **Key-value list**: "Platform firmware" · span 5
- Source chip (Expert): `/sys/class/dmi/id · mokutil`

| Key | Value |
| --- | --- |
| BIOS vendor | LENOVO |
| Version | `JHCN52WW` |
| Release date | 11 Mar 2024 |
| Boot mode | UEFI |
| Secure Boot | {ok:Enabled} |
| TPM | 2.0 · present |

2. **Data table**: "Firmware updates" · span 7
- Source chip (Expert): `fwupdmgr get-updates`

| Device | Current | Available | Vendor | (action) |
| --- | --- | --- | --- | --- |
| Touchpad | `1.4` | `1.6` | Synaptics | [**Update**](act:toast:Firmware update scheduled for next reboot.) |
| System firmware | `JHCN52WW` | - | Lenovo | Up to date |
| NVMe SSD | `3L1QFXV7` | - | Samsung | Not on LVFS |

### 12.7 Power (`power`)

- Group: System · Icon: `bat` · Status badge: {warn:Warning} (rule in section 3)
- Sub-pages: Battery, Power Profile, Sleep

#### 12.7.1 Power › Battery: `power/battery`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Charge | 64 | % | - | Discharging | Usage bar 64% | - |
| Health | 71 | % | {warn:Warning} | of design capacity | - | - |
| Time remaining | 3 h 10 m |  | - | Estimate | - | - |
| Power draw | 8.4 | W | - |   | Sparkline slot 2, gen[81, 8, 3, 2, 3, 25, 8.4] | - |
| Capacity | 40.5 | Wh | - | Design 57 Wh | - | - |
| Cycle count | N/A |  | - | Not reported | - | - |

2. **Line chart**: "Charge level" · span 8
- Source chip (Expert): `upower history`
- Height 220px · Y axis 0 to 100 · unit "%"
- Series: Charge (slot 1, gen[91, 60, 30, 8, 1, 100, 64])
- X axis: time (current range)
- Threshold line at 10: "Low 10%"

3. **Gauge**: "Battery health" · span 4
- Info tooltip: "How much charge the battery holds compared with when it was new."
- Value 71 of 100 · arc `warn` · center "71%" · subtitle "40.5 of 57 Wh" · caption below: "Replace below 60%"

4. **Line chart**: "Health trend" · span 6
- Subtitle: "12 months"
- Height 180px · Y axis 50 to 100 · unit "%"
- Series: Health (slot 1, data [83,82,81,80,79,78,77,76,75,74,72,71])
- X labels: Nov, Dec, Jan, Feb, Mar, Apr, May, Jun, Jul, Aug, Sep, Oct

5. **Data table**: "Top power consumers" · span 6
- Subtitle: "estimate"
- Source chip (Expert): `powertop`
- Empty text: "Requires Full access"

| Process | Power [n] | Share [n] |
| --- | --- | --- |
| chrome | 2.9 W | [meter 35%] |
| Display backlight | 1.9 W | [meter 23%] |
| gnome-shell | 1.4 W | [meter 17%] |
| code | 0.8 W | [meter 10%] |
| Wi-Fi | 0.7 W | [meter 8%] |

#### 12.7.2 Power › Power Profile: `power/profile`

1. **Radio cards (section 5.21)**: "Power mode" · span 6
- Source chip (Expert): `powerprofilesctl`

| Option | Description | Checked |
| --- | --- | --- |
| Power Saver | Longer battery life | No |
| Balanced | Default | Yes |
| Performance | Plugged in only | No |

Radio group name: `pp`.

2. **Key-value list**: "Battery policies" · span 6
- Source chip (Expert): `/etc/UPower/UPower.conf`

| Key | Value |
| --- | --- |
| Low battery warning | 10% |
| Critical action | {warn:Not configured} [Configure](act:fix:batt) |
| Charge threshold | Supported · Off |
| Screen blank | 5 min |
| Automatic suspend | 20 min on battery |

#### 12.7.3 Power › Sleep: `power/sleep`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Suspends | 9 |  | - | last 7 days | - | - |
| Failures | 0 |  | - |   | - | - |
| Avg resume time | 1.8 | s | - |   | - | - |
| Time asleep | 41 | h | - |   | - | - |

2. **Data table**: "Sleep events" · span 12
- Source chip (Expert): `journalctl -u systemd-suspend`

| Time | Event | Duration | Wake source | Result |
| --- | --- | --- | --- | --- |
| Mon 23:40 | Suspend | 7 h 48 m | Lid open | {ok:Success} |
| Mon 13:10 | Suspend | 42 m | Power button | {ok:Success} |
| Sun 22:55 | Suspend | 9 h 12 m | Not recorded | {ok:Success} |
| Sat 18:05 | Suspend | 1 h 30 m | USB | {ok:Success} |

### 12.8 Display & Audio (`display`)

- Group: System · Icon: `mon` · Status badge: {ok:Healthy} (rule in section 3)
- Sub-pages: Displays, Audio

#### 12.8.1 Display & Audio › Displays: `display/displays`

1. **Key-value list**: "Session" · span 4
- Source chip (Expert): `$XDG_SESSION_TYPE`

| Key | Value |
| --- | --- |
| Display server | Wayland |
| Compositor | Mutter (GNOME 46) |
| Fractional scaling | Off |

2. **Data table**: "Connected displays" · span 8

| Output | Resolution | Refresh | Scale | Primary | Status |
| --- | --- | --- | --- | --- | --- |
| `eDP-1` | 1920 × 1080 | 60 Hz | 100% | ✓ | {ok:Connected} |
| `HDMI-A-1` | - | - | - | - | {off:Disconnected} |

#### 12.8.2 Display & Audio › Audio: `display/audio`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Output | Speakers |  | - | Volume 45% | - | - |
| Input | Internal mic |  | - | Not muted | - | - |
| Sound server | PipeWire |  | - | 1.0.5 | - | - |
| Active streams | 1 |  | - | chrome | - | - |

2. **Data table**: "Output devices" · span 6
- Source chip (Expert): `pactl list sinks`

| Device | Volume | Default |
| --- | --- | --- |
| Laptop speakers | [meter:ok 45%] | ✓ |
| Wireless earbuds (A2DP) | [meter:ok 60%] | - |
| HDMI output | [meter:ok 100%] | - |

3. **Data table**: "Input devices" · span 6
- Source chip (Expert): `pactl list sources`

| Device | Level | Default |
| --- | --- | --- |
| Internal microphone | [meter:ok 70%] | ✓ |
| Earbuds microphone (HFP) | [meter:ok 50%] | - |

### 12.9 Processes (`processes`)

- Group: Workloads · Icon: `layers` · Status badge: {ok:Healthy} (rule in section 3)
- Sub-pages: All Processes, Top Consumers

#### 12.9.1 Processes › All Processes: `processes/all`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Total | 312 |  | - |   | - | - |
| Running | 3 |  | - |   | - | - |
| Sleeping | 309 |  | - |   | - | - |
| Zombie | 0 |  | - |   | - | - |
| Threads | 1,204 |  | - |   | - | - |

2. **Data table**: "Processes" · span 12
- Source chip (Expert): `ps aux`
- Search: "Filter by name, PID or user"
- Filter chips on column 3: All / riya / root
- Page size 14
- Total count shown: 312
- Inspect: row click opens the row-inspect drawer with actions "End process", "Open Performance"
- Table ID: `procs`

| Name | PID [n] | User | State | CPU [n] | Memory [n] | Threads [n, ,x] | Started [x] |
| --- | --- | --- | --- | --- | --- | --- | --- |
| chrome | 4121 | riya | Sleeping | 4.2% | 842 MB | 38 | 08:59 |
| chrome (renderer) | 4388 | riya | Sleeping | 2.6% | 612 MB | 19 | 09:01 |
| gnome-shell | 1874 | riya | {ok:Running} | 3.8% | 410 MB | 24 | 08:59 |
| code | 5102 | riya | Sleeping | 1.9% | 486 MB | 31 | 15:24 |
| telegram-desktop | 2230 | riya | Sleeping | 0.4% | 280 MB | 22 | 08:59 |
| dropbox | 2301 | riya | Sleeping | 0.3% | 210 MB | 61 | 08:59 |
| postgres | 2811 | 999 | Sleeping | 0.2% | 142 MB | 1 | 09:00 |
| dockerd | 1320 | root | Sleeping | 0.3% | 96 MB | 18 | 08:58 |
| tracker-miner-fs-3 | 2051 | riya | Sleeping | 0.2% | 96 MB | 5 | 08:59 |
| Xwayland | 1990 | riya | Sleeping | 0.6% | 88 MB | 6 | 08:59 |
| gnome-terminal-server | 6012 | riya | Sleeping | 0.3% | 64 MB | 5 | 20:12 |
| containerd | 1288 | root | Sleeping | 0.1% | 52 MB | 14 | 08:58 |
| snapd | 1043 | root | Sleeping | 0.1% | 44 MB | 16 | 08:58 |
| systemd-journald | 412 | root | Sleeping | 0.1% | 38 MB | 1 | 08:58 |
| NetworkManager | 1011 | root | Sleeping | 0.1% | 22 MB | 4 | 08:58 |
| pipewire | 1702 | riya | Sleeping | 0.5% | 18 MB | 3 | 08:59 |
| redis-server | 2840 | 999 | Sleeping | 0.1% | 12 MB | 5 | 09:00 |
| systemd | 1 | root | Sleeping | 0.0% | 14 MB | 1 | 08:58 |
| bluetoothd | 7740 | root | {ok:Running} | 0.1% | 6 MB | 1 | 21:20 |
| bash | 6024 | riya | Sleeping | 0.0% | 5 MB | 1 | 20:12 |
| top | 6110 | riya | {ok:Running} | 0.3% | 4 MB | 1 | 21:30 |
| cron | 1036 | root | Sleeping | 0.0% | 3 MB | 1 | 08:58 |

#### 12.9.2 Processes › Top Consumers: `processes/top`

1. **Horizontal bar list**: "By CPU" · span 6
- Subtitle: "now"

| Label | Value | Shown as |
| --- | --- | --- |
| chrome | 4.2 | 4.2% |
| gnome-shell | 3.8 | 3.8% |
| chrome (renderer) | 2.6 | 2.6% |
| code | 1.9 | 1.9% |
| Xwayland | 0.6 | 0.6% |
| pipewire | 0.5 | 0.5% |
| telegram-desktop | 0.4 | 0.4% |
| dropbox | 0.3 | 0.3% |

2. **Horizontal bar list**: "By memory" · span 6
- Subtitle: "resident"

| Label | Value | Shown as |
| --- | --- | --- |
| chrome | 842 | 842 MB |
| chrome (renderer) | 612 | 612 MB |
| gnome-shell | 410 | 410 MB |
| code | 486 | 486 MB |
| telegram-desktop | 280 | 280 MB |
| dropbox | 210 | 210 MB |
| postgres | 142 | 142 MB |
| dockerd | 96 | 96 MB |

### 12.10 Services (`services`)

- Group: Workloads · Icon: `cog` · Status badge: {crit:Critical} (rule in section 3)
- Sub-pages: System Services, User Services, Startup Applications

#### 12.10.1 Services › System Services: `services/system`

Tab count pill: 1 (red).

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Running | 142 |  | - |   | - | - |
| Failed | 1 |  | {crit:Critical} | bluetooth.service | - | - |
| Inactive | 38 |  | - |   | - | - |
| Enabled at boot | 61 |  | - |   | - | - |
| Restarts today | 14 |  | {warn:Warning} |   | - | - |

2. **Alert banner**: "bluetooth.service is in a crash loop" · span 12
- State: `crit` · Title: "bluetooth.service is in a crash loop" · Text: "14 restarts since 09:12 · exit status 11 (SIGSEGV)" · Buttons: [**Inspect**](act:bt)

3. **Data table**: "Units" · span 12
- Source chip (Expert): `systemctl list-units --type=service --all`
- Search: "Filter units"
- Filter chips on column 2: All / active / failed / inactive
- Page size 14
- Total count shown: 181
- Inspect: row click opens the row-inspect drawer with actions "Restart", "Stop", "Disable"
- Table ID: `svcs`

| Unit | Active | Sub | Enabled | Restarts [n] | Memory [n] | Row action |
| --- | --- | --- | --- | --- | --- | --- |
| **bluetooth.service** · Bluetooth service | {crit:failed} | auto-restart | {info:enabled} | 14 | - | `bt` |
| **NetworkManager.service** · Network Manager | {ok:active} | running | {info:enabled} | 0 | 22 MB |  |
| **docker.service** · Docker Application Container Engine | {ok:active} | running | {info:enabled} | 0 | 96 MB |  |
| **containerd.service** · containerd container runtime | {ok:active} | running | {info:enabled} | 0 | 52 MB |  |
| **snapd.service** · Snap Daemon | {ok:active} | running | {info:enabled} | 0 | 44 MB |  |
| **gdm.service** · GNOME Display Manager | {ok:active} | running | {info:enabled} | 0 | 9 MB |  |
| **systemd-journald.service** · Journal Service | {ok:active} | running | static | 0 | 38 MB |  |
| **systemd-resolved.service** · Network Name Resolution | {ok:active} | running | {info:enabled} | 0 | 8 MB |  |
| **systemd-timesyncd.service** · Network Time Synchronization | {ok:active} | running | {info:enabled} | 0 | 4 MB |  |
| **cups.service** · CUPS Scheduler | {ok:active} | running | {info:enabled} | 0 | 7 MB |  |
| **cups-browsed.service** · Make remote CUPS printers available locally | {off:inactive} | dead | disabled | 0 | - |  |
| **ufw.service** · Uncomplicated firewall | {ok:active} | exited | {info:enabled} | 0 | - |  |
| **apparmor.service** · Load AppArmor profiles | {ok:active} | exited | {info:enabled} | 0 | - |  |
| **unattended-upgrades.service** · Unattended Upgrades Shutdown | {ok:active} | running | {info:enabled} | 0 | 12 MB |  |
| **power-profiles-daemon.service** · Power Profiles daemon | {ok:active} | running | {info:enabled} | 0 | 6 MB |  |
| **thermald.service** · Thermal Daemon Service | {ok:active} | running | {info:enabled} | 0 | 5 MB |  |
| **avahi-daemon.service** · Avahi mDNS/DNS-SD Stack | {ok:active} | running | {info:enabled} | 0 | 3 MB |  |
| **ModemManager.service** · Modem Manager | {ok:active} | running | {info:enabled} | 0 | 8 MB |  |
| **packagekit.service** · PackageKit Daemon | {ok:active} | running | static | 0 | 28 MB |  |
| **accounts-daemon.service** · Accounts Service | {ok:active} | running | {info:enabled} | 0 | 6 MB |  |
| **polkit.service** · Authorization Manager | {ok:active} | running | static | 0 | 9 MB |  |
| **rsyslog.service** · System Logging Service | {ok:active} | running | {info:enabled} | 0 | 5 MB |  |
| **cron.service** · Regular background program processing daemon | {ok:active} | running | {info:enabled} | 0 | 3 MB |  |
| **anacron.service** · Run anacron jobs | {off:inactive} | dead | {info:enabled} | 0 | - |  |
| **wpa_supplicant.service** · WPA supplicant | {ok:active} | running | {info:enabled} | 0 | 7 MB |  |
| **fwupd.service** · Firmware update daemon | {off:inactive} | dead | static | 0 | - |  |
| **kerneloops.service** · Collects kernel crash signatures | {ok:active} | running | {info:enabled} | 0 | 2 MB |  |

#### 12.10.2 Services › User Services: `services/user`

1. **Data table**: "User units" · span 12
- Subtitle: "riya"
- Source chip (Expert): `systemctl --user`

| Unit | Active | Sub | Memory [n] |
| --- | --- | --- | --- |
| **pipewire.service** · PipeWire Multimedia Service | {ok:active} | running | 18 MB |
| **pipewire-pulse.service** · PipeWire PulseAudio | {ok:active} | running | 9 MB |
| **wireplumber.service** · Multimedia Service Session Manager | {ok:active} | running | 12 MB |
| **xdg-desktop-portal.service** · Portal service | {ok:active} | running | 7 MB |
| **tracker-miner-fs-3.service** · Tracker file system data miner | {ok:active} | running | 96 MB |
| **gvfs-daemon.service** · Virtual filesystem service | {ok:active} | running | 6 MB |
| **gnome-keyring-daemon.service** · GNOME Keyring | {ok:active} | running | 8 MB |
| **org.gnome.Shell@wayland.service** · GNOME Shell on Wayland | {ok:active} | running | 410 MB |
| **snap.telegram-desktop.service** · Telegram (snap) | {ok:active} | running | - |

#### 12.10.3 Services › Startup Applications: `services/startup`

1. **Data table**: "Started at login" · span 12
- Source chip (Expert): `~/.config/autostart · /etc/xdg/autostart`

| Name | Scope | Command [x] | Startup impact [n] | Enabled |
| --- | --- | --- | --- | --- |
| Telegram Desktop | User | `telegram-desktop -startintray` | [meter:ok 57% "1.2 s"] | [toggle:on] |
| Dropbox | User | `dropbox start -i` | [meter:ok 43% "0.9 s"] | [toggle:on] |
| Google Chrome | User | `google-chrome --no-startup-window` | [meter:warn 100% "2.1 s"] | [toggle:on] |
| GNOME Software | System | `gnome-software --gapplication-service` | [meter:ok 29% "0.6 s"] | [toggle:on] |
| Snap user daemon | System | `snap userd --autostart` | [meter:ok 10% "0.2 s"] | [toggle:on] |

### 12.11 Scheduled Jobs (`jobs`)

- Group: Workloads · Icon: `cal` · Status badge: {ok:Healthy} (rule in section 3)
- Sub-pages: Calendar, Cron, Systemd Timers, Anacron, At Jobs

#### 12.11.1 Scheduled Jobs › Calendar: `jobs/calendar`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Total jobs | 10 |  | - | cron · timer · anacron · at | - | - |
| Next run | 26 | min | - | sync-notes.sh | - | - |
| Failed | 0 |  | - | last 7 days | - | - |
| Missed | 1 |  | {warn:Warning} | e2scrub_all.timer | - | - |

2. **Swim-lane schedule (section 5.19)**: "Next 24 hours" · span 12
- Source chip (Expert): `crontab -l · systemctl list-timers`

| Lane (mono) | Runs at (hours from now) | Dot |
| --- | --- | --- |
| sync-notes.sh | 47 runs: 0.4, 0.9, 1.4 … every 0.5 h to 23.4 | small 6px |
| logrotate.timer | 2.4 | 10px |
| apt-daily-upgrade | 8.8 | 10px |
| apt-daily.timer | 9.1, 21.3 | 10px |
| tmpfiles-clean | 11.6 | 10px |
| backup-photos.sh | 23.4 | 10px |

Axis: 0 "Now 21:34", 2.43 "00:00", 8.43 "06:00", 14.43 "12:00", 20.43 "18:00". Each dot has a `title` with the job name.

3. **Heatmap (day × hour)**: "Job runs by hour" · span 12
- Subtitle: "last 7 days"
- Rows: Thu, Fri, Sat, Sun, Mon, Tue, Wed · Columns: hours 00-23 · unit "runs"
- Values (row by row, 24 per row):

```
Thu  1 0 0 0 0 0 2 0 0 2 2 2 2 2 2 2 2 2 3 2 2 2 2 2
Fri  1 0 0 0 0 0 2 0 0 2 2 2 2 2 2 2 2 2 3 2 2 2 2 2
Sat  1 0 0 0 0 0 2 0 0 2 2 2 2 2 2 2 2 2 3 2 2 2 2 2
Sun  1 0 0 0 0 0 2 0 0 2 2 2 2 2 2 2 2 2 3 2 2 2 2 2
Mon  1 0 0 0 0 0 2 0 0 2 2 2 2 2 2 2 2 2 3 2 2 2 2 2
Tue  1 0 0 0 0 0 2 0 0 2 2 2 2 2 2 2 2 2 3 2 2 3 2 2
Wed  1 0 0 0 0 0 2 0 0 2 2 2 2 2 2 2 2 2 3 2 2 3 0 0
```

#### 12.11.2 Scheduled Jobs › Cron: `jobs/cron`

1. **Data table**: "Cron jobs" · span 12
- Source chip (Expert): `crontab -l · /etc/crontab · /etc/cron.d`

| Schedule | Command | User | Source | Next run | Last result |
| --- | --- | --- | --- | --- | --- |
| ***/30 * * * *** · Every 30 minutes | `~/bin/sync-notes.sh` | riya | `crontab (riya)` | 22:00 | {off:Not recorded} [Track](act:track:sync) |
| **0 21 * * *** · Every day at 21:00 | `~/bin/backup-photos.sh` | riya | `crontab (riya)` | Tomorrow 21:00 | {off:Not recorded} [Track](act:track:backup) |
| **17 * * * *** · Hourly at :17 | `run-parts --report /etc/cron.hourly` | root | `/etc/crontab` | 22:17 | {ok:Success} |
| **30 3 * * 0** · Sundays 03:30 | `e2scrub_all_cron` | root | `/etc/cron.d/e2scrub_all` | Sun 25 Oct | {off:Skipped (systemd)} |

2. **Alert banner**: "Cron does not record job results" · span 12
- State: `info` · Title: "Cron does not record job results" · Text: "Track wraps a job so its exit code and output are saved from the next run."

#### 12.11.3 Scheduled Jobs › Systemd Timers: `jobs/timers`

1. **Data table**: "Timers" · span 12
- Source chip (Expert): `systemctl list-timers --all`

| Timer | Activates | Schedule | Last trigger | Next | Result | Persistent |
| --- | --- | --- | --- | --- | --- | --- |
| `logrotate.timer` | `logrotate.service` | `daily` | Today 00:00 | 00:00 | {ok:Success} | ✓ |
| `apt-daily-upgrade.timer` | `apt-daily-upgrade.service` | `*-*-* 6:00` | Today 10:40 | 06:21 | {ok:Success} | ✓ |
| `apt-daily.timer` | `apt-daily.service` | `*-*-* 6,18:00` | Today 18:52 | 06:40 | {ok:Success} | ✓ |
| `systemd-tmpfiles-clean.timer` | `systemd-tmpfiles-clean.service` | `OnUnitActiveSec=1d` | Today 09:13 | Tomorrow 09:13 | {ok:Success} | - |
| `fstrim.timer` | `fstrim.service` | `weekly` | Mon 09:00 | Mon 26 Oct | {ok:Success} | ✓ |
| `e2scrub_all.timer` | `e2scrub_all.service` | `Sun *-*-* 03:10` | Sun 18 Oct | Sun 25 Oct | {warn:Missed} | - |
| `anacron.timer` | `anacron.service` | `hourly` | Today 21:07 | 22:07 | {ok:Success} | ✓ |
| `snapd.snap-repair.timer` | `snapd.snap-repair.service` | `random 6h` | Today 17:40 | Tomorrow 01:12 | {ok:Success} | - |

#### 12.11.4 Scheduled Jobs › Anacron: `jobs/anacron`

1. **Data table**: "Anacron jobs" · span 12
- Source chip (Expert): `/etc/anacrontab · /var/spool/anacron`

| Job | Period | Delay [n] | Command | Last run |
| --- | --- | --- | --- | --- |
| `cron.daily` | 1 day | 5 min | `run-parts /etc/cron.daily` | Today 09:05 |
| `cron.weekly` | 7 days | 10 min | `run-parts /etc/cron.weekly` | Mon 09:10 |
| `cron.monthly` | 1 month | 15 min | `run-parts /etc/cron.monthly` | 01 Oct |

#### 12.11.5 Scheduled Jobs › At Jobs: `jobs/at`

1. **Data table**: "One-time jobs" · span 12
- Source chip (Expert): `atq`

| Job | Run at | Command | User | (action) |
| --- | --- | --- | --- | --- |
| `#4` | Fri 23 Oct 09:00 | `notify-send "Submit the form"` | riya | [Cancel](act:toast:Job #4 removed (atrm 4).) |

### 12.12 Containers (`containers`)

- Group: Workloads · Icon: `cube` · Status badge: {warn:Warning} (rule in section 3)
- Sub-pages: Containers, Images, Volumes, Virtual Machines

#### 12.12.1 Containers › Containers: `containers/containers`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Running | 2 |  | - |   | - | - |
| Stopped | 1 |  | - |   | - | - |
| Images | 11 |  | - | 9.0 GB | - | - |
| Volumes | 3 |  | - | 1.1 GB | - | - |
| Exposed ports | 1 |  | {warn:Warning} | 5432 on all interfaces | - | - |

2. **Data table**: "Docker containers" · span 12
- Source chip (Expert): `docker ps -a`
- Inspect: row click opens the row-inspect drawer with actions "Restart", "Stop", "Logs"

| Name | Image | Status | Ports | CPU [n] | Memory [n] |
| --- | --- | --- | --- | --- | --- |
| pg-dev | `postgres:16` | {ok:Up 12 hours} | `0.0.0.0:5432→5432` {warn:LAN} | 0.2% | 142 MB |
| redis-dev | `redis:7-alpine` | {ok:Up 12 hours} | `127.0.0.1:6379→6379` | 0.1% | 12 MB |
| api-test | `node:20-alpine` | {off:Exited (0) 2 days ago} | - | - | - |

3. **Alert banner**: "Port 5432 bypasses the UFW firewall" · span 12
- State: `warn` · Title: "Port 5432 bypasses the UFW firewall" · Text: "Docker publishes ports with its own iptables rules. Bind to 127.0.0.1 to keep it local." · Buttons: [View firewall](go:network/firewall)

#### 12.12.2 Containers › Images: `containers/images`

1. **Donut chart**: "Image storage" · span 4
- Center: "9.0" / "GB" · unit "GB"

| Segment | Value | Color | Display value |
| --- | --- | --- | --- |
| In use | 5 | slot 1 | - |
| Unused | 4 | kw | - |

2. **Data table**: "Images" · span 8
- Source chip (Expert): `docker images`
- Header actions: [Prune unused · 4.0 GB](act:fix:storage)

| Repository | Tag | Size [n] | Created | In use |
| --- | --- | --- | --- | --- |
| postgres | 16 | 432 MB | 3 weeks ago | ✓ |
| redis | 7-alpine | 41 MB | 1 month ago | ✓ |
| node | 20-alpine | 135 MB | 2 weeks ago | ✓ |
| node | 18 | 1.1 GB | 4 months ago | - |
| python | 3.11 | 1.0 GB | 5 months ago | - |
| mongo | 7 | 760 MB | 3 months ago | - |
| nginx | alpine | 48 MB | 2 months ago | - |
| <none> | <none> | 1.1 GB | dangling | - |

#### 12.12.3 Containers › Volumes: `containers/volumes`

1. **Data table**: "Volumes" · span 12
- Source chip (Expert): `docker volume ls · docker system df -v`

| Name | Driver | Size [n] | Used by |
| --- | --- | --- | --- |
| `pgdata` | local | 980 MB | pg-dev |
| `redisdata` | local | 64 MB | redis-dev |
| `api-node-modules` | local | 86 MB | api-test |

#### 12.12.4 Containers › Virtual Machines: `containers/vms`

1. **Data table**: "Virtual machines" · span 12
- Source chip (Expert): `VBoxManage list vms · virsh list --all`

| Name | Engine | Guest OS | State | vCPU [n] | Memory [n] | Disk [n] |
| --- | --- | --- | --- | --- | --- | --- |
| win11-test | VirtualBox 7.0 | Windows 11 | {off:Powered off} | 2 | 4 GB | 2.9 GB |

### 12.13 Packages (`packages`)

- Group: Software & Data · Icon: `pkg` · Status badge: {warn:Warning} (rule in section 3)
- Sub-pages: Installed, Sources, Cleanup, Permissions

#### 12.13.1 Packages › Installed: `packages/installed`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| APT | 1,842 |  | - | 212 installed by you | - | - |
| Snap | 14 |  | - | 3.9 GB | - | - |
| Flatpak | 6 |  | - | 2.2 GB | - | - |
| AppImage | 2 |  | - | found in ~/Downloads | - | - |
| Language packages | 29 |  | - | pip · npm · cargo | - | - |
| Updates | 12 |  | {warn:Warning} |   | - | `system/updates` |

2. **Donut chart**: "Disk usage by source" · span 4
- Center: "12.4" / "GB" · unit "GB"

| Segment | Value | Color | Display value |
| --- | --- | --- | --- |
| APT | 5.8 | slot 1 | - |
| Snap | 3.9 | slot 2 | - |
| Flatpak | 2.2 | slot 3 | - |
| AppImage | 0.1 | slot 4 | 0.1 GB |
| Other | 0.4 | slot 5 | - |

3. **Data table**: "Packages" · span 8
- Search: "Search packages"
- Filter chips on column 3: All / apt / snap / flatpak / AppImage
- Page size 10
- Total count shown: 1,893
- Inspect: row click opens the row-inspect drawer with actions "Remove", "Show files"
- Table ID: `pkgs`

| Name | Version | Source | Size [n] | Installed by | Last used |
| --- | --- | --- | --- | --- | --- |
| firefox | `131.0.3` | {info:snap} | 286 MB | System | Today |
| code | `1.94.2` | {info:snap} | 340 MB | User | Today |
| google-chrome-stable | `130.0.6723.58` | {off:apt · 3rd-party} | 352 MB | User | Today |
| telegram-desktop | `5.6.1` | {info:snap} | 182 MB | User | Today |
| docker-ce | `27.3.1` | {off:apt · 3rd-party} | 108 MB | User | Today |
| zoom-client | `6.2.3` | {info:snap} | 452 MB | User | 6 days ago |
| com.spotify.Client | `1.2.45` | {ok:flatpak} | 310 MB | User | Yesterday |
| com.obsproject.Studio | `30.2.3` | {ok:flatpak} | 480 MB | User | 9 days ago |
| virtualbox | `7.0.16` | {off:apt} | 190 MB | User | 12 days ago |
| vlc | `3.0.20` | {off:apt} | 12 MB | User | 3 days ago |
| vlc | `3.0.21` | {info:snap} | 320 MB | User | Never |
| gimp | `2.10.36` | {off:apt} | 98 MB | User | 124 days ago {warn:Unused} |
| shotwell | `0.32.6` | {off:apt} | 8 MB | System | 201 days ago {warn:Unused} |
| git | `2.43.0` | {off:apt} | 42 MB | User | Today |
| python3 | `3.12.3` | {off:apt} | - | System | Today |
| libreoffice-writer | `24.2.6` | {off:apt} | 64 MB | System | 18 days ago |
| balenaEtcher | `1.19.21` | {warn:AppImage} | 98 MB | User | 42 days ago |
| dropbox | `2024.04.17` | {off:apt · 3rd-party} | 38 MB | User | Today |
| httpie | `3.2.2` | `pip` | 2 MB | User | 30 days ago |
| typescript | `5.6.3` | `npm` | 22 MB | User | 2 days ago |
| ripgrep | `14.1.1` | `cargo` | 6 MB | User | Today |

#### 12.13.2 Packages › Sources: `packages/sources`

1. **Data table**: "Software sources" · span 12
- Source chip (Expert): `/etc/apt/sources.list.d`

| Repository | Type | Signed | Packages [n] | Status |
| --- | --- | --- | --- | --- |
| `archive.ubuntu.com noble main restricted universe multiverse` | Official | ✓ | 1,689 | {ok:OK} |
| `security.ubuntu.com noble-security` | Official | ✓ | 312 | {ok:OK} |
| `download.docker.com/linux/ubuntu noble stable` | Third-party | ✓ | 6 | {ok:OK} |
| `dl.google.com/linux/chrome/deb stable` | Third-party | ✓ | 1 | {ok:OK} |
| `packages.microsoft.com/repos/code stable` | Third-party | ✓ | 0 | {info:Unused} |
| `ppa:git-core/ppa` | PPA | ✓ | 4 | {ok:OK} |
| `ppa:deadsnakes/ppa` | PPA | ✓ | 0 | {info:Unused} |
| `ppa:obsproject/obs-studio` | PPA | ✓ | 0 | {info:Unused} |

#### 12.13.3 Packages › Cleanup: `packages/cleanup`

1. **Data table**: "Reclaimable" · span 12

| Item | Count [n] | Size [n] | Undo | (action) |
| --- | --- | --- | --- | --- |
| Packages no longer needed (autoremove) | 48 | 310 MB | {info:Reinstallable} | [Remove](act:toast:48 packages removed · 310 MB freed.) |
| Disabled snap revisions | 6 | 8.7 GB | {warn:Irreversible} | [Clean](act:fix:storage) |
| Duplicate app: VLC (apt + snap) | 1 | 320 MB | {info:Reinstallable} | [Review](act:toast:Keep one VLC to save 320 MB.) |
| Unused Flatpak runtimes | 2 | 640 MB | {info:Regenerates} | [Remove](act:toast:flatpak uninstall --unused) |
| Apps not used in 90+ days | 2 | 106 MB | {off:Review} | [Review](go:packages/installed) |
| Old kernel 6.8.0-41 | 1 | 320 MB | {info:Reinstallable} | [Remove](act:toast:Old kernel queued for removal.) |

#### 12.13.4 Packages › Permissions: `packages/permissions`

1. **Data table**: "Sandboxed app permissions" · span 12
- Source chip (Expert): `snap connections · flatpak info --show-permissions`

| App | Source | Camera | Microphone | Home folder | Network | Location |
| --- | --- | --- | --- | --- | --- | --- |
| zoom-client | {info:snap} | ✓ | ✓ | ✓ | ✓ | - |
| code | {info:snap} | - | - | ✓ | ✓ | - |
| telegram-desktop | {info:snap} | ✓ | ✓ | ✓ | ✓ | - |
| firefox | {info:snap} | ✓ | ✓ | ✓ | ✓ | ✓ |
| com.spotify.Client | {ok:flatpak} | - | - | - | ✓ | - |
| com.obsproject.Studio | {ok:flatpak} | ✓ | ✓ | ✓ | ✓ | - |

### 12.14 Storage (`storage`)

- Group: Software & Data · Icon: `db` · Status badge: {crit:Critical} (rule in section 3)
- Sub-pages: Overview, Filesystems, Disk Usage, Cleanup
- Page header actions (all sub-pages without their own): [**Free up space**](act:fix:storage) (shown while reclaimable space > 0)

#### 12.14.1 Storage › Overview: `storage/overview`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Used | 224.2 | GB | {crit:Critical} | 94% of 238 GB | Usage bar 94.2% | - |
| Free | 13.8 | GB | - |   | - | - |
| Reclaimable | 15.2 | GB | - | Safe to clean | - | - |
| Full in | ~8 | days | {crit:Critical} | at 0.69% per day | - | - |
| Inodes used | 21 | % | - |   | - | - |
| Encryption | Off |  | - | LUKS not used | - | - |

2. **Alert banner**: "Root filesystem is 94% full" · span 12
- State: `crit` · Title: "Root filesystem is 94% full" · Text: "15.2 GB can be reclaimed safely. Personal files are not touched." · Buttons: [**Free up space**](act:fix:storage)

3. **Allocation bar (section 5.18)**: "Space allocation" · span 12
- Subtitle: "/dev/nvme0n1p2 · ext4"
- Source chip (Expert): `du -xh --max-depth=1`

Segments (left to right):

| Segment | GB | Color |
| --- | --- | --- |
| Home | 128.0 | --s1 |
| Apps | 46.0 | --s2 |
| System | 22.0 | --s3 |
| Containers | 13.0 | --s4 |
| Reclaimable | 15.2 | crit stripes |
| Free | 13.8 | --track |

Width of each segment = GB / 238. The legend repeats each name with its bold GB value.

4. **Line chart**: "Usage trend and forecast" · span 8
- Subtitle: "30 days"
- Info tooltip: "The dashed line shows when the disk fills at the current rate."
- Height 220px · Y axis 60 to 100 · unit "%"
- Series: Used (slot 1, data [73.7,74.3,75.7,75.6,77.1,78,78,78.6,79.8,80.7,81.4,81.8,82.8,82.9,83.3,84.7,85.2,85.3,86.8,87.5,87.5,88.2,89.3,90,90.7,91.4,92.1,92.5,93.7,94.2])
- X labels: 30 labels: 22 Sept, 23 Sept, 24 Sept … 20 Oct, 21 Oct
- Threshold line at 85: "Warning 85%"
- Forecast: +8 points to 100, label "Full ~29 Oct" (recomputed from state: daysFull)

5. **Data table**: "Mount points" · span 4
- Source chip (Expert): `findmnt`

| Mount | Size [n] | Used |
| --- | --- | --- |
| `/` | 238 GB | [meter:crit 94%] |
| `/boot/efi` | 1 GB | [meter 6%] |
| `/run` | 780 MB | [meter 1%] |
| `/dev/shm` | 3.8 GB | [meter 2%] |

#### 12.14.2 Storage › Filesystems: `storage/filesystems`

1. **Data table**: "Filesystems" · span 12
- Source chip (Expert): `df -hT · df -i`

| Device | Mount | Type | Size [n] | Used [n] | Available [n] | Use | Inodes [n, ,x] |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `/dev/nvme0n1p2` | `/` | ext4 | 238 GB | 224.2 GB | 13.8 GB | [meter:crit 94%] | 21% |
| `/dev/nvme0n1p1` | `/boot/efi` | vfat | 1.0 GB | 61 MB | 963 MB | [meter 6%] | - |
| `tmpfs` | `/run` | tmpfs | 780 MB | 2.3 MB | 778 MB | [meter 1%] | 1% |
| `tmpfs` | `/dev/shm` | tmpfs | 3.8 GB | 84 MB | 3.7 GB | [meter 2%] | 1% |
| `squashfs ×14` | `/snap/*` | squashfs | 3.9 GB | 3.9 GB | 0 | [meter:ok 100%] | 100% |

2. **Checklist**: "Filesystem checks" · span 12

| State | Title | Detail | Action |
| --- | --- | --- | --- |
| `pass` | fstab entries valid | findmnt --verify: 0 errors |   |
| `pass` | No ext4 errors in kernel log |   |   |
| `pass` | Reserved blocks 5% (11.9 GB) for root |   |   |
| `warn` | Disk encryption off | Files are readable if the laptop is stolen · needs reinstall to enable |   |
| `na` | Full filesystem check (fsck) | Runs at boot when needed · last check 34 days ago |   |

#### 12.14.3 Storage › Disk Usage: `storage/usage`

1. **Horizontal bar list**: "Largest folders" · span 6
- Subtitle: "home"
- Source chip (Expert): `du -h --max-depth=1 ~`

| Label | Value | Shown as |
| --- | --- | --- |
| `~/Videos` | 52 | 52 GB |
| `~/Documents` | 41 | 41 GB |
| `~/Downloads` | 22 | 22 GB |
| `~/.local/share` | 5.2 | 5.2 GB |
| `~/.cache` | 3.8 | 3.8 GB |
| `~/snap` | 2.4 | 2.4 GB |
| `~/Pictures` | 1.6 | 1.6 GB |

2. **Horizontal bar list**: "Largest system folders" · span 6
- Source chip (Expert): `du -xh --max-depth=1 /`

| Label | Value | Shown as |
| --- | --- | --- |
| `/usr` | 14 | 14 GB |
| `/var/lib/docker` | 13 | 13 GB |
| `/var/lib/snapd` | 12.6 | 12.6 GB |
| `/var/log` | 1.4 | 1.4 GB |
| `/opt` | 0.9 | 0.9 GB |
| `/boot` | 0.3 | 0.3 GB |

3. **Data table**: "Largest files" · span 12
- Source chip (Expert): `find ~ -size +500M`
- Search: "Filter files"

| File | Size [n] | Type | Modified |
| --- | --- | --- | --- |
| `~/Downloads/ubuntu-24.04.1-desktop-amd64.iso` | 5.8 GB | Disk image | 4 months ago |
| `~/Videos/screen-2026-09-12.mp4` | 4.1 GB | Video | 1 month ago |
| `~/VirtualBox VMs/win11-test/win11-test.vdi` | 2.9 GB | Virtual disk | 3 weeks ago |
| `~/Downloads/dataset-q3.zip` | 2.2 GB | Archive | 2 months ago |
| `~/Videos/meeting-recording-0918.mkv` | 1.8 GB | Video | 1 month ago |
| `~/Documents/thesis-backup.tar.gz` | 1.2 GB | Archive | 6 months ago |

#### 12.14.4 Storage › Cleanup: `storage/cleanup`

Header actions: [**Free 15.2 GB**](act:fix:storage) (hidden once done), then Refresh and Export.

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Reclaimable | 15.2 | GB | {warn:Warning} |   | - | - |
| Cleaned this month | 0.0 | GB | - |   | - | - |

2. **Data table**: "Cleanup candidates" · span 12
- Row selection · bulk button [Clean selected](act:fix:storage)
- Empty text: "Nothing to clean right now"
- Table ID: `cleanup`

| Item | Details | Size [n] | Undo | Password |
| --- | --- | --- | --- | --- |
| Remove disabled snap revisions | 6 old revisions kept after updates | 8.7 GB | {warn:Irreversible} | 🔒 |
| Prune unused Docker images | 7 images not used by any container | 4.0 GB | {info:Regenerates} | - |
| Empty Trash | 312 files | 2.1 GB | {warn:Irreversible} | - |
| Clean APT package cache | Downloaded .deb files | 0.4 GB | {info:Regenerates} | 🔒 |

### 12.15 Backups (`backups`)

- Group: Software & Data · Icon: `backup` · Status badge: {warn:Warning} (rule in section 3)
- Sub-pages: Backup Jobs, System Snapshots, Restore Points

#### 12.15.1 Backups › Backup Jobs: `backups/jobs`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Last backup | 21 | days ago | {warn:Warning} | 30 Sep 21:12 | - | - |
| Tool | Déjà Dup |  | - | Ubuntu Backups | - | - |
| Destination | BACKUP-1TB |  | - | USB drive · not connected | - | - |
| Backup size | 96 | GB | - |   | - | - |
| Schedule | Weekly |  | - | when drive is connected | - | - |

2. **Bar chart**: "Backup history" · span 6
- Subtitle: "size per run"
- Height 200px · Y max auto · unit "GB"
- Bars (label: value · tooltip label): 26 Aug: 4.2 · "26 Aug · incremental"; 2 Sep: 3.1 · "2 Sep · incremental"; 9 Sep: 5.6 · "9 Sep · incremental"; 16 Sep: 2.8 · "16 Sep · incremental"; 23 Sep: 3.9 · "23 Sep · incremental"; 30 Sep: 4.4 · "30 Sep · incremental"; 7 Oct: 0 [kn] · "7 Oct · skipped (drive missing)"; 14 Oct: 0 [kn] · "14 Oct · skipped (drive missing)"; 21 Oct: 0 [kn] · "21 Oct · skipped (drive missing)"

3. **Data table**: "Runs" · span 6
- Source chip (Expert): `deja-dup`

| Date | Type | Size [n] | Duration | Result |
| --- | --- | --- | --- | --- |
| 21 Oct | Scheduled | - | - | {warn:Skipped · drive missing} |
| 30 Sep | Incremental | 4.4 GB | 9 m | {ok:Success} |
| 23 Sep | Incremental | 3.9 GB | 8 m | {ok:Success} |
| 16 Sep | Incremental | 2.8 GB | 6 m | {ok:Success} |
| 9 Sep | Full | 91 GB | 1 h 42 m | {ok:Success} |

#### 12.15.2 Backups › System Snapshots: `backups/snapshots`

1. **Alert banner**: "No system snapshots" · span 12
- State: `warn` · Title: "No system snapshots" · Text: "Timeshift is not installed. Snapshots let you roll back the system after a bad update." · Buttons: [**Install Timeshift**](act:toast:sudo apt install timeshift)

2. **Data table**: "Snapshots" · span 12
- Empty text: "No snapshots yet"

| Created | Type | Tool | Size [n] | Comment |
| --- | --- | --- | --- | --- |
| *(empty: "No snapshots yet")* |   |   |   |   |

#### 12.15.3 Backups › Restore Points: `backups/restore`

1. **Data table**: "SysDoc restore points" · span 12
- Info tooltip: "Saved automatically before every change SysDoc makes."

| Created | Change | Target | Size [n] | (action) |
| --- | --- | --- | --- | --- |
| Tue 11:05 | Disabled service | `cups-browsed.service` | 1 KB | [Restore](act:undo:cups) |
| 14 Oct | Removed old kernel | `linux-image-6.8.0-38` | - | Reinstall from APT |

### 12.16 Network (`network`)

- Group: Network & Access · Icon: `wifi` · Status badge: {ok:Healthy} (rule in section 3)
- Sub-pages: Interfaces, Connections, Open Ports, Firewall, DNS & VPN, Traffic

#### 12.16.1 Network › Interfaces: `network/interfaces`

1. **Data table**: "Interfaces" · span 12
- Source chip (Expert): `ip -br addr · nmcli device`

| Interface | Type | State | IPv4 | MAC [x] | Speed | RX [n] | TX [n] |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `wlp0s20f3` | Wi-Fi | {ok:connected} | `192.168.1.24/24` | `3c:a9:f4:..:1b` | 866 Mb/s | 4.0 GB | 0.6 GB |
| `docker0` | Bridge | {ok:connected} | `172.17.0.1/16` | `02:42:..:9a` | - | 210 MB | 140 MB |
| `lo` | Loopback | {off:unmanaged} | `127.0.0.1/8` | - | - | 88 MB | 88 MB |
| `enx00e04c68` | Ethernet | {off:unavailable} | - | `00:e0:4c:..:68` | - | 0 | 0 |

2. **Key-value list**: "Wi-Fi" · span 6
- Source chip (Expert): `nmcli -f all dev wifi`

| Key | Value |
| --- | --- |
| Network | HomeNet-5G |
| Signal | [meter:ok 78% "−52 dBm"] |
| Band / channel | 5 GHz · 44 |
| Security | {ok:WPA2-PSK} |
| Link rate | 866 Mb/s |

3. **Key-value list**: "Connectivity" · span 6

| Key | Value |
| --- | --- |
| Internet | {ok:Online} |
| Gateway | `192.168.1.1` · 2 ms |
| Public IP | Hidden [Reveal](act:toast:Looking up the public IP contacts an outside service. Off in the prototype.) |
| IPv6 | Link-local only |
| Captive portal | None |

#### 12.16.2 Network › Connections: `network/connections`

1. **Data table**: "Active connections" · span 12
- Source chip (Expert): `ss -tunp`
- Search: "Filter connections"
- Filter chips on column 4: All / ESTAB / LISTEN / UNCONN
- Page size 12
- Total count shown: 68

| Proto | Local | Remote | State | Process |
| --- | --- | --- | --- | --- |
| tcp | `192.168.1.24:51432` | `142.250.183.14:443` | {ok:ESTAB} | chrome |
| tcp | `192.168.1.24:51438` | `142.250.77.110:443` | {ok:ESTAB} | chrome |
| tcp | `192.168.1.24:42210` | `149.154.167.51:443` | {ok:ESTAB} | telegram-desktop |
| tcp | `192.168.1.24:39812` | `162.125.21.3:443` | {ok:ESTAB} | dropbox |
| tcp | `192.168.1.24:50114` | `13.107.6.158:443` | {ok:ESTAB} | code |
| tcp | `0.0.0.0:5432` | `*:*` | {info:LISTEN} | docker-proxy |
| tcp | `127.0.0.1:6379` | `*:*` | {info:LISTEN} | docker-proxy |
| tcp | `127.0.0.1:631` | `*:*` | {info:LISTEN} | cupsd |
| udp | `127.0.0.53:53` | `*:*` | {off:UNCONN} | systemd-resolved |
| udp | `0.0.0.0:5353` | `*:*` | {off:UNCONN} | avahi-daemon |
| tcp | `192.168.1.24:51900` | `185.125.190.20:443` | {off:TIME-WAIT} | - |

#### 12.16.3 Network › Open Ports: `network/ports`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Listening | 5 |  | - |   | - | - |
| Local only | 3 |  | - |   | - | - |
| Network-exposed | 2 |  | {warn:Warning} |   | - | - |
| Reachable from internet | Unknown |  | - | Needs outside test | - | - |

2. **Data table**: "Listening ports" · span 12
- Source chip (Expert): `ss -tulpn`

| Port | Protocol | Bind address | Exposure | Process | Firewall |
| --- | --- | --- | --- | --- | --- |
| `5432` | TCP | `0.0.0.0` | {warn:LAN} | docker-proxy (pg-dev) | {warn:Bypassed} |
| `5353` | UDP | `0.0.0.0` | {info:LAN} | avahi-daemon | {ok:Allowed (mDNS)} |
| `6379` | TCP | `127.0.0.1` | {off:Local} | docker-proxy (redis-dev) | - |
| `631` | TCP | `127.0.0.1` | {off:Local} | cupsd | - |
| `53` | UDP | `127.0.0.53` | {off:Local} | systemd-resolved | - |

3. **Alert banner**: "External reachability is not tested" · span 12
- State: `info` · Title: "External reachability is not tested" · Text: "Whether the router forwards a port can only be checked from outside your network." · Buttons: [Run external test](act:toast:Opt-in: this sends your public IP to a test server. Off in the prototype.)

#### 12.16.4 Network › Firewall: `network/firewall`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Status | Active |  | - | ufw | - | - |
| Incoming | Deny |  | - | default | - | - |
| Outgoing | Allow |  | - | default | - | - |
| Rules | 2 |  | - |   | - | - |
| Blocked today | 37 |  | - | packets | - | - |

2. **Data table**: "Rules" · span 12
- Source chip (Expert): `sudo ufw status numbered`
- Empty text: "Requires Full access"

| # [n] | To | Action | From | Comment |
| --- | --- | --- | --- | --- |
| 1 | `5353/udp` | {ok:ALLOW IN} | Anywhere | mDNS |
| 2 | `1714:1764/tcp` | {ok:ALLOW IN} | `192.168.1.0/24` | GSConnect |

3. **Alert banner**: "Docker rules are not managed by ufw" · span 12
- State: `warn` · Title: "Docker rules are not managed by ufw" · Text: "Port 5432 (pg-dev) is reachable from the LAN even though ufw denies incoming traffic." · Buttons: [View container](go:containers/containers)

#### 12.16.5 Network › DNS & VPN: `network/dns`

1. **Key-value list**: "DNS" · span 6
- Source chip (Expert): `resolvectl status`

| Key | Value |
| --- | --- |
| Resolver | systemd-resolved |
| Servers | `192.168.1.1` |
| Search domain | - |
| DNSSEC | No |
| DNS over TLS | No |
| /etc/hosts entries | 3 |

2. **Data table**: "VPN connections" · span 6
- Source chip (Expert): `nmcli connection show`

| Name | Type | State | (action) |
| --- | --- | --- | --- |
| Office | WireGuard | {off:Disconnected} | [Connect](act:toast:Connecting to Office (WireGuard)…) |
| Tailscale | tailscale | {ok:Connected} |   |

#### 12.16.6 Network › Traffic: `network/traffic`

1. **Line chart**: "Throughput" · span 12
- Source chip (Expert): `/proc/net/dev`
- Height 220px · Y axis 0 to auto · unit "MB/s"
- Series: Download (slot 1, gen[9, 0.8, 0.6, 0.8, 0, 6, 1.2]); Upload (slot 2, gen[10, 0.2, 0.2, 0.3, 0, 3, 0.3])
- X axis: time (current range)

2. **Data table**: "Traffic by application" · span 12
- Subtitle: "today"
- Source chip (Expert): `nethogs (admin)`
- Empty text: "Requires Full access"

| Application | Downloaded [n] | Uploaded [n] | Share |
| --- | --- | --- | --- |
| chrome | 2.6 GB | 210 MB | [meter 62%] |
| snapd | 620 MB | 4 MB | [meter 15%] |
| dropbox | 310 MB | 280 MB | [meter 13%] |
| telegram-desktop | 180 MB | 40 MB | [meter 5%] |
| code | 90 MB | 12 MB | [meter 2%] |

### 12.17 Accounts (`accounts`)

- Group: Network & Access · Icon: `users` · Status badge: {ok:Healthy} (rule in section 3)
- Sub-pages: Users, Groups, Sessions, Login History, SSH Keys

#### 12.17.1 Accounts › Users: `accounts/users`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Human users | 2 |  | - |   | - | - |
| Administrators | 1 |  | - |   | - | - |
| System accounts | 41 |  | - |   | - | - |
| Locked | 0 |  | - |   | - | - |
| Without password | 0 |  | - | Verified | - | - |

2. **Data table**: "Users" · span 12
- Source chip (Expert): `getent passwd · chage -l`
- Inspect: row click opens the row-inspect drawer with actions "Lock account", "Reset password"

| User | UID [n] | Groups | Role | Shell | Last sign-in | Password age |
| --- | --- | --- | --- | --- | --- | --- |
| **riya** · Riya | 1000 | ‹sudo›‹adm›‹docker›‹lxd› | {info:Administrator} | `/bin/bash` | Today 08:59 | 14 months |
| **guest** · Guest | 1001 | ‹guest› | {off:Standard} | `/bin/bash` | 3 weeks ago | 9 months |

3. **Alert banner**: "Password strength cannot be checked" · span 12
- State: `info` · Title: "Password strength cannot be checked" · Text: "Linux stores only salted hashes. SysDoc shows password age instead."

#### 12.17.2 Accounts › Groups: `accounts/groups`

1. **Data table**: "Groups with members" · span 12
- Source chip (Expert): `getent group`
- Search: "Filter groups"

| Group | GID [n] | Members | Privilege |
| --- | --- | --- | --- |
| sudo | 27 | riya | {warn:Admin} |
| adm | 4 | riya, syslog | {info:Read logs} |
| docker | 999 | riya | {warn:Root-equivalent} |
| lxd | 135 | riya | {info:Containers} |
| plugdev | 46 | riya | - |
| lpadmin | 114 | riya | Printers |
| sambashare | 117 | riya | - |

#### 12.17.3 Accounts › Sessions: `accounts/sessions`

1. **Data table**: "Active sessions" · span 12
- Source chip (Expert): `loginctl list-sessions`

| Session | User | Seat | Type | Since | State |
| --- | --- | --- | --- | --- | --- |
| `2` | riya | seat0 | wayland | Today 08:59 | {ok:active} |
| `c1` | gdm | seat0 | wayland | Today 08:58 | {off:online} |

#### 12.17.4 Accounts › Login History: `accounts/logins`

1. **Bar chart**: "Sign-ins per day" · span 5
- Height 180px · Y max auto
- Bars (label: value): Thu: 2; Fri: 3; Sat: 1; Sun: 1; Mon: 3; Tue: 2; Wed: 2

2. **Data table**: "Events" · span 7
- Source chip (Expert): `last · journalctl _COMM=sudo`

| Time | User | Event | Source | Result |
| --- | --- | --- | --- | --- |
| Today 21:12 | riya | sudo | `apt update` | {ok:Success} |
| Today 08:59 | riya | Sign-in | gdm-password | {ok:Success} |
| Tue 18:39 | riya | sudo | `apt upgrade` | {ok:Success} |
| Mon 08:42 | riya | Sign-in | gdm-password | {ok:Success} |
| Sat 10:13 | riya | Sign-in | gdm-password | {warn:Failed ×1} |
| 30 Sep | guest | Sign-in | gdm-password | {ok:Success} |

#### 12.17.5 Accounts › SSH Keys: `accounts/ssh`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| OpenSSH server | Not installed |  | - |   | - | - |
| Authorized keys | 0 |  | - |   | - | - |
| Client keys | 1 |  | - | ed25519 | - | - |

2. **Data table**: "Client keys" · span 12
- Subtitle: "~/.ssh"

| File | Type | Fingerprint | Created |
| --- | --- | --- | --- |
| `id_ed25519` | ED25519 | `SHA256:k3F…Qw8` | 12 Aug 2025 |

### 12.18 Security (`security`)

- Group: Network & Access · Icon: `shield` · Status badge: {warn:Warning} (rule in section 3)
- Sub-pages: Overview, Vulnerabilities, Hardening, File Integrity, Malware Scan

#### 12.18.1 Security › Overview: `security/overview`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Security updates | 4 |  | {warn:Warning} |   | - | `system/updates` |
| Known vulnerabilities | 3 |  | {warn:Warning} | 2 high · 1 medium | - | - |
| Firewall | Active |  | - |   | - | - |
| AppArmor | 112 | profiles | - | enforcing | - | - |
| Secure Boot | On |  | - |   | - | - |
| Disk encryption | Off |  | {info:Info} |   | - | - |

2. **Checklist**: "Security posture" · span 8

| State | Title | Detail | Action |
| --- | --- | --- | --- |
| `fail` | Security updates installed | 4 pending | [**Install**](act:fix:sec) |
| `pass` | Firewall enabled | ufw · default deny incoming |   |
| `pass` | Secure Boot enabled |   |   |
| `pass` | CPU vulnerability mitigations active | Spectre v1/v2, Retbleed, MDS |   |
| `pass` | AppArmor enforcing |   |   |
| `pass` | Screen locks after 5 minutes |   |   |
| `pass` | No remote login (SSH) exposed |   |   |
| `warn` | Docker port 5432 exposed to LAN | Bypasses ufw | [View](go:network/ports) |
| `warn` | Disk encryption off | Requires reinstall to enable |   |
| `na` | Malware scanner | Not installed | [View](go:security/malware) |

3. **Donut chart**: "Checks" · span 4
- Center: "6/10" / "passing"

| Segment | Value | Color | Display value |
| --- | --- | --- | --- |
| Pass | 6 | k3 | - |
| Warning | 2 | kw | - |
| Fail | 1 | kc | - |
| Not applicable | 1 | kn | - |

#### 12.18.2 Security › Vulnerabilities: `security/vulns`

Tab count pill: 3.

1. **Data table**: "Known vulnerabilities" · span 12
- Subtitle: "sample data"
- Source chip (Expert): `pro fix · OVAL`
- Empty text: "No known vulnerabilities in installed packages"

| CVE | Package | Severity | Installed | Fixed in | Action |
| --- | --- | --- | --- | --- | --- |
| `CVE-2026-10421` | openssl | {crit:High} | `3.0.13-0ubuntu3.4` | `3.0.13-0ubuntu3.5` | [**Fix**](act:fix:sec) |
| `CVE-2026-10388` | sudo | {crit:High} | `1.9.15p5-3ubuntu5.1` | `1.9.15p5-3ubuntu5.2` | [**Fix**](act:fix:sec) |
| `CVE-2026-09917` | linux-firmware | {warn:Medium} | `…0ubuntu2.5` | `…0ubuntu2.6` | [Fix](act:fix:sec) |

#### 12.18.3 Security › Hardening: `security/hardening`

1. **Data table**: "Hardening checks" · span 12

| Check | Result | Value | Recommendation |
| --- | --- | --- | --- |
| Kernel ASLR | {ok:Pass} | `kernel.randomize_va_space = 2` | - |
| Unprivileged user namespaces | {ok:Pass} | `restricted by AppArmor` | - |
| kptr_restrict | {ok:Pass} | `1` | - |
| dmesg_restrict | {ok:Pass} | `1` | - |
| Home directory permissions | {ok:Pass} | `drwxr-x---` | - |
| Automatic security updates | {ok:Pass} | `Unattended-Upgrade "1"` | - |
| Docker group membership | {warn:Review} | riya | docker group equals root access |
| Disk encryption | {warn:Fail} | None | Enable LUKS at next reinstall |

#### 12.18.4 Security › File Integrity: `security/integrity`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Files verified | 284,112 |  | - |   | - | - |
| Modified | 0 |  | - |   | - | - |
| Missing | 0 |  | - |   | - | - |
| Last scan | Today |  | - | 03:00 | - | - |

2. **Data table**: "Programs with SUID bit" · span 12
- Info tooltip: "These programs run with their owner’s rights (usually root). All match their packages."
- Source chip (Expert): `find / -perm -4000`

| Path | Owner | Package | Verified |
| --- | --- | --- | --- |
| `/usr/bin/sudo` | root | sudo | ✓ |
| `/usr/bin/passwd` | root | passwd | ✓ |
| `/usr/bin/mount` | root | mount | ✓ |
| `/usr/bin/pkexec` | root | pkexec | ✓ |
| `/usr/lib/openssh/ssh-keysign` | root | openssh-client | ✓ |
| `/usr/bin/fusermount3` | root | fuse3 | ✓ |

#### 12.18.5 Security › Malware Scan: `security/malware`

1. **Alert banner**: "No malware scanner installed" · span 12
- State: `info` · Title: "No malware scanner installed" · Text: "Optional on Ubuntu. Signature scans reduce risk but cannot prove a system is clean." · Buttons: [**Install ClamAV**](act:toast:sudo apt install clamav)

2. **Checklist**: "Heuristic checks" · span 12

| State | Title | Detail | Action |
| --- | --- | --- | --- |
| `pass` | No startup items running from /tmp or ~/Downloads |   |   |
| `pass` | No cron jobs that download and execute scripts |   |   |
| `pass` | No unknown kernel modules |   |   |
| `pass` | No hidden processes |   |   |

### 12.19 Privacy (`privacy`)

- Group: Network & Access · Icon: `eye` · Status badge: {ok:Healthy} (rule in section 3)
- Sub-pages: Device Access, App Permissions, Privacy Settings

#### 12.19.1 Privacy › Device Access: `privacy/access`

1. **State tiles (section 5.21)**: "Right now" · span 12

| Icon | Title | State |
| --- | --- | --- |
| `mic` | Microphone | Not in use |
| `cam` | Camera | Not in use |
| `screen` | Screen sharing | Off |
| `pin` | Location | Off |

2. **Data table**: "Recent access" · span 12
- Subtitle: "7 days"
- Source chip (Expert): `PipeWire · xdg-desktop-portal`

| Time | Device | Application | Duration |
| --- | --- | --- | --- |
| Tue 16:00 | Camera | zoom-client | 48 min |
| Tue 16:00 | Microphone | zoom-client | 48 min |
| Mon 11:30 | Microphone | chrome | 12 min |
| Mon 11:30 | Screen share | chrome | 9 min |

#### 12.19.2 Privacy › App Permissions: `privacy/perms`

1. **Data table**: "Permissions by app" · span 12
- Source chip (Expert): `snap connections · flatpak permissions`

| App | Camera | Microphone | Location | Files | Network |
| --- | --- | --- | --- | --- | --- |
| zoom-client | ✓ | ✓ | - | ✓ | ✓ |
| chrome | ✓ | ✓ | - | ✓ | ✓ |
| telegram-desktop | ✓ | ✓ | - | ✓ | ✓ |
| firefox | ✓ | ✓ | ✓ | ✓ | ✓ |
| com.spotify.Client | - | - | - | - | ✓ |

#### 12.19.3 Privacy › Privacy Settings: `privacy/settings`

1. **Settings form (field rows, section 5.21)**: "Settings" · span 8

| Label | Description | Control | Initial | Control ID |
| --- | --- | --- | --- | --- |
| Location services | Apps can request your location | Switch | Off | `-` |
| Usage reporting to Ubuntu | ubuntu-report · popularity-contest | Switch | Off | `-` |
| Problem reporting | Send crash reports to Canonical | Switch | On | `-` |
| Recent files history | Remember files you opened | Switch | On | `-` |
| Remove old temporary files | After 30 days | Switch | On | `-` |

### 12.20 Diagnostics (`diagnostics`)

- Group: Tools · Icon: `gauge` · Status badge: none
- Sub-pages: Performance Check, Network Test, Hardware Tests

#### 12.20.1 Diagnostics › Performance Check: `diagnostics/performance`

1. **Diagnostics runner (section 5.23)**: "Performance check" · span 12
- Info tooltip: "Run while the computer feels slow. Finds the main cause in about 10 seconds."

Idle steps: CPU load → Memory → Memory pressure → Disk I/O → Temperature → Top processes. Button: [**Run check**](act:diag:run).

> After the run completes, these widgets appear below the runner: **KPI row**: Memory used 7.6 GB `{crit:Critical}` "99% of 7.6 GB"; Memory pressure 23.4 % `{crit}` "some avg10"; Swap activity 18 MB/s `{warn}`; CPU 23 % "normal"; Temperature 61 °C "normal". **Data table "Contributing processes"** span 7: columns Process, Memory [n], CPU [n], Impact; rows: chrome (42 tabs) 6.1 GB 14% {crit:High}; code 640 MB 4% {off:Low}; gnome-shell 410 MB 3% {off:Low}; telegram-desktop 280 MB 1% {off:Low}. **Checklist "Recommendations"** span 5: `warn` Close unused browser tabs / Or enable Memory Saver in Chrome; `warn` Increase swap to 4 GB / Reduces freezes when memory fills / [Configure](act:toast:Swap resize opens a guided fix in the real app.); `pass` CPU and temperature are normal. The runner panel result: inline crit alert "Primary cause: memory saturation" / "chrome uses 6.1 GB of 7.6 GB (42 tabs) · swap active · memory pressure 23%". Header action [Run again](act:diag:run).

#### 12.20.2 Diagnostics › Network Test: `diagnostics/network`

1. **Diagnostics runner (section 5.23)**: "Network test" · span 12

Idle steps: Gateway → DNS resolution → Internet latency → Packet loss → IPv6. Button: [**Run test**](act:net:run).

> After the run completes: runner result is an inline ok alert "Network is healthy" / "All 5 checks passed", header action [Run again](act:net:run), and a **KPI row** appears: Gateway 2 ms; DNS lookup 14 ms; Latency 18 ms "1.1.1.1"; Jitter 3 ms; Packet loss 0 %; IPv6 Link-local.

#### 12.20.3 Diagnostics › Hardware Tests: `diagnostics/hardware`

1. **Data table**: "Available tests" · span 12

| Test | Duration | Last run | Result | (action) |
| --- | --- | --- | --- | --- |
| SMART short self-test (NVMe) | 2 min | 7 days ago | {ok:Passed} | [Run](act:toast:SMART short self-test started.) |
| Memory test (memtest86+) | 30-60 min | Never | {off:-} | [Schedule](act:toast:Runs at next restart from the boot menu.) |
| Battery capacity report | 1 min | Today | {warn:71% health} | [Open](go:power/battery) |
| Sensor check | 10 s | Today | {ok:Passed} | [Open](go:hardware/sensors) |
| Speaker and microphone test | 30 s | Never | {off:-} | [Run](act:toast:Plays a test tone and records 3 seconds.) |

### 12.21 Assistant (`assistant`)

- Group: Tools · Icon: `chat` · Status badge: none
- Sub-pages: Chat

#### 12.21.1 Assistant › Chat: `assistant/chat`

1. **Chat (section 5.24)**: "Ask SysDoc" · span 8
- Subtitle: "Local model · on-device"

Initial messages: user "Why did my laptop restart last night?", then the `restart` answer (section 10.5). Composer: input "Ask about this computer…" + primary Send. When the assistant is off, the panel body is instead a centered muted line "The assistant is turned off." + [**Turn on**](go:settings/assistant), and the subtitle reads "Off".

2. **Question list**: "Suggested questions" · span 4

Full-width rows (chat icon + text), each runs `ask:<key>`:

- Why did my laptop restart last night? → `ask:restart`
- Why is Bluetooth crashing? → `ask:bt`
- Is my SSD healthy? → `ask:ssd`
- What is filling my disk? → `ask:disk`

3. **Key-value list**: "Assistant" · span 4

| Key | Value |
| --- | --- |
| Model | Local · 3B parameters |
| Runtime | Ollama |
| Data access | Read-only |
| Can run commands | No · suggests fixes only |
| Cloud fallback | {off:Off} |

### 12.22 Settings (`settings`)

- Group: Footer · Icon: `sliders` · Status badge: none
- Sub-pages: General, Access, Notifications, Assistant, Data, About

#### 12.22.1 Settings › General: `settings/general`

1. **Settings form (field rows, section 5.21)**: "General" · span 8

| Label | Description | Control | Initial | Control ID |
| --- | --- | --- | --- | --- |
| Expert mode | Show data sources, commands and raw columns | Switch | Off | `set-expert` |
| Default time range |  | Select: 1h / 24h / 7d / 30d | 24h | `set-range` |
| Refresh interval | Live metrics | Select: 2 s / 5 s / 15 s / Manual | 5 s | `set-refresh` |
| Temperature unit |  | Select: Celsius / Fahrenheit | Celsius | `set-temp` |
| Start at login | Collector runs in the background | Switch | On | `set-start` |

#### 12.22.2 Settings › Access: `settings/access`

1. **Radio cards (section 5.21)**: "Access level" · span 8

| Option | Description | Checked |
| --- | --- | --- |
| Basic | No password · about 80% of checks | No |
| Full | Admin helper approved once · all checks | Yes |

Radio group name: `access`.

2. **Key-value list**: "Admin helper" · span 4

| Key | Value |
| --- | --- |
| Service | `sysdoc-helper` |
| Status | {ok:Authorized} |
| Activation | D-Bus, on demand |
| Allowed actions | 24 (fixed list) |
| Audit log | `/var/log/sysdoc/helper.log` |

#### 12.22.3 Settings › Notifications: `settings/notifications`

1. **Settings form (field rows, section 5.21)**: "Notifications" · span 8

| Label | Description | Control | Initial | Control ID |
| --- | --- | --- | --- | --- |
| Critical issues | Disk almost full, SSD failing, overheating | Switch | On | `n1` |
| Warnings | Included in the weekly summary when off | Switch | Off | `n2` |
| Weekly summary | Sunday 18:00 | Switch | On | `n3` |
| Quiet hours |  | Select: Off / 22:00-07:00 / 23:00-08:00 | 22:00-07:00 | `n4` |

#### 12.22.4 Settings › Assistant: `settings/assistant`

1. **Settings form (field rows, section 5.21)**: "Assistant" · span 8

| Label | Description | Control | Initial | Control ID |
| --- | --- | --- | --- | --- |
| Enable assistant | Runs a local model on this computer | Switch | On | `set-ai` |
| Model |  | Select: Local · 3B (fast) / Local · 8B (better answers) | Local · 3B (fast) | `set-model` |
| Cloud fallback | Removes names, hostnames, IPs and serials before sending | Switch | Off | `set-cloud` |

#### 12.22.5 Settings › Data: `settings/data`

1. **Settings form (field rows, section 5.21)**: "Data retention" · span 8

| Label | Description | Control | Initial | Control ID |
| --- | --- | --- | --- | --- |
| Keep history for | Metrics and change history | Select: 90 days / 1 year / 2 years | 2 years | `ret` |
| Database size | `~/.local/share/sysdoc/sysdoc.db` | Read-only value | 74 MB | `-` |
| Export report | HTML or JSON snapshot of all pages | [Export](act:toast:Report export is disabled in the prototype.) | - | `-` |
| Delete history | Restore points are kept | [Delete…](act:toast:Prototype: nothing was deleted.) | - | `-` |

#### 12.22.6 Settings › About: `settings/about`

1. **Key-value list**: "About" · span 6

| Key | Value |
| --- | --- |
| Version | SysDoc 0.5 · prototype |
| Collector | sysdoc-agent 0.5 · running |
| Knowledge base | 1,240 entries · updated today |
| Distro adapter | Ubuntu / Debian |
| License | Apache-2.0 |


