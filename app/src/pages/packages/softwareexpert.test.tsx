// SPDX-License-Identifier: Apache-2.0
// Expert software: package drawer, software health, and the full file check (only on a click).

import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { VerifyAnswer } from "../../api";
import { sourceParts } from "./Packages";
import { FileCheck, PackageExpertDetail, SoftwareHealthExpert, snapGroups, sourceText } from "./SoftwareExpert";

let verify: VerifyAnswer = { running: false };
const asked: string[] = [];

vi.mock("../../api", () => ({
  getInventory: (kind: string) => {
    asked.push(kind);
    if (kind === "package_verify_run") verify = { running: true };
    return Promise.resolve(
      kind === "software_health"
        ? {
            held: [],
            broken: [],
            local: [["libpython3.10-dev", "3.10.20-1+noble1"]],
            snaps: [
              { name: "firefox", version: "131", revision: "5000", tracking: "latest/stable", publisher: "mozilla", notes: [] },
              { name: "firefox", version: "130", revision: "4900", tracking: "latest/stable", publisher: "mozilla", notes: ["disabled"] },
              { name: "code", version: "1.9", revision: "10", publisher: "vscode", notes: ["classic"] },
            ],
            snap_refresh: { timer: "00:00~24:00/4", last: "today at 13:20", next: "today at 23:20" },
            runtimes: [{ id: "org.gnome.Platform", branch: "44", installation: "system", used: false }],
          }
        : verify,
    );
  },
  getPackageDetail: () =>
    Promise.resolve({
      name: "htop",
      fields: { Version: "3.3.0", "Installed-Size": "500", Maintainer: "Ubuntu" },
      manual: true,
      needs: ["libc6", "libncursesw6"],
      needed_by: [],
      installed: "3.3.0",
      candidate: "3.3.1",
      sources: [["3.3.1", ["http://archive.ubuntu.com/ubuntu noble-updates/main amd64 Packages"]], ["3.3.0", ["/var/lib/dpkg/status"]]],
      files: 12,
      changed_files: [],
    }),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(() => {
  cleanup();
  verify = { running: false };
  asked.length = 0;
});

describe("Expert software", () => {
  it("package drawer: newer version, sources, needs, and unchanged files", async () => {
    render(<PackageExpertDetail name="htop" />);
    await waitFor(() => expect(screen.getByText("3.3.1 (update waiting)")).toBeTruthy());
    expect(screen.getByText("this computer (no repository)")).toBeTruthy();
    expect(screen.getByText("libc6, libncursesw6")).toBeTruthy();
    expect(screen.getByText("None: every file matches the package.")).toBeTruthy();
  });

  it("software health: local packages, old snap versions, classic snaps, unused runtimes", async () => {
    render(<SoftwareHealthExpert />);
    await waitFor(() => expect(screen.getByText("1 package: no updates")).toBeTruthy());
    expect(screen.getByText("1 (removed by Storage › Cleanup)")).toBeTruthy();
    expect(screen.getByText("Old version")).toBeTruthy();
    expect(screen.getByText("Classic: no sandbox")).toBeTruthy();
    expect(screen.getByText("Not used by any app")).toBeTruthy();
  });

  it("the file check starts only on the button, and separates unreadable files", async () => {
    render(<FileCheck />);
    await waitFor(() => expect(screen.getByText(/Not checked yet/)).toBeTruthy());
    expect(asked).not.toContain("package_verify_run");
    fireEvent.click(screen.getByRole("button", { name: "Check all files" }));
    await waitFor(() => expect(screen.getByText(/Checking every file at low priority/)).toBeTruthy());
    expect(asked.filter((k) => k === "package_verify_run")).toHaveLength(1);
    verify = {
      running: false,
      finished_at: Date.now(),
      took_s: 37,
      problems: [
        { file: "/usr/bin/ls", problem: "changed", config: false, package: "coreutils" },
        { file: "/etc/sysctl.d/10-ptrace.conf", problem: "changed", config: true, package: "procps" },
        { file: "/boot/vmlinuz", problem: "unreadable", config: false },
      ],
    };
    await act(async () => {
      window.dispatchEvent(new Event("deecure-refresh"));
    });
    await waitFor(() => expect(screen.getByText("1 program or data file differ")).toBeTruthy());
    expect(screen.getByText(/1 configuration file changed \(normal\)/)).toBeTruthy();
    expect(screen.getByText(/1 file only the admin may read, not checked/)).toBeTruthy();
    expect(asked.filter((k) => k === "package_verify_run")).toHaveLength(1);
  });

  it("sources in plain words", () => {
    expect(sourceText("http://archive.ubuntu.com/ubuntu noble-updates/main amd64 Packages")).toBe("archive.ubuntu.com/ubuntu noble-updates/main");
    expect(sourceText("/var/lib/dpkg/status")).toBe("this computer (no repository)");
  });

  it("splits a source line and groups snap versions", () => {
    expect(sourceParts("https://packages.microsoft.com/repos/vscode stable main")).toEqual({ host: "packages.microsoft.com", path: "/repos/vscode", rest: "stable main" });
    expect(sourceParts("deb [arch=amd64] http://archive.ubuntu.com/ubuntu/ noble main")).toEqual({ host: "archive.ubuntu.com", path: "/ubuntu", rest: "noble main" });
    const s = (name: string, version: string, notes: string[] = []) => ({ name, version, revision: "1", tracking: "latest/stable", publisher: "canonical", notes });
    expect(snapGroups([s("core22", "2", []), s("core22", "1", ["disabled"]), s("code", "1.9", ["classic"])])).toEqual([
      { name: "code", version: "1.9", tracking: "latest/stable", classic: true, old: 0 },
      { name: "core22", version: "2", tracking: "latest/stable", classic: false, old: 1 },
    ]);
  });
});
