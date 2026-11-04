// SPDX-License-Identifier: Apache-2.0
// One Optimize scan shared by the Optimize page and the Overview strip. The agent runs every
// check live (about 10 s of system tools), so two views never start two scans: they read the
// same answer, and a new scan starts only when the last one is older than SCAN_EVERY_MS.

import { useEffect, useSyncExternalStore } from "react";
import { type AppError, type OptimizeCheck, getInventory, toAppError } from "../../api";
import { REFRESH_EVENT } from "../../units";
import { useWindowShown } from "../../windowShown";

/** How old an answer may be before a view that is open scans again. */
export const SCAN_EVERY_MS = 120_000;
/** How long a scan usually takes on a laptop, for the "usually about" hint. */
export const SCAN_USUAL_MS = 10_000;

export interface ScanState {
  checks?: OptimizeCheck[];
  /** When the newest answer arrived (ms). */
  at?: number;
  error: AppError | null;
  /** When the running scan started (ms); undefined while no scan runs. */
  startedAt?: number;
}

let state: ScanState = { error: null };
let running: Promise<void> | null = null;
/** When the last scan started, so a failing agent is asked again only every SCAN_EVERY_MS. */
let tried: number | undefined;
const listeners = new Set<() => void>();

function set(next: Partial<ScanState>) {
  state = { ...state, ...next };
  for (const listener of listeners) listener();
}

/** Starts a scan, or returns the one already running. */
export function scanOptimize(): Promise<void> {
  if (running) return running;
  tried = Date.now();
  set({ startedAt: tried });
  running = getInventory("optimize")
    .then(
      (checks) => set({ checks, at: Date.now(), error: null }),
      // The last answer stays on screen with the error.
      (e) => set({ error: toAppError(e) }),
    )
    .finally(() => {
      running = null;
      set({ startedAt: undefined });
    });
  return running;
}

/** After a fix: scan again at once if a view is open, else on the next open. */
export function optimizeChanged() {
  if (listeners.size) void scanOptimize();
  else tried = undefined;
}

/** For tests: forget every answer. */
export function resetOptimizeScan() {
  running = null;
  tried = undefined;
  state = { error: null };
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

const stale = () => !running && (tried === undefined || Date.now() - tried >= SCAN_EVERY_MS);

/** The shared scan; scans while shown when the answer is missing or old. The Refresh button scans now. */
export function useOptimizeScan(): ScanState {
  const shown = useWindowShown();
  useEffect(() => {
    if (!shown) return;
    const check = () => void (stale() && scanOptimize());
    check();
    const timer = setInterval(check, 15_000);
    const onRefresh = () => void scanOptimize();
    window.addEventListener(REFRESH_EVENT, onRefresh);
    return () => {
      clearInterval(timer);
      window.removeEventListener(REFRESH_EVENT, onRefresh);
    };
  }, [shown]);
  return useSyncExternalStore(subscribe, () => state);
}
