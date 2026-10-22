# Storage (`storage`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Software & Data · Icon: `db` · Status badge: {crit:Critical} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Overview, Filesystems, Disk Usage, Cleanup
- Page header actions (all sub-pages without their own): [**Free up space**](act:fix:storage) (shown while reclaimable space > 0)

## Storage › Overview — `storage/overview`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Used | 224.2 | GB | {crit:Critical} | 94% of 238 GB | Usage bar 94.2% | — |
| Free | 13.8 | GB | — |   | — | — |
| Reclaimable | 15.2 | GB | — | Safe to clean | — | — |
| Full in | ~8 | days | {crit:Critical} | at 0.69% per day | — | — |
| Inodes used | 21 | % | — |   | — | — |
| Encryption | Off |  | — | LUKS not used | — | — |

2. **Alert banner** — "Root filesystem is 94% full" · span 12
- State: `crit` · Title: "Root filesystem is 94% full" · Text: "15.2 GB can be reclaimed safely. Personal files are not touched." · Buttons: [**Free up space**](act:fix:storage)

3. **Allocation bar ([Allocation bar (disk)](../03-components/18-allocation-bar.md))** — "Space allocation" · span 12
- Subtitle: "/dev/nvme0n1p2 · ext4"
- Source chip (Expert): `du -xh --max-depth=1`

Segments (left to right):

| Segment | GB | Color |
| --- | --- | --- |
| Home | 128.0 | --s1 |
| Apps | 46.0 | --s2 |
| System | 22.0 | --s3 |
| Containers | 13.0 | --s4 |
| Reclaimable | 15.2 | crit stripes |
| Free | 13.8 | --track |

Width of each segment = GB / 238. The legend repeats each name with its bold GB value.

4. **Line chart** — "Usage trend and forecast" · span 8
- Subtitle: "30 days"
- Info tooltip: "The dashed line shows when the disk fills at the current rate."
- Height 220px · Y axis 60 to 100 · unit "%"
- Series: Used (slot 1, data [73.7,74.3,75.7,75.6,77.1,78,78,78.6,79.8,80.7,81.4,81.8,82.8,82.9,83.3,84.7,85.2,85.3,86.8,87.5,87.5,88.2,89.3,90,90.7,91.4,92.1,92.5,93.7,94.2])
- X labels: 30 labels: 22 Sept, 23 Sept, 24 Sept … 20 Oct, 21 Oct
- Threshold line at 85: "Warning 85%"
- Forecast: +8 points to 100, label "Full ~29 Oct" (recomputed from state: daysFull)

5. **Data table** — "Mount points" · span 4
- Source chip (Expert): `findmnt`

| Mount | Size [n] | Used |
| --- | --- | --- |
| `/` | 238 GB | [meter:crit 94%] |
| `/boot/efi` | 1 GB | [meter 6%] |
| `/run` | 780 MB | [meter 1%] |
| `/dev/shm` | 3.8 GB | [meter 2%] |

## Storage › Filesystems — `storage/filesystems`

1. **Data table** — "Filesystems" · span 12
- Source chip (Expert): `df -hT · df -i`

| Device | Mount | Type | Size [n] | Used [n] | Available [n] | Use | Inodes [n, ,x] |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `/dev/nvme0n1p2` | `/` | ext4 | 238 GB | 224.2 GB | 13.8 GB | [meter:crit 94%] | 21% |
| `/dev/nvme0n1p1` | `/boot/efi` | vfat | 1.0 GB | 61 MB | 963 MB | [meter 6%] | — |
| `tmpfs` | `/run` | tmpfs | 780 MB | 2.3 MB | 778 MB | [meter 1%] | 1% |
| `tmpfs` | `/dev/shm` | tmpfs | 3.8 GB | 84 MB | 3.7 GB | [meter 2%] | 1% |
| `squashfs ×14` | `/snap/*` | squashfs | 3.9 GB | 3.9 GB | 0 | [meter:ok 100%] | 100% |

2. **Checklist** — "Filesystem checks" · span 12

| State | Title | Detail | Action |
| --- | --- | --- | --- |
| `pass` | fstab entries valid | findmnt --verify: 0 errors |   |
| `pass` | No ext4 errors in kernel log |   |   |
| `pass` | Reserved blocks 5% (11.9 GB) for root |   |   |
| `warn` | Disk encryption off | Files are readable if the laptop is stolen · needs reinstall to enable |   |
| `na` | Full filesystem check (fsck) | Runs at boot when needed · last check 34 days ago |   |

## Storage › Disk Usage — `storage/usage`

1. **Horizontal bar list** — "Largest folders" · span 6
- Subtitle: "home"
- Source chip (Expert): `du -h --max-depth=1 ~`

| Label | Value | Shown as |
| --- | --- | --- |
| `~/Videos` | 52 | 52 GB |
| `~/Documents` | 41 | 41 GB |
| `~/Downloads` | 22 | 22 GB |
| `~/.local/share` | 5.2 | 5.2 GB |
| `~/.cache` | 3.8 | 3.8 GB |
| `~/snap` | 2.4 | 2.4 GB |
| `~/Pictures` | 1.6 | 1.6 GB |

2. **Horizontal bar list** — "Largest system folders" · span 6
- Source chip (Expert): `du -xh --max-depth=1 /`

| Label | Value | Shown as |
| --- | --- | --- |
| `/usr` | 14 | 14 GB |
| `/var/lib/docker` | 13 | 13 GB |
| `/var/lib/snapd` | 12.6 | 12.6 GB |
| `/var/log` | 1.4 | 1.4 GB |
| `/opt` | 0.9 | 0.9 GB |
| `/boot` | 0.3 | 0.3 GB |

3. **Data table** — "Largest files" · span 12
- Source chip (Expert): `find ~ -size +500M`
- Search: "Filter files"

| File | Size [n] | Type | Modified |
| --- | --- | --- | --- |
| `~/Downloads/ubuntu-24.04.1-desktop-amd64.iso` | 5.8 GB | Disk image | 4 months ago |
| `~/Videos/screen-2026-09-12.mp4` | 4.1 GB | Video | 1 month ago |
| `~/VirtualBox VMs/win11-test/win11-test.vdi` | 2.9 GB | Virtual disk | 3 weeks ago |
| `~/Downloads/dataset-q3.zip` | 2.2 GB | Archive | 2 months ago |
| `~/Videos/meeting-recording-0918.mkv` | 1.8 GB | Video | 1 month ago |
| `~/Documents/thesis-backup.tar.gz` | 1.2 GB | Archive | 6 months ago |

## Storage › Cleanup — `storage/cleanup`

Header actions: [**Free 15.2 GB**](act:fix:storage) (hidden once done), then Refresh and Export.

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Reclaimable | 15.2 | GB | {warn:Warning} |   | — | — |
| Cleaned this month | 0.0 | GB | — |   | — | — |

2. **Data table** — "Cleanup candidates" · span 12
- Row selection · bulk button [Clean selected](act:fix:storage)
- Empty text: "Nothing to clean right now"
- Table ID: `cleanup`

| Item | Details | Size [n] | Undo | Password |
| --- | --- | --- | --- | --- |
| Remove disabled snap revisions | 6 old revisions kept after updates | 8.7 GB | {warn:Irreversible} | 🔒 |
| Prune unused Docker images | 7 images not used by any container | 4.0 GB | {info:Regenerates} | — |
| Empty Trash | 312 files | 2.1 GB | {warn:Irreversible} | — |
| Clean APT package cache | Downloaded .deb files | 0.4 GB | {info:Regenerates} | 🔒 |

---

← Previous: [Packages](13-packages.md) · [Index](README.md) · Next: [Backups](15-backups.md) →
