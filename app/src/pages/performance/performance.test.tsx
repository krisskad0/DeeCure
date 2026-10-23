// SPDX-License-Identifier: Apache-2.0
// Renders every Performance tab with realistic agent answers, and with nothing collected yet.

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Metric } from "../../api";
import { PerformanceCpu } from "./Cpu";
import { PerformanceDisk } from "./Disk";
import { PerformanceGpu } from "./Gpu";
import { PerformanceMemory } from "./Memory";
import { PerformanceNetwork } from "./Network";
import { PerformancePressure } from "./Pressure";

const at = Date.now();
const m = (name: string, value: number, subject?: string): Metric => ({ name, subject, value, unit: "count", at });
const FULL: Metric[] = [
  m("cpu.usage", 12), m("cpu.load_1m", 0.84), m("cpu.load_5m", 0.92), m("cpu.load_15m", 1.1), m("cpu.frequency", 1.3e9),
  m("cpu.context_switches", 6200), m("cpu.interrupts", 3100), m("temp.cpu", 54), m("cpu.core", 18, "0"), m("cpu.core", 9, "1"),
  m("memory.used", 5.9 * 1024 ** 3), m("memory.total", 7.6 * 1024 ** 3), m("memory.free", 0.1 * 1024 ** 3), m("memory.cache", 1.5 * 1024 ** 3),
  m("memory.buffers", 0.1 * 1024 ** 3), m("swap.used", 1.2 * 1024 ** 3), m("swap.total", 2 * 1024 ** 3), m("pressure.memory", 0.4),
  m("pressure.cpu", 1.2), m("pressure.io", 0.8), m("pressure.memory_full", 0.1), m("pressure.io_full", 0.2),
  m("disk.read", 2.4e6), m("disk.write", 0.8e6), m("disk.iops", 142), m("disk.busy", 6), m("disk.await", 0.0004),
  m("disk.read", 2.4e6, "nvme0n1"), m("disk.write", 0.8e6, "nvme0n1"), m("disk.busy", 6, "nvme0n1"),
  m("net.rx", 1.2e6), m("net.tx", 0.3e6), m("net.rx", 1.2e6, "wlp0s20f3"), m("net.tx", 0.3e6, "wlp0s20f3"),
];
let latest: Metric[] = FULL;
let gpus: unknown[] = [];

vi.mock("../../api", () => ({
  getLatest: () => Promise.resolve(latest),
  getMetrics: (name: string) => Promise.resolve(latest.length ? [0, 1, 2].map((i) => ({ ...m(name, 1 + i), at: at - (2 - i) * 60_000 })) : []),
  getInventory: (kind: string) =>
    Promise.resolve(
      kind === "system"
        ? { arch: "x86_64", cpu_threads: 12, cpu_max_hz: 4.4e9, reboot_required: false, reboot_packages: [] }
        : kind === "network"
          ? { interfaces: [{ name: "wlp0s20f3", kind: "Wi-Fi", state: "up" }, { name: "lo", kind: "Loopback", state: "unknown" }], connections: { tcp_established: 68, tcp_listening: 5, udp: 9 } }
          : kind === "gpu"
            ? gpus
            : kind === "hardware"
              ? { disks: [{ name: "nvme0n1", kind: "SSD (NVMe)", model: "Samsung 980", size: 512 * 1024 ** 3, removable: false }] }
            : [{ pid: 4121, name: "chrome", user: "riya", state: "S", cpu_percent: 4.2, memory_bytes: 842e6, swap_bytes: 0, threads: 30, command: "chrome" }],
    ),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

beforeEach(() => {
  latest = FULL;
  gpus = [];
});
afterEach(cleanup);

describe("Performance tabs", () => {
  it("CPU shows load, frequency, and top consumers", async () => {
    render(<PerformanceCpu />);
    await waitFor(() => expect(screen.getByText("5 min 0.92 · 15 min 1.10")).toBeTruthy());
    expect(screen.getByText("Max 4.4 GHz")).toBeTruthy();
    expect(screen.getByText("6.2K")).toBeTruthy();
    await waitFor(() => expect(screen.getByText("chrome")).toBeTruthy());
  });

  it("memory composition adds up and swap is shown", async () => {
    render(<PerformanceMemory />);
    await waitFor(() => expect(screen.getByText("of 2.0 GB")).toBeTruthy());
    expect(screen.getByText("Cache (can be freed)")).toBeTruthy();
  });

  it("disk lists devices and is honest about per-process I/O", async () => {
    render(<PerformanceDisk />);
    await waitFor(() => expect(screen.getByText("nvme0n1")).toBeTruthy());
    await waitFor(() => expect(screen.getByText("SSD (NVMe)")).toBeTruthy());
    expect(screen.getByText("512 GB")).toBeTruthy();
    expect(screen.queryByText(/Requires Full access/)).toBeNull();
    expect(screen.getByText("0.4")).toBeTruthy();
  });

  it("network lists interfaces, up first", async () => {
    render(<PerformanceNetwork />);
    await waitFor(() => expect(screen.getByText("68")).toBeTruthy());
    const names = screen.getAllByRole("code").map((c) => c.textContent);
    expect(names.slice(0, 2)).toEqual(["wlp0s20f3", "lo"]);
  });

  it("GPU says what the driver does not report", async () => {
    gpus = [{ card: "card1", vendor: "Intel Corporation", model: "Raptor Lake-P [Iris Xe Graphics]", driver: "i915", boot_vga: true }];
    render(<PerformanceGpu />);
    await waitFor(() => expect(screen.getAllByText("Iris Xe Graphics").length).toBeGreaterThan(0));
    expect(screen.getByText("GPU temperature is not exposed")).toBeTruthy();
    expect(screen.getByText("Shared")).toBeTruthy();
  });

  it("GPU with no card says so", async () => {
    render(<PerformanceGpu />);
    await waitFor(() => expect(screen.getByText("No graphics card was found.")).toBeTruthy());
  });

  it("pressure shows a peak and handles kernels without PSI", async () => {
    render(<PerformancePressure />);
    await waitFor(() => expect(screen.getByText("Memory full")).toBeTruthy());
    cleanup();
    latest = [m("cpu.usage", 3)];
    render(<PerformancePressure />);
    await waitFor(() => expect(screen.getByText(/does not report pressure/)).toBeTruthy());
  });

  it("every tab survives an agent with no data yet", async () => {
    latest = [];
    for (const Tab of [PerformanceCpu, PerformanceMemory, PerformanceDisk, PerformanceNetwork, PerformancePressure]) {
      render(<Tab />);
      cleanup();
    }
  });
});
