# app

The SysDoc desktop app: Tauri v2 + React + TypeScript.

| Path (planned) | What |
| --- | --- |
| `src/` | React UI: shell, components, pages |
| `src-tauri/` | Tauri Rust side. Connects to the agent socket. |
| `package.json` | Scripts: `dev`, `build`, `test`, `lint` |

- UI spec: [docs/03-ui-design](../docs/03-ui-design/README.md)
- Why Tauri: [ADR 0002](../docs/02-architecture/decisions/0002-tauri-react-desktop-ui.md)
