// SPDX-License-Identifier: Apache-2.0
// Expert drive sections: an honest PCIe verdict, drive details, TRIM, filesystems, disk use.

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { PciDevice } from "../../api";
import { DiskIoExpertSections, DriveExpertSections, FilesystemExpert, PcieLinks, linkVerdict } from "./DiskExpert";

const ssd: PciDevice = {
  address: "0000:01:00.0",
  class: "Storage",
  model: "WD PC SN5000S",
  link_speed: "8.0 GT/s PCIe",
  max_link_speed: "16.0 GT/s PCIe",
  link_width: 4,
  max_link_width: 4,
  slot_max_link_speed: "16.0 GT/s PCIe",
  slot_max_link_width: 4,
  errors: { corrected: 2, nonfatal: 0, fatal: 0 },
  power_control: "on",
};

let timerOn = true;
vi.mock("../../api", () => ({
  getInventory: (kind: string) =>
    Promise.resolve(
      ({
        pci_detail: [ssd, { address: "0000:00:07.0", class: "Bridge", model: "Thunderbolt", link_speed: "2.5 GT/s PCIe", max_link_speed: "2.5 GT/s PCIe", link_width: 0, max_link_width: 4 }],
        storage_detail: {
          disks: [
            {
              name: "nvme0n1",
              rotational: false,
              scheduler: "none",
              schedulers: ["none", "mq-deadline"],
              trim: true,
              table: "gpt",
              partitions: [
                { name: "nvme0n1p1", start_sector: 2048, size: 1e9, aligned: true },
                { name: "nvme0n1p2", start_sector: 2099263, size: 1e9, aligned: false },
              ],
              nvme: { firmware: "34012146", pci_address: "0000:01:00.0", temp_warning: 82.85, temp_critical: 87.85 },
            },
          ],
          filesystems: [{ device: "/dev/sdb1", mount: "/media/data", fstype: "ext4", options: ["rw", "discard"], inodes: 1000, inodes_free: 250, errors: 2 }],
          nvme_max_latency_us: 100000,
        },
        jobs: { cron: [], anacron: [], at: [], timers: timerOn ? [{ name: "fstrim.timer", scope: "system", activates: "fstrim.service", schedule: [], next: Date.now() + 1e8, last: Date.now() - 1e8, persistent: true }] : [] },
        cgroups: { at: 1, groups: [{ kind: "service", unit: "systemd-journald.service", name: "systemd-journald", disk_read: 10, disk_write: 9 * 1024 ** 3 }] },
        process_io: { at: 1, processes: [{ pid: 5, name: "code", read_bytes: 1, write_bytes: 2048, cancelled_write_bytes: 0 }], unreadable: 232 },
      } as Record<string, unknown>)[kind],
    ),
  toAppError: (e: unknown) => ({ kind: "other", message: String(e) }),
}));

afterEach(() => {
  cleanup();
  timerOn = true;
});

describe("PCIe link verdict", () => {
  it("compares against the slower of device and slot", () => {
    expect(linkVerdict({ ...ssd, link_speed: "16.0 GT/s PCIe" }).text).toBe("Full speed");
    expect(linkVerdict({ ...ssd, slot_max_link_speed: "8.0 GT/s PCIe" }).text).toBe("Full speed");
    expect(linkVerdict(ssd)).toEqual({ variant: "warn", text: "Slower than both sides support: 8 of 16 GT/s" });
    expect(linkVerdict({ ...ssd, link_speed: "16.0 GT/s PCIe", link_width: 2 }).text).toContain("2 of 4 lanes");
  });

  it("does not blame cards that slow down when idle, or empty ports", () => {
    expect(linkVerdict({ ...ssd, class: "Display" }).variant).toBe("info");
    expect(linkVerdict({ ...ssd, power_state: "suspended" }).variant).toBe("info");
    expect(linkVerdict({ ...ssd, link_width: 0 }).text).toBe("Nothing connected");
    expect(linkVerdict({ address: "x" }).text).toBe("Not reported");
  });
});

describe("Expert drive sections", () => {
  it("drive details: link, alignment, TRIM, and temperature limits", async () => {
    render(<DriveExpertSections />);
    await waitFor(() => expect(screen.getByText(/Slower than both sides support/)).toBeTruthy());
    expect(screen.getByText("No: slower on an SSD")).toBeTruthy();
    expect(screen.getByText("Off (always awake)")).toBeTruthy();
    expect(screen.getByText("warning 83 °C · critical 88 °C")).toBeTruthy();
    expect(screen.getByText("On (wake-up up to 100 ms)")).toBeTruthy();
    await waitFor(() => expect(screen.getByText("/media/data")).toBeTruthy());
  });

  it("says so when weekly TRIM is not set up", async () => {
    timerOn = false;
    render(<DriveExpertSections />);
    await waitFor(() => expect(screen.getByText("Not set up")).toBeTruthy());
  });

  it("filesystems show inode use and recorded errors", async () => {
    render(<FilesystemExpert />);
    await waitFor(() => expect(screen.getByText("75.0% of 1,000")).toBeTruthy());
    expect(screen.getByText("2")).toBeTruthy();
  });

  it("disk use per service and process, and who is not counted", async () => {
    render(<DiskIoExpertSections />);
    await waitFor(() => expect(screen.getByText("systemd-journald")).toBeTruthy());
    expect(screen.getByText("9.0 GB")).toBeTruthy();
    await waitFor(() => expect(screen.getByText(/232 processes of other users/)).toBeTruthy());
  });

  it("PCIe links list empty ports and link errors", async () => {
    render(<PcieLinks />);
    await waitFor(() => expect(screen.getByText("Nothing connected")).toBeTruthy());
    expect(screen.getByText("2 corrected")).toBeTruthy();
  });
});
