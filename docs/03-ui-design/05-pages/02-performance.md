# Performance (`performance`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Monitoring · Icon: `perf` · Status badge: {ok:Healthy} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: CPU, Memory, Disk I/O, Network, GPU, Pressure (PSI). Beginner mode shows all but Pressure (PSI) ([Beginner mode](../02-shell/sidebar.md#beginner-mode)).

## Performance › CPU: `performance/cpu`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Utilization | 12 | % | - |   | Sparkline slot 1, gen[3, 14, 8, 6, 2, 60, 12] | - |
| Load average | 0.84 |  | - | 5 min 0.92 · 15 min 1.10 | - | - |
| Frequency | 1.3 | GHz | - | Max 4.4 GHz | - | - |
| Temperature | 54 | °C | - |   | Sparkline slot 2, gen[21, 52, 6, 3, 35, 90, 54] | - |
| Processes | 312 |  | - | 1,204 threads | - | - |
| Context switches | 6.2K | /s | - | Interrupts 3.1K/s | - | - |

2. **Line chart**: "CPU utilization by type" · span 12
- Info tooltip: "User = your apps. System = the kernel. I/O wait = waiting for the disk."
- Source chip (Expert): `/proc/stat`
- Height 240px · Y axis 0 to 50 · unit "%" · **stacked**
- Series: User (slot 1, gen[3, 8, 5, 4, 0, 60]); System (slot 2, gen[13, 3, 2, 2, 0, 30]); I/O wait (slot 3, gen[17, 0.8, 0.6, 1, 0, 15])
- X axis: time (current range)

3. **Bar chart**: "Per-core utilization" · span 6
- Subtitle: "now"
- Source chip (Expert): `mpstat -P ALL`
- Height 200px · Y max 100 · unit "%"
- Bars (label: value · tooltip label): P0: 18 · "Thread P0"; P1: 9 · "Thread P1"; P2: 14 · "Thread P2"; P3: 6 · "Thread P3"; E0: 22 · "Thread E0"; E1: 11 · "Thread E1"; E2: 4 · "Thread E2"; E3: 7 · "Thread E3"; E4: 16 · "Thread E4"; E5: 3 · "Thread E5"; E6: 8 · "Thread E6"; E7: 5 · "Thread E7"

4. **Data table**: "Top CPU consumers" · span 6
- Source chip (Expert): `ps --sort=-%cpu`

| Process | PID [n] | User | CPU [n] |
| --- | --- | --- | --- |
| chrome | 4121 | riya | [meter 21% "4.2%"] |
| gnome-shell | 1874 | riya | [meter 19% "3.8%"] |
| chrome (renderer) | 4388 | riya | [meter 13% "2.6%"] |
| code | 5102 | riya | [meter 10% "1.9%"] |
| Xwayland | 1990 | riya | [meter 3% "0.6%"] |
| pipewire | 1702 | riya | [meter 3% "0.5%"] |
| telegram-desktop | 2230 | riya | [meter 2% "0.4%"] |

## Performance › Memory: `performance/memory`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Used | 5.9 | GB | - | 78% of 7.6 GB | Sparkline slot 2, gen[5, 70, 8, 3, 40, 95, 78] | - |
| Available | 1.7 | GB | - |   | - | - |
| Cache and buffers | 1.6 | GB | - |   | - | - |
| Swap | 1.2 | GB | - | of 2.0 GB | Usage bar 60% | - |
| Pressure | 0.4 | % | - | some avg10 | - | - |
| OOM kills | 1 |  | {warn:Warning} | last 7 days | - | - |

2. **Line chart**: "Memory usage" · span 8
- Source chip (Expert): `/proc/meminfo`
- Height 240px · Y axis 0 to 8 · unit "GB" · **stacked**
- Series: Used (slot 1, gen[5, 4.6, 0.9, 0.5, 2.5, 6.4, 5.9]); Cache (slot 2, gen[7, 1.4, 0.3, 0.2, 0.5, 2.5, 1.6])
- X axis: time (current range)
- Threshold line at 7.6: "Total 7.6 GB"

3. **Donut chart**: "Composition" · span 4
- Subtitle: "now"
- Center: "7.6" / "GB total" · unit "GB"

| Segment | Value | Color | Display value |
| --- | --- | --- | --- |
| Used | 5.9 | slot 1 | - |
| Cache | 1.6 | slot 2 | - |
| Free | 0.1 | kn | 0.1 GB |

4. **Data table**: "Top memory consumers" · span 12
- Source chip (Expert): `ps --sort=-rss`

| Process | PID [n] | User | Resident [n] | Share of total [n] | Swap [n, ,x] |
| --- | --- | --- | --- | --- | --- |
| chrome | 4121 | riya | 842 MB | [meter 44% "11.1%"] | 120 MB |
| chrome (renderer) | 4388 | riya | 612 MB | [meter 32% "8.1%"] | 84 MB |
| gnome-shell | 1874 | riya | 410 MB | [meter 22% "5.4%"] | 0 |
| code | 5102 | riya | 486 MB | [meter 26% "6.4%"] | 62 MB |
| telegram-desktop | 2230 | riya | 280 MB | [meter 15% "3.7%"] | 0 |
| dropbox | 2301 | riya | 210 MB | [meter 11% "2.8%"] | 0 |
| postgres | 2811 | 999 | 142 MB | [meter 7% "1.9%"] | 0 |
| dockerd | 1320 | root | 96 MB | [meter 5% "1.3%"] | 0 |

## Performance › Disk I/O: `performance/disk`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Read | 2.4 | MB/s | - |   | Sparkline slot 1, gen[31, 2, 2, 3, 0, 40, 2.4] | - |
| Write | 0.8 | MB/s | - |   | Sparkline slot 2, gen[32, 1, 1, 1.5, 0, 30, 0.8] | - |
| IOPS | 142 |  | - |   | - | - |
| Utilization | 6 | % | - |   | - | - |
| Avg latency | 0.4 | ms | - |   | - | - |
| Queue depth | 0.02 |  | - |   | - | - |

2. **Line chart**: "Throughput" · span 12
- Source chip (Expert): `/proc/diskstats`
- Height 220px · Y axis 0 to auto · unit "MB/s"
- Series: Read (slot 1, gen[31, 2, 2, 3, 0, 40, 2.4]); Write (slot 2, gen[32, 1, 1, 1.5, 0, 30, 0.8])
- X axis: time (current range)

3. **Data table**: "Devices" · span 6
- Source chip (Expert): `iostat -x`

| Device | Type | Model [x] | Size [n] | Read/s [n] | Write/s [n] | Util [n] | Await [n, ,x] |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `nvme0n1` | SSD (NVMe) | Samsung 980 | 477 GB | 2.4 MB | 0.8 MB | 6% | 0.4 ms |
| `sda` | Hard disk | ST1000LM035 | 932 GB | 0 | 0 | 0% | 0 ms |

- Whole disks only (no partitions, loop, or RAM disks). Type, Model, and Size come from the hardware inventory (the same as Hardware › Storage Devices): SSD (NVMe), SSD, Hard disk, USB drive, or Memory card. A drive plugged in since the last inventory shows "-" until the next one (within 30 s).

4. **Data table**: "Top I/O processes" · span 6 (the Devices table takes the full width until then)
- **Not in this version:** this needs a per-process collector that DeeCure does not have yet, so the panel is hidden rather than shown empty ([pending work](../../05-roadmap/pending.md)).
- Source chip (Expert): `iotop -b -n1`
- Empty text: "Requires Full access · Settings › Access"

| Process | Read [n] | Write [n] |
| --- | --- | --- |
| chrome | 1.1 MB/s | 0.4 MB/s |
| tracker-miner-fs-3 | 0.8 MB/s | 0.1 MB/s |
| code | 0.2 MB/s | 0.1 MB/s |
| postgres | 0.1 MB/s | 0.1 MB/s |

## Performance › Network: `performance/network`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Download | 1.2 | MB/s | - |   | Sparkline slot 1, gen[9, 0.8, 0.6, 0.8, 0, 6, 1.2] | - |
| Upload | 0.3 | MB/s | - |   | Sparkline slot 2, gen[10, 0.2, 0.2, 0.3, 0, 3, 0.3] | - |
| Data today | 4.2 | GB | - |   | - | - |
| Connections | 68 |  | - | 5 listening | - | - |
| Packet loss | 0 | % | - |   | - | - |
| Latency | 18 | ms | - | to 1.1.1.1 | - | - |

2. **Line chart**: "Throughput" · span 12
- Source chip (Expert): `/proc/net/dev`
- Height 220px · Y axis 0 to auto · unit "MB/s"
- Series: Download (slot 1, gen[9, 0.8, 0.6, 0.8, 0, 6, 1.2]); Upload (slot 2, gen[10, 0.2, 0.2, 0.3, 0, 3, 0.3])
- X axis: time (current range)

3. **Bar chart**: "Daily data usage" · span 6
- Subtitle: "14 days"
- Source chip (Expert): `vnstat -d`
- Height 200px · Y max auto · unit "GB"
- Bars (label: value · tooltip label): 8: 3.1 · "8 Oct"; 9: 4.4 · "9 Oct"; 10: 2.2 · "10 Oct"; 11: 1.9 · "11 Oct"; 12: 5.8 · "12 Oct"; 13: 3.6 · "13 Oct"; 14: 4.1 · "14 Oct"; 15: 3.3 · "15 Oct"; 16: 6.2 · "16 Oct"; 17: 2.8 · "17 Oct"; 18: 1.5 · "18 Oct"; 19: 4.9 · "19 Oct"; 20: 5.1 · "20 Oct"; 21: 4.2 · "21 Oct"

4. **Data table**: "Interfaces" · span 6
- Source chip (Expert): `ip -s link`

| Interface | Type | State | RX/s [n] | TX/s [n] |
| --- | --- | --- | --- | --- |
| `wlp0s20f3` | Wi-Fi | {ok:up} | 1.2 MB | 0.3 MB |
| `docker0` | Bridge | {ok:up} | 12 KB | 9 KB |
| `lo` | Loopback | {ok:up} | 4 KB | 4 KB |
| `enx00e04c68` | Ethernet (USB) | {off:down} | 0 | 0 |

## Performance › GPU: `performance/gpu`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Device | Iris Xe |  | - | Intel · Alder Lake-UP3 GT2 | - | - |
| Utilization | 8 | % | - |   | Sparkline slot 1, gen[41, 7, 5, 4, 0, 80, 8] | - |
| Memory | Shared |  | - | Uses system RAM | - | - |
| Temperature | N/A |  | - | Not reported by i915 | - | - |
| Driver | i915 |  | - | Mesa 24.0.9 | - | - |

2. **Line chart**: "GPU utilization" · span 12
- Source chip (Expert): `intel_gpu_top (admin)`
- Height 220px · Y axis 0 to 100 · unit "%"
- Series: Render (slot 1, gen[41, 7, 5, 4, 0, 80, 8])
- X axis: time (current range)

3. **Alert banner**: "GPU temperature is not exposed" · span 12
- State: `info` · Title: "GPU temperature is not exposed" · Text: "The Intel i915 driver shares the CPU package sensor. See Hardware › Sensors." · Buttons: [Open sensors](go:hardware/sensors)

## Performance › Pressure (PSI): `performance/pressure`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| CPU some | 1.2 | % | - |   | - | - |
| Memory some | 0.4 | % | - |   | - | - |
| Memory full | 0.1 | % | - |   | - | - |
| I/O some | 0.8 | % | - |   | - | - |
| I/O full | 0.2 | % | - |   | - | - |

2. **Line chart**: "Pressure stall information" · span 12
- Info tooltip: "Share of time tasks waited for CPU, memory or disk. The most reliable sign of a slow system."
- Source chip (Expert): `/proc/pressure/{cpu,memory,io}`
- Height 240px · Y axis 0 to auto · unit "%"
- Series: CPU (slot 1, gen[51, 1.4, 1, 1.5, 0, 40, 1.2]); Memory (slot 2, gen[52, 0.6, 0.5, 1.2, 0, 40, 0.4]); I/O (slot 3, gen[53, 0.9, 0.6, 1, 0, 40, 0.8])
- X axis: time (current range)

3. **Key-value list**: "Thresholds" · span 12

| Key | Value |
| --- | --- |
| Healthy | below 5% (avg10) |
| Noticeable slowdown | 5-20% |
| Severe | above 20% |
| Peak in range | {warn:Memory 23.4% · Mon 16:12} |

---

← Previous: [Overview](01-overview.md) · [Index](README.md) · Next: [Logs](03-logs.md) →
