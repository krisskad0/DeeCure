// SPDX-License-Identifier: Apache-2.0
// The admin-only hardware views, with answers shaped like the helper's.

import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { BiosSettingsView, MemorySlotsView, NvmeDetailsView, SelfTestsView, channelsInUse, moduleSpeed } from "./AdminDetails";

afterEach(cleanup);

describe("memory slots", () => {
  it("counts channels from bank names", () => {
    const m = (bank: string, filled = true) => ({ "Bank Locator": bank, Locator: "DIMM", Filled: String(filled) });
    expect(channelsInUse([m("P0 CHANNEL A"), m("P0 CHANNEL B")])).toBe(2);
    expect(channelsInUse([m("P0 CHANNEL A"), m("P0 CHANNEL B", false)])).toBe(1);
    expect(channelsInUse([m("BANK 0"), m("BANK 1")])).toBe(2);
    expect(channelsInUse([{ Locator: "ChannelA-DIMM0", Filled: "true" }])).toBe(1);
    expect(channelsInUse([m("Not Specified")])).toBeUndefined();
  });

  it("compares the running speed with the rated one", () => {
    expect(moduleSpeed({ Speed: "5600 MT/s", "Configured Memory Speed": "4800 MT/s" })).toEqual({ text: "4800 MT/s (rated 5600 MT/s)", slower: true });
    expect(moduleSpeed({ Speed: "4267 MT/s", "Configured Memory Speed": "4267 MT/s" })).toEqual({ text: "4267 MT/s", slower: false });
    expect(moduleSpeed({ Speed: "Unknown" }).text).toBe("-");
  });

  it("warns about single-channel memory and shows free slots", () => {
    render(
      <MemorySlotsView
        data={{
          arrays: [{ "Maximum Capacity": "64 GB", "Error Correction Type": "None" }],
          modules: [
            { Locator: "DIMM A", "Bank Locator": "P0 CHANNEL A", Size: "16 GB", Type: "DDR5", "Form Factor": "SODIMM", Speed: "5600 MT/s", "Configured Memory Speed": "5600 MT/s", Filled: "true" },
            { Locator: "DIMM B", "Bank Locator": "P0 CHANNEL B", Size: "No Module Installed", Filled: "false" },
          ],
        }}
      />,
    );
    expect(screen.getByText("64 GB")).toBeTruthy();
    expect(screen.getByText("2 (1 free)")).toBeTruthy();
    expect(screen.getByText(/single-channel/)).toBeTruthy();
    expect(screen.getByText("Empty")).toBeTruthy();
  });

  it("says soldered memory cannot be upgraded, without a channel warning", () => {
    render(<MemorySlotsView data={{ arrays: [], modules: [{ Locator: "Channel-0", "Bank Locator": "BANK 0", Size: "8 GB", "Form Factor": "Row Of Chips", Filled: "true" }] }} />);
    expect(screen.getByText(/cannot be upgraded/)).toBeTruthy();
    expect(screen.queryByText(/single-channel/)).toBeNull();
  });
});

describe("BIOS settings", () => {
  it("lists settings, marks changed ones, and shows password state", () => {
    render(
      <BiosSettingsView
        data={[
          {
            name: "dell-wmi-sysman",
            admin_password: false,
            system_password: true,
            settings: [
              { name: "WakeOnLan", label: "Wake on LAN", kind: "enumeration", value: "LanOnly", default: "Disabled", options: ["Disabled", "LanOnly"] },
              { name: "PrimaryBattChargeCfg", kind: "integer", value: "80", default: "80", min: "50", max: "100" },
            ],
          },
        ]}
      />,
    );
    expect(screen.getByText("Wake on LAN")).toBeTruthy();
    expect(screen.getAllByText("changed")).toHaveLength(1);
    expect(screen.getByText("50 to 100")).toBeTruthy();
    expect(screen.getByText("Not set")).toBeTruthy();
    expect(screen.getByText("2 (1 changed from the default)")).toBeTruthy();
  });

  it("explains an empty answer", () => {
    render(<BiosSettingsView data={[]} />);
    expect(screen.getByText(/offers no BIOS settings/)).toBeTruthy();
  });
});

describe("NVMe and self-tests", () => {
  it("shows firmware slots and a faster block format", () => {
    render(
      <NvmeDetailsView
        data={{
          controller: { firmware_slots: 3, slot1_read_only: true, power_states: 5, autonomous_power_saving: true, namespaces: 1, write_cache: true, self_test: true },
          namespace: {
            size_bytes: 512e9,
            formats: [
              { block_size: 512, metadata: 0, performance: "good", in_use: true },
              { block_size: 4096, metadata: 0, performance: "best", in_use: false },
            ],
          },
          firmware: { active: 2, next: null, slots: [{ slot: 1, version: "1.0" }, { slot: 2, version: "1.2" }] },
        }}
      />,
    );
    expect(screen.getByText(/slot 1 is read-only/)).toBeTruthy();
    expect(screen.getByText("1: 1.0 · 2: 1.2 (running)")).toBeTruthy();
    expect(screen.getByText(/4096-byte blocks are faster/)).toBeTruthy();
  });

  it("shows the self-test log and a missing one", () => {
    render(<SelfTestsView data={{ supported: true, running: null, tests: [{ kind: "Short", result: "Completed without error", passed: true, hours: 1200 }] }} />);
    expect(screen.getByText("Completed without error")).toBeTruthy();
    cleanup();
    render(<SelfTestsView data={{ supported: false, tests: [] }} />);
    expect(screen.getByText(/no self-test log/)).toBeTruthy();
  });
});
