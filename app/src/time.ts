// SPDX-License-Identifier: Apache-2.0

/** "just now", "5 min ago", "3 h ago", "2 days ago" — the same words as `deecure status`. */
export function ago(then: number, now: number = Date.now()): string {
  const minutes = Math.floor((now - then) / 60_000);
  if (minutes < 1) return "just now";
  if (minutes < 60) return `${minutes} min ago`;
  if (minutes < 1440) return `${Math.floor(minutes / 60)} h ago`;
  const days = Math.floor(minutes / 1440);
  return `${days} ${days === 1 ? "day" : "days"} ago`;
}
