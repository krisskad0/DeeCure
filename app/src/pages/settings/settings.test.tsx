// SPDX-License-Identifier: Apache-2.0
// Diagnostics runner, shared preferences, and the Diagnostics, Settings, and Assistant pages.

import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { usePref } from "../../prefs";
import { AssistantChat } from "../assistant/Assistant";
import { DiagnosticsHardware, DiagnosticsNetwork, DiagnosticsPerformance } from "../diagnostics/Diagnostics";
import { SettingsAbout, SettingsData, SettingsGeneral, SettingsNotifications } from "./Settings";

const saved: unknown[] = [];

vi.mock("../../api", () => ({
  getInventory: (kind: string) =>
    kind === "diagnose_performance"
      ? Promise.resolve({ steps: [{ name: "Memory pressure", result: "crit", value: "tasks waited 23.4% of the time" }], verdict: "Memory is the bottleneck.", tone: "crit", top: [["chrome", 80]] })
      : kind === "diagnose_network"
        ? Promise.reject({ kind: "agent_not_running", message: "The agent is not running." })
        : Promise.resolve({ version: "0.1.0", database: "/home/riya/.local/share/deecure/deecure.db", database_bytes: 74e6, rules: 8, rules_dir: "/usr/share/deecure/rules", uptime_seconds: 3700, memory_bytes: 9e6 }),
  getLatest: () => Promise.resolve([{ name: "temp.cpu", value: 54, unit: "celsius", at: 0 }]),
  getTrayState: () => Promise.resolve({ enabled: false, active: false, available: false }),
  setTray: () => Promise.reject("this desktop has no tray area"),
  getNotifySettings: () => Promise.resolve({ critical: true, warnings: false, weekly: true, quiet_hours: "22-07" }),
  setNotifySettings: (s: unknown) => {
    saved.push(s);
    return Promise.resolve(s);
  },
  sendTestNotification: () => Promise.reject({ message: "no notification service is running (is a desktop session open?)" }),
  getNotifications: () => Promise.resolve([{ at: Date.now(), title: "Main disk is almost full", body: "Only 3 GB left." }]),
  toAppError: (e: { message?: string }) => ({ kind: "other", message: e.message ?? String(e) }),
}));

afterEach(cleanup);

function Mode({ id }: { id: string }) {
  const [mode, setMode] = usePref<"beginner" | "expert">("mode", "beginner");
  return (
    <button type="button" data-testid={id} onClick={() => setMode(mode === "expert" ? "beginner" : "expert")}>
      {mode}
    </button>
  );
}

describe("preferences", () => {
  it("a change in one place updates every hook with the same key", () => {
    render(
      <>
        <Mode id="a" />
        <Mode id="b" />
      </>,
    );
    fireEvent.click(screen.getByTestId("a"));
    expect(screen.getByTestId("b").textContent).toBe("expert");
    fireEvent.click(screen.getByTestId("b"));
    expect(screen.getByTestId("a").textContent).toBe("beginner");
  });
});

describe("Diagnostics", () => {
  it("runs the performance check and names the cause", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    render(<DiagnosticsPerformance />);
    fireEvent.click(screen.getByRole("button", { name: /Run check/ }));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
    });
    vi.useRealTimers();
    await waitFor(() => expect(screen.getByText("Memory is the bottleneck.")).toBeTruthy());
    expect(screen.getByText("chrome")).toBeTruthy();
    expect(screen.getByRole("button", { name: /Run again/ })).toBeTruthy();
  });

  it("says when the check cannot run", async () => {
    render(<DiagnosticsNetwork />);
    fireEvent.click(screen.getByRole("button", { name: /Run test/ }));
    await waitFor(() => expect(screen.getByText("The check could not run")).toBeTruthy());
  });

  it("hardware tests show real results", async () => {
    render(<DiagnosticsHardware />);
    await waitFor(() => expect(screen.getByText("CPU 54 °C")).toBeTruthy());
  });
});

describe("Settings and Assistant", () => {
  it("expert mode switch works", () => {
    render(<SettingsGeneral />);
    const toggle = screen.getByRole("switch", { name: "Expert mode" }) as HTMLInputElement;
    const before = toggle.checked;
    fireEvent.click(toggle);
    expect(toggle.checked).toBe(!before);
  });

  it("data and about show the agent's facts", async () => {
    render(<SettingsData />);
    await waitFor(() => expect(screen.getByText("70.6 MB")).toBeTruthy());
    cleanup();
    render(<SettingsAbout />);
    await waitFor(() => expect(screen.getByText("8 from /usr/share/deecure/rules")).toBeTruthy());
  });

  it("the tray switch says when the desktop has no tray area", async () => {
    render(<SettingsGeneral />);
    await waitFor(() => expect(screen.getByText("This desktop has no tray area")).toBeTruthy());
    const tray = screen.getByRole("switch", { name: "Tray icon" }) as HTMLInputElement;
    expect(tray.checked).toBe(false);
    fireEvent.click(tray);
    await waitFor(() => expect((screen.getByRole("switch", { name: "Tray icon" }) as HTMLInputElement).checked).toBe(false));
  });

  it("notification choices are saved in the agent", async () => {
    render(<SettingsNotifications />);
    await waitFor(() => expect(screen.getByRole("switch", { name: "Warnings" })).toBeTruthy());
    expect(screen.getByText("Main disk is almost full")).toBeTruthy();
    fireEvent.click(screen.getByRole("switch", { name: "Warnings" }));
    await waitFor(() => expect(saved).toEqual([{ critical: true, warnings: true, weekly: true, quiet_hours: "22-07" }]));
    fireEvent.change(screen.getByRole("combobox", { name: "Quiet hours" }), { target: { value: "off" } });
    await waitFor(() => expect(saved[1]).toEqual({ critical: true, warnings: true, weekly: true, quiet_hours: "off" }));
    fireEvent.click(screen.getByRole("button", { name: "Send a test notification" }));
    await waitFor(() => expect(screen.getByRole("button", { name: "Send a test notification" })).toBeTruthy());
  });

  it("assistant is honest that it comes later", () => {
    render(<AssistantChat />);
    expect(screen.getByText("Coming in a later version")).toBeTruthy();
  });
});
