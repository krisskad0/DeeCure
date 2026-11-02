# Form controls

[Docs](../../README.md) › [UI design](../README.md) › [Components](README.md)

- **Number box** (`.input-num`, such as warning limits and charge limits): the same field look as the select: 30px tall, 72px wide, right-aligned tabular numbers, 1px `--line-2` border, radius 7, `--panel`, `--btn-shadow`, no spin arrows. Hover `--muted` border; focus `--accent` border with a 3px `--accent-soft` ring; disabled at 0.45 opacity.
- **Checkbox and radio:** the system control in `--accent` (`accent-color`), 15px.
- **Switch:** 36×20, track `--line-2` (off) or `--accent` (on), 16px white knob with a light shadow, 16px travel, 0.15s transition. A real checkbox overlays it.
- **Select:** DeeCure's own dropdown, not the system one (the system's open list cannot be styled and looks out of place in the desktop app). The closed box is a `button` with `role="combobox"`; the open list is a `role="listbox"` card under it (or above it when there is no room below): `--panel`, 1px `--line-2` border, radius 8, `--shadow`, padding 4, at most 268px tall with scrolling; each option 30px, radius 6, `--hover` under the mouse or keyboard, the chosen one in `--accent` 600 with a check mark. Keys: click, Space, Enter, or the arrows open it; the arrows, Home, and End move; a letter jumps to the next option starting with it; Enter picks; Escape, Tab, or a click outside closes without a change. Tests pick an option with `choose(label, value)` from `src/test-utils.ts`. The closed box looks like a button: 30px tall, 1px `--line-2` border, radius 7, `--panel` background, `--btn-shadow`, padding 0 30 0 10, the system arrow replaced by DeeCure's own chevron (`--chev`, light and dark versions). Hover: `--muted` border; keyboard focus: the same ring as buttons. In a settings row it is at least 180px wide and right-aligned, so the controls line up.
- **Radio cards:** grid `repeat(auto-fit, minmax(200px, 1fr))`, gap 10, padding 12 14. Card: 1px border, radius 7, padding 10 12, radio + bold title + 12px muted description. Checked: `--accent` border, `--accent-soft` background.
- **Field row:** grid `1fr | auto`, gap 16, padding 10 14, bottom border: bold label + 12px muted description | control.
- **State tiles:** grid `repeat(auto-fit, minmax(150px, 1fr))`, gap 10. Tile: 1px border, radius 7, padding 10 12, 32px icon square (`--neutral-soft`) + bold title + 12px muted state.

---

← Previous: [Lifecycle bar](20-lifecycle-bar.md) · [Index](README.md) · Next: [Code and log blocks](22-code-and-log-blocks.md) →
