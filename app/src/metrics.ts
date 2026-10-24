// SPDX-License-Identifier: Apache-2.0
// Helpers for metric data: time ranges, lining up series, and plain-English numbers.

import type { Metric } from "./api";
import type { Range } from "./ui";

export const RANGE_MS: Record<Range, number> = { "1h": 3_600_000, "24h": 86_400_000, "7d": 604_800_000, "30d": 2_592_000_000 };

/** The newest value of a metric, or `undefined` when there is none. */
export function pick(latest: Metric[] | undefined, name: string, subject?: string): number | undefined {
  return latest?.find((m) => m.name === name && m.subject === subject)?.value;
}

/** All subjects of one metric, such as every temperature sensor. */
/**
 * A temperature against the part's own limits (`temp.limit` and `temp.critical`, or the CPU's
 * `temp.cpu_limit` and `temp.cpu_critical`), as the health warnings judge it; never one number
 * for every computer. `undefined` (no colour) when the part reports no limit.
 */
export function heatStatus(celsius: number | undefined, limit?: number, critical?: number): "ok" | "warn" | "crit" | undefined {
  if (celsius === undefined || (limit === undefined && critical === undefined)) return undefined;
  if (critical !== undefined && celsius >= critical) return "crit";
  if (limit !== undefined && celsius >= limit) return "warn";
  return "ok";
}

export function pickAll(latest: Metric[] | undefined, name: string): Metric[] {
  return (latest ?? []).filter((m) => m.name === name && m.subject !== undefined);
}

/** Lines up several series on the times they all share. */
export function align(series: Pick<Metric, "at" | "value">[][]): { times: number[]; data: number[][] } {
  if (series.length === 0) return { times: [], data: [] };
  const maps = series.map((s) => new Map(s.map((p) => [p.at, p.value])));
  const times = series[0].map((p) => p.at).filter((t) => maps.every((m) => m.has(t)));
  return { times, data: maps.map((m) => times.map((t) => m.get(t) ?? 0)) };
}

const UNITS = ["B", "KB", "MB", "GB", "TB"];

/** 9197629440 → "8.6 GB". Uses 1024 steps, like the GNOME System Monitor. */
export function bytes(value: number, digits = 1): string {
  let v = value;
  let u = 0;
  while (Math.abs(v) >= 1024 && u < UNITS.length - 1) {
    v /= 1024;
    u++;
  }
  return `${u === 0 ? Math.round(v) : v.toFixed(v >= 100 ? 0 : digits)} ${UNITS[u]}`;
}

/** Bytes per second → "1.2 MB/s". */
export function rate(value: number): string {
  return `${bytes(value)}/s`;
}

/** Seconds → "3 h 10 m", "12 m", "2 days 4 h". */
export function duration(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes} m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours} h ${minutes % 60} m`;
  const days = Math.floor(hours / 24);
  return `${days} ${days === 1 ? "day" : "days"} ${hours % 24} h`;
}

/** A date and time in the user's locale, with "Today" for today. */
export function when(ms: number, now = Date.now()): string {
  const d = new Date(ms);
  const time = d.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
  if (new Date(now).toDateString() === d.toDateString()) return `Today ${time}`;
  return `${d.toLocaleDateString(undefined, { day: "2-digit", month: "short" })} ${time}`;
}

/** Score colour: 90+ healthy, 75-89 fair, below 75 at risk. */
export function scoreTone(score: number): "ok" | "warn" | "crit" {
  return score >= 90 ? "ok" : score >= 75 ? "warn" : "crit";
}

const DAY_MS = 86_400_000;

/** Day × hour counts for the last `days` days (oldest first), for the heatmap. */
export function heatRows(times: number[], now = Date.now(), days = 7): { label: string; values: number[] }[] {
  const start = new Date(now);
  start.setHours(0, 0, 0, 0);
  const first = start.getTime() - (days - 1) * DAY_MS;
  const rows = Array.from({ length: days }, (_, i) => ({
    label: new Date(first + i * DAY_MS + 12 * 3_600_000).toLocaleDateString(undefined, { weekday: "short" }),
    values: Array<number>(24).fill(0),
  }));
  for (const t of times) {
    const d = new Date(t);
    const day = new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
    const i = Math.round((day - first) / DAY_MS);
    if (i >= 0 && i < days) rows[i].values[d.getHours()]++;
  }
  return rows;
}

/** systemd writes "-" in unit names as a backslash and x2d. */
export function unescapeUnit(name: string): string {
  return name.replace(/\\x([0-9a-f]{2})/gi, (_, hex: string) => String.fromCharCode(parseInt(hex, 16)));
}

/** apt writes local time as "2026-10-23  00:34:58". */
export function aptTime(start: string): number {
  return new Date(start.replace(/\s+/, "T")).getTime();
}

