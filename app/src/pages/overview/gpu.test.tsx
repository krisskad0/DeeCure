// SPDX-License-Identifier: Apache-2.0
// The GPU tile and chart line on Overview › Summary: no graphics card, a driver without busy
// time, two cards, and GPU history that starts later than CPU and memory.

import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Gpu, Metric } from "../../api";
import { LineChart } from "../../components/charts/LineChart";
import { gpuName, gpuSeries, gpuTile } from "./Summary";

afterEach(cleanup);

const intel: Gpu = { card: "card1", vendor: "Intel Corporation", model: "Raptor Lake-P [Iris Xe Graphics]", driver: "i915", boot_vga: true };
const nvidia: Gpu = { card: "card0", vendor: "NVIDIA Corporation", model: "AD107M [GeForce RTX 4060 Max-Q / Mobile]", driver: "nvidia", boot_vga: false };
const busy = (card: string, value: number): Metric => ({ name: "gpu.busy", subject: card, value, unit: "percent", at: 1 });
const history = (gpu: Record<string, { at: number; value: number }[]>) => ({ cpu: [], memory: [], rx: [], tx: [], gpu }) as never;

describe("GPU tile", () => {
  it("is not there without a graphics card", () => {
    expect(gpuTile([], [], undefined)).toBeNull();
    expect(gpuTile(undefined, [], undefined)).toBeNull();
  });

  it("shows the busy share and the short name", () => {
    const t = gpuTile([intel], [busy("card1", 23.4)], history({ card1: [{ at: 1, value: 20 }, { at: 2, value: 23 }] }));
    expect(t?.value).toBe("23");
    expect(t?.subtext).toBe("Iris Xe Graphics");
    expect(t?.visual).toEqual({ kind: "spark", data: [20, 23], slot: 4 });
  });

  it("says why a driver gives no reading", () => {
    expect(gpuTile([nvidia], [], undefined)?.subtext).toBe("The NVIDIA driver does not report busy time");
    expect(gpuTile([intel], [], undefined)?.subtext).toBe("Measuring…");
    expect(gpuTile([{ ...intel, driver: "virtio-pci" }], [], undefined)?.subtext).toBe("Its driver does not report busy time");
  });

  it("shows the busiest of two cards", () => {
    const amd: Gpu = { card: "card2", model: "Navi 33 [Radeon RX 7600]", driver: "amdgpu", boot_vga: false };
    const t = gpuTile([intel, amd], [busy("card1", 5), busy("card2", 60)], undefined);
    expect(t?.value).toBe("60");
    expect(t?.subtext).toBe("Radeon RX 7600 · busiest of 2");
  });

  it("names cards in plain words", () => {
    expect(gpuName(intel)).toBe("Iris Xe Graphics");
    expect(gpuName({ card: "card3", boot_vga: false })).toBe("card3");
  });

  it("charts only cards with readings", () => {
    expect(gpuSeries(history({ card1: [{ at: 1, value: 3 }], card0: [] }), [nvidia, intel]).map((g) => g.card)).toEqual(["card1"]);
    expect(gpuSeries(history({}), undefined)).toEqual([]);
  });
});

describe("chart gaps", () => {
  it("draws a line that starts later without breaking the others", () => {
    render(
      <LineChart
        series={[
          { name: "CPU", data: [10, 20, 30, 40], slot: 1 },
          { name: "GPU", data: [Number.NaN, Number.NaN, 5, 8], slot: 4 },
        ]}
        x={{ kind: "time", times: [1, 2, 3, 4], range: "1h" }}
      />,
    );
    const label = screen.getByRole("img").getAttribute("aria-label") ?? "";
    expect(label).toContain("CPU");
    expect(label).toContain("GPU");
    const paths = [...document.querySelectorAll("path")].map((p) => p.getAttribute("d") ?? "");
    expect(paths.some((d) => d.includes("NaN"))).toBe(false);
  });
});
