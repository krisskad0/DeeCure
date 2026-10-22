# Phase 1 — MVP: see and understand

[Docs](../README.md) › [Roadmap](README.md)

**Goal:** the desktop app shows every insight that needs no password, explained in plain English.
**Rough time:** 8–10 weeks.

## Progress

The insights catalog (task 1) moved to just before the collectors (task 7), so the app window could come first.

| Task | Status |
| --- | --- |
| 2. App scaffold | ✅ Done. Tauri v2 + React 19 + TypeScript; one live screen (Overview › Active issues) with real findings from the agent, refreshed every 10 s; agent-not-running and first-check states. `make app`. |
| 3. Foundations | ✅ Done. Design tokens (light and dark), bundled Ubuntu Sans fonts, spec typography, and the 43 icons plus the 22-page list generated from the spec by `scripts/gen-ui.py`. |
| 4. Shell | ✅ Done. Top bar, sidebar (22 pages, sub-page trees, rail mode, narrow-window drawer), sticky page header with tabs, 12-column grid, and overlays: Ctrl K command palette (pages and services), notifications list, machines list, toasts. The side drawer comes with the components that open it (row inspect, fixes). |
| 5. Components | ✅ Done. All of 01–22 are built, with UI tests and the component gallery (Ctrl K → "Component gallery"). Charts are our own SVG ([ADR 0006](../02-architecture/decisions/0006-charts-as-own-svg-components.md)). 23 (diagnostics runner) and 24 (chat) come with their pages. |
| 9. Health score | ✅ Done. Score and per-area deductions in `status`, saved as `health.score` after each check, shown in the CLI and the app. Equal 15-point shares until open decision #15. |
| 10. Pages | Started. **Overview** (Summary, Issues, Health Score) shows real data: KPI row, CPU and memory chart over the chosen range, subsystems, active issues, score gauge and deductions, disk donut, top processes, and system facts. Not shown yet: disk breakdown by folder, recent activity, and the error heatmap (they come with Storage, Activity, and Logs), and resolved-issue counts. **Performance** (CPU, Memory, Disk I/O, Network, GPU, Pressure) shows real data; not shown yet: OOM kills (Logs), queue depth, ping latency and loss, 14-day data usage, GPU load history, and per-process disk use (needs Full access). **System** (Overview, Boot, Updates, Kernel, Time & Locale) and **Processes** (All, Top Consumers, with a row-inspect drawer) show real data; installing updates, removing old kernels, and ending processes wait for the fix engine. **Hardware** (all 8 tabs), **Power** (Battery, Power Profile, Sleep), and **Display & Audio** show real data; values only root can read (serial number, memory modules, SMART, per-process power) say they need Full access. **Services › System Services** shows real data: KPI row, an alert per failed service, and the units table (search, filters, 14 per page). Gaps, kept honest: the Memory column waits for live per-service memory, and "Restarts today" shows restarts since boot until Phase 3 adds event history. |
| 7. Collectors | Started. Network, disk I/O (rates, IOPS, busy, wait), pressure (some and full), sensors, per-core CPU, load, clock speed, context switches, memory and swap breakdown, and battery time left, boot timing, and pending updates, plus live `system`, `os`, `kernel`, `update_history`, `processes`, `network`, and `gpu` inventories. New rules: `security.updates_pending`, `system.reboot_required`. |
| 1, 6, 8, 11–13 | Not started |

## Tasks (in order)

1. **Write the insights catalog:** every check, marked Yes / Admin / No. This phase builds all "Yes" checks.
   → [Product](../01-product/README.md#missing-document)
2. **App scaffold:** Tauri v2 + React + TypeScript in `app/`. The Tauri side connects to the agent socket.
   → [ADR 0002](../02-architecture/decisions/0002-tauri-react-desktop-ui.md)
3. **Foundations:** design tokens for light and dark, bundled fonts, icons.
   → [Foundations](../03-ui-design/01-foundations/README.md)
4. **Shell:** top bar, sidebar, page header, content grid, responsive rules, overlays.
   → [Shell](../03-ui-design/02-shell/README.md)
5. **Components 01–22**, each with its own tests. (23 Diagnostics runner comes in step 11; 24 Chat comes in Phase 4.)
   → [Components](../03-ui-design/03-components/README.md)
6. **Behavior:** state, time range, Beginner / Expert mode, page status rules, keyboard, command palette.
   → [Behavior](../03-ui-design/04-behavior/README.md)
7. **All "Yes" collectors**, using the catalog from step 1.
   → [Collectors](../02-architecture/collectors.md)
8. **Knowledge base:** first entries for the most common Ubuntu services, errors, ports, and packages.
   → [Knowledge base](../02-architecture/knowledge-base.md)
9. **Health score** with its breakdown.
   → [Health score](../02-architecture/health-score.md)
10. **Pages, in this order.** Admin-only values show "Requires Full access" until Phase 2.
    1. [Overview](../03-ui-design/05-pages/01-overview.md)
    2. [Performance](../03-ui-design/05-pages/02-performance.md)
    3. [Storage](../03-ui-design/05-pages/14-storage.md)
    4. [System](../03-ui-design/05-pages/05-system.md)
    5. [Hardware](../03-ui-design/05-pages/06-hardware.md)
    6. [Power](../03-ui-design/05-pages/07-power.md)
    7. [Display & Audio](../03-ui-design/05-pages/08-display.md)
    8. [Processes](../03-ui-design/05-pages/09-processes.md)
    9. [Services](../03-ui-design/05-pages/10-services.md)
    10. [Scheduled Jobs](../03-ui-design/05-pages/11-jobs.md) (including the calendar)
    11. [Containers](../03-ui-design/05-pages/12-containers.md)
    12. [Packages](../03-ui-design/05-pages/13-packages.md)
    13. [Logs](../03-ui-design/05-pages/03-logs.md)
    14. [Network](../03-ui-design/05-pages/16-network.md)
    15. [Accounts](../03-ui-design/05-pages/17-accounts.md)
    16. [Security](../03-ui-design/05-pages/18-security.md) (read-only checks)
    17. [Privacy](../03-ui-design/05-pages/19-privacy.md)
    18. [Backups](../03-ui-design/05-pages/15-backups.md) (read-only)
    19. [Settings](../03-ui-design/05-pages/22-settings.md): General, Data, About
11. **"Why is my PC slow?"** Diagnostics › Performance Check with the runner component.
    → [Diagnostics page](../03-ui-design/05-pages/20-diagnostics.md), [Runner](../03-ui-design/03-components/23-diagnostics-runner.md), [User journey](../01-product/user-journey.md#why-is-my-pc-slow-diagnostics--performance-check)
12. **Design and build onboarding and first scan.** These screens are not in the prototype yet.
    → [User journey](../01-product/user-journey.md#first-launch-3-short-screens)
13. **Desktop integration:** `.desktop` file, icon, AppStream metainfo. Add the app to the `.deb`.
    → [Debian package](../06-packaging/debian-package.md)

## Done when

- [ ] Every page shows real data, or says honestly "your hardware does not report this".
- [ ] First useful results in under 10 seconds. Full first scan in under 60 seconds.
- [ ] Desktop app RAM under 200 MB. Agent inside its budget.
- [ ] 5 beginners can answer "what is using my disk?" without help.

---

← Previous: [Phase 0](phase-0-foundations.md) · [Index](README.md) · Next: [Phase 2 — Fix safely](phase-2-fix-safely.md) →
