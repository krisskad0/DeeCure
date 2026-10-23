# Using DeeCure

[Docs](../README.md) › [User guide](README.md)

Everything you need for everyday use, in short steps.

## 1. First launch

DeeCure shows three short screens:

1. **Welcome:** what DeeCure does.
2. **Privacy:** everything stays on this computer. No account. Nothing is sent anywhere.
3. **Access level:**
   - **Basic** (no password): DeeCure sees everything your account can read. That is most things.
   - **Full**: DeeCure can also read admin-only things (disk health, firewall rules) and run admin fixes. Your password is asked each time, in the normal Ubuntu box.

Then a first check runs. Each area turns green, yellow, or red as it finishes. You can change the access level later in **Settings › Access**.

## 2. The home screen

**Overview › Summary** shows:

- **Health score** (0–100): 90 or more is healthy, 75–89 is fair, below 75 needs attention. Each problem takes points off its area.
- **Subsystems:** all 13 areas (hardware, power, storage, network, security, and more), each with a number and a status. Click a row to open its page.
- **Active issues:** the problems, most serious first. Each has a button to fix or look.

Status words always come with a color and an icon: **Healthy** (green), **Warning** (yellow), **Critical** (red), **Info** (blue).

## 3. Fix a problem

1. Click **Fix** on a problem.
2. DeeCure shows each step: what it does, whether it can be undone, and (in Expert mode) the exact command.
3. Untick any step you do not want.
4. Click **Apply**. Steps marked **Admin** ask for your password (Full access only; in Basic access they are locked).
5. DeeCure checks again and tells you if the problem is gone.

Every fix is listed in **Activity › DeeCure Actions**, with **Undo** when it can be undone.

## 4. "Something broke after a change"

Open **Activity › Timeline**. It lists everything that changed in the last 7 days, newest first:

- packages installed, upgraded, or removed (apt, snap, Flatpak),
- configuration files changed (click **View** to see exactly which lines),
- USB devices plugged in and out,
- starts, unclean shutdowns, crashes, and failed services,
- fixes DeeCure made (with **Undo** right there).

If a new kernel arrived, DeeCure shows how to start once with the previous one, to check whether the new kernel is the cause.

## 5. "Why is my PC slow?"

Open **Diagnostics › Performance** and click **Run check**. In about 10 seconds DeeCure names the main cause (CPU, memory, disk, or heat) and the app behind it.

## 6. Notifications and the weekly summary

DeeCure tells you about serious problems, even when the window is closed:

- **Critical problems** appear right away, for example "Disk will be full in about 3 days".
- **Warnings** wait for the weekly summary, unless you turn them on.
- **Quiet hours** (22:00–07:00 by default): nothing appears then; it waits until the quiet hours end.
- The same alert never appears twice within 24 hours.
- **Weekly summary**, Sunday 18:00: your health score and how it moved, what changed, and one tip. Past summaries are on **Overview › Health Score**.

Change all of this in **Settings › Notifications**. **Send a test notification** checks that it works.

## 7. Settings worth knowing

| Where | Setting |
| --- | --- |
| Settings › General | **Expert mode** (data sources, commands, raw columns), default time range, **refresh interval** (2 s, 5 s, 15 s, or Manual), **temperature unit** (°C or °F), narrow sidebar, **tray icon** |
| Settings › Access | Basic or Full access |
| Settings › Notifications | Critical, warnings, weekly summary, quiet hours |
| Settings › Data | Keep history for 90 days, 1 year, or 2 years; database size; **Export report** (an HTML file in your Downloads folder); **Delete history** (restore points and settings are kept) |

**Tray icon** (off by default): when it is on, closing the window keeps DeeCure in the tray, and it starts there when you log in. It needs a tray area on your desktop (Ubuntu has one).

## 8. In the Terminal

The same information is available in the Terminal:

```bash
deecure status                     # health score and problems
deecure history --since 2d         # what changed in the last 2 days
deecure history --kind packages    # only package changes
deecure report                     # a 7-day summary
deecure report --html report.html  # the same, as a file to share
deecure fix --list                 # every fix
deecure fix storage.free_space     # preview a fix, then run it after you say yes
deecure history --fixes            # what DeeCure changed, with numbers
deecure undo 12                    # take fix number 12 back
```

Add `--json` to `status`, `history`, or `report` for scripts.

## 9. Privacy

- DeeCure reads your computer to explain it. Everything stays here: no account, no tracking, nothing sent.
- The history lives in `~/.local/share/deecure/deecure.db`, readable only by you.
- Small copies of text configuration files are kept to show what changed. They stay in the same private database.
- **Settings › Data › Delete history** removes the history at any time.

---

← Previous: [Install](install.md) · [Index](README.md)
