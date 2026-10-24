// SPDX-License-Identifier: Apache-2.0

import { describe, expect, it } from "vitest";
import type { Metric } from "./api";
import { align, bytes, duration, heatStatus, pick, scoreTone } from "./metrics";

const m = (at: number, value: number, name = "cpu.usage", subject?: string): Metric => ({ name, subject, value, unit: "percent", at });

describe("metric helpers", () => {
  it("formats sizes and durations in plain words", () => {
    expect(bytes(512)).toBe("512 B");
    expect(bytes(9_197_629_440)).toBe("8.6 GB");
    expect(bytes(250 * 1024 ** 3)).toBe("250 GB");
    expect(duration(45)).toBe("0 m");
    expect(duration(11_400)).toBe("3 h 10 m");
    expect(duration(190_000)).toBe("2 days 4 h");
  });

  it("aligns series on shared times only", () => {
    const { times, data } = align([
      [{ at: 1, value: 10 }, { at: 2, value: 20 }, { at: 3, value: 30 }],
      [{ at: 2, value: 5 }, { at: 3, value: 6 }],
    ]);
    expect(times).toEqual([2, 3]);
    expect(data).toEqual([[20, 30], [5, 6]]);
  });

  it("picks by name and subject", () => {
    const latest = [m(1, 40, "disk.used_percent", "/"), m(1, 12, "disk.used_percent", "/boot"), m(1, 7)];
    expect(pick(latest, "disk.used_percent", "/boot")).toBe(12);
    expect(pick(latest, "cpu.usage")).toBe(7);
    expect(pick(latest, "memory.used")).toBeUndefined();
  });

  it("colours the score by its band", () => {
    expect([scoreTone(95), scoreTone(80), scoreTone(60)]).toEqual(["ok", "warn", "crit"]);
  });
});

describe("heatStatus", () => {
  it("judges a temperature only against the part's own limits", () => {
    // A laptop CPU at 93 °C whose own limit is 95 °C is working normally.
    expect(heatStatus(93, 95, 100)).toBe("ok");
    expect(heatStatus(96, 95, 100)).toBe("warn");
    expect(heatStatus(100, 95, 100)).toBe("crit");
    // No limit reported: no colour, rather than a guess.
    expect(heatStatus(97)).toBeUndefined();
    expect(heatStatus(undefined, 95, 100)).toBeUndefined();
  });
});

