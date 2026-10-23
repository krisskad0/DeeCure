# Accounts (`accounts`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Network & Access · Icon: `users` · Status badge: {ok:Healthy} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Users, Groups, Sessions, Login History, SSH Keys

## Accounts › Users: `accounts/users`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| Human users | 2 |  | - |   | - | - |
| Administrators | 1 |  | - |   | - | - |
| System accounts | 41 |  | - |   | - | - |
| Locked | 0 |  | - |   | - | - |
| Without password | 0 |  | - | Verified | - | - |

2. **Data table**: "Users" · span 12
- Source chip (Expert): `getent passwd · chage -l`
- Inspect: row click opens the row-inspect drawer with actions "Lock account", "Reset password"

| User | UID [n] | Groups | Role | Shell | Last sign-in | Password age |
| --- | --- | --- | --- | --- | --- | --- |
| **riya** · Riya | 1000 | ‹sudo›‹adm›‹docker›‹lxd› | {info:Administrator} | `/bin/bash` | Today 08:59 | 14 months |
| **guest** · Guest | 1001 | ‹guest› | {off:Standard} | `/bin/bash` | 3 weeks ago | 9 months |

3. **Alert banner**: "Password strength cannot be checked" · span 12
- State: `info` · Title: "Password strength cannot be checked" · Text: "Linux stores only salted hashes. DeeCure shows password age instead."

## Accounts › Groups: `accounts/groups`

1. **Data table**: "Groups with members" · span 12
- Source chip (Expert): `getent group`
- Search: "Filter groups"

| Group | GID [n] | Members | Privilege |
| --- | --- | --- | --- |
| sudo | 27 | riya | {warn:Admin} |
| adm | 4 | riya, syslog | {info:Read logs} |
| docker | 999 | riya | {warn:Root-equivalent} |
| lxd | 135 | riya | {info:Containers} |
| plugdev | 46 | riya | - |
| lpadmin | 114 | riya | Printers |
| sambashare | 117 | riya | - |

## Accounts › Sessions: `accounts/sessions`

1. **Data table**: "Active sessions" · span 12
- Source chip (Expert): `loginctl list-sessions`

| Session | User | Seat | Type | Since | State |
| --- | --- | --- | --- | --- | --- |
| `2` | riya | seat0 | wayland | Today 08:59 | {ok:active} |
| `c1` | gdm | seat0 | wayland | Today 08:58 | {off:online} |

## Accounts › Login History: `accounts/logins`

1. **Bar chart**: "Sign-ins per day" · span 5
- Height 180px · Y max auto
- Bars (label: value): Thu: 2; Fri: 3; Sat: 1; Sun: 1; Mon: 3; Tue: 2; Wed: 2

2. **Data table**: "Events" · span 7
- Source chip (Expert): `last · journalctl _COMM=sudo`

| Time | User | Event | Source | Result |
| --- | --- | --- | --- | --- |
| Today 21:12 | riya | sudo | `apt update` | {ok:Success} |
| Today 08:59 | riya | Sign-in | gdm-password | {ok:Success} |
| Tue 18:39 | riya | sudo | `apt upgrade` | {ok:Success} |
| Mon 08:42 | riya | Sign-in | gdm-password | {ok:Success} |
| Sat 10:13 | riya | Sign-in | gdm-password | {warn:Failed ×1} |
| 30 Sep | guest | Sign-in | gdm-password | {ok:Success} |

## Accounts › SSH Keys: `accounts/ssh`

1. **KPI row** · span 12

| Label | Value | Unit | Status badge | Subtext | Visual | Opens |
| --- | --- | --- | --- | --- | --- | --- |
| OpenSSH server | Not installed |  | - |   | - | - |
| Authorized keys | 0 |  | - |   | - | - |
| Client keys | 1 |  | - | ed25519 | - | - |

2. **Data table**: "Client keys" · span 12
- Subtitle: "~/.ssh"

| File | Type | Fingerprint | Created |
| --- | --- | --- | --- |
| `id_ed25519` | ED25519 | `SHA256:k3F…Qw8` | 12 Aug 2025 |

---

← Previous: [Network](16-network.md) · [Index](README.md) · Next: [Security](18-security.md) →
