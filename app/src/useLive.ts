// SPDX-License-Identifier: Apache-2.0
// Loads data from the agent now and every few seconds while the page is open.

import { useEffect, useState } from "react";
import { type AppError, toAppError } from "./api";

/** `key`: load again at once when it changes, such as the time range. */
export function useLive<T>(load: () => Promise<T>, everyMs = 10_000, key = ""): { data: T | undefined; error: AppError | null } {
  const [data, setData] = useState<T | undefined>(undefined);
  const [error, setError] = useState<AppError | null>(null);
  useEffect(() => {
    let alive = true;
    const run = () =>
      load().then(
        (value) => alive && (setData(value), setError(null)),
        (e) => alive && setError(toAppError(e)),
      );
    void run();
    const timer = setInterval(run, everyMs);
    return () => {
      alive = false;
      clearInterval(timer);
    };
    // `load` may be a new closure on every render; `key` names what it depends on.
  }, [everyMs, key]);
  return { data, error };
}
