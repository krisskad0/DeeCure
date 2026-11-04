# Optimize (`optimize`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Tools · Icon: `bolt` · Status badge: none
- Sub-pages: Improvements
- Shown in the menus at the Expert level only ([View levels](../../01-product/view-levels.md)). What it offers and when: [Optimize catalog](../../01-product/optimize-catalog.md).

## Optimize › Improvements: `optimize/all`

Data: the agent's Optimize checks, asked one part at a time (inventory kinds `optimize_boot`, `optimize_disks`, `optimize_kernels`, `optimize_devices`, `optimize_hardware`, `optimize_cleanup`, `optimize_history`, `optimize_drivers`), so the page shows each part live. An agent from before the parts answers "unknown inventory"; the page then asks `optimize` for everything in one call and shows the area tiles without live parts. Every check says what it measured on this computer, also when nothing is needed. Each check belongs to an area by the first part of its ID (Start-up, Storage, Software, System, Power, Hardware, Memory, Graphics, Drivers, Firmware, Security; Other for an ID the app does not know).

**When it checks.** One scan is shared with the Overview card (`pages/optimize/scan.ts`). The last answer is kept on this computer (`localStorage`, key `deecure.optimize`), also after the app restarts. DeeCure checks by itself only when Optimize or the Overview is open and the answer is missing or older than 24 hours (after a failed try, again after 1 hour at the earliest). Otherwise it checks only when the user clicks the **Scan** button on this page (the Overview's Scan row only opens this page). The Refresh button does not start a scan. After a fix, nothing is checked; the page says "A fix ran after this check" until the user checks again.

1. **Panel "Improvements"** · span 12 · info "Offered only when a measurement on this computer shows it is needed. Each fix shows a preview first, and can be undone where undo is possible. DeeCure checks by itself once a day; Scan now checks at once." Action: "Last checked Today 10:42" (hidden on narrow screens).
   - The panel body has two sides: the result, or the live scan, on the left; on the right the **Scan button** (`pages/optimize/ScanButton.tsx`), a round scanner 132px wide: a thin ring, a face with a viewfinder (four corner brackets) and a scan line, and the label "Scan" ("Scan again" once there is a result). Under it: "About 15 seconds". On hover the ring and face turn blue and the line moves. While a scan runs it is disabled and busy: the face turns light blue, the scan line sweeps up and down, a dashed ring turns, the outer ring fills with the parts done, the label shows "5/8", and the caption "Scanning… 5 of 8 parts done". On narrow screens it sits under the result.
   - While a scan runs, the panel shows only the scan: a radar animation (rings, a turning beam, blips), "Checking this computer", "Part N of 8: <part>", a bar that fills as parts finish, "N s so far, usually about 15 s · found N improvements so far", and one row per part with its icon, name, and what it checks. A row is dimmed and says "Waiting", is highlighted with a spinner and "Checking… N s" while it runs, and then shows {warn:N to improve} or {ok:Nothing needed} with "N checks · 1.2 s", or {crit:Could not check} with the reason as a tooltip. **Found so far** lists each found item as a chip as soon as its part finishes. Measured times on a laptop: start-up 3.9 s, space, packages, and updates 4.8 s, drivers and firmware 4.5 s, the rest under 1 s each.
   - No answer yet and no scan running: "Not checked yet" and what Optimize does, next to the Scan button. No answer because of an error: the usual agent help message, next to the Scan button.
   - Result: a donut (112px, 10px ring on a grey track) of needed (amber) and passed (green) checks, with rounded ends and a small gap between the two, drawn in on first show; the count to improve in amber in the middle ("all passed" with a green check when none), the heading "N improvements for this computer" ("Nothing to improve now" when none), "N checks of M passed", and chips: fixes with a preview, advice to do by hand, need your password, passed (a chip with 0 is left out).
   - Notes over the result: warn "Could not check again" (the last answer stays), info "A fix ran after this check" / "Scan again to see what is left.", warn "N parts could not be checked" with each part and its reason (the other parts' checks are shown).
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
