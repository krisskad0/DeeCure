// SPDX-License-Identifier: Apache-2.0
// Hardware › All Parts: every part in one table, and the slots and sockets with one password.

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { HardwareInfo, Metric } from "../../api";
import { HardwareAllParts, partsList, pointerKind } from "./AllParts";

const at = Date.now();
const HW: HardwareInfo = {
  machine: { vendor: "Dell Inc.", model: "XPS 15", boot_mode: "UEFI" },
  processor: { model: "13th Gen Intel(R) Core(TM) i7-13700H", cores: 14, threads: 20, features: [], cpus: [] },
  memory_total: 16 * 1024 ** 3,
  ecc_reporting: false,
  disks: [
    { name: "nvme0n1", model: "KXG8AZNV1T02", kind: "SSD (NVMe)", size: 1024e9, removable: false },
    { name: "sda", model: "WD Blue", kind: "Hard disk", size: 2000e9, removable: false },
  ],
  devices: [
    { name: "AT Translated Set 2 keyboard", bus: "Input", drivers: [], class: "Keyboard" },
    { name: "VEN_04F3:00 04F3:32DC Touchpad", bus: "Input", drivers: [], class: "Pointer" },
    { name: "Logitech M705 Mouse", bus: "Input", drivers: [], class: "Pointer" },
    { name: "Unknown USB gadget", bus: "USB", vendor: "Acme", drivers: [] },
    { name: "Wi-Fi 6E AX211", bus: "PCI", vendor: "Intel", drivers: ["iwlwifi"], class: "Network" },
  ],
  bluetooth_adapters: 1,
  sata_ports: [{ port: 1, speed: "6.0 Gbps", disk: "sda" }, { port: 2 }],
};
const fan: Metric = { name: "fan", subject: "dell_smm/fan1", value: 2400, unit: "count", at };

const BOARD = {
  ports: [
    { "Internal Reference Designator": "J3A1", "Internal Connector Type": "None", "External Reference Designator": "USB 1", "External Connector Type": "Access Bus (USB)", "Port Type": "USB" },
    { "Internal Reference Designator": "JAUD1", "Internal Connector Type": "None", "External Reference Designator": "Headphone", "External Connector Type": "Mini Jack (headphones)", "Port Type": "Audio Port" },
  ],
  slots: [
    { Designation: "M.2 SSD 1", Type: "x4 PCI Express 4 x4", "Current Usage": "In Use", "Bus Address": "0000:02:00.0" },
    { Designation: "M.2 SSD 2", Type: "x4 PCI Express 4 x4", "Current Usage": "Available" },
  ],
};
const MEMORY = { arrays: [{ "Maximum Capacity": "64 GB", "Number Of Devices": "2" }], modules: [{ Size: "16 GB", Locator: "DIMM A", Type: "DDR5", Filled: "true" }, { Size: "No Module Installed", Locator: "DIMM B", Filled: "false" }] };
const reads: string[] = [];

vi.mock("../../api", () => ({
  getInventory: (kind: string) =>
    Promise.resolve(
      {
        hardware: HW,
        gpu: [{ card: "card1", model: "Intel Corporation Raptor Lake-P [Iris Xe Graphics]", driver: "i915", boot_vga: true }],
        bluetooth: { available: true, adapters: 1, powered: true, devices: [{ name: "WH-1000XM4", kind: "audio-headphones", paired: true, connected: true, battery: 80 }] },
        power: { batteries: [{ name: "BAT0", status: "Discharging", charge: 76, manufacturer: "SMP", model: "DELL 70N2F", technology: "Li-poly" }] },
        power_detail: { batteries: [], chargers: [], power_delivery: [], wake_sources: [], usb: [], radios: [], keyboard_backlight: { level: 1, max: 2 } },
        platform_detail: { security_checks: [], boot_entries: [], disk_unlocked_by_tpm: false, disk_encrypted: false, iommu_units: 0, kvm_ready: false, usb: [], thunderbolt: [], typec: [{ name: "port0", connected: false, partner_power_delivery: false, alt_modes: [] }] },
        devices_detail: {
          cameras: [{ node: "/dev/video0", name: "Integrated_Webcam_HD", driver: "uvcvideo" }],
          fingerprint: { device: "Goodix MOC Fingerprint Sensor", enrolled: [] },
          network: [{ interface: "wlp0s20f3", driver: "iwlwifi", bus: "PCI" }],
          pointing: [],
          batteries: [],
          screens: [{ connector: "eDP-1", maker: "Sharp", model: "LQ156R1", product_code: 1, native: [3840, 2400], refresh_max: 60, hdr: [] }],
          backlights: [],
          sound: [{ index: 0, name: "sof-hda-dsp", codecs: ["Realtek ALC3281"] }],
        },
      }[kind] ?? null,
    ),
  getLatest: () => Promise.resolve([fan]),
  getHelperStatus: () => Promise.resolve(true),
  helperRead: (what: string) => {
    reads.push(what);
    return Promise.resolve(JSON.stringify(what === "memory_slots" ? MEMORY : BOARD));
  },
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(() => {
  cleanup();
  localStorage.clear();
});

describe("All Parts", () => {
  it("names touchpads, touchscreens, pens, and mice from the device name", () => {
    expect(pointerKind("VEN_04F3:00 04F3:32DC Touchpad")).toBe("Touchpad");
    expect(pointerKind("ELAN Touchscreen")).toBe("Touchscreen");
    expect(pointerKind("Wacom HID 52A2 Pen")).toBe("Pen");
    expect(pointerKind("PS/2 Generic Mouse")).toBe("Mouse");
  });

  it("lists every part, even when some answers are missing", () => {
    const parts = partsList({ hw: HW, fans: [] });
    const kinds = parts.map((p) => p.kind);
    expect(kinds).toEqual(["Processor", "Memory", "Drive", "Drive", "Keyboard", "Touchpad", "Mouse", "USB device"]);
    expect(parts.find((p) => p.kind === "Keyboard")?.name).toBe("Built-in keyboard");
    expect(parts.find((p) => p.kind === "USB device")?.status).toEqual(["warn", "No driver"]);
    expect(parts.find((p) => p.kind === "Touchpad")?.place).toEqual(["devices", "ports"]);
  });

  it("shows the parts, the SATA ports, and the slots and sockets after one password", async () => {
    localStorage.setItem("deecure.access", JSON.stringify("full"));
    render(<HardwareAllParts />);
    await waitFor(() => expect(screen.getByText("Built-in keyboard")).toBeTruthy());
    await waitFor(() => expect(screen.getByText("Keyboard backlight")).toBeTruthy());
    expect(screen.getByText("Level 1 of 2")).toBeTruthy();
    expect(screen.getByText("2400 RPM · dell_smm/fan1")).toBeTruthy();
    expect(screen.getByText("Integrated_Webcam_HD")).toBeTruthy();
    expect(screen.getByText("WH-1000XM4")).toBeTruthy();
    expect(screen.getByText("Free")).toBeTruthy();
    // SATA ports need no password.
    expect(screen.getByText("SATA 2")).toBeTruthy();
    expect(screen.getByText("Nothing connected")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Read slots and sockets (asks for your password)" }));
    await waitFor(() => expect(screen.getByText("M.2 SSD 2")).toBeTruthy());
    expect(reads).toEqual(["memory_slots", "slots_ports"]);
    expect(screen.getByText("Headphone")).toBeTruthy();
    expect(screen.getByText("DIMM B")).toBeTruthy();
  });
});
