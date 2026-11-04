// SPDX-License-Identifier: Apache-2.0
// Optimize lists only what was measured as needed, fixes open the preview, the path counts what
// was looked at, every other check shows what it measured, a scan shows each part live, and one
// scan serves every view. DeeCure checks by itself only once a day.

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { OptimizeCheck } from "../../api";
import { FixContext } from "../../shell/FixDrawer";
import { OptimizeStrip } from "../overview/OptimizeStrip";
import { OptimizeAll, areaOf, offered, resetSeen } from "./Optimize";
import { optimizeChanged, resetOptimizeScan } from "./scan";

let checks: OptimizeCheck[] = [];
const calls: string[] = [];
/** A part kind whose answer waits until `release` is called. */
let hold: string | null = null;
let release: (() => void) | null = null;
let fail: Record<string, unknown> = {};

vi.mock("../../api", () => ({
  getInventory: (kind: string) => {
    calls.push(kind);
    if (fail[kind]) return Promise.reject(fail[kind]);
    // Every check comes from the first part; the other parts find nothing.
    const answer = kind === "optimize" || kind === "optimize_boot" ? checks : [];
    if (kind === hold) return new Promise((done) => (release = () => done(answer)));
    return Promise.resolve(answer);
  },
  toAppError: (e: unknown) => (typeof e === "object" && e !== null && "kind" in e ? e : { kind: "other", message: String(e) }),
}));

const opened: [string, string | undefined][] = [];
const withFix = (ui: React.ReactNode) => render(<FixContext.Provider value={(a, s) => opened.push([a, s])}>{ui}</FixContext.Provider>);

afterEach(() => {
  cleanup();
  opened.length = 0;
  calls.length = 0;
  hold = null;
  release = null;
  fail = {};
  resetOptimizeScan();
  resetSeen();
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

const keep = (kept: OptimizeCheck[], at: number) => window.localStorage.setItem("deecure.optimize", JSON.stringify({ checks: kept, at }));

describe("Optimize", () => {
  it("offers measured fixes first, then advice, and opens the preview", async () => {
    checks = [HEAT, TRIM, WAIT];
    expect(offered(checks).map((c) => c.id)).toEqual(["boot.wait_online", "hardware.heat"]);
    withFix(<OptimizeAll />);
    await waitFor(() => expect(screen.getByText(WAIT.title)).toBeTruthy());
    expect(screen.getByText("2 improvements for this computer")).toBeTruthy();
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

  it("walks through every item and counts what was looked at", async () => {
    checks = [WAIT, HEAT];
    withFix(<OptimizeAll />);
    await waitFor(() => expect(screen.getByText("0 of 2")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Next" }));
    expect(screen.getByText("1 of 2")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Got it" }));
    expect(screen.getByText("You went through every item")).toBeTruthy();
    // The last card has nothing left to step to.
    expect(screen.queryByRole("button", { name: "Got it" })).toBeNull();
  });

  it("filters by area when items come from more than one", async () => {
    checks = [WAIT, HEAT];
    withFix(<OptimizeAll />);
    await waitFor(() => expect(screen.getByRole("button", { name: /Hardware/ })).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: /Hardware/ }));
    expect(screen.queryByText(WAIT.title)).toBeNull();
    expect(screen.getByText(HEAT.title)).toBeTruthy();
  });

  it("says so when nothing is needed", async () => {
    checks = [TRIM];
    withFix(<OptimizeAll />);
    await waitFor(() => expect(screen.getByText("Nothing to improve now")).toBeTruthy());
    expect(screen.queryByRole("button", { name: "Preview" })).toBeNull();
    expect(screen.queryByText(/Your path/)).toBeNull();
  });

  it("shows each part live, and the page and the Overview share one scan", async () => {
    checks = [WAIT];
    hold = "optimize_disks";
    withFix(
      <>
        <OptimizeAll />
        <OptimizeStrip />
      </>,
    );
    await waitFor(() => expect(screen.getAllByText("Part 2 of 8: Disks")).toHaveLength(2));
    expect(screen.getByRole("list", { name: "Parts of the check" })).toBeTruthy();
    // The first part is done and says what it found; the found item shows at once.
    expect(screen.getByText("1 to improve")).toBeTruthy();
    expect(screen.getByText(/found 1 improvement so far/)).toBeTruthy();
    expect(screen.getByText(WAIT.title)).toBeTruthy();
    expect(screen.getAllByText("Waiting")).toHaveLength(6);
    release?.();
    await waitFor(() => expect(screen.getByText("1 improvement for this computer")).toBeTruthy());
    expect(screen.getByText(/improvement ready · 1 fix/)).toBeTruthy();
    expect(calls.filter((k) => k === "optimize_boot")).toHaveLength(1);
    expect(calls).toHaveLength(8);
  });

  it("asks an older agent for everything in one call", async () => {
    checks = [WAIT];
    fail = { optimize_boot: { kind: "agent_outdated", message: "older" } };
    withFix(<OptimizeAll />);
    await waitFor(() => expect(screen.getByText(WAIT.title)).toBeTruthy());
    expect(calls).toEqual(["optimize_boot", "optimize"]);
  });

  it("says which part could not be checked, and shows the rest", async () => {
    checks = [WAIT];
    fail = { optimize_drivers: { kind: "other", message: "fwupd did not answer" } };
    withFix(<OptimizeAll />);
    await waitFor(() => expect(screen.getByText("1 part could not be checked")).toBeTruthy());
    expect(screen.getByText("Drivers and firmware: fwupd did not answer")).toBeTruthy();
    expect(screen.getByText(WAIT.title)).toBeTruthy();
  });

  it("does not check by itself when the kept answer is less than a day old", async () => {
    keep([WAIT], Date.now() - 3_600_000);
    resetOptimizeScan(true);
    withFix(<OptimizeAll />);
    expect(screen.getByText(WAIT.title)).toBeTruthy();
    expect(calls).toEqual([]);
    // After a fix, it says the list may be out of date, and still does not check.
    optimizeChanged();
    await waitFor(() => expect(screen.getByText("A fix ran after this check")).toBeTruthy());
    expect(calls).toEqual([]);
    // The button checks now.
    fireEvent.click(screen.getByRole("button", { name: "Check again" }));
    await waitFor(() => expect(calls).toHaveLength(8));
  });

  it("checks by itself when the kept answer is a day old", async () => {
    checks = [HEAT];
    keep([WAIT], Date.now() - 86_400_000 - 1000);
    resetOptimizeScan(true);
    withFix(<OptimizeAll />);
    await waitFor(() => expect(screen.getByText(HEAT.title)).toBeTruthy());
    expect(calls).toHaveLength(8);
  });

  it("puts an ID it does not know under Other", () => {
    expect(areaOf({ ...TRIM, id: "new.thing:x" }).label).toBe("Other");
    expect(areaOf({ ...TRIM, id: "power.device_saving:usb1" }).label).toBe("Power");
  });
});

