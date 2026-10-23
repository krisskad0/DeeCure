// SPDX-License-Identifier: Apache-2.0
// Renders the Overview pages with realistic agent answers, to catch mistakes a type check misses.

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Metric, StatusReport } from "../../api";
import { OverviewScore } from "./HealthScore";
import { OverviewIssues } from "./Issues";
import { OverviewSummary } from "./Summary";

const at = Date.now();
const metric = (name: string, value: number, unit: Metric["unit"], subject?: string): Metric => ({ name, subject, value, unit, at });
const LATEST: Metric[] = [
  metric("cpu.usage", 17.6, "percent"),
  metric("cpu.load_1m", 1.51, "count"),
  metric("memory.used", 9_197_629_440, "bytes"),
  metric("memory.total", 33_320_591_360, "bytes"),
  metric("memory.used_percent", 27.6, "percent"),
  metric("disk.used_percent", 94.1, "percent", "/"),
  metric("disk.free", 14_000_000_000, "bytes", "/"),
  metric("net.rx", 1_200_000, "bytes_per_second"),
  metric("net.tx", 300_000, "bytes_per_second"),
  metric("battery.charge", 66, "percent", "BAT0"),
  metric("battery.health", 79.3, "percent", "BAT0"),
  metric("battery.time_left", 11_400, "seconds", "BAT0"),
  metric("temp.cpu", 57, "celsius"),
  metric("services.running", 37, "count"),
];

vi.mock("../../api", () => ({
  getWeeklyReports: () => Promise.resolve([{ created_at: Date.now(), since: Date.now() - 7 * 86_400_000, until: Date.now(), host: "riya-laptop", os: "Ubuntu", score: 82, score_trend: [], findings: [], changes: { packages: 12, config: 3, devices: 0, crashes: 0, failed_services: 0, boots: 2, unclean_shutdowns: 0, fixes: 0 }, highlights: [], disks: [], tip: "Set up backups." }]),
  getEvents: () => Promise.resolve({ events: [{ id: 1, kind: "boot", at: Date.now() - 3_600_000, title: "Computer started", source: "journal", detail: "Kernel 6.8.0-47-generic" }], truncated: false }),
  getLatest: () => Promise.resolve(LATEST),
  getMetrics: (name: string) => Promise.resolve([0, 1, 2].map((i) => ({ ...metric(name, 10 + i, "percent"), at: at - (2 - i) * 60_000 }))),
  getSnapshot: () => Promise.resolve({ kind: "filesystems", taken_at: at, data: [{ mount_point: "/", device: "/dev/nvme0n1p2", fs_type: "ext4", size: 238_000_000_000 }] }),
  getInventory: (kind: string) =>
    Promise.resolve(
      kind === "system"
        ? { hostname: "riya-laptop", os: "Ubuntu 24.04.1 LTS", kernel: "6.8.0-47-generic", arch: "x86_64", cpu_threads: 12, uptime_seconds: 45_360, reboot_required: true, reboot_packages: ["libc6"] }
        : kind === "processes"
          ? [{ pid: 1, name: "firefox", user: "riya", state: "S", cpu_percent: 4.2, memory_bytes: 842_000_000, threads: 80, command: "firefox" }]
          : kind === "errors"
            ? { total: 1, groups: [], times: [Date.now()] }
            : kind === "boots"
              ? [{ started: Date.now() - 3_600_000, ended: Date.now(), shutdown: "running", kernel: "6.8.0-47-generic" }]
              : [],
    ),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

const REPORT: StatusReport = {
  generated_at: at,
  checked_at: at,
  checked: ["storage", "services", "power"],
  score: 80,
  deductions: [{ category: "storage", points: 15 }, { category: "power", points: 5 }],
  findings: [
    { rule: "storage.root_almost_full", subject: "/", severity: "critical", state: "open", title: "Main disk is almost full", explain: "Only 14 GB left.", first_seen: at, last_seen: at },
    { rule: "power.battery_worn", subject: "BAT0", severity: "warning", state: "open", title: "Battery has lost capacity", explain: "79%.", first_seen: at, last_seen: at },
  ],
};

afterEach(cleanup);

describe("Overview pages", () => {
  it("summary shows real values in every widget", async () => {
    render(<OverviewSummary report={REPORT} error={null} now={at} />);
    await waitFor(() => expect(screen.getByText("riya-laptop")).toBeTruthy());
    expect(screen.getByText("8.6")).toBeTruthy(); // memory used, GB
    expect(screen.getByText("Health 79% · 3 h 10 m left")).toBeTruthy();
    expect(screen.getByText("CPU 57 °C")).toBeTruthy();
    // Areas without checks say so, instead of a worrying "Unknown".
    expect(screen.getAllByText("Not checked").length).toBeGreaterThan(0);
    expect(screen.getByText("firefox")).toBeTruthy();
    expect(screen.getByText("−15")).toBeTruthy();
    expect(screen.getByText("Main disk is almost full")).toBeTruthy();
    await waitFor(() => expect(screen.getByText("Computer started")).toBeTruthy());
  });

  it("issues counts findings by severity", () => {
    render(<OverviewIssues report={REPORT} error={null} now={at} />);
    expect(screen.getAllByText("Critical").length).toBeGreaterThan(0);
    expect(screen.getByText("Battery has lost capacity")).toBeTruthy();
  });

  it("score breakdown lists checked areas, worst first", async () => {
    render(<OverviewScore report={REPORT} error={null} now={at} />);
    const cells = screen.getAllByRole("cell").map((c) => c.textContent);
    expect(cells[0]).toBe("Storage");
    expect(cells).toContain("−5");
    expect(cells).toContain("Services");
    await waitFor(() => expect(screen.getByText("Set up backups.")).toBeTruthy());
    expect(screen.getByText("17")).toBeTruthy();
  });

  it("before the first check there is no score", () => {
    render(<OverviewScore report={{ ...REPORT, checked_at: undefined, score: undefined }} error={null} now={at} />);
    expect(screen.getByText("The score appears after DeeCure's first check.")).toBeTruthy();
  });
});
