# Swim-lane schedule (next 24 hours)

[Docs](../../README.md) › [UI design](../README.md) › [Components](README.md)

Scroll container with a min-width 620 inner. Rows 28px: grid `160px | 1fr`, gap 10. Name in mono 12.5px `--ink-2`. Track: 1px left border, 1px `--grid` horizontal center line. Each run is a 10px `--s1` circle with a 2px `--panel` ring at `left = hoursFromNow / 24`. Very frequent jobs use 6px dots without a ring. Axis row: labels at hour offsets 0 "Now 21:34", 2.43 "00:00", 8.43 "06:00", 14.43 "12:00", 20.43 "18:00".

---

← Previous: [Allocation bar (disk)](18-allocation-bar.md) · [Index](README.md) · Next: [Lifecycle bar](20-lifecycle-bar.md) →
