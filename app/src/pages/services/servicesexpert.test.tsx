// SPDX-License-Identifier: Apache-2.0
// Expert services and boot: cost, setup facts (who blocked what), exposure, and the boot chain.

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { BootExpertSections, ServicesExpertSections, exposureBadge, stateText } from "./ServicesExpert";

vi.mock("../../api", () => ({
  getInventory: (kind: string) =>
    Promise.resolve(
      ({
        cgroups: { at: 1, groups: [{ kind: "service", unit: "cups.service", name: "cups", cpu_usec: 5e6, memory_own: 4e6, tasks: 3 }] },
        unit_files: [
          { unit: "cups.service", local_overrides: ["/etc/systemd/system/cups.service.d/x.conf"], package_overrides: [], wanted_by: ["multi-user.target"], restart: "on-failure", restarts: 4, state: "enabled" },
          { unit: "sudo.service", local_overrides: [], package_overrides: [], wanted_by: [], restarts: 0, state: "masked", masked_by: "package" },
          { unit: "saned.service", local_overrides: [], package_overrides: [], wanted_by: [], restarts: 0, state: "masked", masked_by: "local" },
        ],
        service_security: [{ unit: "snapd.service", score: 9.9, level: "UNSAFE" }, { unit: "chrony.service", score: 1.8, level: "OK" }],
        boot_detail: {
          critical_chain: [
            { unit: "graphical.target", at_ms: 12107, depth: 0 },
            { unit: "NetworkManager-wait-online.service", at_ms: 6054, took_ms: 5988, depth: 1 },
            { unit: "dbus.service", at_ms: 3243, took_ms: 159, depth: 2 },
          ],
          blame: [["NetworkManager-wait-online.service", 5988], ["plymouth-quit-wait.service", 3612]],
        },
      } as Record<string, unknown>)[kind],
    ),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(cleanup);

describe("Expert services and boot", () => {
  it("services: local changes, loops, who blocked a service, and exposure", async () => {
    render(<ServicesExpertSections />);
    await waitFor(() => expect(screen.getByText("Blocked by its package")).toBeTruthy());
    expect(screen.getByText("Blocked on this computer")).toBeTruthy();
    expect(screen.getByText("1 override")).toBeTruthy();
    expect(screen.getByText("4 (on-failure)")).toBeTruthy();
    expect(screen.getByText("2")).toBeTruthy();
    await waitFor(() => expect(screen.getByText("9.9 almost every right")).toBeTruthy());
    expect(screen.getByText("5 s")).toBeTruthy();
  });

  it("boot: the step that held the desktop up is pointed out", async () => {
    render(<BootExpertSections />);
    await waitFor(() => expect(screen.getByText("took 6.0 s")).toBeTruthy());
    expect(screen.getByText("plymouth-quit-wait.service")).toBeTruthy();
  });

  it("plain words", () => {
    expect(exposureBadge({ unit: "x", score: 7.8, level: "EXPOSED" }).variant).toBe("warn");
    expect(stateText({ unit: "x", local_overrides: [], package_overrides: [], wanted_by: [], restarts: 0, state: "static" })).toBe("Started by others");
  });
});
