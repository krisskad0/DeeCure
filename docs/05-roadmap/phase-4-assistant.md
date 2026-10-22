# Phase 4 — Ask SysDoc

[Docs](../README.md) › [Roadmap](README.md)

**Goal:** users ask questions in plain words. A local AI answers using real data and shows its sources.
**Rough time:** 4–6 weeks.

## Tasks (in order)

1. **Ollama integration:** find a running Ollama, guide the user to install it if needed, pick a model (3B fast or 8B better).
   → [AI assistant](../02-architecture/ai-assistant.md)
2. **Read-only tool API** for the AI: `get_findings`, `get_events(since)`, `get_boot_list`, `get_logs(filter)`.
3. **Grounded answers:** every answer shows the data or log lines it used. Answers not backed by the KB carry the label "AI explanation — may be wrong".
4. **Chat UI** and Settings › Assistant.
   → [Chat](../03-ui-design/03-components/24-chat.md), [Assistant page](../03-ui-design/05-pages/21-assistant.md), [Settings page](../03-ui-design/05-pages/22-settings.md)
5. **Fix suggestions:** the AI can only suggest a known fix ID. It then goes through the normal preview and approval.
   → [Fix engine](../02-architecture/fix-engine.md)
6. **Cloud fallback (opt-in):** remove user names, host names, IPs, and serial numbers before sending.
7. **Question test set:** at least 30 real questions with expected answers, such as "Why did my laptop restart last night?"

## Done when

- [ ] The journey "Ask SysDoc" works fully offline.
  → [User journey](../01-product/user-journey.md#ask-sysdoc-assistant--chat)
- [ ] A test proves the AI cannot run any command or call the helper.
- [ ] The question test set passes at the agreed rate.

---

← Previous: [Phase 3](phase-3-history.md) · [Index](README.md) · Next: [Phase 5 — Reach](phase-5-reach.md) →
