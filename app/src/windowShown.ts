// SPDX-License-Identifier: Apache-2.0
// Whether anyone can see the window: not hidden in the tray and not minimized. While it is
// hidden, panels stop reading data (useLive), so a window left in the tray costs no CPU.

import { useSyncExternalStore } from "react";

/** The event the desktop sends when it hides the window in the tray or shows it (src-tauri/src/tray.rs). */
export const SHOWN_EVENT = "window-shown";

let fromApp = true;
let fromPage = typeof document === "undefined" || document.visibilityState !== "hidden";
const listeners = new Set<() => void>();
let started = false;

/** True unless the window is hidden in the tray or minimized. */
export function isWindowShown(): boolean {
  return fromApp && fromPage;
}

function changed() {
  for (const listener of listeners) listener();
}

/** Sets what the desktop says (also for tests). */
export function setShownByApp(shown: boolean) {
  if (fromApp !== shown) {
    fromApp = shown;
    changed();
  }
}

function start() {
  if (started || typeof window === "undefined") return;
  started = true;
  document.addEventListener("visibilitychange", () => {
    fromPage = document.visibilityState !== "hidden";
    changed();
  });
  // Only in the desktop app, not in tests or a plain browser.
  if (!("__TAURI_INTERNALS__" in window)) return;
  void import("@tauri-apps/api/event").then(({ listen }) => listen<boolean>(SHOWN_EVENT, (e) => setShownByApp(e.payload)));
  // A window started hidden (`--hidden`) sent its event before this page listened.
  void import("@tauri-apps/api/window").then(({ getCurrentWindow }) => getCurrentWindow().isVisible().then(setShownByApp, () => undefined));
}

function subscribe(listener: () => void) {
  start();
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** True while the window can be seen; the component draws again when this changes. */
export function useWindowShown(): boolean {
  return useSyncExternalStore(subscribe, isWindowShown);
}
