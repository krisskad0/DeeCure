# Fix definitions and state changes

[Docs](../../README.md) › [UI design](../README.md) › [Behavior](README.md)

## Fix definitions

**`fix:storage`: Free up disk space** (subtitle "Root filesystem ( / )")

| Step ID | Title | Description | Size | Admin | Undo | Checked by default | Command |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `snap` | Remove disabled snap revisions | 6 old revisions kept after updates | 8.7 GB | Yes | `none` | Yes | `snap remove firefox --revision=4793 # +5 more` |
| `docker` | Prune unused Docker images | 7 images not used by any container | 4.0 GB | No | `regen` | Yes | `docker image prune -a` |
| `trash` | Empty Trash | 312 files | 2.1 GB | No | `none` | Yes | `gio trash --empty` |
| `apt` | Clean APT package cache | Downloaded .deb files | 0.4 GB | Yes | `regen` | Yes | `sudo apt-get clean` |

**`fix:sec`: Install updates** (subtitle "APT · noble-security")

| Step ID | Title | Description | Size | Admin | Undo | Checked by default | Command |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `secu` | Install 4 security updates | openssl, libssl3t64, sudo, linux-firmware | - | Yes | `none` | Yes | `sudo apt-get install --only-upgrade openssl libssl3t64 sudo linux-firmware` |
| `other` | Install 8 regular updates | google-chrome-stable, git and 6 more | - | Yes | `none` | No | `sudo apt-get upgrade` |

**`fix:batt`: Configure low-battery action** (subtitle "UPower · /etc/UPower/UPower.conf")

| Step ID | Title | Description | Size | Admin | Undo | Checked by default | Command |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `up` | Warn at 10%, power off cleanly at 5% | Prevents unclean shutdowns like today at 02:14 | - | Yes | `full` | Yes | `sudo sed -i 's/^PercentageAction=.*/PercentageAction=5/' /etc/UPower/UPower.conf` |

Undo badge mapping: `full` → `{ok:Reversible}`, `regen` → `{info:Regenerates}`, `none` → `{warn:Irreversible}`. Admin steps also show `{off:Admin}`.

## What changes after each action

| Action | State change | Visible effects |
| --- | --- | --- |
| Free up disk space (chosen steps) | `cleaned[step]=true`; `used -= step sizes` | Disk %, free GB, reclaimable, "full in" days, donut, allocation bar, forecast and mount meters update. With all steps: 94% → 88%, the Storage issue becomes Warning "Root filesystem above 85%" (action View → Storage › Disk Usage), score 82 → 88, the Storage page turns Warning. Cleaned rows leave Storage › Cleanup. If the Docker step ran: unused images leave Containers › Images and the Prune button is hidden. If the snap step ran: Packages › Cleanup shows 0 snap revisions. Adds a DeeCure log entry (not reversible). |
| Install updates | `secFixed=true` | Security issue removed; score +3. Updates: 12 → 8 pending, security 0, the security rows leave the pending table, the update history gains a "DeeCure" row. Security › Overview shows all checks passing (donut 7/10) and an empty Vulnerabilities table. Security page Healthy; Packages page Info. Adds a DeeCure log entry. |
| Configure low-battery action | `battFixed=true`, `battTime=now` | Issue "Low-battery action not set" removed. Power › Power Profile shows `{ok:Power off at 5%}`. Activity › Config Changes gains an UPower.conf row. Adds a DeeCure log entry **with Undo** (also listed in Backups › Restore Points). |
| Restart Bluetooth | `btRestarted=true`, `btTime=now` | The service row becomes `{ok:active}` running 6 MB. The crash-loop alert is hidden. KPIs show Running 143, Failed 0 `ok`. The issue text becomes "Restarted at HH:MM · monitoring". The Services tab count is hidden and the Services page turns Warning. Toast "bluetooth.service restarted · monitoring for new crashes." |
| Undo cups-browsed | `cupsUndone=true` | The cups-browsed row becomes active/running/enabled, 5 MB. Restore points show "Restored"; DeeCure Actions shows "Undone". Toast "Undone · cups-browsed.service enabled and started." |
| Undo low-battery change | `battFixed=false` | Reverts the battery change and marks the log entry undone. Toast "Undone · UPower.conf restored from the restore point." |
| Track cron job | `tracked[id]=true` | The cron result becomes `{info:Tracking}`. Toast "Job wrapped · results are recorded from the next run." |
| Access = Basic | `access='basic'` | Admin-only values show `{off:Requires Full access}` or `{off:Locked}`; admin-only tables are empty with "Requires Full access". Toast "Basic access · admin-only checks are locked." (Full: "Full access · admin helper authorized.") |
| Expert mode | `expert=true` | Panel source chips appear; columns marked `x` appear. |
| Time range | `range` | All generated series and sparklines regenerate with that range's point count and labels. |
