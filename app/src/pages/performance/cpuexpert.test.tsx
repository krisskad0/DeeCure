// SPDX-License-Identifier: Apache-2.0
// Expert processor sections: real values, rates from two readings, and honest gaps.

import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { CpuDetail } from "../../api";
import { REFRESH_EVENT } from "../../units";
import { CpuExpertSections, CpuSleepStates, ProcessorExpertSections, irqDevice } from "./CpuExpert";

const core = (cpu: number, throttles: number, deep: number) => ({
  cpu,
  now_khz: 1_800_000,
  min_khz: 400_000,
  max_khz: 4_600_000,
  hardware_max_khz: 4_600_000,
  governor: "powersave",
  energy_preference: "balance_power",
  highest_perf: cpu === 0 ? 59 : 34,
  throttle_count: throttles,
  idle: [
    { name: "C1", latency_us: 1, time_us: 1000, disabled: false },
    { name: "C10", latency_us: 890, time_us: deep, disabled: false },
  ],
});

const reading = (at: number, step: number): CpuDetail => ({
  at,
  driver: "intel_pstate",
  driver_mode: "active",
  turbo: true,
  vendor: "GenuineIntel",
  family: 6,
  model: 186,
  stepping: 3,
  microcode_loaded: "0x4121",
  microcode_on_disk: "0x4123",
  package_throttle_count: 100 + step * 50,
  package_throttle_max_ms: 52,
  cores: [core(0, 10 + step * 5, 9000 + step * 9000), core(1, 4, 9000)],
  caches: [
    { level: 1, kind: "Data", bytes: 48 * 1024, shared_cpus: "0-1", copies: 2 },
    { level: 3, kind: "Unified", bytes: 12 * 1024 * 1024, shared_cpus: "0-1", copies: 2 },
  ],
  power_limits: [{ zone: "package-0", name: "long_term", watts: 15, window_s: 28, enabled: true }],
  flaws: [
    { name: "meltdown", state: "not_affected", detail: "Not affected" },
    { name: "retbleed", state: "vulnerable", detail: "Vulnerable: no microcode" },
  ],
  interrupts: [
    { irq: "147", count: 1000 + step * 5000, device: "IR-PCI-MSI-0000:00:14.3 0-edge iwlwifi" },
    { irq: "LOC", count: 50, device: "Local timer interrupts" },
  ],
});

let step = 0;
vi.mock("../../api", () => ({
  getInventory: () => Promise.resolve(reading(1_000_000 + step * 10_000, step)),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(() => {
  cleanup();
  step = 0;
});

async function secondReading() {
  step = 1;
  await act(async () => {
    window.dispatchEvent(new Event(REFRESH_EVENT));
  });
}

describe("Expert processor sections", () => {
  it("shows heat slowdowns, preferred cores, and rates after a second reading", async () => {
    render(<CpuExpertSections waitingForDisk={["jbd2/nvme0n1p2-8"]} />);
    await waitFor(() => expect(screen.getByText("intel_pstate")).toBeTruthy());
    expect(screen.getByText("100")).toBeTruthy();
    expect(screen.getAllByText("Yes")).toHaveLength(1);
    expect(screen.getByText("jbd2/nvme0n1p2-8")).toBeTruthy();
    await secondReading();
    await waitFor(() => expect(screen.getByText(/5\.0 per second now/)).toBeTruthy());
    expect(screen.getByText("iwlwifi")).toBeTruthy();
    expect(screen.getByText("500")).toBeTruthy();
  });

  it("explains power limits, a newer microcode, and open flaws", async () => {
    render(<ProcessorExpertSections />);
    await waitFor(() => expect(screen.getByText("Long-term (sustained)")).toBeTruthy());
    expect(screen.getByText("15 W")).toBeTruthy();
    expect(screen.getByText(/newer: restart to load it/)).toBeTruthy();
    expect(screen.getByText("2 checked · 1 open")).toBeTruthy();
    expect(screen.getByText("Open")).toBeTruthy();
    expect(screen.getByText("48 KB")).toBeTruthy();
  });

  it("sleep states show the share since start, then now", async () => {
    render(<CpuSleepStates />);
    await waitFor(() => expect(screen.getByText("C10")).toBeTruthy());
    expect(screen.getByText("90.0%")).toBeTruthy();
    await secondReading();
    await waitFor(() => expect(screen.getByText("100.0%")).toBeTruthy());
  });

  it("names the device behind an interrupt line", () => {
    expect(irqDevice("IR-PCI-MSI-0000:00:14.3 0-edge iwlwifi")).toBe("iwlwifi");
    expect(irqDevice("Local timer interrupts")).toBe("Local timer interrupts");
  });
});
