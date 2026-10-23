// SPDX-License-Identifier: Apache-2.0
// Expert power sections: battery depth, chargers, drain while suspended, wake sources, savers.

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { PowerDetail } from "../../api";
import { BatteryExpertSections, WakeExpertSections, wakeLabel } from "./PowerExpert";

const HOUR = 3_600_000;
const t0 = Date.now() - 48 * HOUR;
const power: PowerDetail = {
  batteries: [{ name: "BAT0", status: "Discharging", watts: 14.75, volts: 11.97, volts_min_design: 11.4, amps: 1.23, temp_c: 32.6, health: "Good", made: "2024-12-16", charge_start: 50, charge_stop: 80, charge_mode: { current: "Custom", offered: ["Trickle", "Fast", "Standard", "Adaptive", "Custom"] } }],
  chargers: [{ name: "AC", kind: "Mains", online: false }, { name: "ucsi-source-psy-USBC000:001", kind: "USB", online: true, usb_type: { current: "PD", offered: ["C", "PD"] }, max_watts: 45 }],
  power_delivery: [{ name: "pd0", offered: [{ kind: "fixed_supply", millivolts: 20000, milliamps: 2250 }], accepted: [{ kind: "fixed_supply", millivolts: 5000, milliamps: 3000 }] }],
  suspend: { current: "s2idle", offered: ["s2idle"] },
  wake_sources: [
    { name: "PNP0C0D:00", events: 4, wakeups: 0, active: 0, may_wake: true },
    { name: "device:53", pci_address: "0000:2c:00.0", events: 2, wakeups: 1, active: 0, may_wake: true },
    { name: "serio0", events: 9, wakeups: 0, active: 0, may_wake: false },
  ],
  last_wake_irq: "9",
  last_wake_device: "IR-IO-APIC 9-fasteoi acpi",
  usb: [
    { port: "3-1", product: "Dell Universal Receiver", control: "on", keep_awake: true },
    { port: "3-7", product: "Fingerprint reader", control: "on", keep_awake: false, state: "active" },
  ],
};

vi.mock("../../api", () => ({
  getInventory: (kind: string) =>
    Promise.resolve(
      ({
        power_detail: power,
        pci_detail: [{ address: "0000:2c:00.0", class: "Network", model: "RTL8111 Ethernet", power_control: "on", power_state: "suspended" }],
        sleep: [
          { at: t0, kind: "suspend", duration_ms: 8 * HOUR, failed: false },
          { at: t0 + 20 * HOUR, kind: "suspend", duration_ms: 2 * HOUR, failed: false },
          { at: t0 + 30 * HOUR, kind: "suspend", duration_ms: 60_000, failed: false },
        ],
      } as Record<string, unknown>)[kind],
    ),
  // 80% before the first suspend and 76% after it (0.5% per hour); the second one charged.
  getMetrics: (_name: string, since: number) =>
    Promise.resolve([{ name: "battery.charge", value: since < t0 ? 80 : since < t0 + 10 * HOUR ? 76 : since < t0 + 21 * HOUR ? 50 : 60, at: since, unit: "percent" }]),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(cleanup);

describe("Expert power sections", () => {
  it("battery depth, charge limits and modes, and what the charger offers", async () => {
    render(<BatteryExpertSections />);
    await waitFor(() => expect(screen.getByText("starts below 50%, stops at 80%")).toBeTruthy());
    expect(screen.getByText(/Custom \(offered: Trickle/)).toBeTruthy();
    expect(screen.getByText("11.97 V (design minimum 11.40 V)")).toBeTruthy();
    expect(screen.getByText("Connected · PD · up to 45 W")).toBeTruthy();
    expect(screen.getByText(/20 V · 2.25 A \(most 45 W\)/)).toBeTruthy();
  });

  it("battery used while suspended, per hour, and a suspend that charged", async () => {
    render(<BatteryExpertSections />);
    await waitFor(() => expect(screen.getByText("0.5%")).toBeTruthy());
    expect(screen.getByText("80% to 76%")).toBeTruthy();
    expect(screen.getByText("Charging")).toBeTruthy();
    expect(screen.queryByText("0.0 h")).toBeNull();
  });

  it("wake sources by name, the last wake, and devices that could save power", async () => {
    render(<WakeExpertSections />);
    await waitFor(() => expect(screen.getByText("Lid")).toBeTruthy());
    expect(screen.getByText("interrupt 9 (acpi)")).toBeTruthy();
    expect(screen.queryByText("Built-in keyboard")).toBeNull();
    await waitFor(() => expect(screen.getAllByText("RTL8111 Ethernet").length).toBe(2));
    expect(screen.getByText("Fingerprint reader")).toBeTruthy();
    expect(screen.queryByText("Dell Universal Receiver")).toBeNull();
    expect(screen.getByText("1 input or sound device stays awake on purpose.")).toBeTruthy();
  });

  it("wake labels fall back to the kernel name", () => {
    expect(wakeLabel({ name: "PNP0C0C:00", events: 0, wakeups: 0, active: 0 }, [])).toBe("Power button");
    expect(wakeLabel({ name: "device:99", events: 0, wakeups: 0, active: 0 }, [])).toBe("device:99");
    expect(wakeLabel({ name: "3-1", product: "Mouse", events: 0, wakeups: 0, active: 0 }, [])).toBe("Mouse");
  });
});
