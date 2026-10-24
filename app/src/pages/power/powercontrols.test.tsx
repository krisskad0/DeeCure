// SPDX-License-Identifier: Apache-2.0
// Power controls open the fix preview with the exact change; protected devices get no button.

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { PowerDetail } from "../../api";
import { FixContext } from "../../shell/FixDrawer";
import { PowerControls, describeKept, lidSummary, limitsSubject, pciPowerSavingSafe, radioKinds } from "./PowerControls";
import { WakeExpertSections, mustKeepWaking } from "./PowerExpert";

const power: PowerDetail = {
  batteries: [{ name: "BAT0", charge_start: 50, charge_stop: 80, charge_mode: { current: "Custom", offered: ["Standard", "Adaptive", "Custom"] } }],
  chargers: [],
  power_delivery: [],
  suspend: { current: "deep", offered: ["s2idle", "deep"] },
  aspm_policy: { current: "powersave", offered: ["default", "performance", "powersave", "powersupersave"] },
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
  lid: { on_battery: "suspend", on_power: "suspend", ours: true, on_battery_from: "/etc/systemd/logind.conf.d/50-deecure-lid.conf", running_battery: "suspend", running_power: "suspend" },
  kept: [
    { setting: "charge_limits:BAT0", call: { op: "set_charge_limits", battery: "BAT0", start: 50, stop: 80 }, boot_ok: true, boot_message: "charge_control_start_threshold = 50" },
    { setting: "power_saving:pci0000:00/0000:00:14.0/usb3/3-1", call: { op: "set_device_power_saving", device: "pci0000:00/0000:00:14.0/usb3/3-1", on: false }, boot_ok: false, boot_message: "a different device is there now, so nothing was changed" },
    { setting: "turbo", call: { op: "set_turbo", on: false } },
    { setting: "suspend_mode", call: { op: "set_suspend_mode", mode: "deep", keep: true }, boot_ok: true, boot_message: "mem_sleep = deep" },
  ],
};

vi.mock("../../api", () => ({
  getInventory: (kind: string) =>
    Promise.resolve(
      ({
        power_detail: power,
        cpu_detail: { turbo: true, cores: [{ cpu: 0, hardware_max_khz: 4_700_000, idle: [] }], caches: [], power_limits: [], flaws: [], interrupts: [], at: 1 },
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

  it("kept settings show in words with this start's result", async () => {
    withFix(<PowerControls />);
    await waitFor(() => expect(screen.getByText("Kept after restart")).toBeTruthy());
    expect(screen.getByText(/Charge limits · BAT0: start below 50%, stop at 80%/)).toBeTruthy();
    expect(screen.getByText(/Receiver: power saving off/)).toBeTruthy();
    expect(screen.getAllByText("Set at this start")).toHaveLength(2);
    expect(screen.getByText("Not set at this start")).toBeTruthy();
    expect(screen.getByText("a different device is there now, so nothing was changed")).toBeTruthy();
    expect(screen.getByText("From the next start")).toBeTruthy();
    // Without a known product, the device's last path part names it.
    expect(describeKept({ setting: "zswap", call: { op: "set_zswap", on: true } }, { usb: [], wake_sources: [] })).toBe("Compressed swap (zswap): on");
    expect(describeKept({ setting: "scheduler:sda", call: { op: "set_disk_scheduler", disk: "sda", scheduler: "mq-deadline" } }, { usb: [], wake_sources: [] })).toBe("Disk sda: mq-deadline scheduler");
    expect(describeKept({ setting: "wakeup:x", call: { op: "set_device_wakeup", device: "pci0000:00/0000:00:1f.3", enabled: false } }, { usb: [], wake_sources: [] })).toBe(
      "0000:00:1f.3: does not wake the computer",
    );
  });

  it("the CPU limit, PCIe power saving, and suspend type are tried first, then kept", async () => {
    withFix(<PowerControls />);
    await waitFor(() => expect(screen.getByText("60% (2.8 GHz)")).toBeTruthy());
    expect((screen.getByText("No limit") as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(screen.getByText("60% (2.8 GHz)"));
    // PCIe saving was tried but not kept: it offers Keep. Suspend is kept: it shows Kept.
    expect(screen.getByText(/Now: Save power, for this session only/)).toBeTruthy();
    fireEvent.click(screen.getByText("Keep Save power"));
    fireEvent.click(screen.getByText("Try Firmware default"));
    expect(screen.getByText(/Now: deep \(suspend to RAM\), kept after restart/)).toBeTruthy();
    expect(screen.queryByText("Keep deep (suspend to RAM)")).toBeNull();
    fireEvent.click(screen.getByText("Try s2idle (modern standby)"));
    expect(screen.getByText(/Suspend type: deep \(suspend to RAM\)/)).toBeTruthy();
    expect(opened).toEqual([
      ["power.set_battery_cpu_limit", "60"],
      ["power.set_aspm_policy", "powersave|keep"],
      ["power.set_aspm_policy", "default|try"],
      ["power.set_suspend_mode", "s2idle|try"],
    ]);
  });

  it("the lid action is chosen per power source and says when it applies", async () => {
    withFix(<PowerControls />);
    await waitFor(() => expect(screen.getByText("Closing the lid")).toBeTruthy());
    const lidChange = screen.getByRole("button", { name: "Set lid action" }) as HTMLButtonElement;
    expect(lidChange.disabled).toBe(true);
    fireEvent.change(screen.getByRole("combobox", { name: "Closing the lid plugged in" }), { target: { value: "lock" } });
    fireEvent.click(lidChange);
    fireEvent.click(screen.getByText("Use the system setting"));
    expect(opened).toEqual([
      ["power.set_lid_action", "suspend|lock"],
      ["power.set_lid_action", "default|default"],
    ]);
    const lid = { on_battery: "lock", on_power: "ignore", ours: true, on_battery_from: "/etc/systemd/logind.conf.d/50-deecure-lid.conf", on_power_from: "/etc/systemd/logind.conf.d/90-vendor.conf", running_battery: "suspend", running_power: "ignore" };
    const text = lidSummary(lid);
    expect(text).toContain("On battery: lock the screen. Plugged in: do nothing.");
    expect(text).toContain("Applies after the next restart (now: suspend on battery, do nothing plugged in).");
    expect(text).toContain("/etc/systemd/logind.conf.d/90-vendor.conf comes later and decides part of this.");
    expect(lidSummary({ ...lid, running_battery: "lock" })).not.toContain("after the next restart");
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
