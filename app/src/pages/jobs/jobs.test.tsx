// SPDX-License-Identifier: Apache-2.0
// Renders the Storage, Services, and Scheduled Jobs pages with realistic answers and gaps.

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Metric } from "../../api";
import { ServicesStartup, ServicesUser } from "../services/OtherServices";
import { SystemServices } from "../services/SystemServices";
import type { StatusReport } from "../../api";
import { ReportContext } from "../../report";
import { StorageCleanup, StorageFilesystems, StorageOverview, StorageUsage, fullIn } from "../storage/Storage";
import { JobsAnacron, JobsAt, JobsCalendar, JobsCron, JobsTimers, shortCommand } from "./Jobs";

const now = Date.now();
const m = (name: string, value: number, subject?: string, at = now): Metric => ({ name, subject, value, unit: "count", at });
let usage: unknown = { scanning: true };

vi.mock("../../api", () => ({
  getLatest: () => Promise.resolve([m("disk.used_percent", 94, "/"), m("disk.used", 224e9, "/"), m("disk.free", 13.8e9, "/"), m("disk.inodes_percent", 21, "/")]),
  // 30 daily points rising 0.7% a day, ending at 94%.
  getMetrics: () => Promise.resolve(Array.from({ length: 30 }, (_, i) => m("disk.used_percent", 73.7 + i * 0.7, "/", now - (29 - i) * 86_400_000))),
  getSnapshot: (kind: string) =>
    Promise.resolve(
      kind === "filesystems"
        ? { data: [{ mount_point: "/", device: "/dev/nvme0n1p2", fs_type: "ext4", size: 238e9 }] }
        : { data: [{ name: "bluetooth.service", description: "Bluetooth", load: "loaded", active: "failed", sub: "failed", restarts: 14 }, { name: "cron.service", description: "Cron", load: "loaded", active: "active", sub: "running" }] },
    ),
  getMachine: () => Promise.resolve({ hostname: "h", os: "o", user: "riya" }),
  getInventory: (kind: string) =>
    Promise.resolve(
      {
        storage: {
          encrypted: false,
          root_chain: ["nvme0n1p2"],
          root_reserved: 11.9e9,
          fstab_entries: 3,
          fstab_problems: [{ line: 3, problem: "UUID=2222 is not connected" }],
          root_forecast: { percent_per_day: 0.7, days_until_full: 8.6 },
          root_history_days: 7,
          candidates: [
            { action: "storage.remove_old_snap_revisions", title: "Remove disabled snap revisions", details: "6 old revisions", size: 8.7e9, undo: "irreversible", needs_password: true },
            { action: "storage.empty_trash", title: "Empty Trash", details: "312 files", size: 2.1e9, undo: "irreversible", needs_password: false },
          ],
        },
        disk_usage: usage,
        service_memory: { "cron.service": 3_000_000 },
        user_services: [{ name: "pipewire.service", description: "PipeWire", active: "active", sub: "running", memory: 18e6 }],
        startup: [{ id: "t.desktop", name: "Telegram Desktop", command: "telegram-desktop -startintray", scope: "User", enabled: true }],
        jobs: {
          cron: [
            { schedule: "*/30 * * * *", command: "~/bin/sync-notes.sh", user: "riya", source: "crontab (riya)" },
            { schedule: "17 * * * *", command: "cd / && run-parts --report /etc/cron.hourly", user: "root", source: "/etc/crontab" },
          ],
          timers: [{ name: "apt-daily.timer", scope: "system", activates: "apt-daily.service", schedule: ["*-*-* 6,18:00"], next: now + 3_600_000, last: now - 3_600_000, result: "exit-code", persistent: true }],
          anacron: [{ id: "cron.daily", period: "1", delay: 5, command: "run-parts /etc/cron.daily", last_run: now - 86_400_000 }],
          at: [],
          at_installed: false,
          runs: [{ at: now - 3_600_000, job: "apt-daily.service", failed: true }, { at: now - 1_800_000, job: "x", failed: false }],
        },
      }[kind],
    ),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

beforeEach(() => {
  usage = { scanning: true };
});
afterEach(cleanup);

describe("Storage", () => {
  it("the Full in tile uses the agent's forecast and says when it needs more readings", () => {
    const base = { encrypted: false, root_chain: [], root_reserved: 0, fstab_entries: 0, fstab_problems: [], candidates: [] };
    expect(fullIn({ ...base, root_forecast: { percent_per_day: 0.7, days_until_full: 8.6 } }, 238e9)).toEqual({ value: "~9", unit: "days", subtext: "at 1.7 GB a day, over the last 3 days", days: 8.6 });
    expect(fullIn({ ...base, root_forecast: { percent_per_day: 0.02 } }, 238e9).value).toBe("Not filling");
    expect(fullIn({ ...base, root_history_days: 1.4 }).subtext).toBe("needs 3 days of readings (1.4 so far)");
    expect(fullIn(undefined).value).toBe("-");
  });

  it("overview shows the agent's low-space warning, forecasts, and waits for the folder scan", async () => {
    const report = {
      findings: [{ rule: "storage.root_almost_full", severity: "warning", state: "open", title: "Main disk is low on space", explain: "Your main disk has 13.8 GB (5.8%) left.", first_seen: now, last_seen: now }],
    } as unknown as StatusReport;
    render(
      <ReportContext.Provider value={report}>
        <StorageOverview />
      </ReportContext.Provider>,
    );
    await waitFor(() => expect(screen.getByText("Main disk is low on space")).toBeTruthy());
    await waitFor(() => expect(screen.getByText("~9")).toBeTruthy());
    expect(screen.getByText(/Scanning folders/)).toBeTruthy();
  });

  it("filesystems check fstab and encryption", async () => {
    render(<StorageFilesystems />);
    await waitFor(() => expect(screen.getByText("1 problem in /etc/fstab")).toBeTruthy());
    expect(screen.getByText("Disk encryption off")).toBeTruthy();
  });

  it("usage shows scan results; cleanup lists candidates without deleting", async () => {
    usage = { scanning: false, scan: { scanned_at: now, seconds: 3, home: [{ path: "/home/riya/Videos", size: 52e9 }], system: [{ path: "/usr", size: 18e9 }], files: [{ path: "/home/riya/Videos/film.mkv", size: 4e9, modified: now }], unreadable: 2, partial: false } };
    render(<StorageUsage />);
    await waitFor(() => expect(screen.getByText("~/Videos")).toBeTruthy());
    expect(screen.getByText("~/Videos/film.mkv")).toBeTruthy();
    cleanup();
    render(<StorageCleanup />);
    await waitFor(() => expect(screen.getByText("Remove disabled snap revisions")).toBeTruthy());
    expect(screen.getByRole("button", { name: "Clean up…" })).toBeTruthy();
  });
});

describe("Services", () => {
  it("system services show live memory", async () => {
    render(<SystemServices />);
    await waitFor(() => expect(screen.getByText("2.9 MB")).toBeTruthy());
  });

  it("user services and startup apps render", async () => {
    render(<ServicesUser />);
    await waitFor(() => expect(screen.getByText("pipewire.service")).toBeTruthy());
    cleanup();
    render(<ServicesStartup />);
    await waitFor(() => expect(screen.getByText("Telegram Desktop")).toBeTruthy());
  });
});

describe("Scheduled jobs", () => {
  it("names commands briefly", () => {
    expect(shortCommand("~/bin/sync-notes.sh")).toBe("sync-notes.sh");
    expect(shortCommand("cd / && run-parts --report /etc/cron.hourly")).toBe("run-parts cron.hourly");
    expect(shortCommand("test -x /usr/sbin/anacron || { cd / && run-parts --report /etc/cron.daily; }")).toBe("run-parts cron.daily");
  });

  it("calendar plans the next 24 hours and counts failures", async () => {
    render(<JobsCalendar />);
    await waitFor(() => expect(screen.getAllByText("sync-notes.sh").length).toBeGreaterThan(0));
    expect(screen.getByText("apt-daily.service")).toBeTruthy();
  });

  it("cron, timers, anacron, and at render", async () => {
    for (const [Tab, text] of [
      [JobsCron, "Every 30 minutes"],
      [JobsTimers, "Failed"],
      [JobsAnacron, "1 day"],
      [JobsAt, "The at tools are not installed, so there are no one-time jobs."],
    ] as const) {
      render(<Tab />);
      await waitFor(() => expect(screen.getAllByText(text).length).toBeGreaterThan(0));
      cleanup();
    }
  });
});
