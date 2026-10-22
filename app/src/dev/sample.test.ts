// SPDX-License-Identifier: Apache-2.0
import { describe, expect, it } from "vitest";
import { RANGES, gen, heatData, rng } from "./sample";

describe("sample data (sample-data.md)", () => {
  it("rng is deterministic and between 0 and 1", () => {
    const a = rng(42);
    const b = rng(42);
    const first = [a(), a(), a()];
    expect([b(), b(), b()]).toEqual(first);
    for (const v of first) expect(v).toBeGreaterThan(0), expect(v).toBeLessThan(1);
  });

  it("gen has one point per range step, stays in bounds, and ends on `end`", () => {
    for (const range of ["1h", "24h", "7d", "30d"] as const) {
      const data = gen(range, [3, 14, 8, 6, 2, 60, 12]);
      expect(data).toHaveLength(RANGES[range].n);
      expect(Math.min(...data)).toBeGreaterThanOrEqual(2);
      expect(Math.max(...data)).toBeLessThanOrEqual(60);
      expect(data[data.length - 1]).toBe(12);
    }
  });

  it("heatData applies spikes", () => {
    const grid = heatData(7, 7, 24, 4, [[2, 9, 14]]);
    expect(grid).toHaveLength(7);
    expect(grid[2][9]).toBe(14);
  });
});
