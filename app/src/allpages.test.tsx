// SPDX-License-Identifier: Apache-2.0
// Every page and tab, at the Expert level, through the real app code with the desktop bridge
// replaced. A page passes when nothing crashed (no "could not be shown" message) and React
// logged no error or warning.
//
// - "agent not running" and "older agent": always run. Every page must show its help message.
// - "real answers": runs when DEECURE_UI_DATA names a JSON file of the agent's real answers
//   (recorded on a real computer; it holds that computer's details, so it is never committed).

import { act, cleanup, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import { VISIBLE_PAGES } from "./visible";

type Data = Record<string, unknown> & { inventory: Record<string, unknown>; snapshot: Record<string, unknown>; metrics: Record<string, unknown> };

// The test runs in Node; Node's own modules are not in the app's types, so they are loaded by name.
declare const process: { env: Record<string, string | undefined> };
const nodeFs = "node:fs";
const fs = (await import(/* @vite-ignore */ nodeFs)) as { existsSync(path: string): boolean; readFileSync(path: string, encoding: string): string };
const file = process.env.DEECURE_UI_DATA;
const real: Data | undefined = file && fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, "utf8")) : undefined;

/** Kinds and methods older agents (0.1.0-alpha.3) do not know. */
const NEW_KINDS = new Set([
  "cpu_detail", "memory_detail", "process_memory", "cgroups", "storage_detail", "process_io", "pci_detail", "gpu_detail", "graphics_stack",
  "net_deep", "tcp_traffic", "power_detail", "thermal", "boot_detail", "unit_files", "service_security", "kernel_detail", "platform_detail",
  "devices_detail", "hw_errors", "software_health", "security_detail", "package_verify", "package_verify_run", "optimize",
]);

let scenario: "real" | "down" | "older" = "down";

const outdated = { kind: "agent_outdated", message: "The background agent is older than this app, so it cannot answer this yet. Restart the agent to use the new version." };
const down = { kind: "agent_not_running", message: "The DeeCure agent is not running" };

function answer(cmd: string, args: Record<string, unknown> = {}): Promise<unknown> {
  // What the desktop itself answers, without the agent.
  if (cmd === "machine") return Promise.resolve({ hostname: "test-laptop", os: "Ubuntu 24.04 LTS", user: "test" });
  if (cmd === "tray_state") return Promise.resolve({ enabled: false, active: false, available: true });
  if (cmd === "helper_status") return Promise.resolve(true);
  if (cmd === "helper_read") return Promise.reject({ kind: "other", message: "not in this test" });
  if (scenario === "down" || !real) return Promise.reject(down);
  const older = scenario === "older";
  switch (cmd) {
    case "inventory": {
      const kind = String(args.kind);
      if (older && NEW_KINDS.has(kind)) return Promise.reject(outdated);
      return Promise.resolve(real.inventory[kind] ?? null);
    }
    case "process_detail":
    case "package_detail":
      return older ? Promise.reject({ kind: "agent_outdated", message: outdated.message }) : Promise.resolve(real[cmd]);
    case "metrics":
      return Promise.resolve(real.metrics[String(args.name)] ?? real.metrics["cpu.usage"] ?? []);
    case "snapshot":
      return Promise.resolve(real.snapshot[String(args.kind)] ?? null);
    case "status":
    case "latest":
    case "events":
    case "warning_settings":
      // An agent from before this version does not know it.
      return older ? Promise.reject({ kind: "agent_outdated", message: outdated.message }) : Promise.resolve(real[cmd]);
    case "notify_settings":
    case "data_settings":
    case "weekly_reports":
    case "fix_runs":
    case "notifications":
    case "fix_plan":
      return Promise.resolve(real[cmd]);
    default:
      // Writes (settings, fixes) are not pressed in this test.
      return Promise.resolve(null);
  }
}

vi.mock("@tauri-apps/api/core", () => ({ invoke: (cmd: string, args?: Record<string, unknown>) => answer(cmd, args) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => {}) }));

const ROUTES = VISIBLE_PAGES.flatMap((p) => p.tabs.map((t) => `${p.id}/${t.id}`));

let problems: string[] = [];

beforeEach(() => {
  localStorage.setItem("deecure.onboarded", "true");
  localStorage.setItem("deecure.mode", JSON.stringify("expert"));
  Element.prototype.scrollIntoView = () => {};
  problems = [];
  const keep = (kind: string) => (...a: unknown[]) => {
    const text = a.map((x) => (x instanceof Error ? x.message : String(x))).join(" ");
    // Updates after the test's last wait are a test artifact, not a page problem.
    if (text.includes("not wrapped in act")) return;
    problems.push(`${kind}: ${text.slice(0, 300)}`);
  };
  vi.spyOn(console, "error").mockImplementation(keep("error"));
  vi.spyOn(console, "warn").mockImplementation(keep("warn"));
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  localStorage.clear();
});

async function visit(route: string) {
  window.location.hash = `#/${route}`;
  const view = render(<App />);
  for (let i = 0; i < 6; i++) {
    await act(async () => {
      await new Promise((r) => setTimeout(r, 15));
    });
  }
  // Drawers open on a click: open the first row's drawer where a table has one.
  const row = view.container.querySelector("tr.clickable");
  if (row) {
    await act(async () => {
      (row as HTMLElement).click();
      await new Promise((r) => setTimeout(r, 30));
    });
    for (let i = 0; i < 4; i++) {
      await act(async () => {
        await new Promise((r) => setTimeout(r, 15));
      });
    }
  }
  const broken = [...document.querySelectorAll('[role="alert"]')].map((e) => e.textContent ?? "").filter((t) => t.includes("could not be shown"));
  view.unmount();
  return broken;
}

describe("every page at the Expert level", () => {
  it("with the agent not running, every page shows its help instead of crashing", async () => {
    scenario = "down";
    const failed: string[] = [];
    for (const route of ROUTES) {
      const broken = await visit(route);
      if (broken.length || problems.length) failed.push(`${route}: ${[...broken, ...problems].join(" | ")}`);
      problems = [];
    }
    expect(failed).toEqual([]);
  }, 120_000);

  it.skipIf(!real)("with an older agent, the Expert parts say the agent needs a restart", async () => {
    scenario = "older";
    const failed: string[] = [];
    for (const route of ROUTES) {
      const broken = await visit(route);
      if (broken.length || problems.length) failed.push(`${route}: ${[...broken, ...problems].join(" | ")}`);
      problems = [];
    }
    expect(failed).toEqual([]);
  }, 120_000);

  it.skipIf(!real)("with this computer's real answers, every page draws without an error", async () => {
    scenario = "real";
    const failed: string[] = [];
    for (const route of ROUTES) {
      const broken = await visit(route);
      if (broken.length || problems.length) failed.push(`${route}: ${[...broken, ...problems].join(" | ")}`);
      problems = [];
    }
    expect(failed).toEqual([]);
  }, 180_000);
});
