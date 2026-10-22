# Health score

[Docs](../README.md) › [Architecture](README.md)

1. Each category starts with its share of the score (for example, Storage 15, Security 15, Hardware 15).
2. Each open finding deducts points: critical −15, warning −5, info 0. A category can never go below 0.
3. The total is out of 100, with a breakdown shown on tap: "Storage −10 (disk 94% full)".
4. The score only uses findings the system can actually check, so a missing sensor does not lower it.

## What is built

The code is `health::score` in `crates/deecure-core/src/health.rs`. For now every area has the same share, 15 points (`AREA_SHARE`). Different shares per area are [open decision #15](../01-product/open-decisions.md).

- Only **open** findings count. Resolved and dismissed findings cost nothing.
- The agent's `status` call returns the score and the points each area lost, largest first ([IPC protocol](ipc-protocol.md#methods)).
- Before the first check there is no score at all. An empty list would look like a perfect 100.
- After every check the agent saves the score as the metric `health.score`, so the app can draw a trend.
- Colours in the app: 90+ Healthy, 75–89 Fair, below 75 At risk.

Example: one critical storage finding and one warning power finding give 100 − 15 − 5 = **80**.

