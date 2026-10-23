// SPDX-License-Identifier: Apache-2.0
// Expert firmware and ports: HSI and its checks, boot entries, USB speed, USB-C, Thunderbolt.

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { PlatformDetail } from "../../api";
import { FirmwareExpertSections, PortsExpertSections, hsiMeaning, usbSpeed } from "./PlatformExpert";

const d: PlatformDetail = {
  hsi: "HSI:1!",
  security_checks: [
    { id: "org.fwupd.hsi.Kernel.Swap", name: "Linux swap", result: "not-encrypted", passed: false, runtime: true },
    { id: "org.fwupd.hsi.Uefi.SecureBoot", name: "UEFI secure boot", result: "enabled", passed: true, runtime: false },
  ],
  boot_entries: [
    { number: "0000", name: "UEFI WD SSD", active: true, file: "\\EFI\\Boot\\BootX64.efi", order: 1, current: false },
    { number: "0006", name: "Ubuntu", active: true, file: "\\EFI\\ubuntu\\shimx64.efi", order: 0, current: true },
  ],
  secure_boot: "SecureBoot enabled",
  mok_keys: ["CN=grub"],
  management_engine: "0:16.1.40.2765",
  tpm_version: "2",
  tpm_driver: "tpm_tis",
  disk_unlocked_by_tpm: false,
  disk_encrypted: true,
  iommu_units: 2,
  thunderbolt_security: "user",
  thunderbolt_dma_protection: true,
  rtc_drift_s: 0,
  virtualization: "vmx",
  kvm_ready: true,
  usb: [
    { port: "4-1", product: "Portable SSD", version: "3.20", speed_mbps: 480, max_power_ma: 896, hub: false, drivers: ["uas"] },
    { port: "3-6", product: "Webcam", version: "2.01", speed_mbps: 480, max_power_ma: 500, hub: false, drivers: ["uvcvideo"] },
  ],
  typec: [{ name: "port0", data_role: "host", power_role: "source", power_mode: "usb_power_delivery", connected: true, partner_power_delivery: true, alt_modes: [{ name: "DisplayPort", active: true }] }],
  thunderbolt: [{ id: "0-1", name: "Dock", vendor: "CalDigit", authorized: false }],
};

vi.mock("../../api", () => ({
  getInventory: () => Promise.resolve(d),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(cleanup);

describe("Expert firmware and ports", () => {
  it("firmware: level, failed checks first, protections, and boot order", async () => {
    render(<FirmwareExpertSections />);
    await waitFor(() => expect(screen.getByText("1 of 2")).toBeTruthy());
    expect(screen.getByText("not-encrypted (running system)")).toBeTruthy();
    expect(screen.getByText("On, unlocked with a password")).toBeTruthy();
    expect(screen.getByText("Ask before new devices · DMA protection on")).toBeTruthy();
    expect(screen.getByText("Intel VT-x on · KVM ready")).toBeTruthy();
    expect(screen.getByText("this boot")).toBeTruthy();
  });

  it("ports: a USB 3 device held back, USB-C modes, and unapproved Thunderbolt", async () => {
    render(<PortsExpertSections />);
    await waitFor(() => expect(screen.getByText(/a USB 3 device on a USB 2 link/)).toBeTruthy());
    expect(screen.getByText("480 Mbit/s (USB 2)")).toBeTruthy();
    expect(screen.getByText("DisplayPort")).toBeTruthy();
    expect(screen.getByText(/Not approved/)).toBeTruthy();
  });

  it("plain words for HSI and speeds", () => {
    expect(hsiMeaning("HSI:1!")).toContain("Some protections are off");
    expect(hsiMeaning(undefined)).toBe("fwupd did not report a level.");
    expect(usbSpeed({ port: "x", speed_mbps: 5000, version: "3.10", hub: false, drivers: [] }).text).toBe("5 Gbit/s (USB 3)");
  });
});
