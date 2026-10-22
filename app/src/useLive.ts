// SPDX-License-Identifier: Apache-2.0
// Loads data from the agent now and every few seconds while the page is open.

import { useEffect, useState } from "react";
import { type AppError, toAppError } from "./api";

export function useLive<T>(load: () => Promise<T>, everyMs = 10_000): { data: T | undefined; error: AppError | null } {
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
    // `load` is a module-level function in every caller, so it never changes.
  }, [everyMs]);
  return { data, error };
}
