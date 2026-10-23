# Phase 3 — History and trends

[Docs](../README.md) › [Roadmap](README.md)

**Goal:** DeeCure remembers what changed and warns about problems before they happen.
**Rough time:** 5–6 weeks.

## Progress

| Task | Status |
| --- | --- |
| 1. Event collectors | ✅ Done. apt and dpkg logs (with the newest rotated copy), snap and Flatpak lists, the journal (USB devices, crashes, memory kills, failed services), boots with kernel changes and unclean shutdowns, config files with diffs, and DeeCure's fixes, saved once each by key ([History events](../02-architecture/collectors.md#history-events)). Config files are compared scan to scan instead of inotify, so DeeCure never uses up the watches editors need. udev is not watched live: the kernel log gives the same USB plugs, also for times the agent was off. |
| 2. Activity pages | ✅ Done. Timeline, Package History (deb, snap, Flatpak), Config Changes (with diffs and who changed it), Device Events, from saved history ([Activity page](../03-ui-design/05-pages/04-activity.md#what-is-built-phase-3)). |
| 3. Retention | ✅ Done, with schema version 2: metric names stored once (`series`), snapshots thinned with age, events capped ([Storage](../02-architecture/storage.md)). |
| 4. Trends and predictions | ✅ Done. Each disk's 7-day trend gives `filling` and `days_until_full`; rule `storage.filling_up` warns within 14 days and is critical within 3 ([Rules engine › Trends](../02-architecture/rules-engine.md#trends)). The Storage page draws the forecast line. |
| 5. Notifications | ✅ Done. Critical right away, warnings when asked, quiet hours, no repeats within 24 hours, bursts as one summary; settings saved by the agent; test button ([Notifications](../02-architecture/notifications.md)). |
| 6. Weekly report and CLI | ✅ Done. Weekly summary on Sunday 18:00; `deecure history`, `deecure report`, `deecure report --html`; Export report in the app ([CLI](../02-architecture/cli.md)). |
| 7. Tray icon | ✅ Done. Off by default; only offered on desktops with a tray area ([Settings page](../03-ui-design/05-pages/22-settings.md)). |

## Tasks (in order)

1. **Event collectors:** package changes (dpkg log, snap, flatpak), config changes (inotify on `/etc`, cron folders, autostart), device events (udev), boots.
   → [Collectors](../02-architecture/collectors.md)
2. **Activity pages:** Timeline, Package History, Config Changes, Device Events.
   → [Activity page](../03-ui-design/05-pages/04-activity.md), [Activity timeline](../03-ui-design/03-components/10-activity-timeline.md)
3. **Retention:** 1-minute data for 7 days, 15-minute averages for 90 days, daily averages for 2 years.
   → [Storage](../02-architecture/storage.md)
4. **Trends and predictions:** "Disk full in about N days" with the chart forecast line.
   → [Line / area chart](../03-ui-design/03-components/12-line-area-chart.md)
5. **Notifications:** critical alerts right away, warnings in the weekly summary, quiet hours, no repeats within 24 hours.
   → [Notifications](../02-architecture/notifications.md), [Settings page](../03-ui-design/05-pages/22-settings.md)
6. **Weekly report** and the CLI commands `deecure history` and `deecure report --html`.
   → [CLI](../02-architecture/cli.md)
7. **Optional tray icon**, off by default.
   → [User journey](../01-product/user-journey.md#everyday-use)

## Done when

- [x] The journey "Something broke after a change" works end to end.
  → [User journey](../01-product/user-journey.md#something-broke-after-a-change)
  The agent saves kernel updates and "Started with a new kernel X (was Y)"; Activity › Timeline shows them with a "new kernel" banner and the how-to "Start once with the previous kernel", and a DeeCure fix has Undo right in its row. Checked by `app/src/pages/activity/activity.test.tsx` and on a real laptop (1,163 events found on the first run).
- [x] The database stays under 200 MB after 2 years of simulated data.
  `make db-size` (in CI): 124 MB for a big machine with 150 metric series ([Storage › Size after 2 years](../02-architecture/storage.md#size-after-2-years)).
- [x] No alert repeats within 24 hours.
  `notify::tests::no_alert_repeats_within_24_hours`; sent alerts are kept in the database, so a restart does not repeat them either.

Checked locally on 2026-10-23: all Rust and web tests, lint, `make perf` (first result 1.5 s, idle CPU 0.37%, peak RAM 18.9 MB, database 10 MB after the first scan with 30 days of history), `make deb` with lintian, and the `.deb` container test up to the agent. The container test of the desktop package was stopped while downloading its libraries (slow network); run `make deb-test` again, and CI runs it on every push.

Not checked yet (needs people or machines): how the timeline and notifications feel to beginners, and the tray on desktops other than Ubuntu's GNOME.

---

← Previous: [Phase 2](phase-2-fix-safely.md) · [Index](README.md) · Next: [Phase 4 — Assistant](phase-4-assistant.md) →
