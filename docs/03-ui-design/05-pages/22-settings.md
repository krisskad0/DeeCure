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

What is built (Phase 3): **Tray icon** (off by default). When on, closing the window keeps DeeCure in the tray (menu: Open DeeCure, Quit DeeCure), and a login entry (`~/.config/autostart/io.github.krisskad0.DeeCure.desktop`, `deecure-desktop --hidden`) starts it there. It can only be turned on when the desktop has a tray area (`org.kde.StatusNotifierWatcher`; on Ubuntu the AppIndicator extension), so the window can never be hidden with no way back. Only one copy of the app runs: starting it again shows the running window. The setting lives in `~/.config/deecure/desktop.json`; DeeCure removes only a login entry it wrote itself.

## Settings › Access — `settings/access`

1. **Radio cards ([Form controls](../03-components/21-form-controls.md))** — "Access level" · span 8

| Option | Description | Checked |
| --- | --- | --- |
| Basic | No password · about 80% of checks | No |
| Full | Admin helper approved once · all checks | Yes |

Radio group name: `access`.

What is built: the choice is saved in the app (`access` preference, default Basic) and is the same one as in onboarding. **Basic:** DeeCure never asks for the password; admin read-outs say how to turn on Full, and admin fix steps are locked (other steps still run). **Full:** admin read-outs and admin steps can be used, and polkit asks for the password each time. Full can be picked only when the admin helper is installed; otherwise its card is greyed out and says why.

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

What is built (Phase 3): the choices are saved by the agent, which sends the notifications ([Notifications](../../02-architecture/notifications.md)). A **Send a test notification** button shows one now, or says why it cannot, and a **Recently shown** panel lists the last 8.

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

What is built: **Keep history for** is a real choice saved by the agent; a shorter time deletes older history at once. **Delete…** asks "Delete all history? This cannot be undone." before anything happens, then says how much space came back. Restore points and settings are kept ([Storage](../../02-architecture/storage.md)).

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
