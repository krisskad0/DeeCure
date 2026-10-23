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
| Database size | Size after the first scan (with 30 days of history read on the first run). Two years of growth are checked by `make db-size` ([Storage](storage.md#size-after-2-years)). |

The limits are at the top of the script. Change them only together with this page.

Last measured on 2026-10-23, on a real Ubuntu 24.04 laptop, with the history task, notifications, and all 16 rules:

| Measure | Result | Budget |
| --- | --- | --- |
| First useful results | 1.5 s | 10 s |
| Agent CPU when idle | 0.37% | 1% |
| Agent RAM (peak) | 18.9 MB | 60 MB |
| Database after the first scan | 10 MB | 200 MB |
| Database after 2 simulated years | 124 MB | 200 MB |

Not checked yet: desktop app RAM and the full first scan.
