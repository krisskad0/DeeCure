// SPDX-License-Identifier: Apache-2.0
// Use per app over time: the windows, the bars, and lines that share one time axis.

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import type { AppUsageInfo } from "../../api";
import { CpuByAppPanel, MemoryByAppPanel, TrafficByAppPanels, entries, union } from "./AppUsage";

const GB = 1024 ** 3;
const info: AppUsageInfo = {
  windows: [
    {
      range: "24h",
      metrics: {
        "app.cpu_seconds": [
          { kind: "app", name: "Google Chrome", subject: "app:Google Chrome", total: 8640, average: 30, peak: 120 },
          { kind: "other", name: "", subject: "other", total: 60, average: 1, peak: 2 },
        ],
        "app.memory": [{ kind: "app", name: "Visual Studio Code", subject: "app:Visual Studio Code", total: 0, average: 1.5 * GB, peak: 2 * GB }],
        "app.net_rx": [{ kind: "app", name: "Firefox", subject: "app:Firefox", total: 3 * GB, average: GB, peak: 2 * GB }],
      },
    },
    { range: "7d", metrics: { "app.cpu_seconds": [{ kind: "service", name: "snapd", subject: "service:snapd", total: 3600, average: 5, peak: 9 }] } },
    { range: "30d", metrics: {} },
  ],
};

vi.mock("../../api", () => ({
  getInventory: () => Promise.resolve(info),
  getMetrics: (_name: string, _since: number, _until: number, opts: { subject?: string }) =>
    Promise.resolve([{ name: "app.cpu_seconds", subject: opts.subject, value: 30, at: 1, unit: "seconds" }, { name: "app.cpu_seconds", subject: opts.subject, value: 60, at: 2, unit: "seconds" }]),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

beforeAll(() => Object.defineProperty(navigator, "hardwareConcurrency", { value: 4, configurable: true }));
afterEach(cleanup);

describe("use per app over time", () => {
  it("joins times so an app that was closed does not cut the others", () => {
    expect(union([[{ at: 1, value: 5 }, { at: 3, value: 7 }], [{ at: 2, value: 1 }]])).toEqual({ times: [1, 2, 3], data: [[5, 0, 7], [0, 1, 0]] });
    expect(entries(info, "30d", "app.cpu_seconds")).toEqual([]);
  });

  it("shows CPU time and the average share of the whole CPU", async () => {
    render(<CpuByAppPanel />);
    // 8640 s in a day on 4 cores = 2.5%.
    await waitFor(() => expect(screen.getByText("2 h 24 m · 3% avg")).toBeTruthy());
    expect(screen.getByText("Other apps and services")).toBeTruthy();
    fireEvent.click(screen.getByRole("combobox", { name: "Time" }));
    fireEvent.mouseDown(await screen.findByRole("option", { name: "Last 30 days" }));
    await waitFor(() => expect(screen.getByText(/Nothing recorded in the last 30 days/)).toBeTruthy());
  });

  it("ranks memory by its average and shows the peak", async () => {
    render(<MemoryByAppPanel />);
    await waitFor(() => expect(screen.getByText("1.5 GB avg · 2.0 GB peak")).toBeTruthy());
  });

  it("shows download per app, says TCP only, and an empty upload", async () => {
    render(<TrafficByAppPanels />);
    await waitFor(() => expect(screen.getByText("3.0 GB")).toBeTruthy());
    expect(screen.getAllByText("TCP only").length).toBe(1);
    expect(screen.getByText(/Nothing recorded in the last 24 hours/)).toBeTruthy();
  });
});
