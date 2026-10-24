// SPDX-License-Identifier: Apache-2.0
// Renders the Hardware, Power, and Display & Audio pages with realistic answers and with gaps.

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Metric } from "../../api";
import { DisplayAudio, DisplayDisplays } from "../display/Display";
import { PowerBattery, PowerProfile, PowerSleep } from "../power/Power";
import { HardwareFirmware } from "./Firmware";
import { HardwareGraphics } from "./Graphics";
import { HardwareMemory } from "./Memory";
import { HardwarePeripherals } from "./Peripherals";
import { HardwareProcessor } from "./Processor";
import { HardwareSensors } from "./Sensors";
import { HardwareStorage } from "./Storage";
import { HardwareSummary } from "./Summary";

const at = Date.now();
const m = (name: string, value: number, subject?: string): Metric => ({ name, subject, value, unit: "count", at });
const LATEST = [m("cpu.usage", 12), m("temp.cpu", 54), m("cpu.frequency", 1.3e9), m("cpu.core", 16, "0"), m("temp", 56, "coretemp/Core 0"), m("temp", 38, "nvme/Composite"), m("temp", 31, "BAT0/temp"), m("battery.health", 71, "BAT0"), m("battery.time_left", 11_400, "BAT0"), m("battery.power", 8.4, "BAT0"), m("memory.total", 7.6e9)];
const HW = {
  machine: { vendor: "LENOVO", model: "IdeaPad 5 14IAL7", chassis: "Laptop", bios_version: "JHCN52WW", bios_date: "03/11/2024", boot_mode: "UEFI", secure_boot: true, tpm: "2.0" },
  processor: { model: "12th Gen Intel(R) Core(TM) i5-1235U", cores: 10, hybrid: [2, 8], threads: 12, base_hz: 1.3e9, max_hz: 4.4e9, l3_bytes: 12582912, microcode: "0x4123", features: ["VT-x", "AVX2"], cpus: [{ cpu: 0, core_id: 0, kind: "Performance", mhz: 1640 }, { cpu: 4, core_id: 8, kind: "Efficient", mhz: 1180 }] },
  memory_total: 8 * 1024 ** 3,
  ecc_reporting: false,
  disks: [{ name: "nvme0n1", model: "Samsung PM991a", kind: "SSD (NVMe)", size: 256e9, removable: false }],
  devices: [
    { name: "Wi-Fi 6 AX201", bus: "PCI", vendor: "Intel", drivers: ["iwlwifi"], class: "Network" },
    { name: "Mystery network card", bus: "PCI", vendor: "Acme", drivers: [], class: "Network" },
    { name: "Shared SRAM", bus: "PCI", drivers: [], class: "Memory" },
    { name: "USB mouse", bus: "USB", vendor: "Logitech", drivers: ["usbhid"] },
  ],
  bluetooth_adapters: 1,
};
let battery = true;
let audio: unknown = {};
let firmware: unknown = { available: false, devices: [] };

vi.mock("../../api", () => ({
  // Two apps stopped for memory this week, one event of two.
  getEvents: () => Promise.resolve({ events: [{ id: 1, kind: "out_of_memory_kill", at, title: "chrome was stopped", source: "journal", data: { count: 2 } }], truncated: false }),
  getLatest: () => Promise.resolve(LATEST),
  getMetrics: (name: string) => Promise.resolve([0, 1, 2].map((i) => ({ ...m(name, 70 + i), at: at - (2 - i) * 86_400_000 }))),
  getMachine: () => Promise.resolve({ hostname: "h", os: "o", user: "u", session_type: "wayland" }),
  getInventory: (kind: string) =>
    Promise.resolve(
      {
        hardware: HW,
        gpu: [{ card: "card1", vendor: "Intel Corporation", model: "Alder Lake-UP3 GT2 [Iris Xe Graphics]", driver: "i915", boot_vga: true }],
        bluetooth: { available: true, adapters: 1, powered: true, devices: [{ name: "Earbuds", paired: true, connected: true, battery: 80 }] },
        firmware,
        power: {
          batteries: battery ? [{ name: "BAT0", status: "Discharging", charge: 64, full_wh: 40.5, design_wh: 57, charge_limit: 100 }] : [],
          on_ac: !battery,
          profile: "balanced",
          profiles: ["power-saver", "balanced", "performance"],
          policy: { low_percent: 10, critical_percent: 5, action_percent: 2, critical_action: "HybridSleep" },
          screen_blank_seconds: 300,
          suspend_on_battery_seconds: 1200,
          suspend_on_ac_seconds: 0,
          lid_action: "suspend",
          sleep_mode: "s2idle",
        },
        sleep: [{ at: at - 3_600_000, kind: "suspend", duration_ms: 1_800_000, failed: false }, { at: at - 7_200_000, kind: "suspend", failed: true }],
        display: { compositor: "Mutter", shell_version: "46.0", fractional_scaling: false, outputs: [{ connector: "eDP-1", connected: true, name: "Built-in display", builtin: true, width: 1920, height: 1080, refresh_hz: 60.02, scale: 1, primary: true }, { connector: "HDMI-A-1", connected: false, builtin: false, primary: false }] },
        audio,
      }[kind],
    ),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

beforeEach(() => {
  battery = true;
  audio = { server: "PipeWire 1.0.5", outputs: [{ name: "spk", description: "Raptor Lake Speaker + Headphones", volume: 45, muted: false, default: true }], inputs: [{ name: "mic", description: "Raptor Lake Digital Microphone", volume: 70, muted: false, default: true }], streams: ["Firefox"] };
  firmware = { available: false, devices: [] };
});
afterEach(cleanup);

describe("Hardware pages", () => {
  it("summary lists machine and components", async () => {
    render(<HardwareSummary />);
    await waitFor(() => expect(screen.getByText("IdeaPad 5 14IAL7")).toBeTruthy());
    expect(screen.getByText("Intel Core i5-1235U · 54 °C")).toBeTruthy();
    await waitFor(() => expect(screen.getByText(/needs Full access/)).toBeTruthy());
    await waitFor(() => expect(screen.getByText("fwupd not running")).toBeTruthy());
  });

  it("processor names P and E cores and shows their temperature", async () => {
    render(<HardwareProcessor />);
    await waitFor(() => expect(screen.getByText("10 (2 performance + 8 efficient)")).toBeTruthy());
    expect(screen.getByText("P0")).toBeTruthy();
    expect(screen.getByText("E0")).toBeTruthy();
    expect(screen.getByText("56 °C")).toBeTruthy();
  });

  it("memory, storage, graphics, and sensors render", async () => {
    for (const [Tab, text] of [
      [HardwareMemory, "2 apps were stopped because memory ran out"],
      [HardwareStorage, "Samsung PM991a"],
      [HardwareGraphics, "Hardware acceleration available"],
      [HardwareSensors, "Fan speed not reported"],
    ] as const) {
      render(<Tab />);
      await waitFor(() => expect(screen.getAllByText(text).length).toBeGreaterThan(0));
      cleanup();
    }
  });

  it("peripherals count only real missing drivers and include Bluetooth", async () => {
    render(<HardwarePeripherals />);
    await waitFor(() => expect(screen.getByText("Earbuds · battery 80%")).toBeTruthy());
    expect(screen.getByText("No driver needed")).toBeTruthy();
    expect(screen.getAllByText("No driver").length).toBe(1);
  });

  it("firmware is honest when fwupd is not running, and lists updates when it is", async () => {
    render(<HardwareFirmware />);
    await waitFor(() => expect(screen.getByText(/DeeCure does not start it/)).toBeTruthy());
    expect(screen.getByText(/2024/)).toBeTruthy();
    cleanup();
    firmware = { available: true, devices: [{ name: "Touchpad", vendor: "Synaptics", version: "1.4", available: "1.6", updatable: true }] };
    render(<HardwareFirmware />);
    await waitFor(() => expect(screen.getByText("Update available")).toBeTruthy());
  });
});

describe("Power and Display pages", () => {
  it("battery shows health, time left, and capacity", async () => {
    render(<PowerBattery />);
    await waitFor(() => expect(screen.getByText("Design 57 Wh")).toBeTruthy());
    expect(screen.getByText("3 h 10 m")).toBeTruthy();
    expect(screen.getByText("40.5 of 57 Wh")).toBeTruthy();
  });

  it("a desktop without a battery says so", async () => {
    battery = false;
    render(<PowerBattery />);
    await waitFor(() => expect(screen.getByText("This computer has no battery.")).toBeTruthy());
  });

  it("profile and sleep render", async () => {
    render(<PowerProfile />);
    await waitFor(() => expect(screen.getByText("Modern standby (s2idle)")).toBeTruthy());
    expect(screen.getByText("20 m on battery · Never plugged in")).toBeTruthy();
    cleanup();
    render(<PowerSleep />);
    await waitFor(() => expect(screen.getAllByText("Failed").length).toBeGreaterThan(0));
  });

  it("displays and audio render, and audio without PipeWire is honest", async () => {
    render(<DisplayDisplays />);
    await waitFor(() => expect(screen.getByText("1920 × 1080")).toBeTruthy());
    expect(screen.getByText("Mutter (GNOME 46.0)")).toBeTruthy();
    cleanup();
    render(<DisplayAudio />);
    await waitFor(() => expect(screen.getByText("Volume 45%")).toBeTruthy());
    cleanup();
    audio = { outputs: [], inputs: [], streams: [] };
    render(<DisplayAudio />);
    await waitFor(() => expect(screen.getByText(/No sound server answered/)).toBeTruthy());
  });
});
