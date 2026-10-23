# app

The DeeCure desktop app: Tauri v2 + React + TypeScript.

| Path | What |
| --- | --- |
| `src/` | React UI. `tokens.css` holds the design tokens from the prototype. Fonts (Ubuntu Sans, Ubuntu Sans Mono) are bundled, so the app works offline. |
| `src/components/` | Reusable components, one file each ([spec](../docs/03-ui-design/03-components/README.md)) |
| `src/api.ts` | Calls into the Tauri side. Types mirror `crates/deecure-core`. |
| `src/pages/<page>/` | One folder per page; every tab reads real data from the agent |
| `src/shell/` | Top bar, sidebar, page header, command palette, drawers, fix drawer, onboarding, toasts |
| `src/visible.ts` | Pages hidden until they are built (the assistant, Phase 4) |
| `src/access.ts`, `src/units.ts`, `src/prefs.ts` | Access level (Basic / Full), refresh interval and temperature unit, saved view preferences |
| `src/spec/` | **Generated** from the UI spec by `scripts/gen-ui.py`: icons and the page list. Do not edit; change the spec and run the script. CI fails if they drift. |
| `src/dev/` | Development-only: the **component gallery** (Ctrl K → "Component gallery", or `#/gallery`) and the prototype's sample-data generator |
| `src-tauri/` | Tauri Rust side. `main.rs`: commands that talk to the agent's socket through `deecure_core::client`, run fixes with `deecure-fix`, and export reports. `tray.rs`: the optional tray icon and the one-copy socket. |
| `src-tauri/icons/` | Window icons, made from `packaging/desktop/deecure.svg` with `npx tauri icon` |

## Run it live

```bash
make agent   # terminal 1: the agent
make app     # terminal 2: the app window; edits to src/ show up at once
```

Node.js 22 or newer is needed (`nvm install 22`). `make app` installs the packages the first time.

## Check it

```bash
make web     # UI tests (Vitest), type-check (TypeScript), and build
make lint    # Rust lints, including src-tauri
```

- UI spec: [docs/03-ui-design](../docs/03-ui-design/README.md)
- Why Tauri: [ADR 0002](../docs/02-architecture/decisions/0002-tauri-react-desktop-ui.md)
