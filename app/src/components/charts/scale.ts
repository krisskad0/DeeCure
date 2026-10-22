// SPDX-License-Identifier: Apache-2.0
// Shared chart maths, from docs/03-ui-design/03-components/12–16.

import type { Range } from "../../ui";

/** The smallest "nice" number (1, 2, 2.5, 5, or 10 × 10ⁿ) that is at least `value`. */
export function niceCeil(value: number): number {
  if (!(value > 0)) return 1;
  const base = 10 ** Math.floor(Math.log10(value));
  for (const step of [1, 2, 2.5, 5, 10]) {
    if (step * base >= value - 1e-9) return step * base;
  }
  return 10 * base;
}

/** Axis and tooltip numbers: rounded at 10 and above or when whole, else one decimal; then the unit. */
export function formatValue(value: number, unit = ""): string {
  const text = Math.abs(value) >= 10 || Number.isInteger(value) ? String(Math.round(value)) : value.toFixed(1);
  if (!unit) return text;
  return unit === "%" || unit.startsWith("°") ? `${text}${unit}` : `${text} ${unit}`;
}

const pad2 = (n: number) => String(n).padStart(2, "0");

/** Time labels by range: axis (`tooltip` false) or tooltip header (true). */
export function timeLabel(ms: number, range: Range, tooltip: boolean): string {
  const d = new Date(ms);
  const hhmm = `${pad2(d.getHours())}:${pad2(d.getMinutes())}`;
  const weekday = d.toLocaleDateString("en-GB", { weekday: "short" });
  const dayMonth = d.toLocaleDateString("en-GB", { day: "numeric", month: "short" });
  if (range === "1h") return hhmm;
  if (range === "24h") return tooltip ? `${weekday} ${hhmm}` : hhmm;
  if (range === "7d") return tooltip ? `${weekday} ${hhmm}` : weekday;
  return tooltip ? `${dayMonth} ${hhmm}` : dayMonth;
}

/** Point indices that get an x-axis label on a time series: 0, 25, 50, 75, and 100%. */
export function timeLabelIndices(n: number): number[] {
  return [...new Set([0, 0.25, 0.5, 0.75, 1].map((f) => Math.round(f * (n - 1))))];
}

/** Indices that get a label from a label array: every ceil(n/6)-th, plus the last. */
export function labelArrayIndices(n: number): number[] {
  const every = Math.ceil(n / 6);
  const out = [];
  for (let i = 0; i < n; i += every) out.push(i);
  if (out[out.length - 1] !== n - 1) out.push(n - 1);
  return out;
}

/** Bar charts show every k-th label so labels do not overlap. */
export function barLabelEvery(n: number, width: number): number {
  return Math.ceil(n / Math.max(4, Math.floor(width / 46)));
}

/** Heatmap colour level 1–5 (--h1 … --h5) for a cell. */
export function heatLevel(value: number, max: number): number {
  if (value <= 0 || max <= 0) return 1;
  return Math.min(5, 2 + Math.floor((value / max) * 3.99));
}
