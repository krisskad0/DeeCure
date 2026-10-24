# Optimize (`optimize`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Tools · Icon: `bolt` · Status badge: none
- Sub-pages: Improvements
- Shown in the menus at the Expert level only ([View levels](../../01-product/view-levels.md)). What it offers and when: [Optimize catalog](../../01-product/optimize-catalog.md).

## Optimize › Improvements: `optimize/all`

Data: the agent's `optimize` inventory. Every check says what it measured on this computer, also when nothing is needed.

1. **List "Improvements"** · span 12 · info "Offered only when a measurement on this computer shows it is needed. Each fix shows a preview first."
   - One row per offered check, fixes first: the title (bold), the measurement under it (`--ink-2`), and on the right either a badge `{info:Needs your password}` (admin) or nothing (you), and the button [**Preview**](act:fix) that opens the fix preview; or, for advice, a `{warn:Advice}` badge and the advice text under the measurement.
   - Empty: ok alert "Nothing to improve now" / "Every check below passed."
2. **Data table "Checked, nothing needed"** · span 12 · collapsed by default with a count: columns Check, What DeeCure measured. Example rows: "Stop "wait for network" from delaying boot" · "It delayed boot by 5.9 s, but docker.service waits for the network at this boot, so it stays."; "Turn on the weekly TRIM" · "fstrim.timer is enabled: nvme0n1 get TRIM every week."

Agent older than the app: the help message "The background agent is older than this app…" with the restart hint, like other Expert parts.

---

← Previous: [Settings](22-settings.md) · [Index](README.md)
