// SPDX-License-Identifier: Apache-2.0
// Expert security: automatic updates that really install (or stop), and AppArmor per process.

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { SecurityDetail } from "../../api";
import { SecurityExpertSections, lastRunVerdict } from "./SecurityExpert";

const d: SecurityDetail = {
  confinement: [
    { profile: "unconfined", mode: "none", processes: 305, names: ["systemd"] },
    { profile: "chrome", mode: "unconfined", processes: 29, names: ["chrome"] },
    { profile: "/usr/sbin/cupsd", mode: "enforce", processes: 1, names: ["cupsd"] },
  ],
  periodic: [["Update-Package-Lists", "1"], ["Unattended-Upgrade", "1"]],
  unattended_installed: true,
  upgrade_runs: [
    { at: "2026-10-22 10:43:11", level: "WARNING", message: "System is on metered connection, stopping" },
    { at: "2026-10-14 06:00:00", level: "INFO", message: "All upgrades installed" },
  ],
};

vi.mock("../../api", () => ({
  getInventory: () => Promise.resolve(d),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(cleanup);

describe("Expert security", () => {
  it("automatic updates that stop on a metered connection are pointed out", async () => {
    render(<SecurityExpertSections />);
    await waitFor(() => expect(screen.getByText(/Stopped: the connection is marked as metered/)).toBeTruthy());
    expect(screen.getByText("Every day")).toBeTruthy();
    expect(screen.getByText("1 of 335 processes enforced")).toBeTruthy();
    expect(screen.getByText("Profile that allows everything")).toBeTruthy();
  });

  it("plain verdicts for the last run", () => {
    expect(lastRunVerdict([{ at: "x", level: "INFO", message: "All upgrades installed" }]).variant).toBe("ok");
    expect(lastRunVerdict([{ at: "x", level: "ERROR", message: "dpkg was interrupted" }]).variant).toBe("crit");
    expect(lastRunVerdict(undefined).text).toBe("The log cannot be read");
    expect(lastRunVerdict([]).text).toBe("No run in the log yet");
  });
});
