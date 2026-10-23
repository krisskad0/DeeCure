# State model

[Docs](../../README.md) › [UI design](../README.md) › [Behavior](README.md)

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

## Issues list (`issues()`, in this order)

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
