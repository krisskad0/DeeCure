// SPDX-License-Identifier: Apache-2.0
// The prototype's deterministic sample data (docs/03-ui-design/04-behavior/sample-data.md),
// ported exactly, for the component gallery and tests. Not used for real data.

export type Range = "1h" | "24h" | "7d" | "30d";

export const NOW = new Date(2026, 9, 21, 21, 34).getTime();
export const RANGES: Record<Range, { n: number; step: number }> = {
  "1h": { n: 60, step: 1 / 60 },
  "24h": { n: 96, step: 0.25 },
  "7d": { n: 168, step: 1 },
  "30d": { n: 120, step: 6 },
};

export function rng(seed: number): () => number {
  let s = (seed * 9301 + 49297) % 2147483647;
  if (s <= 0) s += 2147483646;
  return () => (s = (s * 16807) % 2147483647) / 2147483647;
}

export function timeAt(i: number, n: number, stepHours: number): Date {
  return new Date(NOW - (n - 1 - i) * stepHours * 3600e3);
}

/** gen[seed, base, amp, noise, lo, hi, end] from the page specs. */
export function gen(range: Range, [seed, base, amp, noise, lo = 0, hi = 100, end]: number[]): number[] {
  const { n, step } = RANGES[range];
  const r = rng(seed * 31 + n);
  const out: number[] = [];
  let w = 0;
  for (let i = 0; i < n; i++) {
    const t = timeAt(i, n, step);
    const hour = t.getHours() + t.getMinutes() / 60;
    w = w * 0.72 + (r() - 0.5) * noise;
    out.push(Math.max(lo, Math.min(hi, base + amp * Math.sin(((hour - 9) / 24) * 2 * Math.PI) + w)));
  }
  if (end !== undefined) {
    out[n - 1] = end;
    out[n - 2] = (out[n - 2] + end) / 2;
  }
  return out;
}

/** heatData(seed, rows, cols, base, spikes) from the spec. */
export function heatData(seed: number, rows: number, cols: number, base: number, spikes: [number, number, number][] = []): number[][] {
  const r = rng(seed);
  return Array.from({ length: rows }, (_, ri) =>
    Array.from({ length: cols }, (_, ci) => {
      let v = Math.max(0, Math.round((r() - 0.55) * base));
      for (const [sr, sc, sv] of spikes) if (sr === ri && sc === ci) v = sv;
      return v;
    }),
  );
}
