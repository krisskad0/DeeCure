# app

The DeeCure desktop app: Tauri v2 + React + TypeScript.

| Path | What |
| --- | --- |
| `src/` | React UI. `tokens.css` holds the design tokens from the prototype. |
| `src/components/` | Reusable components, one file each ([spec](../docs/03-ui-design/03-components/README.md)) |
| `src/api.ts` | Calls into the Tauri side. Types mirror `crates/deecure-core`. |
| `src-tauri/` | Tauri Rust side. Its commands talk to the agent's socket through `deecure_core::client`. |
| `src-tauri/icons/` | Window icons, made from `packaging/desktop/deecure.svg` with `npx tauri icon` |

## Run it live

```bash
make agent   # terminal 1: the agent
make app     # terminal 2: the app window; edits to src/ show up at once
```

Node.js 22 or newer is needed (`nvm install 22`). `make app` installs the packages the first time.

## Check it

```bash
make web     # type-check (TypeScript) and build the web UI
make lint    # Rust lints, including src-tauri
```

- UI spec: [docs/03-ui-design](../docs/03-ui-design/README.md)
- Why Tauri: [ADR 0002](../docs/02-architecture/decisions/0002-tauri-react-desktop-ui.md)
