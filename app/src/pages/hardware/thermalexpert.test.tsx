// SPDX-License-Identifier: Apache-2.0
// Expert temperature sections: zones with their limits, cooling in plain words, slowdowns.

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ThermalExpertSections, coolingText, perInterval } from "./ThermalExpert";

vi.mock("../../api", () => ({
  getInventory: () =>
    Promise.resolve({
      zones: [
        { zone: "thermal_zone1", kind: "TSKN", celsius: 62, trips: [{ kind: "active", celsius: 60.05 }, { kind: "passive", celsius: 65.05 }, { kind: "critical", celsius: 127.05 }] },
        { zone: "thermal_zone9", kind: "MYSTERY", trips: [] },
      ],
      cooling: [
        { kind: "Processor", count: 12, level: 0, max_level: 3 },
        { kind: "dell-smm-fan1", count: 1, level: 2, max_level: 2 },
      ],
    }),
  getMetrics: () => Promise.resolve([]),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(cleanup);

describe("Expert temperature sections", () => {
  it("zones show where the system acts, and little room left is pointed out", async () => {
    render(<ThermalExpertSections />);
    await waitFor(() => expect(screen.getByText("Case surface (skin)")).toBeTruthy());
    expect(screen.getByText("MYSTERY")).toBeTruthy();
    expect(screen.getByText("3 °C")).toBeTruthy();
    expect(screen.getByText("Fan (dell-smm-fan1)")).toBeTruthy();
    expect(screen.getByText("2 of 2 (full speed)")).toBeTruthy();
  });

  it("cooling levels in plain words", () => {
    expect(coolingText({ kind: "TCC Offset", count: 1, level: 8, max_level: 63 }).now).toBe("8 °C below the chip's maximum");
    expect(coolingText({ kind: "Processor", count: 12, level: 0, max_level: 3 }).now).toBe("Not limiting");
  });

  it("slowdowns per interval from a counter, and a restart does not count as negative", () => {
    const at = (n: number, value: number) => ({ name: "cpu.throttle_total", value, at: n, unit: "count" as const });
    const h = perInterval({ range: "24h", series: [[at(1, 100), at(2, 150), at(3, 4)]] });
    expect(h?.series[0].map((p) => p.value)).toEqual([50, 0]);
  });
});
