# Optimize (`optimize`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Tools · Icon: `bolt` · Status badge: none
- Sub-pages: Improvements
- Shown in the menus at the Expert level only ([View levels](../../01-product/view-levels.md)). What it offers and when: [Optimize catalog](../../01-product/optimize-catalog.md).

## Optimize › Improvements: `optimize/all`

Data: the agent's `optimize` inventory. Every check says what it measured on this computer, also when nothing is needed. One scan is shared with the Overview strip (`pages/optimize/scan.ts`): it runs when the answer is missing or older than 2 minutes, on Refresh, on **Check again**, and right after a fix. Each check belongs to an area by the first part of its ID (Start-up, Storage, Software, System, Power, Hardware, Memory, Graphics, Drivers, Firmware, Security; Other for an ID the app does not know).

1. **Panel "Improvements"** · span 12 · info "Offered only when a measurement on this computer shows it is needed. Each fix shows a preview first, and can be undone where undo is possible." Actions: "Last checked Today 10:42" (hidden on narrow screens) and [**Check again**] (shows "Checking…" and is disabled while a scan runs).
   - First scan: a radar animation (rings, a turning beam, blips), "Checking this computer", "Nothing is changed while it checks", a moving bar, "N s so far, usually about 10 s", and one tile per area with what it checks. No made-up progress: the agent sends one answer at the end.
   - Result: a ring of passed (green) and needed (amber) checks with the count in the middle, the heading "N improvements for this computer" ("Nothing to improve now" when none), "N checks of M passed", and chips: fixes with a preview, advice to do by hand, need your password, passed (a chip with 0 is left out).
   - Checking again: an info line "Checking again: N s so far…" over the last answer. A failed scan with an older answer: a warn alert "Could not check again". No answer at all: the usual agent help message.
   - The agent answered with no checks: info alert "No checks ran".
2. **Panel "Your path (N)"** · span 12 · only when something is offered. Items in order: fixes first, then advice.
   - "N of M looked at" and one bar segment per item; a segment turns green when its item was looked at, and opens its card. When all are looked at: "You went through every item".
   - Area chips with counts ("All", then each area), only with items from 2 or more areas.
   - One card per item: step number (a check mark once looked at), area, "Item N of M", the title, **Measured** with the measurement, and for advice **What to do** in an amber box. Badges: {info:Fix with preview} or {warn:Advice}; for fixes {info:Needs your password} or {off:No password}; {ok:Looked at}. Buttons: [**Preview**](act:fix) for fixes, and **Next** ("Got it, next" for advice; "Done" or "Got it" on the last card) that marks the item looked at and moves to the next card. Preview also marks it looked at. "Looked at" lasts until the app closes.
3. **Data table "Checked, nothing needed"** · span 12 · collapsed by default with a count: columns Area, Check, What DeeCure measured. Example rows: "Start-up" · "Stop "wait for network" from delaying boot" · "It delayed boot by 5.9 s, but docker.service waits for the network at this boot, so it stays."; "Storage" · "Turn on the weekly TRIM" · "fstrim.timer is enabled: nvme0n1 get TRIM every week."

Motion stops when the system asks for reduced motion; cards never start hidden, so nothing is lost if an animation does not run.

Agent older than the app: the help message "The background agent is older than this app…" with the restart hint, like other Expert parts.

---

← Previous: [Settings](22-settings.md) · [Index](README.md)
