# Interactions

[Docs](../../README.md) › [UI design](../README.md) › [Behavior](README.md)

## Navigation

- `go(page/tab)`: sets the page and sub-page, opens that page's sidebar tree, closes popovers and the mobile drawer, re-renders, and scrolls the main area to the top.
- Deep link: on load, `#page.tab` (for example `#storage.cleanup`) opens that sub-page.
- Re-render on state change keeps the scroll position.

## Actions (`act:` names used in the page map)

| Action | Behavior |
| --- | --- |
| `go:page/tab` | Navigate |
| `toast:Message` | Show a toast with that message (prototype stand-in for a real operation) |
| `fix:storage`, `fix:sec`, `fix:batt` | Open the fix drawer (10.3). If already done, show a toast instead: "Nothing left to clean." / "Security updates are already installed." / "Low-battery action is already configured." |
| `bt` | Open the Bluetooth service drawer (10.4) |
| `btrestart` | Restart Bluetooth ([What changes after each action](fixes.md#what-changes-after-each-action)) and close the drawer |
| `askgo:bt` | Close the drawer, open Assistant › Chat, ask the Bluetooth question |
| `ask:<key>` | Ask a suggested question (10.5) |
| `undo:cups`, `undo:batt` | Undo ([What changes after each action](fixes.md#what-changes-after-each-action)) |
| `track:sync`, `track:backup` | Track a cron job ([What changes after each action](fixes.md#what-changes-after-each-action)) |
| `diag:run`, `net:run` | Start the diagnostics runner ([Diagnostics runner](../03-components/23-diagnostics-runner.md)) |
| `range:X`, `mode:b`, `mode:e` | Time range; Beginner / Expert |
| `rail`, `navtoggle`, `pop:notes`, `pop:host`, `cmdk` | Collapse sidebar, mobile menu, popovers, command palette |
| `refresh` | Re-render; toast "Refreshed · 21:34" |
| `export` | Toast "Export is disabled in the prototype." |
| `close`, `close-go:page/tab` | Close overlays (and navigate) |

## Fix drawer flow

1. **Preview.** Title and subtitle from the fix definition. Section label "Steps". Each step is a checkbox row: bold title, muted description, badges (undo type, plus `{off:Admin}` for admin steps), command block, and the size in GB on the right. Steps marked "off by default" start unchecked. Storage cleanup already done is not listed. When opened from Storage › Cleanup with rows selected, only those rows start checked.
   - Footer note: "Password required once" if any chosen step is admin, "No password needed" if none, "Select at least one step" if nothing is chosen.
   - Buttons: Cancel, and primary Apply. Its label is "Free X.X GB" for storage, "Apply N steps" for more than one step, otherwise "Apply". Disabled with no steps chosen. The label and note update live as boxes change.
2. **Authenticate** (only if a chosen step needs admin). A centered card on `--panel-2`: 40px lock circle, bold "Authentication required", muted "deecure-helper wants to run N privileged action(s)", a fake password field (`••••••••`), small note "Simulated polkit prompt. In the real app Ubuntu shows this window and DeeCure never sees the password." Buttons: Back, primary Authenticate.
3. **Run.** An ordered list of chosen steps. Finished steps show a green check circle and their command; the current step a spinner; later steps a dot. One step every 700ms. Footer note "Stops on the first error". Scrim clicks and Esc are ignored.
4. **Done.** Apply the state change ([What changes after each action](fixes.md#what-changes-after-each-action)). Inline success alert: bold result title and "Logged in Activity › DeeCure Actions at HH:MM". A 2-column grid of result cells (12px muted label, 16px 600 value). An optional info note. Buttons: View log (→ Activity › DeeCure Actions), primary Done.
   - Storage result: "X.X GB freed"; cells Root filesystem `94% → 88%`, Health score `82 → 88`, Free space, Full in `~N days`. Note when still ≥85%: "Still above 85%. ~/Videos uses 52 GB; moving it to external storage brings / to about N%."
   - Updates result: "4 updates installed" (12 when both steps); cells Security updates pending `4 → 0`, Health score, Known CVEs `3 → 0`, Reboot `Required`. Note "Restart when convenient to load the updated libraries."
   - Battery result: "Low-battery action configured"; cells At 10% `Warning`, At 5% `Clean power-off`. Note "Reversible from Activity › DeeCure Actions or Backups › Restore Points."

## Bluetooth service drawer (`bt`)

- Title `bluetooth.service`, subtitle "Bluetooth service · crash loop".
- 2×2 result cells: Active state `failed` (or `active` after restart), Restarts today `14`, Exit status `11 (SIGSEGV)`, First crash `Today 09:12`.
- Section "Correlated change": info alert "Kernel upgraded 6.8.0-45 → 6.8.0-47" / "Tue 18:40 · crashes started at next boot".
- Section "Log excerpt": two log lines: `09:12:44 kernel: bluetoothd[1342]: segfault at 0 ip 000055d1c2a4 error 4` and `systemd: bluetooth.service: Main process exited, code=dumped, status=11/SEGV`.
- Section "Remediation", a bordered checklist with 3 rows:
  1. Refresh icon (pass style) · "Restart service" / `sudo systemctl restart bluetooth` · [**Restart**](act:btrestart)
  2. Terminal icon (na) · "Boot previous kernel once" / "Boot menu › Advanced options › 6.8.0-45" · [Guide](toast:Hold Shift at startup → Advanced options for Ubuntu → 6.8.0-45-generic.)
  3. Chat icon (na) · "Ask the assistant" / "Explain the crash in plain words" · [Ask](act:askgo:bt)
- Footer: [Crash reports](go:logs/crashes), Close.

## Assistant

Starts with one exchange already shown: the user question for `restart` and its answer. Suggested questions append a user bubble, then a waiting bubble, then the answer after 700ms. Free text in the composer matches keywords: restart / shut / power off / turned off → `restart`; bluetooth / earbud → `bt`; ssd / nvme / drive health → `ssd`; disk / space / full / storage → `disk`. Anything else gets the fallback: "This prototype answers the suggested questions only. The real assistant reads this computer's logs and settings to answer anything."

| Key | Question | Answer | Sources | Action button |
| --- | --- | --- | --- | --- |
| `restart` | Why did my laptop restart last night? | It did not restart. **The battery ran out at 02:14**, so the laptop powered off without a clean shutdown. You started it again at 08:58. No low-battery action is configured, so this can happen again. | `02:09:12 upowerd: battery BAT0 is critically low (2%)`<br>`02:13:58 upowerd: battery BAT0 at 1%, critical action not configured`<br>`-- boot ended without clean shutdown --` | [**Configure low-battery action**](act:fix:batt) |
| `bt` | Why is Bluetooth crashing? | **bluetoothd crashes with SIGSEGV each time the earbuds connect.** It started at 09:12 today, after yesterday’s kernel upgrade 6.8.0-45 → 6.8.0-47. Restart the service first. If it keeps crashing, boot once with 6.8.0-45 to confirm the regression. | `09:12:44 kernel: bluetoothd[1342]: segfault at 0 … error 4`<br>`systemd: bluetooth.service: Main process exited, status=11/SEGV`<br>`Tue 18:40 apt: upgrade linux-image-6.8.0-47-generic` | [**Inspect bluetooth.service**](act:bt) |
| `ssd` | Is my SSD healthy? | **Yes.** SMART passed, 88% life left, 0 media errors. At your write rate that is many years. | `SMART overall-health self-assessment: PASSED`<br>`Percentage Used: 12%`<br>`Media and Data Integrity Errors: 0` | — |
| `disk` | What is filling my disk? | **~/Videos (52 GB) and ~/Documents (41 GB)** are the largest folders. System leftovers worth 15.2 GB can also be removed safely. | `du -h --max-depth=1 ~`<br>`snap list --all \| grep disabled` | [**Free up space**](act:fix:storage) |

## Row-inspect drawer

Tables marked **Inspect** open a drawer when a row is clicked: title = the first cell's text, subtitle = the table title, body = a key-value list of every column header and that row's cell. Footer = the table's row actions (listed in the page map) + primary Close. The prototype stand-ins: "End process" → toast "SIGTERM sent to <name> (prototype)."; service actions → toast "<Action> requested for <unit> (prototype)."; other row actions run as listed.

## Keyboard

- Ctrl/Cmd + K opens the command palette.
- Esc closes the palette, popovers, the mobile sidebar and drawers (not while a fix is running).
- All controls are buttons or inputs, reachable by Tab, with a visible focus ring.

## Form changes

- Any switch or select not listed below shows the toast "Saved."
- Power profile radios → toast "Power profile set to <name>."
- Settings › Assistant "Enable assistant" → toggles the assistant (toast "Assistant enabled." / "Assistant disabled."); "Cloud fallback" → toast "Cloud fallback on · data redacted before sending." / "Cloud fallback off."
- Settings › General "Expert mode" toggles Expert mode; "Default time range" sets the range.

## Charts

- Charts redraw when the window or main area resizes (debounced 80–120ms) and after every render.
