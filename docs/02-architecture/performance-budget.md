# Performance budget

[Docs](../README.md) › [Architecture](README.md)

| Measure | Target |
| --- | --- |
| Agent CPU when idle | Under 1% |
| Agent RAM | Under 60 MB |
| Desktop app RAM (open) | Under 200 MB |
| First useful results | Under 10 seconds |
| Full first scan (without deep disk scan) | Under 60 seconds |
| Database size | Under 200 MB |

## How it is checked

`make perf` (`scripts/perf-check.sh`) runs on every push in CI (job "Performance budget") and fails the build when the agent is over budget:

| Measure | How |
| --- | --- |
| First useful results | Time from starting the agent until `deecure status` shows the rules have run |
| Agent CPU when idle | CPU time used over 65 seconds with no clients, which includes one full collection cycle |
| Agent RAM | Peak memory (`VmHWM`) after 300 `status` requests |
| Database size | Size after the first scan. Long-term growth is checked in Phase 3, with two years of simulated data. |

The limits are at the top of the script. Change them only together with this page.

Not checked yet: desktop app RAM (Phase 1) and the full first scan (it grows as collectors are added).
