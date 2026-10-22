# 0002. Tauri v2 + React + TypeScript for the desktop app

- Status: Accepted (from the implementation plan)
- Date: 2026-10-22

## Context

The UI is a large dashboard: 22 pages, 91 sub-pages, many charts and tables. The prototype is already built with web technology (HTML, CSS, SVG). The app must stay under 200 MB RAM while open.

## Decision

Build the desktop app with Tauri v2. Write the UI in React and TypeScript. The Tauri Rust side talks to the agent over its local socket.

## Why

- Tauri uses the system web view (WebKitGTK on Linux), so it is much lighter than Electron.
- The team can reuse web skills and the prototype's CSS tokens.
- The Rust side shares types with the agent through `sysdoc-core`.
- **Options not picked:** Electron (too heavy), GTK4 + libadwaita (native look, but the dashboard and charts are much more work), Svelte instead of React (fine, choose it if the team prefers).

## Consequences

- Runtime depends on WebKitGTK 4.1 (`libwebkit2gtk-4.1-0`). It is in Ubuntu 22.04 and newer.
- The prototype's build rules (one HTML file, vanilla JavaScript) do not apply to the real app. The visual spec stays the same.
- Charts: still open. Either custom SVG components that match the spec exactly, or a library such as uPlot for long time series. See [Open decisions](../../01-product/open-decisions.md).
