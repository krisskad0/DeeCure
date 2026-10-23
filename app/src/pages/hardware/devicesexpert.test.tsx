// SPDX-License-Identifier: Apache-2.0
// Expert built-in devices, screens, sound, and hardware errors.

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { DevicesDetail } from "../../api";
import { BuiltInDevicesExpert, HardwareErrorsPanel, ScreensExpert, SoundHardwareExpert, screenSize } from "./DevicesExpert";

const devices: DevicesDetail = {
  cameras: [{ node: "video0", name: "Integrated_Webcam_FHD", driver: "uvcvideo" }],
  fingerprint: { enrolled: [] },
  network: [
    { interface: "enp44s0", driver: "r8169", firmware: "rtl8168h-2", speed: "100Mb/s", duplex: "Full", fastest: "1 Gb/s", link: true, energy_saving: "enabled - active" },
    { interface: "wlp45s0", driver: "rtw89_8852be" },
  ],
  pointing: ["VEN_04F3:00 04F3:32DC Touchpad"],
  batteries: [{ kind: "mouse", model: "MX Master 3", percent: 55 }],
  printers: [],
  screens: [{ connector: "eDP-1", maker: "LGD", model: "RNTRJ 140WFH", product_code: 0x753, year: 2022, width_cm: 31, height_cm: 17, native: [1920, 1080], bits: 6, hdr: [] }],
  backlights: [{ name: "intel_backlight", level: 29472, max: 96000, kind: "raw" }],
  sound: [{ index: 0, name: "sof-hda-dsp", codecs: ["Realtek ALC3204", "Intel Raptor Lake P HDMI"] }],
};

vi.mock("../../api", () => ({
  getInventory: (kind: string) =>
    Promise.resolve(
      ({
        devices_detail: devices,
        hw_errors: { kinds: [{ kind: "usb", count: 2, last_at: Date.now() - 3_600_000, examples: ["usb 3-2: device descriptor read/64, error -71"] }, { kind: "gpu", count: 0, examples: [] }], crash_dumps: [], driver_offers: [{ vendor: "NVIDIA", model: "RTX 3050", drivers: [["nvidia-driver-535", true]] }] },
        pci_detail: [{ address: "x", errors: { corrected: 3, nonfatal: 0, fatal: 0 } }],
        memory_detail: { controllers: [{ name: "MC0", corrected: 0, uncorrected: 0, dimms: [] }] },
      } as Record<string, unknown>)[kind],
    ),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(cleanup);

describe("Expert devices, screens, sound, and errors", () => {
  it("built-in devices and a slow Ethernet link", async () => {
    render(<BuiltInDevicesExpert />);
    await waitFor(() => expect(screen.getByText("Integrated_Webcam_FHD (uvcvideo)")).toBeTruthy());
    expect(screen.getByText("None supported by Ubuntu")).toBeTruthy();
    expect(screen.getByText("MX Master 3: 55%")).toBeTruthy();
    expect(screen.getByText("100Mb/s full (chip supports 1 Gb/s)")).toBeTruthy();
  });

  it("screens: maker, size, density, depth; backlight; sound codecs", async () => {
    render(<ScreensExpert />);
    await waitFor(() => expect(screen.getByText("LG Display")).toBeTruthy());
    expect(screen.getByText("13.9″ (31 × 17 cm)")).toBeTruthy();
    expect(screen.getByText("1920 × 1080 · 158 ppi")).toBeTruthy();
    expect(screen.getByText("6 bits")).toBeTruthy();
    expect(screen.getByText(/31% \(29,472 of 96,000/)).toBeTruthy();
    cleanup();
    render(<SoundHardwareExpert />);
    await waitFor(() => expect(screen.getByText("Realtek ALC3204, Intel Raptor Lake P HDMI")).toBeTruthy());
  });

  it("hardware errors gather the log and the counters", async () => {
    render(<HardwareErrorsPanel />);
    await waitFor(() => expect(screen.getByText("USB errors")).toBeTruthy());
    expect(screen.getByText("usb 3-2: device descriptor read/64, error -71")).toBeTruthy();
    await waitFor(() => expect(screen.getByText("PCIe link errors (counted by the hardware since start)")).toBeTruthy());
    expect(screen.getByText("5 in the last 7 days or since start")).toBeTruthy();
    expect(screen.getByText("nvidia-driver-535 (recommended)")).toBeTruthy();
  });

  it("screen size from the EDID", () => {
    expect(Math.round(screenSize(devices.screens[0]).inches ?? 0)).toBe(14);
    expect(screenSize({ ...devices.screens[0], width_cm: undefined })).toEqual({});
  });
});
