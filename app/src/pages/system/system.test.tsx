// SPDX-License-Identifier: Apache-2.0
// Renders the System and Processes pages with realistic agent answers and with gaps.

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ProcessesAll, ProcessesTop } from "../processes/Processes";
import { SystemBoot } from "./Boot";
import { SystemKernel } from "./Kernel";
import { SystemOverview } from "./Overview";
import { SystemTime } from "./Time";
import { SystemUpdates } from "./Updates";

const at = Date.now();
let boot: unknown = null;
let updates: unknown = null;

const OS = {
  distribution: "Ubuntu 24.04.1 LTS (Noble Numbat)",
  lifecycle: { release: "2024-04-25", eol: "2029-05-31", eol_esm: "2034-04-25" },
  virtualization: "None (bare metal)",
  machine_id: "4f1c…a72e",
  users_signed_in: 1,
  pro_attached: false,
  livepatch: "disabled",
  timezone: "Asia/Kolkata",
  ntp_synchronized: true,
  time_server: "ntp.ubuntu.com (default)",
  rtc_local: false,
  locale: "LANG=en_US.UTF-8",
  keyboard: "us",
};
const PROCESSES = [
  { pid: 4121, name: "chrome", user: "riya", state: "S", cpu_percent: 4.2, memory_bytes: 842e6, swap_bytes: 0, threads: 38, command: "/opt/google/chrome/chrome", started_at: at },
  { pid: 1874, name: "gnome-shell", user: "riya", state: "R", cpu_percent: 3.8, memory_bytes: 410e6, swap_bytes: 0, threads: 24, command: "/usr/bin/gnome-shell" },
  { pid: 1, name: "systemd", user: "root", state: "S", cpu_percent: 0, memory_bytes: 14e6, swap_bytes: 0, threads: 1, command: "/sbin/init splash" },
];

vi.mock("../../api", () => ({
  getLatest: () => Promise.resolve([{ name: "cpu.load_1m", value: 0.84, unit: "count", at }, { name: "cpu.load_5m", value: 0.92, unit: "count", at }, { name: "cpu.load_15m", value: 1.1, unit: "count", at }, { name: "services.running", value: 142, unit: "count", at }]),
  getMachine: () => Promise.resolve({ hostname: "riya-laptop", os: "Ubuntu", user: "riya", desktop: "ubuntu:GNOME", session_type: "wayland" }),
  getMetrics: () => Promise.resolve([28, 30, 41.2].map((value, i) => ({ name: "boot.time", value, unit: "seconds", at: at - (2 - i) * 86_400_000 }))),
  getSnapshot: (kind: string) => Promise.resolve(kind === "boot" ? boot : updates),
  getInventory: (kind: string) =>
    Promise.resolve(
      {
        system: { hostname: "riya-laptop", os: "Ubuntu 24.04.1 LTS", kernel: "6.8.0-47-generic", arch: "x86_64", cpu_threads: 12, uptime_seconds: 45_360, reboot_required: true, reboot_packages: ["libc6", "openssl"] },
        os: OS,
        kernel: {
          running: "6.8.0-47-generic",
          tainted: 0,
          command_line: "BOOT_IMAGE=/vmlinuz-6.8.0-47-generic ro quiet splash",
          installed: [{ version: "6.8.0-47-generic", status: "running", size: 312e6 }, { version: "6.8.0-45-generic", status: "fallback", size: 310e6 }],
          modules: [{ name: "i915", size: 4.2e6, used: 34, used_by: [] }],
        },
        update_history: [{ start: "2026-10-23  00:34:58", source: "unattended-upgrades", packages: 3, changes: ["upgrade openssl"], failed: false }],
        processes: PROCESSES,
      }[kind],
    ),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

beforeEach(() => {
  boot = null;
  updates = null;
});
afterEach(cleanup);

describe("System pages", () => {
  it("overview shows OS, status, and the release lifecycle", async () => {
    boot = { data: { boot_id: "x", booted_at: at - 45_360_000, systemd: "255.4-1ubuntu8", times: { kernel: 3.9, userspace: 26.8, total: 41.2 }, slowest: [] } };
    render(<SystemOverview />);
    await waitFor(() => expect(screen.getByText("Ubuntu 24.04.1 LTS (Noble Numbat)")).toBeTruthy());
    expect(screen.getByText("GNOME · Wayland")).toBeTruthy();
    await waitFor(() => expect(screen.getByText("systemd 255.4-1ubuntu8")).toBeTruthy());
    expect(screen.getByText("Yes · libc6, openssl")).toBeTruthy();
    expect(screen.getByText("Not attached")).toBeTruthy();
    expect(screen.getByText(/Standard support ends/)).toBeTruthy();
  });

  it("boot waits honestly, then shows phases and history", async () => {
    render(<SystemBoot />);
    await waitFor(() => expect(screen.getByText(/once the boot has finished/)).toBeTruthy());
    cleanup();
    boot = { data: { boot_id: "x", booted_at: at, times: { firmware: 8.1, loader: 2.4, kernel: 3.9, userspace: 26.8, total: 41.2 }, slowest: [{ name: "dev-mapper-ubuntu\\x2dvg.device", seconds: 2.2 }] } };
    render(<SystemBoot />);
    await waitFor(() => expect(screen.getByText("41.2")).toBeTruthy());
    expect(screen.getByText("Average 29.0 s")).toBeTruthy();
    expect(screen.getByText("dev-mapper-ubuntu-vg.device")).toBeTruthy();
  });

  it("updates list security first and show history", async () => {
    updates = { data: { pending: [{ name: "openssl", installed: "1", candidate: "2", origin: "noble-security", security: true }], lists_updated_at: at - 7_200_000, automatic_security: true } };
    render(<SystemUpdates />);
    await waitFor(() => expect(screen.getByText("openssl")).toBeTruthy());
    expect(screen.getByText("2 h ago")).toBeTruthy();
    await waitFor(() => expect(screen.getByText("unattended-upgrades")).toBeTruthy());
  });

  it("kernel and time render", async () => {
    render(<SystemKernel />);
    await waitFor(() => expect(screen.getByText("Fallback")).toBeTruthy());
    cleanup();
    render(<SystemTime />);
    await waitFor(() => expect(screen.getByText(/^Asia\/Kolkata/)).toBeTruthy());
    expect(screen.getByText("Imperial · °F")).toBeTruthy();
  });
});

describe("Processes pages", () => {
  it("counts states, filters by user, and inspects a row", async () => {
    render(<ProcessesAll />);
    await waitFor(() => expect(screen.getByText("chrome")).toBeTruthy());
    fireEvent.click(screen.getByText("chrome"));
    expect(screen.getByRole("dialog", { name: "chrome" })).toBeTruthy();
    expect(screen.getByText("/opt/google/chrome/chrome")).toBeTruthy();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("dialog", { name: "chrome" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "root" }));
    expect(screen.queryByText("chrome")).toBeNull();
    expect(screen.getByText("systemd")).toBeTruthy();
  });

  it("top consumers list by CPU and memory", async () => {
    render(<ProcessesTop />);
    await waitFor(() => expect(screen.getAllByText("chrome (4121)").length).toBe(2));
  });
});
