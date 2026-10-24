# Privacy (`privacy`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

- Group: Network & Access · Icon: `eye` · Status badge: {ok:Healthy} (rule in [Page status rules](../04-behavior/page-status.md))
- Sub-pages: Device Access, App Permissions, Privacy Settings

## Privacy › Device Access: `privacy/access`

1. **State tiles ([Form controls](../03-components/21-form-controls.md))**: "Right now" · span 12

| Icon | Title | State |
| --- | --- | --- |
| `mic` | Microphone | Not in use |
| `cam` | Camera | Not in use |
| `screen` | Screen sharing | Off |
| `pin` | Location | Off |

2. **Data table**: "Recent access" · span 12 · **not built yet:** it needs a stored history of camera and microphone use (PipeWire polled every minute), so the app shows only "Right now" until then, with no placeholder
- Subtitle: "7 days"
- Source chip (Expert): `PipeWire · xdg-desktop-portal`

| Time | Device | Application | Duration |
| --- | --- | --- | --- |
| Tue 16:00 | Camera | zoom-client | 48 min |
| Tue 16:00 | Microphone | zoom-client | 48 min |
| Mon 11:30 | Microphone | chrome | 12 min |
| Mon 11:30 | Screen share | chrome | 9 min |

## Privacy › App Permissions: `privacy/perms`

1. **Data table**: "Permissions by app" · span 12
- Source chip (Expert): `snap connections · flatpak permissions`

| App | Camera | Microphone | Location | Files | Network |
| --- | --- | --- | --- | --- | --- |
| zoom-client | ✓ | ✓ | - | ✓ | ✓ |
| chrome | ✓ | ✓ | - | ✓ | ✓ |
| telegram-desktop | ✓ | ✓ | - | ✓ | ✓ |
| firefox | ✓ | ✓ | ✓ | ✓ | ✓ |
| com.spotify.Client | - | - | - | - | ✓ |

## Privacy › Privacy Settings: `privacy/settings`

1. **Settings form (field rows, [Form controls](../03-components/21-form-controls.md))**: "Settings" · span 8

| Label | Description | Control | Initial | Control ID |
| --- | --- | --- | --- | --- |
| Location services | Apps can request your location | Switch | Off | `-` |
| Usage reporting to Ubuntu | ubuntu-report · popularity-contest | Switch | Off | `-` |
| Problem reporting | Send crash reports to Canonical | Switch | On | `-` |
| Recent files history | Remember files you opened | Switch | On | `-` |
| Remove old temporary files | After 30 days | Switch | On | `-` |

---

← Previous: [Security](18-security.md) · [Index](README.md) · Next: [Diagnostics](20-diagnostics.md) →
