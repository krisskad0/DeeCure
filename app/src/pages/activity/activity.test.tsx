// SPDX-License-Identifier: Apache-2.0
// The Activity pages, from stored history events, including the journey
// "Something broke after a change" (docs/01-product/user-journey.md).

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { HistoryEvent } from "../../api";
import { resetKb } from "../../kb";
import { ActivityConfig, ActivityDevices, ActivityPackages, ActivityTimelinePage, groupChanges, newKernel } from "./Activity";

const now = Date.now();
const H = 3_600_000;
const run = now - 26 * H;
let id = 0;
const ev = (e: Partial<HistoryEvent> & Pick<HistoryEvent, "kind" | "at" | "title">): HistoryEvent => ({ id: ++id, source: "journal", ...e });

const EVENTS: HistoryEvent[] = [
  ev({ kind: "fix_applied", at: now - 1 * H, title: "Turn off cups-browsed.service", source: "DeeCure", detail: "DeeCure · can be undone", data: { run_id: 7, undoable: true, status: "succeeded" } }),
  ev({ kind: "boot", at: now - 2 * H, title: "Started with a new kernel 6.11.0-9-generic (was 6.8.0-47-generic)", data: { kernel_changed: true, kernel: "6.11.0-9-generic", previous_kernel: "6.8.0-47-generic" } }),
  ev({ kind: "crash", at: now - 3 * H, title: "bluetoothd crashed (14 times)", data: { signal: "SIGSEGV", count: 14 } }),
  ev({ kind: "config_changed", at: now - 5 * H, title: "Changed /etc/apt/sources.list.d/vscode.list", subject: "/etc/apt/sources.list.d/vscode.list", source: "apt", data: { change: "changed", added: 1, removed: 0, count: 1 } }),
  ev({ kind: "config_changed", at: now - 5 * H + 20_000, title: "Added /etc/apt/keyrings/vscode.gpg", subject: "/etc/apt/keyrings/vscode.gpg", source: "apt", data: { change: "added", count: 1 } }),
  ev({ kind: "device_added", at: now - 6 * H, title: "Ultra Fit connected", subject: "Ultra Fit", source: "kernel", data: { bus: "USB", id: "0781:5583", count: 1 } }),
  ev({ kind: "package_installed", at: run, title: "Installed linux-image-6.11.0-9-generic 6.11.0-9.9", subject: "linux-image-6.11.0-9-generic", source: "Software Updater", data: { action: "install", format: "deb", to: "6.11.0-9.9", run, kernel: true } }),
  ev({ kind: "package_upgraded", at: run, title: "Upgraded linux-firmware a → b", subject: "linux-firmware", source: "Software Updater", data: { action: "upgrade", format: "deb", from: "a", to: "b", run } }),
  ev({ kind: "package_upgraded", at: now - 30 * H, title: "Updated snap code 1.94.2", subject: "code", source: "snap", data: { action: "upgrade", format: "snap", to: "1.94.2" } }),
  ev({ kind: "unclean_shutdown", at: now - 40 * H, title: "Unclean shutdown", detail: "Power loss, a freeze, or a forced power-off" }),
];

let events: HistoryEvent[] = EVENTS;
const undoFix = vi.fn((_id: number) => Promise.resolve(["Turned cups-browsed.service back on"]));

vi.mock("../../api", () => ({
  getEvents: (_since: number, opts: { kinds?: string[] } = {}) => Promise.resolve({ events: events.filter((e) => !opts.kinds || opts.kinds.includes(e.kind)), truncated: false }),
  getEvent: (eventId: number) => Promise.resolve({ ...events.find((e) => e.id === eventId), data: { change: "changed", added: 1, removed: 0, diff: "  deb old\n+ deb [arch=amd64] https://packages.microsoft.com/repos/code stable main\n" } }),
  getInventory: (kind: string) =>
    Promise.resolve(kind === "kb" ? { services: [], ports: [], errors: [], packages: [], topics: [{ id: "topics/system/previous-kernel", title: "Start once with the previous kernel", summary: "s", steps: ["Restart the computer."] }], rules: {} } : null),
  getFixRuns: () => Promise.resolve([]),
  undoFix: (n: number) => undoFix(n),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

beforeEach(() => {
  events = EVENTS;
  resetKb();
});
afterEach(cleanup);

describe("grouping", () => {
  it("joins one apt run and config files changed together", () => {
    const rows = groupChanges(EVENTS);
    const titles = rows.map((r) => r.title);
    expect(titles).toContain("Changed linux-image-6.11.0-9-generic, linux-firmware");
    expect(titles).toContain("Changed /etc/apt/sources.list.d/vscode.list, /etc/apt/keyrings/vscode.gpg");
    expect(titles).toContain("Updated snap code 1.94.2");
    expect(rows.find((r) => r.type === "Configuration")?.detail).toBe("Configuration · 2 files · apt");
    expect(rows).toHaveLength(8);
  });

  it("finds the newest kernel change", () => {
    expect(newKernel(EVENTS)?.kind).toBe("boot");
    expect(newKernel(EVENTS.filter((e) => e.kind !== "boot"))?.subject).toBe("linux-image-6.11.0-9-generic");
    expect(newKernel([])).toBeUndefined();
  });
});

describe("Activity", () => {
  it("something broke after a change: the timeline shows the kernel, a tip, and Undo", async () => {
    render(<ActivityTimelinePage />);
    await waitFor(() => expect(screen.getByText(/Started with a new kernel 6.11.0-9-generic/)).toBeTruthy());
    expect(screen.getByText(/A new kernel arrived/)).toBeTruthy();
    await waitFor(() => expect(screen.getByText(/Start once with the previous kernel/)).toBeTruthy());
    expect(screen.getByText("bluetoothd crashed (14 times)")).toBeTruthy();
    expect(screen.getByText("Unclean shutdown")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Undo" }));
    await waitFor(() => expect(undoFix).toHaveBeenCalledWith(7));
    await waitFor(() => expect(screen.getByText("Undone")).toBeTruthy());
  });

  it("no tip without a new kernel, and an empty week says so", async () => {
    events = [];
    render(<ActivityTimelinePage />);
    await waitFor(() => expect(screen.getByText("No changes in the last 7 days.")).toBeTruthy());
    expect(screen.queryByText(/A new kernel arrived/)).toBeNull();
  });

  it("config changes open a diff", async () => {
    render(<ActivityConfig />);
    await waitFor(() => expect(screen.getAllByText("/etc/apt/sources.list.d/vscode.list").length).toBeGreaterThan(0));
    fireEvent.click(screen.getAllByRole("button", { name: "View" })[0]);
    await waitFor(() => expect(screen.getByText(/packages.microsoft.com/)).toBeTruthy());
    expect(screen.getByText(/1 lines added/)).toBeTruthy();
  });

  it("package and device tabs render stored events", async () => {
    for (const [Tab, text] of [
      [ActivityPackages, "linux-firmware"],
      [ActivityPackages, "snap"],
      [ActivityDevices, "Ultra Fit"],
    ] as const) {
      render(<Tab />);
      await waitFor(() => expect(screen.getAllByText(text).length).toBeGreaterThan(0));
      cleanup();
    }
  });
});
