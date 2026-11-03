// SPDX-License-Identifier: Apache-2.0
// Battery use by app: days added up, shares of a full charge, and the empty text.

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { AppBatteryInfo } from "../../api";
import { AppBatteryPanel, usesIn } from "./AppBattery";

const info: AppBatteryInfo = {
  days: [
    { start: 2, uses: [{ kind: "app", name: "Google Chrome", mwh: 9000 }, { kind: "rest", name: "", mwh: 6000 }] },
    { start: 1, uses: [{ kind: "rest", name: "", mwh: 2000 }, { kind: "app", name: "Google Chrome", mwh: 1000 }, { kind: "service", name: "snapd", mwh: 300 }] },
    ...Array.from({ length: 6 }, (_, i) => ({ start: -i, uses: [] })),
  ],
};
let answer: AppBatteryInfo = info;

vi.mock("../../api", () => ({
  getInventory: () => Promise.resolve(answer),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(cleanup);

describe("Battery use by app", () => {
  it("adds up the same app over several days", () => {
    expect(usesIn(info, "today").map((u) => [u.name, u.mwh])).toEqual([["Google Chrome", 9000], ["", 6000]]);
    const week = usesIn(info, "week");
    expect(week[0]).toEqual({ kind: "app", name: "Google Chrome", mwh: 10000 });
    expect(week[1].mwh).toBe(8000);
    expect(usesIn(info, "yesterday")[0].kind).toBe("rest");
  });

  it("shows each app as a share of a full charge", async () => {
    answer = info;
    render(<AppBatteryPanel fullWh={50} />);
    // 15 Wh of a 50 Wh battery today; Chrome 9 Wh = 18%.
    await waitFor(() => expect(screen.getByText("30% · 15.0 Wh")).toBeTruthy());
    expect(screen.getByText("Google Chrome")).toBeTruthy();
    expect(screen.getByText("18% · 9.0 Wh")).toBeTruthy();
    expect(screen.getByText("Screen and system at rest")).toBeTruthy();
    fireEvent.click(screen.getByRole("combobox", { name: "Time" }));
    fireEvent.mouseDown(await screen.findByRole("option", { name: "Yesterday" }));
    await waitFor(() => expect(screen.getByText("snapd")).toBeTruthy());
  });

  it("says when nothing was counted", async () => {
    answer = { days: [{ start: 0, uses: [] }] };
    render(<AppBatteryPanel />);
    await waitFor(() => expect(screen.getByText(/Nothing counted today/)).toBeTruthy());
  });
});
