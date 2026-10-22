// SPDX-License-Identifier: Apache-2.0
import { describe, expect, it } from "vitest";
import type { Finding, StatusReport } from "./api";
import { pageStatus } from "./status";

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
    expect(pageStatus("logs", r, null)).toBe("off");
    expect(pageStatus("overview", r, null)).toBe("ok");
    expect(pageStatus("overview", report([], []), null)).toBe("off");
  });

  it("shows no badge on tools and settings", () => {
    expect(pageStatus("diagnostics", report([]), null)).toBeNull();
    expect(pageStatus("settings", report([]), null)).toBeNull();
  });
});
