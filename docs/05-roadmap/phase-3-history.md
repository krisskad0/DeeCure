# Phase 3 — History and trends

[Docs](../README.md) › [Roadmap](README.md)

**Goal:** DeeCure remembers what changed and warns about problems before they happen.
**Rough time:** 5–6 weeks.

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

- [ ] The journey "Something broke after a change" works end to end.
  → [User journey](../01-product/user-journey.md#something-broke-after-a-change)
- [ ] The database stays under 200 MB after 2 years of simulated data.
- [ ] No alert repeats within 24 hours.

---

← Previous: [Phase 2](phase-2-fix-safely.md) · [Index](README.md) · Next: [Phase 4 — Assistant](phase-4-assistant.md) →
