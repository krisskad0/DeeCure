// SPDX-License-Identifier: Apache-2.0
// The knowledge base: loaded from the agent once per window, then looked up locally.

import { useEffect, useState } from "react";
import { type Kb, getInventory } from "./api";

let cached: Promise<Kb | null> | null = null;

/** The knowledge base, or `null` while loading or when the agent has none. */
export function useKb(): Kb | null {
  const [kb, setKb] = useState<Kb | null>(null);
  useEffect(() => {
    let alive = true;
    cached ??= getInventory("kb")
      // An older agent, or an unexpected answer, means "no knowledge base", not a crash.
      .then((k) => (k && Array.isArray(k.services) && Array.isArray(k.topics) && typeof k.rules === "object" ? k : null))
      .catch(() => {
        cached = null; // try again next time
        return null;
      });
    void cached.then((k) => alive && setKb(k));
    return () => {
      alive = false;
    };
  }, []);
  return kb;
}

export const kbService = (kb: Kb | null, unit: string) => kb?.services?.find((s) => s.name === unit);
export const kbPort = (kb: Kb | null, port: number, proto: string) => kb?.ports?.find((p) => p.port === port && p.proto === proto);
export const kbPackage = (kb: Kb | null, name: string) => kb?.packages?.find((p) => p.name === name);
export const kbTopic = (kb: Kb | null, rule: string) => {
  const id = kb?.rules?.[rule];
  return id ? kb?.topics?.find((t) => t.id === id) : undefined;
};

/** The first error pattern a log message contains. Patterns like `nvme.*timeout` are regular expressions. */
export function kbError(kb: Kb | null, message: string) {
  return kb?.errors?.find((e) => {
    if (message.includes(e.match)) return true;
    if (!/[.*+?[\]()|]/.test(e.match)) return false;
    try {
      return new RegExp(e.match).test(message);
    } catch {
      return false;
    }
  });
}

/** For tests: forget the cached copy. */
export function resetKb() {
  cached = null;
}
