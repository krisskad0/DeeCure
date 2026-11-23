// SPDX-License-Identifier: Apache-2.0
// One Optimize scan shared by the Optimize page and the Overview card. The checks run many system
// tools (about 10 s in all), so DeeCure checks by itself only once a day (SCAN_EVERY_MS), and
// otherwise only when the user asks. The last answer is kept between app starts.
//
// A scan asks the agent for one part at a time (PARTS), so the page shows which part runs now and
// what each part found. An agent older than the parts (0.1.0-alpha.9) answers everything in one call.

import { useEffect, useSyncExternalStore } from "react";
import { type AppError, type OptimizeCheck, getInventory, toAppError } from "../../api";
import type { IconName } from "../../spec/icons";
import { useWindowShown } from "../../windowShown";

/** How old an answer may be before an open view checks again by itself. */
export const SCAN_EVERY_MS = 86_400_000;
/** After a failed scan, how long before an open view tries again by itself. */
const RETRY_MS = 3_600_000;
/** How long a scan usually takes on a laptop, for the "usually about" hint. */
export const SCAN_USUAL_MS = 15_000;
/** Where the last answer is kept (this computer only). */
const KEY = "deecure.optimize";

/** The parts of the checks, in the agent's order (inventory kind `optimize_<id>`). */
export const PARTS: { id: string; label: string; icon: IconName; what: string }[] = [
  { id: "boot", label: "Start-up", icon: "play", what: 'Whether "wait for network" slows boot' },
  { id: "disks", label: "Disks", icon: "db", what: "Weekly TRIM for SSDs" },
  { id: "kernels", label: "Old kernels", icon: "pkg", what: "Kernels kept besides the running one and a fallback" },
  { id: "devices", label: "Devices", icon: "usb", what: "Power saving, PCIe link width, USB speed, and services with no hardware" },
  { id: "hardware", label: "Hardware health", icon: "chip", what: "Heat slowdowns, battery health, restart needed, and memory errors" },
  { id: "cleanup", label: "Space, packages, and updates", icon: "layers", what: "Trash, caches, logs, snaps, Docker, unused packages, updates, firewall, and file watches" },
  { id: "history", label: "Usage history", icon: "hist", what: "Charge limit, battery drain, power saver and turbo on battery, power mode, wake-ups, and swapping" },
  { id: "drivers", label: "Drivers and firmware", icon: "mon", what: "Video driver, second graphics card, recommended drivers, firmware, and DKMS" },
];

export interface PartState {
  id: string;
  status: "waiting" | "running" | "done" | "failed";
  /** When this part started (ms). */
  startedAt?: number;
  /** How long it took (ms). */
  ms?: number;
  /** Checks it ran, and how many of them offer something. */
  checked?: number;
  found?: number;
  error?: string;
}

export interface ScanState {
  checks?: OptimizeCheck[];
  /** When the newest answer arrived (ms). */
  at?: number;
  error: AppError | null;
  /** When the running scan started (ms); undefined while no scan runs. */
  startedAt?: number;
  /** Each part of the newest scan; undefined for an older agent's one-call scan. */
  parts?: PartState[];
  /** What the running scan offers so far, as parts finish. */
  found?: OptimizeCheck[];
  /** When a fix ran (ms); the answer may be out of date when this is newer than `at`. */
  changedAt?: number;
}

function load(): Pick<ScanState, "checks" | "at"> {
  try {
    const saved = JSON.parse(window.localStorage.getItem(KEY) ?? "null");
    if (saved && Array.isArray(saved.checks) && typeof saved.at === "number") return { checks: saved.checks, at: saved.at };
  } catch {
    // No storage, or a damaged value: start without an answer.
  }
  return {};
}

function save(checks: OptimizeCheck[], at: number) {
  try {
    window.localStorage.setItem(KEY, JSON.stringify({ checks, at }));
  } catch {
    // Storage full or blocked: the answer still shows until the app closes.
  }
}

let state: ScanState = { error: null, ...load() };
let running: Promise<void> | null = null;
/** How many full scans have started, so a one-part re-check can tell one began after it. */
let scans = 0;
/** How many fixes have run, so a re-check clears the "a fix ran" mark only when no other fix ran after it. */
let fixes = 0;
/** When the last scan started, so a failing agent is asked again only every RETRY_MS. */
let tried: number | undefined;
const listeners = new Set<() => void>();

function set(next: Partial<ScanState>) {
  state = { ...state, ...next };
  for (const listener of listeners) listener();
}

function setPart(id: string, next: Partial<PartState>) {
  set({ parts: state.parts?.map((p) => (p.id === id ? { ...p, ...next } : p)) });
}

/** Offered checks first, keeping the agent's order within each. */
function sorted(checks: OptimizeCheck[]): OptimizeCheck[] {
  return [...checks.filter((c) => c.offered), ...checks.filter((c) => !c.offered)];
}

function finish(checks: OptimizeCheck[]) {
  const at = Date.now();
  set({ checks: sorted(checks), at, error: null });
  save(state.checks ?? [], at);
}

async function run() {
  const all: OptimizeCheck[] = [];
  let first: AppError | null = null;
  for (const [i, part] of PARTS.entries()) {
    const started = Date.now();
    setPart(part.id, { status: "running", startedAt: started });
    try {
      const got = tagged(await getInventory(`optimize_${part.id}`), part.id);
      all.push(...got);
      const found = got.filter((c) => c.offered);
      setPart(part.id, { status: "done", ms: Date.now() - started, checked: got.length, found: found.length });
      set({ found: [...(state.found ?? []), ...found] });
    } catch (e) {
      const error = toAppError(e);
      if (i === 0 && error.kind === "agent_outdated") {
        // An agent from before the parts: one call for everything, without live parts.
        set({ parts: undefined });
        try {
          finish(await getInventory("optimize"));
        } catch (e2) {
          set({ error: toAppError(e2) });
        }
        return;
      }
      if (error.kind === "agent_not_running") {
        set({ error, parts: undefined });
        return;
      }
      first ??= error;
      setPart(part.id, { status: "failed", ms: Date.now() - started, error: error.message });
    }
  }
  // Every part failed: keep the last answer and say why.
  if (state.parts?.every((p) => p.status === "failed")) set({ error: first });
  else finish(all);
}

/** Starts a scan, or returns the one already running. */
export function scanOptimize(): Promise<void> {
  if (running) return running;
  tried = Date.now();
  scans += 1;
  set({ startedAt: tried, error: null, found: [], parts: PARTS.map((p) => ({ id: p.id, status: "waiting" })) });
  running = run().finally(() => {
    running = null;
    set({ startedAt: undefined });
  });
  return running;
}

/** Marks each check with the part it came from. */
function tagged(checks: OptimizeCheck[], part: string): OptimizeCheck[] {
  return checks.map((c) => ({ ...c, part }));
}

/**
 * After a fix: the answer may be out of date. When the fix came from a check whose part is known,
 * only that part is checked again at once (a second or two), so the item shows as done. Otherwise
 * the page says to scan again.
 *
 * The fresh part goes into the newest list when it arrives (not the list from before the wait),
 * so two quick fixes do not undo each other. If a full scan started or ended meanwhile, its
 * answer is newer and this one is dropped. Only this fix's "a fix ran" mark is cleared, so a
 * fix from another page in the meantime still shows it.
 */
export async function optimizeChanged(action?: string): Promise<void> {
  if (state.at === undefined) return;
  const changedAt = Date.now();
  const fix = ++fixes;
  set({ changedAt });
  const parts = [...new Set((state.checks ?? []).filter((c) => action && c.action === action && c.part).map((c) => c.part as string))];
  if (parts.length === 0 || running) return;
  const scan = scans;
  try {
    const fresh: OptimizeCheck[] = [];
    for (const part of parts) fresh.push(...tagged(await getInventory(`optimize_${part}`), part));
    if (scans !== scan) return;
    const kept = (state.checks ?? []).filter((c) => !c.part || !parts.includes(c.part));
    set({ checks: sorted([...kept, ...fresh]), ...(fixes === fix ? { changedAt: undefined } : {}) });
    save(state.checks ?? [], state.at ?? changedAt);
  } catch {
    // The banner stays: "Scan again to see what is left."
  }
}

/** For tests: forget every answer and the kept one, or with `keep`, read the kept one again. */
export function resetOptimizeScan(keep = false) {
  running = null;
  tried = undefined;
  // A re-check still waiting from before is dropped.
  scans += 1;
  try {
    if (!keep) window.localStorage.removeItem(KEY);
  } catch {
    // Nothing kept.
  }
  state = { error: null, ...load() };
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** Whether an open view should check by itself: no answer, or one older than a day. */
function due(): boolean {
  if (running) return false;
  const now = Date.now();
  if (tried !== undefined && now - tried < RETRY_MS) return false;
  return state.at === undefined || now - state.at >= SCAN_EVERY_MS;
}

/** The shared scan. While the window is shown, it checks by itself only when `due`. */
export function useOptimizeScan(): ScanState {
  const shown = useWindowShown();
  useEffect(() => {
    if (!shown) return;
    const check = () => void (due() && scanOptimize());
    check();
    const timer = setInterval(check, 60_000);
    return () => clearInterval(timer);
  }, [shown]);
  return useSyncExternalStore(subscribe, () => state);
}
