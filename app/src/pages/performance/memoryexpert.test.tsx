// SPDX-License-Identifier: Apache-2.0
// Expert memory sections: where memory goes, rates, per-app memory, real memory, and gaps.

import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { MemoryDetail } from "../../api";
import { REFRESH_EVENT } from "../../units";
import { MemoryErrors, MemoryExpertSections, breakdown } from "./MemoryExpert";

const GiB = 1024 ** 3;
let step = 0;
let oomd = true;
let controllers = true;

const detail = (): MemoryDetail => ({
  at: 1_000_000 + step * 10_000,
  meminfo: { MemTotal: 16 * GiB, MemFree: 2 * GiB, AnonPages: 6 * GiB, Cached: 5 * GiB, Shmem: GiB, SUnreclaim: 0.5 * GiB },
  vmstat: { pgmajfault: 100 + step * 1000, pswpin: 10, pswpout: 20, oom_kill: 1, pgscan_direct: 0 },
  swaps: [{ path: "/swap.img", kind: "file", size: 8 * GiB, used: 2 * GiB, priority: -2 }],
  zswap: { enabled: false, compressor: "lzo", max_pool_percent: 20 },
  zram: [],
  huge_pages: "madvise",
  huge_pages_defrag: "madvise",
  controllers: controllers ? [{ name: "MC#0", corrected: 3, uncorrected: 1, dimms: [{ label: "DIMM#0", size_mb: 8192, corrected: 3, uncorrected: 1 }] }] : [],
  oomd: oomd ? { swap_limit_percent: 90, pressure_limit_percent: 60, pressure_duration: "20s", swap_watched: [], pressure_watched: [["/user.slice", 50]] } : undefined,
});

vi.mock("../../api", () => ({
  getInventory: (kind: string) =>
    Promise.resolve(
      kind === "memory_detail"
        ? detail()
        : kind === "cgroups"
          ? { at: 1, groups: [{ kind: "app", unit: "app-gnome-code-1.scope", name: "Visual Studio Code", memory: 18 * GiB, memory_own: 2 * GiB, memory_cache: 16 * GiB, memory_pressure: 0 }] }
          : [
              { pid: 7, name: "chrome", rss: GiB, pss: 0.5 * GiB, uss: 0.4 * GiB, major_faults: 3, oom_score: 877 },
              { pid: 1, name: "systemd", rss: 0.1 * GiB, major_faults: 0, oom_score: 0 },
            ],
    ),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(() => {
  cleanup();
  step = 0;
  oomd = true;
  controllers = true;
});

describe("Expert memory sections", () => {
  it("breaks memory down without counting shared memory twice", () => {
    const rows = breakdown({ AnonPages: 6, Cached: 5, Shmem: 1, MemFree: 2 });
    expect(rows.find((r) => r.part === "File cache")?.bytes).toBe(4);
    expect(rows.map((r) => r.part)).not.toContain("Page tables");
  });

  it("shows rates after a second reading, app memory without file cache, and the guard", async () => {
    render(<MemoryExpertSections />);
    await waitFor(() => expect(screen.getByText("Visual Studio Code")).toBeTruthy());
    const row = screen.getByText("Visual Studio Code").closest("tr")?.textContent ?? "";
    expect(row).toContain("2.0 GB");
    expect(row).toContain("16.0 GB");
    expect(row).not.toContain("18.0 GB");
    expect(screen.getByText(/1 of them belong to other users/)).toBeTruthy();
    expect(screen.getByText("60% for 20s")).toBeTruthy();
    expect(screen.getByText("Off")).toBeTruthy();
    step = 1;
    await act(async () => {
      window.dispatchEvent(new Event(REFRESH_EVENT));
    });
    await waitFor(() => expect(screen.getByText("100")).toBeTruthy());
  });

  it("says so when the guard is not running", async () => {
    oomd = false;
    render(<MemoryExpertSections />);
    await waitFor(() => expect(screen.getByText(/systemd-oomd is not running/)).toBeTruthy());
  });

  it("memory errors: counts per module, or an honest gap", async () => {
    render(<MemoryErrors />);
    await waitFor(() => expect(screen.getByText("3 corrected · 1 uncorrected")).toBeTruthy());
    expect(screen.getByText("DIMM#0")).toBeTruthy();
    cleanup();
    controllers = false;
    render(<MemoryErrors />);
    await waitFor(() => expect(screen.getByText(/does not report errors/)).toBeTruthy());
  });
});
