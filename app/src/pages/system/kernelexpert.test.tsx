// SPDX-License-Identifier: Apache-2.0
// Expert kernel sections: who set each setting, limits, modules, DKMS gaps, and time sync.

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { KernelDetail } from "../../api";
import { KernelExpertSections, missingKernels } from "./KernelExpert";

const d: KernelDetail = {
  settings: [
    { key: "kernel.yama.ptrace_scope", value: "1", now: "1", file: "/etc/sysctl.d/10-ptrace.conf", line: 2, package: "procps", file_changed: false, overridden: false },
    { key: "vm.swappiness", value: "60", now: "10", file: "/usr/lib/sysctl.d/50-default.conf", line: 1, package: "systemd", file_changed: false, overridden: true },
    { key: "vm.swappiness", value: "10", now: "10", file: "/etc/sysctl.d/99-me.conf", line: 1, file_changed: false, overridden: false },
    { key: "kernel.printk", value: "4 4 1 7", now: "3 4 1 7", file: "/etc/sysctl.d/10-console.conf", line: 1, package: "procps", file_changed: true, overridden: false },
  ],
  watch_limit: 65536,
  watches_used: 60000,
  watch_users: [{ pid: 5, name: "code", watches: 50000 }],
  open_files: 16580,
  open_files_limit: 9223372036854775807,
  modules: [{ name: "nvidia", size: 1e6, taint: "POE", used_by: [], devices: 1 }, { name: "i915", size: 4e6, used_by: ["kvmgt"], devices: 1 }],
  dkms: [{ module: "nvidia", version: "535", kernels: ["6.8.0-45-generic"] }],
  kernels: ["6.8.0-45-generic", "6.8.0-49-generic"],
  lockdown: "integrity",
  time_sync: { Server: "ntp.ubuntu.com", Offset: "+7.740ms" },
};

vi.mock("../../api", () => ({
  getInventory: () => Promise.resolve(d),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(cleanup);

describe("Expert kernel sections", () => {
  it("settings say who set them and what is in effect", async () => {
    render(<KernelExpertSections />);
    await waitFor(() => expect(screen.getByText("4 settings · 2 set or changed on this computer")).toBeTruthy());
    expect(screen.getByText("No: a later file sets it")).toBeTruthy();
    expect(screen.getByText("procps, file changed here")).toBeTruthy();
    expect(screen.getByText("3 4 1 7 (changed while running)")).toBeTruthy();
    expect(screen.getAllByText("This computer").length).toBeGreaterThan(0);
  });

  it("limits, modules, DKMS gaps, and time sync", async () => {
    render(<KernelExpertSections />);
    await waitFor(() => expect(screen.getByText("92")).toBeTruthy());
    expect(screen.getByText("no system-wide limit")).toBeTruthy();
    expect(screen.getByText("POE")).toBeTruthy();
    expect(screen.getByText("6.8.0-49-generic")).toBeTruthy();
    expect(screen.getAllByText("+7.740ms")).toHaveLength(2);
    expect(missingKernels(d, { kernels: ["6.8.0-45-generic", "6.8.0-49-generic"] })).toEqual([]);
  });
});
