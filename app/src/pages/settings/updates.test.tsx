// SPDX-License-Identifier: Apache-2.0
// The Updates panel: what it says, Check now, Install through the fix preview, and setting up
// automatic updates (which needs the password, through the preview too).

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { UpdateSettings, UpdateStatus } from "../../api";
import { FixContext } from "../../shell/FixDrawer";
import { Updates, updateLine } from "./Updates";

const BASE: UpdateStatus = {
  settings: { check: "daily", channel: "prerelease", auto: false },
  current: "0.1.0-alpha.3",
  checked_at: Date.now() - 3_600_000,
  latest: { version: "0.1.0-alpha.4", prerelease: true, url: "https://github.com/krisskad0/DeeCure/releases/tag/v0.1.0-alpha.4", notes: "Health warnings and updates.\nMore." },
  available: true,
  from_apt: false,
  auto_ready: false,
};

let status: UpdateStatus = BASE;
const saved: UpdateSettings[] = [];
let checks = 0;
vi.mock("../../api", () => ({
  getUpdateStatus: () => Promise.resolve(status),
  setUpdateSettings: (s: UpdateSettings) => {
    saved.push(s);
    return Promise.resolve({ ...status, settings: s });
  },
  checkUpdates: () => {
    checks++;
    return Promise.resolve({ ...status, available: false, latest: undefined });
  },
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

const opened: [string, string | undefined][] = [];
const withFix = (ui: React.ReactNode) => render(<FixContext.Provider value={(a, s) => opened.push([a, s])}>{ui}</FixContext.Provider>);

afterEach(() => {
  cleanup();
  opened.length = 0;
  saved.length = 0;
  checks = 0;
  status = BASE;
});

describe("Updates", () => {
  it("says what it knows in plain words", () => {
    expect(updateLine(BASE)).toBe("DeeCure 0.1.0-alpha.4 is available.");
    expect(updateLine({ ...BASE, available: false })).toBe("DeeCure is up to date.");
    expect(updateLine({ ...BASE, available: false, error: "could not reach GitHub" })).toBe("The last check failed: could not reach GitHub.");
    expect(updateLine({ ...BASE, from_apt: true })).toMatch(/Software Updater/);
    expect(updateLine({ ...BASE, available: false, checked_at: undefined, settings: { ...BASE.settings, check: "off" } })).toBe("Update checks are off.");
  });

  it("installs through the fix preview, and checks now on request", async () => {
    withFix(<Updates />);
    await waitFor(() => expect(screen.getByText("Install 0.1.0-alpha.4")).toBeTruthy());
    expect(screen.getByText("Health warnings and updates.")).toBeTruthy();
    fireEvent.click(screen.getByText("Install 0.1.0-alpha.4"));
    expect(opened).toEqual([["system.update_deecure", "0.1.0-alpha.4"]]);
    fireEvent.click(screen.getByText("Check now"));
    await waitFor(() => expect(checks).toBe(1));
    await waitFor(() => expect(screen.getByText(/DeeCure is up to date/)).toBeTruthy());
  });

  it("automatic updates save the choice and ask for the password through the preview", async () => {
    withFix(<Updates />);
    await waitFor(() => expect(screen.getByRole("combobox", { name: "Install updates" })).toBeTruthy());
    fireEvent.change(screen.getByRole("combobox", { name: "Install updates" }), { target: { value: "auto" } });
    await waitFor(() => expect(saved.at(-1)?.auto).toBe(true));
    await waitFor(() => expect(opened).toEqual([["system.set_auto_update", "on"]]));
    // Not set up until the password was given: the panel says so.
    expect(screen.getByText("Automatic updates are not set up yet")).toBeTruthy();
    // Never: no internet at all, and no automatic updates.
    fireEvent.change(screen.getByRole("combobox", { name: "Check for updates" }), { target: { value: "off" } });
    await waitFor(() => expect(saved.at(-1)).toEqual({ check: "off", channel: "prerelease", auto: false }));
  });
});
