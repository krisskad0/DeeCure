// SPDX-License-Identifier: Apache-2.0
// Optimize lists only what was measured as needed, fixes open the preview, and every other
// check shows what it measured.

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { OptimizeCheck } from "../../api";
import { FixContext } from "../../shell/FixDrawer";
import { OptimizeAll, offered } from "./Optimize";

let checks: OptimizeCheck[] = [];

vi.mock("../../api", () => ({
  getInventory: () => Promise.resolve(checks),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

const opened: [string, string | undefined][] = [];
const withFix = (ui: React.ReactNode) => render(<FixContext.Provider value={(a, s) => opened.push([a, s])}>{ui}</FixContext.Provider>);

afterEach(() => {
  cleanup();
  opened.length = 0;
});

const WAIT: OptimizeCheck = {
  id: "boot.wait_online",
  kind: "fix",
  title: 'Stop "wait for network" from delaying boot',
  measured: "It delayed boot by 5.9 s, and nothing here needs the network before sign-in.",
  offered: true,
  action: "services.disable",
  subject: "NetworkManager-wait-online.service",
  access: "admin",
};
const HEAT: OptimizeCheck = { id: "hardware.heat", kind: "advice", title: "Clean the fans, or check the cooling", measured: "The CPU slowed down from heat 40 times in the last 24 hours.", offered: true, advice: "Dust in the fan or vents is the usual cause." };
const TRIM: OptimizeCheck = { id: "storage.trim", kind: "fix", title: "Turn on the weekly TRIM", measured: "fstrim.timer is enabled: nvme0n1 get TRIM every week.", offered: false, access: "admin" };

describe("Optimize", () => {
  it("offers measured fixes first, then advice, and opens the preview", async () => {
    checks = [HEAT, TRIM, WAIT];
    expect(offered(checks).map((c) => c.id)).toEqual(["boot.wait_online", "hardware.heat"]);
    withFix(<OptimizeAll />);
    await waitFor(() => expect(screen.getByText(WAIT.title)).toBeTruthy());
    expect(screen.getByText(WAIT.measured)).toBeTruthy();
    expect(screen.getByText("Needs your password")).toBeTruthy();
    expect(screen.getByText("Dust in the fan or vents is the usual cause.")).toBeTruthy();
    expect(screen.getAllByRole("button", { name: "Preview" })).toHaveLength(1);
    fireEvent.click(screen.getByRole("button", { name: "Preview" }));
    expect(opened).toEqual([["services.disable", "NetworkManager-wait-online.service"]]);
    // What passed is listed with its measurement.
    expect(screen.getByText("Checked, nothing needed (1)")).toBeTruthy();
    expect(screen.getByText(TRIM.measured)).toBeTruthy();
  });

  it("says so when nothing is needed", async () => {
    checks = [TRIM];
    withFix(<OptimizeAll />);
    await waitFor(() => expect(screen.getByText("Nothing to improve now")).toBeTruthy());
    expect(screen.queryByRole("button", { name: "Preview" })).toBeNull();
  });
});
