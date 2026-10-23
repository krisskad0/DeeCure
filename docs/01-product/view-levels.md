# View levels: Beginner, Advanced, Expert

[Docs](../README.md) › [Product](README.md)

**Status: approved by the owner on 2026-10-23. Being built step by step; see [pending work](../05-roadmap/pending.md).**

DeeCure has three view levels. Each level shows everything the level below shows, plus more. The menus stay short at every level: depth goes **inside** pages (detail drawers, sections, history), not into more tabs.

| Level | For | Menus | What it adds |
| --- | --- | --- | --- |
| Beginner (default) | People new to Linux | 8 pages, 22 sub-pages | Plain words, the health score, safe one-click fixes |
| Advanced | People who know their computer | All 21 pages, Settings included (tabs merged, see below) | Every page and tab, with clear columns |
| Expert | People who want every detail | Same menus as Advanced, plus **Optimize** | Data sources, commands, raw columns, and the deep data in [Expert coverage](expert-coverage.md) |

## Rules for every level

- **Measured, never guessed.** Every number comes from a named source on this computer. If Linux cannot measure something exactly, DeeCure says so and does not show a made-up number.
- **No myths.** DeeCure does not offer common "speed-up tips" that do not help on Ubuntu or that trade safety for speed ([Optimize › Not offered](optimize-catalog.md#not-offered-and-why)).
- **Suggestions follow a measurement.** An optimization is offered only when a measurement on this computer shows it is needed ([Optimize](optimize-catalog.md)).
- **Light on the computer.** Deep data is read only while its page is open, and slow reads only when asked. The agent stays within its [performance budget](../02-architecture/performance-budget.md).
- **Admin reads go through the helper.** Items marked "Admin" need Full access and your password, like the drive health read today. Each new admin read is added to the helper's security review.

## How Expert adds depth without more tabs

- **Detail drawer on every row.** Click any process, service, drive, device, package, or connection to see everything about it.
- **Expert sections.** Extra panels at the bottom of a tab, shown only in Expert.
- **History.** Expert values are kept, so trends (drive wear, battery health, boot time) are charts, not single numbers.
- **One new page: Optimize.** It shows every measured improvement in one place, with a preview and undo for each.

## Fewer tabs for Advanced and Expert

✅ Done. The full view had 89 tabs. Merging tabs that show the same kind of list into one tab brings this to 77. A merged tab shows the old tabs as parts, one after another; in Beginner, Activity › Timeline shows only the timeline part.

| Page | Today | Proposed |
| --- | --- | --- |
| Scheduled Jobs | Calendar, Cron, Systemd Timers, Anacron, At Jobs | Calendar, All jobs (cron, timers, anacron, at) |
| Logs | Journal, Errors, Crashes, Kernel, Boots | Journal (with kernel messages), Errors and crashes, Boots |
| Activity | Timeline, Package History, Config Changes, Device Events, DeeCure Actions | Timeline (with package, device, and DeeCure history), Config Changes |
| Accounts | Users, Groups, Sessions, Login History, SSH Keys | Users and groups, Sign-ins (sessions and history), SSH Keys |
| Containers | Containers, Images, Volumes, Virtual Machines | Containers, Images and volumes, Virtual Machines |
| Backups | Backup Jobs, System Snapshots, Restore Points | Backups (jobs and snapshots), Restore Points |

Links to old tabs keep working: they open the merged tab and scroll to that part (for example `#/jobs/cron` opens All jobs at the cron part).

---

[Expert coverage](expert-coverage.md) · [Optimize](optimize-catalog.md)
