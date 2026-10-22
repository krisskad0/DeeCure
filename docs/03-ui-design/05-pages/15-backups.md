# Backups (`backups`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Software & Data · Icon: `backup` · Status badge: {warn:Warning} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Backup Jobs, System Snapshots, Restore Points

## Backups › Backup Jobs — `backups/jobs`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Last backup | 21 | days ago | {warn:Warning} | 30 Sep 21:12 | — | — |
| Tool | Déjà Dup |  | — | Ubuntu Backups | — | — |
| Destination | BACKUP-1TB |  | — | USB drive · not connected | — | — |
| Backup size | 96 | GB | — |   | — | — |
| Schedule | Weekly |  | — | when drive is connected | — | — |

2. **Bar chart** — "Backup history" · span 6
- Subtitle: "size per run"
- Height 200px · Y max auto · unit "GB"
- Bars (label: value · tooltip label): 26 Aug: 4.2 · "26 Aug · incremental"; 2 Sep: 3.1 · "2 Sep · incremental"; 9 Sep: 5.6 · "9 Sep · incremental"; 16 Sep: 2.8 · "16 Sep · incremental"; 23 Sep: 3.9 · "23 Sep · incremental"; 30 Sep: 4.4 · "30 Sep · incremental"; 7 Oct: 0 [kn] · "7 Oct · skipped (drive missing)"; 14 Oct: 0 [kn] · "14 Oct · skipped (drive missing)"; 21 Oct: 0 [kn] · "21 Oct · skipped (drive missing)"

3. **Data table** — "Runs" · span 6
- Source chip (Expert): `deja-dup`

| Date | Type | Size [n] | Duration | Result |
| --- | --- | --- | --- | --- |
| 21 Oct | Scheduled | — | — | {warn:Skipped · drive missing} |
| 30 Sep | Incremental | 4.4 GB | 9 m | {ok:Success} |
| 23 Sep | Incremental | 3.9 GB | 8 m | {ok:Success} |
| 16 Sep | Incremental | 2.8 GB | 6 m | {ok:Success} |
| 9 Sep | Full | 91 GB | 1 h 42 m | {ok:Success} |

## Backups › System Snapshots — `backups/snapshots`

1. **Alert banner** — "No system snapshots" · span 12
- State: `warn` · Title: "No system snapshots" · Text: "Timeshift is not installed. Snapshots let you roll back the system after a bad update." · Buttons: [**Install Timeshift**](act:toast:sudo apt install timeshift)

2. **Data table** — "Snapshots" · span 12
- Empty text: "No snapshots yet"

| Created | Type | Tool | Size [n] | Comment |
| --- | --- | --- | --- | --- |
| *(empty: "No snapshots yet")* |   |   |   |   |

## Backups › Restore Points — `backups/restore`

1. **Data table** — "DeeCure restore points" · span 12
- Info tooltip: "Saved automatically before every change DeeCure makes."

| Created | Change | Target | Size [n] | (action) |
| --- | --- | --- | --- | --- |
| Tue 11:05 | Disabled service | `cups-browsed.service` | 1 KB | [Restore](act:undo:cups) |
| 14 Oct | Removed old kernel | `linux-image-6.8.0-38` | — | Reinstall from APT |

---

← Previous: [Storage](14-storage.md) · [Index](README.md) · Next: [Network](16-network.md) →
