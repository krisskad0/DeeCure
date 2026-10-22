# Diagnostics runner

[Docs](../../README.md) › [UI design](../README.md) › [Components](README.md)

- **Idle:** step grid (`repeat(auto-fit, minmax(170px, 1fr))`, gap 8). Each step: 1px border, radius 6, padding 8 10, a grey 8px dot, muted name. Below it, a primary button with a play icon.
- **Running:** a 6px progress bar (`--accent` fill, 0.35s width transition) above the step grid. Finished steps show a 20px green check circle and `--ink` text; the current step a 14px spinner (2px ring, `--accent` top, 0.8s rotation); later steps a grey dot. Steps advance every 450ms.
- **Done:** the panel shows the result as an inline alert (no border or padding), adds a "Run again" action to the header, and extra widgets appear below (see the page map).

---

← Previous: [Code and log blocks](22-code-and-log-blocks.md) · [Index](README.md) · Next: [Chat](24-chat.md) →
