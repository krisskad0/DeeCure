// SPDX-License-Identifier: Apache-2.0
// Warnings can be turned off and their limits changed; only changes from the default are saved.

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { WarningRule, WarningSettings } from "../../api";
import { WarningLimits, toSettings } from "./WarningLimits";

const RULES: WarningRule[] = [
  { id: "hardware.cpu_hot", name: "CPU too hot", category: "hardware", off: false, limits: [{ name: "minutes", label: "Minutes near its limit before warning", unit: "minutes", default: 5, value: 5, min: 1, max: 60 }] },
  { id: "network.weak_wifi", name: "Weak Wi-Fi", category: "network", off: false, limits: [{ name: "minutes", label: "Minutes before telling", unit: "minutes", default: 10, value: 15, min: 2, max: 120 }] },
];

const saved: WarningSettings[] = [];
vi.mock("../../api", () => ({
  getWarningSettings: () => Promise.resolve(RULES),
  setWarningSettings: (s: WarningSettings) => {
    saved.push(s);
    return Promise.resolve(
      RULES.map((r) => ({ ...r, off: s.off.includes(r.id), limits: r.limits.map((l) => ({ ...l, value: s.limits[r.id]?.[l.name] ?? l.default })) })),
    );
  },
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(() => {
  cleanup();
  saved.length = 0;
});

describe("Warnings and limits", () => {
  it("saves only what differs from the default", () => {
    expect(toSettings(RULES)).toEqual({ off: [], limits: { "network.weak_wifi": { minutes: 15 } } });
  });

  it("turns a warning off, changes a limit inside its range, and goes back to the default", async () => {
    render(<WarningLimits />);
    await waitFor(() => expect(screen.getByText("Hardware and temperatures")).toBeTruthy());
    expect(screen.getByText("Network")).toBeTruthy();
    fireEvent.click(screen.getByRole("switch", { name: "CPU too hot" }));
    await waitFor(() => expect(saved.at(-1)?.off).toEqual(["hardware.cpu_hot"]));

    // On again, then 90 minutes: kept at the most allowed, 60.
    fireEvent.click(screen.getByRole("switch", { name: "CPU too hot" }));
    await waitFor(() => expect(saved.at(-1)?.off).toEqual([]));
    const input = screen.getByRole("spinbutton", { name: "CPU too hot: Minutes near its limit before warning" });
    fireEvent.change(input, { target: { value: "90" } });
    fireEvent.blur(input);
    await waitFor(() => expect(saved.at(-1)?.limits["hardware.cpu_hot"]).toEqual({ minutes: 60 }));

    fireEvent.click(screen.getByText("Use default (10)"));
    await waitFor(() => expect(saved.at(-1)?.limits["network.weak_wifi"]).toBeUndefined());
  });
});
