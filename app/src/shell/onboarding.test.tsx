// SPDX-License-Identifier: Apache-2.0
// The first-launch screens and the live first check.

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Onboarding } from "./Onboarding";

let running = true;

vi.mock("../api", () => ({
  getStatus: () => (running ? Promise.resolve({ generated_at: 0, checked_at: 1, score: 95, findings: [] }) : Promise.reject({ kind: "agent_not_running", message: "not running" })),
  getLatest: () => Promise.resolve([{ name: "disk.used_percent", subject: "/", value: 43, unit: "percent", at: 0 }]),
  getSnapshot: () => Promise.resolve(null),
  getInventory: (kind: string) =>
    ({
      hardware: Promise.resolve({ machine: { model: "Latitude 3450" }, processor: { threads: 12 } }),
      system: Promise.resolve({ os: "Ubuntu 24.04.5 LTS", kernel: "6.8.0" }),
      packages: Promise.resolve({ packages: new Array(2150).fill({}) }),
      net_details: Promise.resolve({ connectivity: "full", wifi: { ssid: "HomeNet" } }),
      security: Promise.resolve({ hardening: [{ result: "pass" }, { result: "fail" }] }),
      disk_usage: Promise.resolve({ scanning: true }),
    })[kind] ?? Promise.reject(new Error("unknown")),
  toAppError: (e: { kind?: string; message?: string }) => ({ kind: e.kind ?? "other", message: e.message ?? String(e) }),
}));

afterEach(() => {
  cleanup();
  running = true;
  localStorage.clear();
});

describe("onboarding", () => {
  it("walks through the four screens and runs a real first check", async () => {
    const done = vi.fn();
    render(<Onboarding onDone={done} />);
    expect(screen.getByRole("heading", { name: "Welcome to DeeCure" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Next" }));
    expect(screen.getByText(/Nothing about your computer is sent anywhere/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Next" }));
    expect(screen.getByRole("heading", { name: "Are you new to Linux?" })).toBeTruthy();
    expect((screen.getByRole("radio", { name: /keep it simple/ }) as HTMLInputElement).checked).toBe(true);
    expect(screen.getAllByRole("radio")).toHaveLength(3);
    fireEvent.click(screen.getByRole("radio", { name: /show me everything/i }));
    expect(localStorage.getItem("deecure.mode")).toBe('"expert"');
    fireEvent.click(screen.getByRole("radio", { name: /know my way around/ }));
    expect(localStorage.getItem("deecure.mode")).toBe('"advanced"');
    fireEvent.click(screen.getByRole("button", { name: "Next" }));
    fireEvent.click(screen.getByRole("button", { name: "Start the first check" }));
    await waitFor(() => expect(screen.getByText("Latitude 3450 · 12 threads")).toBeTruthy());
    expect(screen.getByText("2,150 packages installed")).toBeTruthy();
    expect(screen.getByText("Wi-Fi HomeNet · online")).toBeTruthy();
    expect(screen.getByText(/Not available yet: read within a few minutes/)).toBeTruthy();
    await waitFor(() => expect(screen.getByText("95")).toBeTruthy());
    const open = screen.getByRole("button", { name: "Open DeeCure" }) as HTMLButtonElement;
    await waitFor(() => expect(open.disabled).toBe(false));
    fireEvent.click(open);
    expect(done).toHaveBeenCalled();
  });

  it("can be skipped, and helps when the agent is not running", async () => {
    const done = vi.fn();
    render(<Onboarding onDone={done} />);
    fireEvent.click(screen.getByRole("button", { name: "Skip" }));
    expect(done).toHaveBeenCalled();
    cleanup();
    running = false;
    render(<Onboarding onDone={done} />);
    for (const name of ["Next", "Next", "Next", "Start the first check"]) fireEvent.click(screen.getByRole("button", { name }));
    await waitFor(() => expect(screen.getByRole("button", { name: "Try again" })).toBeTruthy());
  });
});
