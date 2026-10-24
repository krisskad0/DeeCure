// SPDX-License-Identifier: Apache-2.0
// How many times something was recorded in the last 7 days (grouped events count each time),
// such as apps stopped for memory or disk read errors (history/journal.rs in the agent).

import { type EventKind, getEvents } from "./api";
import { useLive } from "./useLive";

const WEEK = 7 * 86_400_000;

/** The count, or `undefined` while loading or when the history cannot be read. */
export function useEventCount(kinds: EventKind[], subject?: string): number | undefined {
  const { data } = useLive(() => getEvents(Date.now() - WEEK, { kinds, limit: 500 }), 300_000, `${kinds.join(",")}|${subject ?? ""}`);
  if (!data) return undefined;
  return data.events.filter((e) => subject === undefined || e.subject === subject).reduce((n, e) => n + Math.max(1, Number((e.data as { count?: number } | undefined)?.count ?? 1)), 0);
}
