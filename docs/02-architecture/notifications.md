# Notifications

[Docs](../README.md) › [Architecture](README.md)

- **Critical** findings notify right away: disk almost full, SSD failing, overheating, and similar.
- **Warnings** wait for the weekly summary.
- Quiet hours are supported, and the same alert never repeats within 24 hours.

## How it works

The code is in `crates/deecure-agent/src/notify.rs`. The agent sends notifications, so they come even when the app is closed.

Every minute the agent looks at the open findings and decides what to show:

| Rule | Detail |
| --- | --- |
| Critical | Shown right away (setting **Critical issues**, on by default). |
| Warning | Shown only when **Warnings** is on (off by default). Otherwise it is in the weekly summary. |
| Info | Never shown. |
| No repeats | One alert is a rule, a subject, and a severity, such as `storage.root_almost_full|/|critical`. It is not shown again within 24 hours, even after a restart: the `notification` table remembers. A finding that closes and opens again within 24 hours stays quiet. A warning that becomes critical is a new alert. A critical problem still open after 24 hours is shown once more, as a reminder. |
| Quiet hours | Off, 22:00–07:00 (default), or 23:00–08:00, local time. Nothing is shown then. Alerts are not dropped: when the quiet hours end, whatever is still open is shown. |
| Bursts | More than 3 alerts at once become one: "5 problems need your attention", with the first 3 titles. Each is remembered, so none repeats. |
| Text | The finding's title, and its explanation cut at a sentence end (220 characters at most). |
| Clock changes | A record up to an hour in the future still counts as recent. A record further ahead (the clock was set back a lot) does not block alerts. |

### Delivery

- Through the desktop's notification service, `org.freedesktop.Notifications` on the session bus (GNOME Shell on Ubuntu). Critical alerts are marked urgent.
- The hint `desktop-entry` is the app ID (`deecure_core::APP_ID`), so the desktop shows DeeCure's icon and opens the app when the notification is clicked.
- DeeCure asks whether the service is running first (`NameHasOwner`), so it never starts one. When none runs (a server without a desktop, or right after login while the desktop starts), nothing is recorded and the next minute tries again. The failure is logged once.

### Settings

Kept by the agent (setting `notify`), edited in Settings › Notifications, and checked when saved (unknown values are refused).

| Field | Default | Meaning |
| --- | --- | --- |
| `critical` | `true` | Critical problems right away |
| `warnings` | `false` | Warnings right away too |
| `weekly` | `true` | The weekly summary |
| `quiet_hours` | `"22-07"` | `"off"`, `"22-07"`, or `"23-08"` |

Agent calls: `notify_settings`, `set_notify_settings`, `notify_test` (shows a test notification, or says why it cannot), and `notifications` (the last 30 shown) ([IPC protocol](ipc-protocol.md)).

### Tested

- Unit tests for every rule above, including "no alert repeats within 24 hours".
- The real D-Bus call, against a fake notification server on a private session bus (`dbus-run-session`), so the test never touches the desktop you work on.
