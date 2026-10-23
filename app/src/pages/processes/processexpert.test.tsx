// SPDX-License-Identifier: Apache-2.0
// The Expert process drawer: origin, sandbox, limits, costs, and what other users' processes hide.

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { ProcessDetail } from "../../api";
import { ProcessExpertDetail, duration, sandbox, unitOf } from "./ProcessExpert";

const base: ProcessDetail = {
  pid: 20,
  name: "Web Content",
  state: "S",
  ancestors: [{ pid: 10, name: "firefox" }, { pid: 1, name: "systemd" }],
  children: [],
  command_line: ["/snap/firefox/x/firefox", "-contentproc"],
  program: "/snap/firefox/x/firefox",
  package: undefined,
  cgroup: "/user.slice/user-1000.slice/user@1000.service/app.slice/snap.firefox.firefox-1.scope",
  cpu_seconds: 3725,
  nice: 5,
  policy: "normal",
  realtime_priority: 0,
  threads: 30,
  top_threads: [{ tid: 21, name: "Renderer", cpu_seconds: 900 }, { tid: 20, name: "Web Content", cpu_seconds: 100 }],
  open_files: 950,
  sockets: 12,
  limits: [{ name: "Max open files", soft: "1024", hard: "524288", unit: "files" }, { name: "Max processes", soft: "63000", hard: "63000", unit: "processes" }],
  apparmor: "snap.firefox.firefox (enforce)",
  capabilities: [],
  seccomp: "filter",
  no_new_privileges: true,
  own_namespaces: ["mnt"],
  rss: 600 * 1024 ** 2,
  pss: 400 * 1024 ** 2,
  uss: 350 * 1024 ** 2,
  major_faults: 7,
  oom_score: 800,
  oom_score_adj: 300,
  disk_read: 0,
  disk_write: 1024,
  gpu_ns: 5e9,
};
let detail: ProcessDetail = base;

vi.mock("../../api", () => ({
  getProcessDetail: () => Promise.resolve(detail),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(() => {
  cleanup();
  detail = base;
});

describe("Expert process drawer", () => {
  it("an own process: origin, sandbox, a file limit close by, and costs", async () => {
    render(<ProcessExpertDetail pid={20} />);
    await waitFor(() => expect(screen.getByText("firefox (10) ← systemd (1)")).toBeTruthy());
    expect(screen.getByText("snap.firefox.firefox-1.scope")).toBeTruthy();
    expect(screen.getByText("Sandboxed (AppArmor: snap.firefox.firefox)")).toBeTruthy();
    expect(screen.getByText("Near its limit")).toBeTruthy();
    expect(screen.getByText("5 (gives way to others)")).toBeTruthy();
    expect(screen.getByText("1h 2m")).toBeTruthy();
    expect(screen.getByText("Renderer (21)")).toBeTruthy();
    expect(screen.getByText("800 (adjusted +300)")).toBeTruthy();
  });

  it("another user's process says what is hidden", async () => {
    detail = { ...base, program: undefined, folder: undefined, open_files: undefined, own_namespaces: undefined, pss: undefined, uss: undefined, disk_read: undefined, disk_write: undefined, gpu_ns: undefined, capabilities: Array.from({ length: 41 }, (_, i) => `c${i}`), apparmor: "unconfined" };
    render(<ProcessExpertDetail pid={1} />);
    await waitFor(() => expect(screen.getAllByText("Hidden: another user's process").length).toBeGreaterThan(2));
    expect(screen.getByText("All 41 (full admin rights)")).toBeTruthy();
    expect(screen.getByText("Not sandboxed")).toBeTruthy();
  });

  it("helpers", () => {
    expect(duration(42)).toBe("42 s");
    expect(duration(200)).toBe("3m 20s");
    expect(unitOf("/system.slice/cups.service")).toBe("cups.service");
    expect(unitOf("/")).toBeUndefined();
    expect(sandbox({ ...base, apparmor: "unconfined", own_namespaces: ["net", "pid"] }).text).toContain("own net, pid space");
  });
});
