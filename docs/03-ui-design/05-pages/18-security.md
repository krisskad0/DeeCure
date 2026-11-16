# Security (`security`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Network & Security · Icon: `shield` · Status badge: {warn:Warning} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Overview, Vulnerabilities, Hardening, Firewall, File Integrity, Malware Scan

## Security › Overview: `security/overview`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Security updates | 4 |  | {warn:Warning} |   | - | `packages/updates` |
| Known vulnerabilities | 3 |  | {warn:Warning} | 2 high · 1 medium | - | - |
| Firewall | Active |  | - |   | - | - |
| AppArmor | 112 | profiles | - | enforcing | - | - |
| Secure Boot | On |  | - |   | - | - |
| Disk encryption | Off |  | {info:Info} |   | - | - |

2. **Checklist**: "Security posture" · span 8

| State | Title | Detail | Action |
| --- | --- | --- | --- |
| `fail` | Security updates installed | 4 pending | [**Install**](act:fix:sec) |
| `pass` | Firewall enabled | ufw · default deny incoming |   |
| `pass` | Secure Boot enabled |   |   |
| `pass` | CPU vulnerability mitigations active | Spectre v1/v2, Retbleed, MDS |   |
| `pass` | AppArmor enforcing |   |   |
| `pass` | Screen locks after 5 minutes |   |   |
| `pass` | No remote login (SSH) exposed |   |   |
| `warn` | Docker port 5432 exposed to LAN | Bypasses ufw | [View](go:network/ports) |
| `warn` | Disk encryption off | Requires reinstall to enable |   |
| `na` | Malware scanner | Not installed | [View](go:security/malware) |

3. **Donut chart**: "Checks" · span 4
- Center: "6/10" / "passing"

| Segment | Value | Color | Display value |
| --- | --- | --- | --- |
| Pass | 6 | k3 | - |
| Warning | 2 | kw | - |
| Fail | 1 | kc | - |
| Not applicable | 1 | kn | - |

Expert: "AppArmor by process" is one bar of running processes by how much AppArmor limits them (enforced, warn only, allows everything, no profile), then ranked bars of the profiles with the most processes. The table is under "Show all".

## Security › Vulnerabilities: `security/vulns`

Tab count pill: 3.

1. **Data table**: "Known vulnerabilities" · span 12
- Subtitle: "sample data"
- Source chip (Expert): `pro fix · OVAL`
- Empty text: "No known vulnerabilities in installed packages"

| CVE | Package | Severity | Installed | Fixed in | Action |
| --- | --- | --- | --- | --- | --- |
| `CVE-2026-10421` | openssl | {crit:High} | `3.0.13-0ubuntu3.4` | `3.0.13-0ubuntu3.5` | [**Fix**](act:fix:sec) |
| `CVE-2026-10388` | sudo | {crit:High} | `1.9.15p5-3ubuntu5.1` | `1.9.15p5-3ubuntu5.2` | [**Fix**](act:fix:sec) |
| `CVE-2026-09917` | linux-firmware | {warn:Medium} | `…0ubuntu2.5` | `…0ubuntu2.6` | [Fix](act:fix:sec) |

## Security › Hardening: `security/hardening`

1. **Data table**: "Hardening checks" · span 12

| Check | Result | Value | Recommendation |
| --- | --- | --- | --- |
| Kernel ASLR | {ok:Pass} | `kernel.randomize_va_space = 2` | - |
| Unprivileged user namespaces | {ok:Pass} | `restricted by AppArmor` | - |
| kptr_restrict | {ok:Pass} | `1` | - |
| dmesg_restrict | {ok:Pass} | `1` | - |
| Home directory permissions | {ok:Pass} | `drwxr-x---` | - |
| Automatic security updates | {ok:Pass} | `Unattended-Upgrade "1"` | - |
| Docker group membership | {warn:Review} | riya | docker group equals root access |
| Disk encryption | {warn:Fail} | None | Enable LUKS at next reinstall |

The panel shows a pass bar, one green line when every check passed (else the checks not passed, with the value now and the advice), and the table under "Show all".

## Security › Firewall: `security/firewall`

It was Network › Firewall until 2026-11-16. Security › Overview checks "Firewall enabled", so the check and the fix sit together.

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Status | Active |  | - | ufw | - | - |
| Incoming | Deny |  | - | default | - | - |
| Outgoing | Allow |  | - | default | - | - |
| Rules | 2 |  | - |   | - | - |
| Blocked today | 37 |  | - | packets | - | - |

2. **Data table**: "Rules" · span 12
- Source chip (Expert): `sudo ufw status numbered`
- Empty text: "Requires Full access"

| # [n] | To | Action | From | Comment |
| --- | --- | --- | --- | --- |
| 1 | `5353/udp` | {ok:ALLOW IN} | Anywhere | mDNS |
| 2 | `1714:1764/tcp` | {ok:ALLOW IN} | `192.168.1.0/24` | GSConnect |

3. **Alert banner**: "Docker rules are not managed by ufw" · span 12
- State: `warn` · Title: "Docker rules are not managed by ufw" · Text: "Port 5432 (pg-dev) is reachable from the LAN even though ufw denies incoming traffic." · Buttons: [View container](go:containers/containers)

## Security › File Integrity: `security/integrity`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Files verified | 284,112 |  | - |   | - | - |
| Modified | 0 |  | - |   | - | - |
| Missing | 0 |  | - |   | - | - |
| Last scan | Today |  | - | 03:00 | - | - |

2. **Data table**: "Programs with SUID bit" · span 12
- Info tooltip: "These programs run with their owner’s rights (usually root). All match their packages."
- Source chip (Expert): `find / -perm -4000`

| Path | Owner | Package | Verified |
| --- | --- | --- | --- |
| `/usr/bin/sudo` | root | sudo | ✓ |
| `/usr/bin/passwd` | root | passwd | ✓ |
| `/usr/bin/mount` | root | mount | ✓ |
| `/usr/bin/pkexec` | root | pkexec | ✓ |
| `/usr/lib/openssh/ssh-keysign` | root | openssh-client | ✓ |
| `/usr/bin/fusermount3` | root | fuse3 | ✓ |

## Security › Malware Scan: `security/malware`

1. **Alert banner**: "No malware scanner installed" · span 12
- State: `info` · Title: "No malware scanner installed" · Text: "Optional on Ubuntu. Signature scans reduce risk but cannot prove a system is clean." · Buttons: [**Install ClamAV**](act:toast:sudo apt install clamav)

2. **Checklist**: "Heuristic checks" · span 12

| State | Title | Detail | Action |
| --- | --- | --- | --- |
| `pass` | No startup items running from /tmp or ~/Downloads |   |   |
| `pass` | No cron jobs that download and execute scripts |   |   |
| `pass` | No unknown kernel modules |   |   |
| `pass` | No hidden processes |   |   |

---

← Previous: [Accounts](17-accounts.md) · [Index](README.md) · Next: [Privacy](19-privacy.md) →
