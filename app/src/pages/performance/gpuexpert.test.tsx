// SPDX-License-Identifier: Apache-2.0
// Expert graphics sections: shares from two readings, the second card, and honest gaps.

import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { GpuDetail, GraphicsStack } from "../../api";
import { REFRESH_EVENT } from "../../units";
import { GpuExpertSections, GraphicsStackSection } from "./GpuExpert";

let step = 0;
let dgpuUser = true;
let stack: GraphicsStack = { session_type: "wayland", mesa_version: "25.2.8", renderer: "Mesa Intel Iris Xe", software_rendering: false, video_drivers: ["iHD"], firmware: ["i915: GuC firmware version 70.36.0"] };

const detail = (): GpuDetail => ({
  at: 1_000_000 + step * 10_000,
  cards: [
    { card: "card0", driver: "i915", pci_address: "0000:00:02.0", boot_vga: true, mhz_now: 0, mhz_min: 100, mhz_max: 1250, power_saving_ms: 1000 + step * 9000, slowed_by: ["pl1"], runtime_status: "active", connectors: [["eDP-1", "connected"]] },
    { card: "card1", driver: "nouveau", pci_address: "0000:01:00.0", boot_vga: false, slowed_by: [], runtime_status: "active", connectors: [] },
  ],
  clients: [
    { pid: 10, name: "chrome", pci_address: "0000:00:02.0", engines: { render: 1e9 + step * 2.5e9, video: 0 }, memory: 128 * 1024 ** 2 },
    ...(dgpuUser ? [{ pid: 11, name: "blender", pci_address: "0000:01:00.0", engines: { render: 5e9 } }] : []),
  ],
});

vi.mock("../../api", () => ({
  getInventory: (kind: string) => Promise.resolve(kind === "gpu_detail" ? detail() : stack),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(() => {
  cleanup();
  step = 0;
  dgpuUser = true;
});

describe("Expert graphics sections", () => {
  it("cards: resting clock, slowdown reason, and use per app from two readings", async () => {
    render(<GpuExpertSections />);
    await waitFor(() => expect(screen.getByText("Resting (0 MHz)")).toBeTruthy());
    expect(screen.getByText("long-term power limit")).toBeTruthy();
    expect(screen.getByText("used by blender")).toBeTruthy();
    step = 1;
    await act(async () => {
      window.dispatchEvent(new Event(REFRESH_EVENT));
    });
    await waitFor(() => expect(screen.getByText("25.0%")).toBeTruthy());
    expect(screen.getByText("90.0% now")).toBeTruthy();
  });

  it("a second card awake with no app of yours is pointed out", async () => {
    dgpuUser = false;
    render(<GpuExpertSections />);
    await waitFor(() => expect(screen.getByText("and no app of yours uses it")).toBeTruthy());
  });

  it("graphics software: the fitting video driver, and gaps", async () => {
    render(<GraphicsStackSection drivers={["i915"]} />);
    await waitFor(() => expect(screen.getByText("iHD installed")).toBeTruthy());
    expect(screen.getByText("Install the vainfo package to list them")).toBeTruthy();
    cleanup();
    stack = { ...stack, video_drivers: [], renderer: "llvmpipe (LLVM 19)", software_rendering: true, firmware: ["Direct firmware load for i915/x.bin failed"] };
    render(<GraphicsStackSection drivers={["i915"]} />);
    await waitFor(() => expect(screen.getByText(/Missing: iHD or i965/)).toBeTruthy());
    expect(screen.getByText(/software, no graphics card/)).toBeTruthy();
    expect(screen.getByText("1 firmware problem")).toBeTruthy();
  });
});
