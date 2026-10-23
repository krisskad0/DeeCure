// SPDX-License-Identifier: Apache-2.0
// The fix drawer: preview, untick a step, apply, and check the finding is gone.

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { FixPlan } from "../api";
import { FixDrawer } from "./FixDrawer";

const PLAN: FixPlan = {
  action: "storage.free_space",
  title: "Free up disk space",
  subtitle: "Main disk ( / )",
  rule: "storage.root_almost_full",
  steps: [
    { id: "trash", title: "Empty Trash", description: "312 files", size: 2.1e9, undo: "none", checked: true, command: "gio trash --empty", call: { op: "empty_trash" } },
    { id: "apt", title: "Clean APT package cache", description: "61 files", size: 4e8, undo: "regenerates", checked: true, command: "sudo apt-get clean", call: { op: "clean_apt_cache" } },
  ],
};
const applied = vi.fn();

vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => undefined) }));
vi.mock("../api", async (orig) => ({
  ...(await orig<typeof import("../api")>()),
  getFixPlan: () => Promise.resolve(PLAN),
  applyFix: (plan: FixPlan, chosen: string[]) => {
    applied(chosen);
    return Promise.resolve({ run: { id: 7, action: plan.action, title: plan.title, started_at: 0, finished_at: 1, status: "succeeded", steps: chosen.map((id) => ({ id, title: id, ok: true, message: "done" })) } });
  },
  getStatus: () => Promise.resolve({ generated_at: 0, findings: [] }),
  getHelperStatus: () => Promise.resolve(true),
}));

const setAccess = (level: "basic" | "full") => window.localStorage.setItem("deecure.access", JSON.stringify(level));

afterEach(cleanup);

describe("fix drawer", () => {
  it("previews, lets steps be unticked, applies the rest, and verifies", async () => {
    setAccess("full");
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const done = vi.fn();
    render(<FixDrawer action="storage.free_space" onClose={() => undefined} onDone={done} />);
    await waitFor(() => expect(screen.getByText("Empty Trash")).toBeTruthy());
    expect(screen.getByText("Irreversible")).toBeTruthy();
    expect(screen.getByText("Admin")).toBeTruthy();
    expect(screen.getByText("Admin steps ask for your password once each")).toBeTruthy();
    fireEvent.click(screen.getByRole("checkbox", { name: "Clean APT package cache" }));
    expect(screen.getByText("No password needed")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /Apply · frees 2.0 GB/ }));
    await vi.advanceTimersByTimeAsync(3000);
    vi.useRealTimers();
    await waitFor(() => expect(screen.getByText("DeeCure checked again: the issue is gone.")).toBeTruthy());
    expect(applied).toHaveBeenCalledWith(["trash"]);
    expect(done).toHaveBeenCalled();
  });

  it("in Basic access, admin steps are locked and the rest still works", async () => {
    setAccess("basic");
    applied.mockClear();
    render(<FixDrawer action="storage.free_space" onClose={() => undefined} />);
    await waitFor(() => expect(screen.getByText("Some steps need Full access")).toBeTruthy());
    const apt = screen.getByRole("checkbox", { name: "Clean APT package cache" }) as HTMLInputElement;
    expect(apt.disabled).toBe(true);
    expect(apt.checked).toBe(false);
    expect(screen.getByText("No password needed")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /Apply · frees 2.0 GB/ }));
    await waitFor(() => expect(applied).toHaveBeenCalledWith(["trash"]));
  });
});
