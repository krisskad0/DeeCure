# Chat

[Docs](../../README.md) › [UI design](../README.md) › [Components](README.md)

- Message column: gap 12, padding 14, min-height 360, max-height 520, scrolls, and auto-scrolls to the bottom after render.
- User bubble: right-aligned, `--accent` background, `--accent-ink` text, padding 8 12, radius 12 12 3 12, max-width 78%.
- Assistant bubble: left-aligned, `--panel-2`, 1px border, radius 12 12 12 3, padding 10 12, max-width 92%, gap 8. Contains the answer paragraphs, an optional **Sources** block (dashed top border, 11px 700 uppercase muted label "Sources", log lines) and an optional small primary action button.
- Waiting bubble: assistant style with muted "Reading logs…", replaced after 700ms.
- Composer (top border, padding 10 14, gap 8): input 34px (radius 7, placeholder "Ask about this computer…") + primary "Send".

---

← Previous: [Diagnostics runner](23-diagnostics-runner.md) · [Index](README.md)
