// SPDX-License-Identifier: Apache-2.0
// Small per-user view settings (sidebar collapsed, mode, time range), kept in localStorage.
// Storage can be missing or blocked, so every access is guarded and the app works without it.
// Every hook using the same key stays in step: a change anywhere updates them all.

import { useEffect, useState } from "react";

const EVENT = "deecure-pref";

function load<T>(key: string, initial: T): T {
  try {
    const saved = window.localStorage.getItem(`deecure.${key}`);
    return saved === null ? initial : (JSON.parse(saved) as T);
  } catch {
    return initial;
  }
}

export function usePref<T extends string | boolean>(key: string, initial: T): [T, (value: T) => void] {
  const [value, setValue] = useState<T>(() => load(key, initial));
  useEffect(() => {
    const onChange = (event: Event) => {
      const detail = (event as CustomEvent<{ key: string; value: T }>).detail;
      if (detail.key === key) setValue(detail.value);
    };
    window.addEventListener(EVENT, onChange);
    return () => window.removeEventListener(EVENT, onChange);
  }, [key]);
  const save = (next: T) => {
    setValue(next);
    try {
      window.localStorage.setItem(`deecure.${key}`, JSON.stringify(next));
    } catch {
      // Not saved; the setting still applies until the window closes.
    }
    window.dispatchEvent(new CustomEvent(EVENT, { detail: { key, value: next } }));
  };
  return [value, save];
}
