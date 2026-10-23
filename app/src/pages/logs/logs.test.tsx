// SPDX-License-Identifier: Apache-2.0
// Renders the Logs and Activity pages with realistic answers and with empty ones.

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { heatRows } from "../../metrics";
import { LogsBoots, LogsCrashes, LogsErrors, LogsJournal, LogsKernel, level } from "./Logs";

const now = Date.now();
const H = 3_600_000;
let empty = false;

const DATA: Record<string, unknown> = {
  journal: { volume: Array.from({ length: 24 }, (_, i) => (i === 20 ? 900 : 40)), total: 1600, entries: [{ at: now - 60_000, priority: 4, unit: "systemd", message: "bluetooth.service: Main process exited, code=dumped, status=11/SEGV" }, { at: now - 120_000, priority: 6, unit: "NetworkManager", message: "device (wlp0s20f3): state change" }] },
  errors: { total: 37, times: [now - H, now - 2 * H], groups: [{ message: "bluetooth.service: Main process exited, code=dumped", unit: "systemd", count: 14, last_seen: now - H, class: "attention" }, { message: "ACPI BIOS Error (bug): AE_NOT_FOUND", unit: "kernel", count: 18, last_seen: now - 2 * H, class: "harmless" }] },
  kernel_log: [{ at: now - H, priority: 3, subsystem: "ACPI", message: "ACPI BIOS Error (bug): AE_NOT_FOUND" }],
  crashes: [{ at: now - H, app: "bluetoothd", signal: "SIGSEGV", source: "kernel", count: 14 }],
  boots: [{ started: now - 12 * H, ended: now, shutdown: "running", kernel: "6.8.0-47-generic" }, { started: now - 60 * H, ended: now - 14 * H, shutdown: "unclean", kernel: "6.8.0-45-generic" }],
  device_events: [{ at: now - 2 * H, event: "connected", device: "Ultra USB 3.0", id: "0781:5583" }],
  config_changes: [{ path: "/etc/apt/sources.list.d/vscode.list", modified: now - 3 * H, owner: "root" }, { path: "/etc/apt/sources.list.d/vscode.gpg", modified: now - 3 * H + 1000, owner: "root" }],
  package_history: [
    { start: "2026-10-23  00:34:58", action: "upgrade", name: "openssl", from: "3.0.13-0ubuntu3.4", to: "3.0.13-0ubuntu3.5", source: "unattended-upgrades" },
    { start: "2026-10-23  00:34:58", action: "upgrade", name: "sudo", from: "1", to: "2", source: "unattended-upgrades" },
  ],
};

vi.mock("../../api", () => ({
  getInventory: (kind: string) => Promise.resolve(empty ? (Array.isArray(DATA[kind]) ? [] : { volume: Array(24).fill(0), entries: [], total: 0, groups: [], times: [] }) : DATA[kind]),
  getFixRuns: () => Promise.resolve([]),
  getMetrics: () => Promise.resolve([{ name: "boot.time", value: 41.2, unit: "seconds", at: now - 12 * H }]),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

beforeEach(() => {
  empty = false;
});
afterEach(cleanup);

describe("Logs", () => {
  it("names priorities in words", () => {
    expect(level(3)).toEqual(["crit", "error"]);
    expect(level(4)).toEqual(["warn", "warning"]);
    expect(level(6)).toEqual(["off", "info"]);
  });

  it("builds a 7-day heatmap with today last", () => {
    const rows = heatRows([now, now - 86_400_000], now);
    expect(rows).toHaveLength(7);
    expect(rows[6].values.reduce((a, b) => a + b)).toBe(1);
    expect(rows[5].values.reduce((a, b) => a + b)).toBe(1);
  });

  it("journal, errors, crashes, kernel, and boots render", async () => {
    for (const [Tab, text] of [
      [LogsJournal, "newest 2 of 1,600"],
      [LogsErrors, "Needs attention"],
      [LogsCrashes, "Recurring"],
      [LogsKernel, "ACPI"],
      [LogsBoots, "Unclean"],
    ] as const) {
      render(<Tab />);
      await waitFor(() => expect(screen.getAllByText(text).length).toBeGreaterThan(0));
      cleanup();
    }
  });

  it("empty logs show honest empty states", async () => {
    empty = true;
    render(<LogsCrashes />);
    await waitFor(() => expect(screen.getByText("No crashes in the last 7 days")).toBeTruthy());
    cleanup();
    render(<LogsErrors />);
    await waitFor(() => expect(screen.getByText("No errors in the last 7 days")).toBeTruthy());
  });
});
