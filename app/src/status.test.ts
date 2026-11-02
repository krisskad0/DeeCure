// SPDX-License-Identifier: Apache-2.0
import { describe, expect, it } from "vitest";
import type { Finding, StatusReport } from "./api";
import { PAGES } from "./spec/pages";
import { RULE_TAB, pageStatus, tabStatus } from "./status";

// The test runs in Node; Node's own modules are not in the app's types, so they are loaded by name.
const nodeFs = "node:fs";
const fs = (await import(/* @vite-ignore */ nodeFs)) as { readdirSync(path: string): string[]; readFileSync(path: string, encoding: string): string };

const finding = (rule: string, severity: Finding["severity"]): Finding => ({
  rule,
  severity,
  state: "open",
  title: "t",
  explain: "e",
  first_seen: 0,
  last_seen: 0,
});

const report = (findings: Finding[], checked = ["power", "services", "storage"]): StatusReport => ({
  generated_at: 1,
  checked_at: 1,
  checked,
  findings,
});

describe("pageStatus", () => {
  it("is Unknown while the agent is unreachable or has not checked yet", () => {
    expect(pageStatus("storage", null, { kind: "agent_not_running", message: "" })).toBe("off");
    expect(pageStatus("storage", { generated_at: 1, findings: [] }, null)).toBe("off");
  });

  it("uses the most serious finding of the page's area", () => {
    const r = report([finding("storage.a", "warning"), finding("storage.b", "critical"), finding("power.c", "info")]);
    expect(pageStatus("storage", r, null)).toBe("crit");
    expect(pageStatus("power", r, null)).toBe("info");
    expect(pageStatus("overview", r, null)).toBe("crit");
  });

  it("says Healthy only for areas that have rules", () => {
    const r = report([]);
    expect(pageStatus("services", r, null)).toBe("ok");
    expect(pageStatus("backups", r, null)).toBe("off");
    expect(pageStatus("overview", r, null)).toBe("ok");
    expect(pageStatus("overview", report([], []), null)).toBe("off");
  });

  it("shows no badge on tools, settings, and pages that are not a health area", () => {
    for (const page of ["diagnostics", "settings", "logs", "activity", "processes", "display"]) {
      expect(pageStatus(page, report([]), null)).toBeNull();
    }
  });
});

describe("tabStatus", () => {
  it("marks the sub-page that shows the finding, with its worst severity", () => {
    const r = report([finding("storage.fs_errors", "warning"), finding("storage.read_only", "critical"), finding("storage.disk_low", "info")]);
    expect(tabStatus("storage", "filesystems", r)).toBe("crit");
    expect(tabStatus("storage", "overview", r)).toBe("info");
    expect(tabStatus("storage", "usage", r)).toBeNull();
    expect(tabStatus("power", "battery", r)).toBeNull();
  });

  it("knows a real sub-page for every rule in rules/", () => {
    const root = "../rules";
    const ids = fs
      .readdirSync(root)
      .filter((d) => !d.includes("."))
      .flatMap((d) => fs.readdirSync(`${root}/${d}`).map((f) => fs.readFileSync(`${root}/${d}/${f}`, "utf8").match(/^id: (\S+)/m)?.[1]))
      .filter((id): id is string => id !== undefined);
    expect(ids.length).toBeGreaterThan(40);
    for (const id of ids) {
      const tab = RULE_TAB[id];
      expect(tab, id).toBeDefined();
      expect(PAGES.find((p) => p.id === id.split(".")[0])?.tabs.some((t) => t.id === tab), id).toBe(true);
    }
  });
});
