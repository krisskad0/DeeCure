// SPDX-License-Identifier: Apache-2.0
// Loads data from the agent now and every few seconds while the page is open.

import { useEffect, useState } from "react";
import { type AppError, toAppError } from "./api";
import { usePref } from "./prefs";
import { REFRESH_EVENT, type RefreshChoice, refreshMs } from "./units";

/** Panels that update this often or faster follow Settings › General › Refresh interval. */
const LIVE_MS = 15_000;

/**
 * `key`: load again at once when it changes, such as the time range. Live panels
 * (every 15 s or faster) use the chosen refresh interval instead, and none at all in
 * Manual mode. The Refresh button reloads every panel.
 */
export function useLive<T>(load: () => Promise<T>, everyMs = 10_000, key = ""): { data: T | undefined; error: AppError | null } {
  const [data, setData] = useState<T | undefined>(undefined);
  const [error, setError] = useState<AppError | null>(null);
  const [choice] = usePref<RefreshChoice>("refresh", "5s");
  const interval = everyMs <= LIVE_MS ? refreshMs(choice) : everyMs;
  useEffect(() => {
    let alive = true;
    const run = () =>
      load().then(
        (value) => alive && (setData(value), setError(null)),
        (e) => alive && setError(toAppError(e)),
      );
    void run();
    const timer = interval === null ? undefined : setInterval(run, interval);
    const onRefresh = () => void run();
    window.addEventListener(REFRESH_EVENT, onRefresh);
    return () => {
      alive = false;
      if (timer !== undefined) clearInterval(timer);
      window.removeEventListener(REFRESH_EVENT, onRefresh);
    };
    // `load` may be a new closure on every render; `key` names what it depends on.
  }, [interval, key]);
  return { data, error };
}
