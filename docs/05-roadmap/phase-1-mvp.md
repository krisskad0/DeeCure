# Phase 1: MVP: see and understand

[Docs](../README.md) › [Roadmap](README.md)

**Goal:** the desktop app shows every insight that needs no password, explained in plain English.
**Rough time:** 8-10 weeks.

## Progress

The insights catalog (task 1) moved to just before the collectors (task 7), so the app window could come first.

| Task | Status |
| --- | --- |
| 2. App scaffold | ✅ Done. Tauri v2 + React 19 + TypeScript; one live screen (Overview › Active issues) with real findings from the agent, refreshed every 10 s; agent-not-running and first-check states. `make app`. |
| 3. Foundations | ✅ Done. Design tokens (light and dark), bundled Ubuntu Sans fonts, spec typography, and the 43 icons plus the 22-page list generated from the spec by `scripts/gen-ui.py`. |
| 4. Shell | ✅ Done. Top bar, sidebar (22 pages, sub-page trees, rail mode, narrow-window drawer), sticky page header with tabs, 12-column grid, and overlays: Ctrl K command palette (pages and services), notifications list, machines list, toasts. The side drawer comes with the components that open it (row inspect, fixes). |
| 5. Components | ✅ Done. All of 01-22 are built, with UI tests and the component gallery (Ctrl K → "Component gallery"). Charts are our own SVG ([ADR 0006](../02-architecture/decisions/0006-charts-as-own-svg-components.md)). 23 (diagnostics runner) and 24 (chat) come with their pages. |
| 9. Health score | ✅ Done. Score and per-area deductions in `status`, saved as `health.score` after each check, shown in the CLI and the app. Equal 15-point shares until open decision #15. |
| 10. Pages | ✅ Done. All 22 pages and their 91 tabs show real data from this computer. What needs admin rights says "Requires Full access" (Phase 2); what needs the fix engine (cleaning, installing, ending processes, changing settings) only shows. Still to come: CVE numbers, firewall rules, per-app traffic, camera and microphone history, full file verification, and the assistant (Phase 4). Diagnostics run real checks: a performance check and a network test. |
| 7. Collectors | Started. Network, disk I/O (rates, IOPS, busy, wait), pressure (some and full), sensors, per-core CPU, load, clock speed, context switches, memory and swap breakdown, and battery time left, boot timing, and pending updates, plus live `system`, `os`, `kernel`, `update_history`, `processes`, `network`, and `gpu` inventories. New rules: `security.updates_pending`, `system.reboot_required`, `security.firewall_off`, `backups.none`, `backups.overdue`. |
| 11. Performance check | ✅ Done. Diagnostics › Performance Check with the runner. |
| 13. Desktop integration | ✅ Done. A second package, `deecure-desktop`: the app with a menu entry, icons, AppStream data, and a man page. Tested by installing in a clean container. |
| 8. Knowledge base | ✅ Done. 125 entries: 59 services, 26 ports, 20 errors, 12 packages, and 8 how-to topics (one per rule). Used in issues, services, ports, errors, and packages. |
| 1. Insights catalog | ✅ Done. [Insights catalog](../01-product/insights-catalog.md). |
| 6. Behavior | ✅ Done. Shared view settings (mode, time range, rail) that stay in step across the app, Beginner/Expert columns, a light Beginner mode (8 pages and 19 sub-pages in the menus; Expert shows all, [Beginner mode](../03-ui-design/02-shell/sidebar.md#beginner-mode)), page status rules, Ctrl K palette, Esc closes overlays. |
| 12. Onboarding | ✅ Done. Four short screens (one asks "Are you new to Linux?" to set the mode) and a live first check ([design](../03-ui-design/05-pages/00-onboarding.md)); can be shown again from Settings. |

## Tasks (in order)

1. **Write the insights catalog:** every check, marked Yes / Admin / No. This phase builds all "Yes" checks.
   → [Insights catalog](../01-product/insights-catalog.md)
2. **App scaffold:** Tauri v2 + React + TypeScript in `app/`. The Tauri side connects to the agent socket.
   → [ADR 0002](../02-architecture/decisions/0002-tauri-react-desktop-ui.md)
3. **Foundations:** design tokens for light and dark, bundled fonts, icons.
   → [Foundations](../03-ui-design/01-foundations/README.md)
4. **Shell:** top bar, sidebar, page header, content grid, responsive rules, overlays.
   → [Shell](../03-ui-design/02-shell/README.md)
5. **Components 01-22**, each with its own tests. (23 Diagnostics runner comes in step 11; 24 Chat comes in Phase 4.)
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
    → [User journey](../01-product/user-journey.md#first-launch-4-short-screens)
13. **Desktop integration:** `.desktop` file, icon, AppStream metainfo. Add the app to the `.deb`.
    → [Debian package](../06-packaging/debian-package.md)

## Done when

- [ ] Every page shows real data, or says honestly "your hardware does not report this".
- [ ] First useful results in under 10 seconds. Full first scan in under 60 seconds.
- [ ] Desktop app RAM under 200 MB. Agent inside its budget.
- [ ] 5 beginners can answer "what is using my disk?" without help.

---

← Previous: [Phase 0](phase-0-foundations.md) · [Index](README.md) · Next: [Phase 2: Fix safely](phase-2-fix-safely.md) →
