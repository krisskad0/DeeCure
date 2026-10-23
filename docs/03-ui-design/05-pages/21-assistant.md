# Assistant (`assistant`)

[Docs](../../README.md) › [UI design](../README.md) › [Pages](README.md)

> **In the app:** hidden until Phase 4, so the first version has no placeholder pages (`app/src/visible.ts`).

- Group: Tools · Icon: `chat` · Status badge: none
- Sub-pages: Chat

## Assistant › Chat: `assistant/chat`

1. **Chat ([Chat](../03-components/24-chat.md))**: "Ask DeeCure" · span 8
- Subtitle: "Local model · on-device"

Initial messages: user "Why did my laptop restart last night?", then the `restart` answer ([Assistant](../04-behavior/interactions.md#assistant)). Composer: input "Ask about this computer…" + primary Send. When the assistant is off, the panel body is instead a centered muted line "The assistant is turned off." + [**Turn on**](go:settings/assistant), and the subtitle reads "Off".

2. **Question list**: "Suggested questions" · span 4

Full-width rows (chat icon + text), each runs `ask:<key>`:

- Why did my laptop restart last night? → `ask:restart`
- Why is Bluetooth crashing? → `ask:bt`
- Is my SSD healthy? → `ask:ssd`
- What is filling my disk? → `ask:disk`

3. **Key-value list**: "Assistant" · span 4

| Key | Value |
| --- | --- |
| Model | Local · 3B parameters |
| Runtime | Ollama |
| Data access | Read-only |
| Can run commands | No · suggests fixes only |
| Cloud fallback | {off:Off} |

---

← Previous: [Diagnostics](20-diagnostics.md) · [Index](README.md) · Next: [Settings](22-settings.md) →
