// SPDX-License-Identifier: Apache-2.0
// Power controls open the fix preview with the exact change; protected devices get no button.

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { PowerDetail } from "../../api";
import { FixContext } from "../../shell/FixDrawer";
import { PowerControls, limitsSubject, pciPowerSavingSafe, radioKinds } from "./PowerControls";
import { WakeExpertSections, mustKeepWaking } from "./PowerExpert";

const power: PowerDetail = {
  batteries: [{ name: "BAT0", charge_start: 50, charge_stop: 80, charge_mode: { current: "Custom", offered: ["Standard", "Adaptive", "Custom"] } }],
  chargers: [],
  power_delivery: [],
  suspend: { current: "s2idle", offered: ["s2idle"] },
  wake_sources: [
    { name: "PNP0C0D:00", device: "LNXSYSTM:00/LNXSYBUS:00/PNP0C0D:00", events: 4, wakeups: 0, active: 0, may_wake: true },
    { name: "device:14", device: "pci0000:00/0000:00:14.0", pci_address: "0000:00:14.0", events: 2, wakeups: 0, active: 0, may_wake: true },
  ],
  usb: [{ port: "3-1", product: "Receiver", control: "on", keep_awake: true, sys_path: "pci0000:00/0000:00:14.0/usb3/3-1" }],
  radios: [
    { name: "dell-wifi", kind: "wlan", soft_blocked: false, hard_blocked: false },
    { name: "phy0", kind: "wlan", soft_blocked: false, hard_blocked: false },
    { name: "dell-bluetooth", kind: "bluetooth", soft_blocked: true, hard_blocked: false },
  ],
  keyboard_backlight: { level: 0, max: 2 },
};

vi.mock("../../api", () => ({
  getInventory: (kind: string) =>
    Promise.resolve(
      ({
        power_detail: power,
        cpu_detail: { turbo: true, cores: [], caches: [], power_limits: [], flaws: [], interrupts: [], at: 1 },
        net_deep: { wifi: [{ interface: "wlp45s0", ssid: "Home", uuid: "798005ca-d9fd-4047-86c5-8ed2ff14b23d", power_save: true }], interfaces: [], counters: {}, routes: [], extra_rules: 0, at: 1 },
        pci_detail: [{ address: "0000:00:14.0", class: "USB controller", model: "USB 3.2 controller", driver: "xhci_hcd", sys_path: "pci0000:00/0000:00:14.0", power_control: "on" }],
      } as Record<string, unknown>)[kind],
    ),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

const opened: [string, string | undefined][] = [];
const withFix = (ui: React.ReactNode) => render(<FixContext.Provider value={(a, s) => opened.push([a, s])}>{ui}</FixContext.Provider>);

afterEach(() => {
  cleanup();
  opened.length = 0;
});

describe("Power controls", () => {
  it("each control opens the preview with the exact change", async () => {
    withFix(<PowerControls />);
    await waitFor(() => expect(screen.getByText("Long life (80%)")).toBeTruthy());
    expect((screen.getByText("Long life (80%)") as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(screen.getByText("Balanced (90%)"));
    fireEvent.change(screen.getByRole("spinbutton", { name: "Start charging below" }), { target: { value: "70" } });
    fireEvent.change(screen.getByRole("spinbutton", { name: "Stop charging at" }), { target: { value: "75" } });
    fireEvent.click(screen.getByRole("button", { name: "Set" }));
    fireEvent.change(screen.getByRole("combobox", { name: "Charge mode" }), { target: { value: "Adaptive" } });
    fireEvent.click(screen.getByRole("button", { name: "Change" }));
    await waitFor(() => expect(screen.getByText("Turn turbo off")).toBeTruthy());
    fireEvent.click(screen.getByText("Turn turbo off"));
    fireEvent.click(screen.getByText("Level 2"));
    fireEvent.click(screen.getAllByText("Turn on")[0]);
    await waitFor(() => expect(screen.getByText("Wi-Fi power saving · Home")).toBeTruthy());
    fireEvent.click(screen.getAllByText("Off").at(-1)!);
    expect(opened).toEqual([
      ["power.set_charge_limits", "BAT0:85:90"],
      ["power.set_charge_limits", "BAT0:70:75"],
      ["power.set_charge_mode", "BAT0:Adaptive"],
      ["power.set_turbo", "off"],
      ["power.set_keyboard_backlight", "2"],
      ["power.set_radio", "bluetooth|on"],
      ["power.set_wifi_power_save", "798005ca-d9fd-4047-86c5-8ed2ff14b23d|off"],
    ]);
  });

  it("an invalid custom limit cannot be sent", async () => {
    withFix(<PowerControls />);
    await waitFor(() => expect(screen.getByRole("button", { name: "Set" })).toBeTruthy());
    fireEvent.change(screen.getByRole("spinbutton", { name: "Start charging below" }), { target: { value: "90" } });
    fireEvent.change(screen.getByRole("spinbutton", { name: "Stop charging at" }), { target: { value: "80" } });
    expect((screen.getByRole("button", { name: "Set" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("the lid keeps waking; other devices get a stop button, savers a try button", async () => {
    withFix(<WakeExpertSections />);
    await waitFor(() => expect(screen.getByText("Always kept")).toBeTruthy());
    fireEvent.click(screen.getByText("Stop waking…"));
    await waitFor(() => expect(screen.getByText("Try power saving…")).toBeTruthy());
    fireEvent.click(screen.getByText("Try power saving…"));
    expect(opened).toEqual([
      ["power.set_device_wakeup", "pci0000:00/0000:00:14.0|off"],
      ["power.set_device_power_saving", "pci0000:00/0000:00:14.0|on"],
    ]);
  });

  it("helpers", () => {
    expect(limitsSubject("BAT0", false, 80)).toBe("BAT0:-:80");
    expect(radioKinds(power.radios)).toEqual([{ kind: "wlan", on: true, hard: false }, { kind: "bluetooth", on: false, hard: false }]);
    expect(mustKeepWaking({ name: "serio0", events: 0, wakeups: 0, active: 0 })).toBe(true);
    expect(pciPowerSavingSafe({ address: "x", class: "Storage", driver: "nvme", sys_path: "p", power_control: "on" })).toBe(false);
    expect(pciPowerSavingSafe({ address: "x", class: "Network", sys_path: "p", power_control: "on" })).toBe(false);
  });
});
