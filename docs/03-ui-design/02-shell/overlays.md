# Overlays

[Docs](../../README.md) › [UI design](../README.md) › [Shell](README.md)

| Overlay | Trigger | Spec |
| --- | --- | --- |
| Scrim | Behind drawers and the palette | Fixed full screen, `--scrim`, z-index 40. Clicking it closes (except while a fix is running). |
| Drawer | Fix actions, Inspect, row click | Fixed right, full height, width min(480px, 100%), `--panel`, 1px left border, shadow, z-index 41. Header (padding 14 16, bottom border): title 16px 700 (wraps), subtitle 12.5px muted, close icon button on the right. Body scrolls (padding 14 16, gap 14). Footer (`--panel-2`, top border, padding 12 16): note text on the left (12px muted), buttons on the right. |
| Popover | Bell, machine picker | Fixed under the trigger (right-aligned for the bell, left-aligned for the machine picker), width min(360px, 100vw − 24px), radius 9, shadow, z-index 30. A transparent full-screen layer behind it closes it on click. |
| Command palette | Search trigger, Ctrl/Cmd + K | Fixed, top 12vh, centered, width min(560px, 100% − 32px), radius 12, shadow, z-index 45, over the scrim. Input row 48px with a search icon, 15px input and an `Esc` kbd. Results list max-height min(380px, 60vh). |
| Toast | Many actions | Fixed bottom center (20px + safe area), `--ink` background, `--panel` text, 13px, radius 8, padding 9 14, z-index 60. Auto-hides after 3.2 s. One at a time. |

**Notifications popover:** header "Notifications" + muted "N open". The first 6 issues: a colored dot by severity, bold title, small "Category · First seen". Clicking an issue runs its action. Footer link "View all issues" opens Overview › Issues.

**Machines popover:** header "Machines". Item 1: green dot, **riya-laptop**, "This computer · Ubuntu 24.04.1 LTS" (closes). Item 2: external-link icon, **Add a machine**, "Planned · small office view" → toast "Multi-machine view is planned for a later phase."

**Command palette results** (case-insensitive substring match, max 40, right-aligned kind label):

- Every page and sub-page as `Page › Sub-page` (or just `Page` when it has one sub-page), kind "Page". Beginner mode lists only its own pages and no services; with no matches it says "No matches. Advanced shows every page and service." ([Beginner mode](sidebar.md#beginner-mode)).
- Services: bluetooth.service, docker.service, NetworkManager.service, snapd.service, cups-browsed.service → Services › System Services.
- Packages: the first 14 package names from Packages › Installed → Packages › Installed.
- Jobs: sync-notes.sh, backup-photos.sh → Scheduled Jobs › Cron; logrotate.timer, e2scrub_all.timer → Scheduled Jobs › Systemd Timers.
- Keyboard: ↑/↓ moves the highlight (wraps, scrolls into view), Enter opens, Esc closes. The highlighted row has an `--accent-soft` background.
