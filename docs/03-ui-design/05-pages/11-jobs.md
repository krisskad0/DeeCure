# Scheduled Jobs (`jobs`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Workloads · Icon: `cal` · Status badge: {ok:Healthy} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Calendar, Cron, Systemd Timers, Anacron, At Jobs

## Scheduled Jobs › Calendar: `jobs/calendar`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Total jobs | 10 |  | - | cron · timer · anacron · at | - | - |
| Next run | 26 | min | - | sync-notes.sh | - | - |
| Failed | 0 |  | - | last 7 days | - | - |
| Missed | 1 |  | {warn:Warning} | e2scrub_all.timer | - | - |

2. **Swim-lane schedule ([Swim-lane schedule (next 24 hours)](../03-components/19-swim-lane-schedule.md))**: "Next 24 hours" · span 12
- Source chip (Expert): `crontab -l · systemctl list-timers`

| Lane (mono) | Runs at (hours from now) | Dot |
| --- | --- | --- |
| sync-notes.sh | 47 runs: 0.4, 0.9, 1.4 … every 0.5 h to 23.4 | small 6px |
| logrotate.timer | 2.4 | 10px |
| apt-daily-upgrade | 8.8 | 10px |
| apt-daily.timer | 9.1, 21.3 | 10px |
| tmpfiles-clean | 11.6 | 10px |
| backup-photos.sh | 23.4 | 10px |

Axis: 0 "Now 21:34", 2.43 "00:00", 8.43 "06:00", 14.43 "12:00", 20.43 "18:00". Each dot has a `title` with the job name.

3. **Heatmap (day × hour)**: "Job runs by hour" · span 12
- Subtitle: "last 7 days"
- Rows: Thu, Fri, Sat, Sun, Mon, Tue, Wed · Columns: hours 00-23 · unit "runs"
- Values (row by row, 24 per row):

```
Thu  1 0 0 0 0 0 2 0 0 2 2 2 2 2 2 2 2 2 3 2 2 2 2 2
Fri  1 0 0 0 0 0 2 0 0 2 2 2 2 2 2 2 2 2 3 2 2 2 2 2
Sat  1 0 0 0 0 0 2 0 0 2 2 2 2 2 2 2 2 2 3 2 2 2 2 2
Sun  1 0 0 0 0 0 2 0 0 2 2 2 2 2 2 2 2 2 3 2 2 2 2 2
Mon  1 0 0 0 0 0 2 0 0 2 2 2 2 2 2 2 2 2 3 2 2 2 2 2
Tue  1 0 0 0 0 0 2 0 0 2 2 2 2 2 2 2 2 2 3 2 2 3 2 2
Wed  1 0 0 0 0 0 2 0 0 2 2 2 2 2 2 2 2 2 3 2 2 3 0 0
```

## Scheduled Jobs › Cron: `jobs/cron`

1. **Data table**: "Cron jobs" · span 12
- Source chip (Expert): `crontab -l · /etc/crontab · /etc/cron.d`

| Schedule | Command | User | Source | Next run | Last result |
| --- | --- | --- | --- | --- | --- |
| ***/30 * * * *** · Every 30 minutes | `~/bin/sync-notes.sh` | riya | `crontab (riya)` | 22:00 | {off:Not recorded} [Track](act:track:sync) |
| **0 21 * * *** · Every day at 21:00 | `~/bin/backup-photos.sh` | riya | `crontab (riya)` | Tomorrow 21:00 | {off:Not recorded} [Track](act:track:backup) |
| **17 * * * *** · Hourly at :17 | `run-parts --report /etc/cron.hourly` | root | `/etc/crontab` | 22:17 | {ok:Success} |
| **30 3 * * 0** · Sundays 03:30 | `e2scrub_all_cron` | root | `/etc/cron.d/e2scrub_all` | Sun 25 Oct | {off:Skipped (systemd)} |

2. **Alert banner**: "Cron does not record job results" · span 12
- State: `info` · Title: "Cron does not record job results" · Text: "Track wraps a job so its exit code and output are saved from the next run."

## Scheduled Jobs › Systemd Timers: `jobs/timers`

1. **Data table**: "Timers" · span 12
- Source chip (Expert): `systemctl list-timers --all`

| Timer | Activates | Schedule | Last trigger | Next | Result | Persistent |
| --- | --- | --- | --- | --- | --- | --- |
| `logrotate.timer` | `logrotate.service` | `daily` | Today 00:00 | 00:00 | {ok:Success} | ✓ |
| `apt-daily-upgrade.timer` | `apt-daily-upgrade.service` | `*-*-* 6:00` | Today 10:40 | 06:21 | {ok:Success} | ✓ |
| `apt-daily.timer` | `apt-daily.service` | `*-*-* 6,18:00` | Today 18:52 | 06:40 | {ok:Success} | ✓ |
| `systemd-tmpfiles-clean.timer` | `systemd-tmpfiles-clean.service` | `OnUnitActiveSec=1d` | Today 09:13 | Tomorrow 09:13 | {ok:Success} | - |
| `fstrim.timer` | `fstrim.service` | `weekly` | Mon 09:00 | Mon 26 Oct | {ok:Success} | ✓ |
| `e2scrub_all.timer` | `e2scrub_all.service` | `Sun *-*-* 03:10` | Sun 18 Oct | Sun 25 Oct | {warn:Missed} | - |
| `anacron.timer` | `anacron.service` | `hourly` | Today 21:07 | 22:07 | {ok:Success} | ✓ |
| `snapd.snap-repair.timer` | `snapd.snap-repair.service` | `random 6h` | Today 17:40 | Tomorrow 01:12 | {ok:Success} | - |

## Scheduled Jobs › Anacron: `jobs/anacron`

1. **Data table**: "Anacron jobs" · span 12
- Source chip (Expert): `/etc/anacrontab · /var/spool/anacron`

| Job | Period | Delay [n] | Command | Last run |
| --- | --- | --- | --- | --- |
| `cron.daily` | 1 day | 5 min | `run-parts /etc/cron.daily` | Today 09:05 |
| `cron.weekly` | 7 days | 10 min | `run-parts /etc/cron.weekly` | Mon 09:10 |
| `cron.monthly` | 1 month | 15 min | `run-parts /etc/cron.monthly` | 01 Oct |

## Scheduled Jobs › At Jobs: `jobs/at`

1. **Data table**: "One-time jobs" · span 12
- Source chip (Expert): `atq`

| Job | Run at | Command | User | (action) |
| --- | --- | --- | --- | --- |
| `#4` | Fri 23 Oct 09:00 | `notify-send "Submit the form"` | riya | [Cancel](act:toast:Job #4 removed (atrm 4).) |

---

← Previous: [Services](10-services.md) · [Index](README.md) · Next: [Containers](12-containers.md) →
