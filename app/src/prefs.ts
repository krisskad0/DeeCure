// SPDX-License-Identifier: Apache-2.0
// Small per-user view settings (sidebar collapsed, mode, time range), kept in localStorage.
// Storage can be missing or blocked, so every access is guarded and the app works without it.

import { useState } from "react";

export function usePref<T extends string | boolean>(key: string, initial: T): [T, (value: T) => void] {
  const [value, setValue] = useState<T>(() => {
    try {
      const saved = window.localStorage.getItem(`deecure.${key}`);
      return saved === null ? initial : (JSON.parse(saved) as T);
    } catch {
      return initial;
    }
  });
  const save = (next: T) => {
    setValue(next);
    try {
      window.localStorage.setItem(`deecure.${key}`, JSON.stringify(next));
    } catch {
      // Not saved; the setting still applies until the window closes.
    }
  };
  return [value, save];
}
