# Diagnostics (`diagnostics`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Tools · Icon: `gauge` · Status badge: none
- Sub-pages: Performance Check, Network Test, Hardware Tests

## Diagnostics › Performance Check: `diagnostics/performance`

1. **Diagnostics runner ([Diagnostics runner](../03-components/23-diagnostics-runner.md))**: "Performance check" · span 12
- Info tooltip: "Run while the computer feels slow. Finds the main cause in about 10 seconds."

Idle steps: CPU load → Memory → Memory pressure → Disk I/O → Temperature → Top processes. Button: [**Run check**](act:diag:run).

> After the run completes, these widgets appear below the runner: **KPI row**: Memory used 7.6 GB `{crit:Critical}` "99% of 7.6 GB"; Memory pressure 23.4 % `{crit}` "some avg10"; Swap activity 18 MB/s `{warn}`; CPU 23 % "normal"; Temperature 61 °C "normal". **Data table "Contributing processes"** span 7: columns Process, Memory [n], CPU [n], Impact; rows: chrome (42 tabs) 6.1 GB 14% {crit:High}; code 640 MB 4% {off:Low}; gnome-shell 410 MB 3% {off:Low}; telegram-desktop 280 MB 1% {off:Low}. **Checklist "Recommendations"** span 5: `warn` Close unused browser tabs / Or enable Memory Saver in Chrome; `warn` Increase swap to 4 GB / Reduces freezes when memory fills / [Configure](act:toast:Swap resize opens a guided fix in the real app.); `pass` CPU and temperature are normal. The runner panel result: inline crit alert "Primary cause: memory saturation" / "chrome uses 6.1 GB of 7.6 GB (42 tabs) · swap active · memory pressure 23%". Header action [Run again](act:diag:run).

## Diagnostics › Network Test: `diagnostics/network`

1. **Diagnostics runner ([Diagnostics runner](../03-components/23-diagnostics-runner.md))**: "Network test" · span 12

Idle steps: Gateway → DNS resolution → Internet latency → Packet loss → IPv6. Button: [**Run test**](act:net:run).

> After the run completes: runner result is an inline ok alert "Network is healthy" / "All 5 checks passed", header action [Run again](act:net:run), and a **KPI row** appears: Gateway 2 ms; DNS lookup 14 ms; Latency 18 ms "1.1.1.1"; Jitter 3 ms; Packet loss 0 %; IPv6 Link-local.

## Diagnostics › Hardware Tests: `diagnostics/hardware`

1. **Data table**: "Available tests" · span 12

| Test | Duration | Last run | Result | (action) |
| --- | --- | --- | --- | --- |
| SMART short self-test (NVMe) | 2 min | 7 days ago | {ok:Passed} | [Run](act:toast:SMART short self-test started.) |
| Memory test (memtest86+) | 30-60 min | Never | {off:-} | [Schedule](act:toast:Runs at next restart from the boot menu.) |
| Battery capacity report | 1 min | Today | {warn:71% health} | [Open](go:power/battery) |
| Sensor check | 10 s | Today | {ok:Passed} | [Open](go:hardware/sensors) |
| Speaker and microphone test | 30 s | Never | {off:-} | [Run](act:toast:Plays a test tone and records 3 seconds.) |

---

← Previous: [Privacy](19-privacy.md) · [Index](README.md) · Next: [Assistant](21-assistant.md) →
