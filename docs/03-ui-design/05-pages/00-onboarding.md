# Onboarding and first scan

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

Shown once, the first time the app opens (before the shell). It can be shown again from Settings › General. Follows the [user journey](../../01-product/user-journey.md#first-launch-4-short-screens).

## Layout

- Full window, `--bg` background, one centered card: width min(560px, 100% − 32px), `--panel`, radius 12, padding 28 32, shadow.
- Top of the card: the app icon (48px) and a row of 5 step dots (8px; the current one `--accent`, done ones `--good`).
- Bottom of the card: "Back" (link style, hidden on step 1) on the left; the primary button on the right.
- Keyboard: Enter presses the primary button; Esc does nothing (onboarding is short and has no skip trap: "Skip" is a link on steps 1-4; skipping keeps Beginner mode).

## Steps

1. **Welcome**: title "Welcome to DeeCure". Text: "DeeCure looks at your whole computer: hardware, system, apps, storage, network, and security: and explains what it finds in plain words. Problems come first, with what to do about them." Button: **Next**.
2. **Your privacy**: title "Everything stays here". Three rows with icons: `lock` "No account and no sign-in." · `shield` "Nothing is sent anywhere. DeeCure works offline." · `eye` "It only reads. Changes always need your OK, with a preview and undo." Button: **Next**.
3. **Mode**: title "Are you new to Linux?". Radio cards: **Yes, keep it simple** (checked) "Beginner mode · the pages most people need, in plain words"; **No, show me everything** "Expert mode · every page and tab, data sources, commands, and raw columns". Note: "You can switch any time with Beginner / Expert at the top." Choosing saves the `mode` pref at once. Button: **Next**.
4. **Access level**: title "How much should DeeCure see?". Radio cards: **Basic** (checked) "No password · everything your account can read"; **Full** "Approve a small admin helper once · disk health, firewall rules, all users' jobs, and fixes" with an `{off:Coming soon}` badge, not selectable until the helper exists. Note: "You can change this later in Settings › Access." Button: **Start the first check**.
5. **First check**: title "Checking your computer…". A list of 7 areas, each a real request to the agent: Hardware, System, Storage, Services, Apps, Network, Security. Each row shows a spinner while its request runs, then a green check and a one-line result (for example "Dell Latitude 3450 · 12 threads", "2,150 packages"), or a grey "Not available" with the reason. Below the list: the health score as soon as the agent has one. The folder-size scan starts in the background. Button: **Open DeeCure** (enabled when every area is done).

If the agent is not running, step 5 shows the same "Agent not running" help as every page, with a **Try again** button.

## State

- Pref `onboarded` (true after "Open DeeCure" or "Skip"). Pref `mode` (`beginner` or `expert`), shared with the top bar and Settings › General. Settings › General has **Show the welcome screens again**.

---

[Index](README.md) · Next: [Overview](01-overview.md) →
