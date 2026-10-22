# Settings (`settings`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Footer · Icon: `sliders` · Status badge: none
- Sub-pages: General, Access, Notifications, Assistant, Data, About

## Settings › General — `settings/general`

1. **Settings form (field rows, [Form controls](../03-components/21-form-controls.md))** — "General" · span 8

| Label | Description | Control | Initial | Control ID |
| --- | --- | --- | --- | --- |
| Expert mode | Show data sources, commands and raw columns | Switch | Off | `set-expert` |
| Default time range |  | Select: 1h / 24h / 7d / 30d | 24h | `set-range` |
| Refresh interval | Live metrics | Select: 2 s / 5 s / 15 s / Manual | 5 s | `set-refresh` |
| Temperature unit |  | Select: Celsius / Fahrenheit | Celsius | `set-temp` |
| Start at login | Collector runs in the background | Switch | On | `set-start` |

## Settings › Access — `settings/access`

1. **Radio cards ([Form controls](../03-components/21-form-controls.md))** — "Access level" · span 8

| Option | Description | Checked |
| --- | --- | --- |
| Basic | No password · about 80% of checks | No |
| Full | Admin helper approved once · all checks | Yes |

Radio group name: `access`.

2. **Key-value list** — "Admin helper" · span 4

| Key | Value |
| --- | --- |
| Service | `deecure-helper` |
| Status | {ok:Authorized} |
| Activation | D-Bus, on demand |
| Allowed actions | 24 (fixed list) |
| Audit log | `/var/log/deecure/helper.log` |

## Settings › Notifications — `settings/notifications`

1. **Settings form (field rows, [Form controls](../03-components/21-form-controls.md))** — "Notifications" · span 8

| Label | Description | Control | Initial | Control ID |
| --- | --- | --- | --- | --- |
| Critical issues | Disk almost full, SSD failing, overheating | Switch | On | `n1` |
| Warnings | Included in the weekly summary when off | Switch | Off | `n2` |
| Weekly summary | Sunday 18:00 | Switch | On | `n3` |
| Quiet hours |  | Select: Off / 22:00–07:00 / 23:00–08:00 | 22:00–07:00 | `n4` |

## Settings › Assistant — `settings/assistant`

1. **Settings form (field rows, [Form controls](../03-components/21-form-controls.md))** — "Assistant" · span 8

| Label | Description | Control | Initial | Control ID |
| --- | --- | --- | --- | --- |
| Enable assistant | Runs a local model on this computer | Switch | On | `set-ai` |
| Model |  | Select: Local · 3B (fast) / Local · 8B (better answers) | Local · 3B (fast) | `set-model` |
| Cloud fallback | Removes names, hostnames, IPs and serials before sending | Switch | Off | `set-cloud` |

## Settings › Data — `settings/data`

1. **Settings form (field rows, [Form controls](../03-components/21-form-controls.md))** — "Data retention" · span 8

| Label | Description | Control | Initial | Control ID |
| --- | --- | --- | --- | --- |
| Keep history for | Metrics and change history | Select: 90 days / 1 year / 2 years | 2 years | `ret` |
| Database size | `~/.local/share/deecure/deecure.db` | Read-only value | 74 MB | `—` |
| Export report | HTML or JSON snapshot of all pages | [Export](act:toast:Report export is disabled in the prototype.) | — | `—` |
| Delete history | Restore points are kept | [Delete…](act:toast:Prototype: nothing was deleted.) | — | `—` |

## Settings › About — `settings/about`

1. **Key-value list** — "About" · span 6

| Key | Value |
| --- | --- |
| Version | DeeCure 0.5 · prototype |
| Collector | deecure-agent 0.5 · running |
| Knowledge base | 1,240 entries · updated today |
| Distro adapter | Ubuntu / Debian |
| License | Apache-2.0 |

---

← Previous: [Assistant](21-assistant.md) · [Index](README.md)
