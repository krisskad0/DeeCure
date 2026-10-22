// SPDX-License-Identifier: Apache-2.0
// Page status (docs/03-ui-design/04-behavior/page-status.md), worked out from real findings.
// A page is "Healthy" only if DeeCure really checks that area; otherwise it is "Unknown".

import type { AppError, Finding, StatusReport } from "./api";
import type { StatusVariant } from "./components/StatusBadge";

/** Tools and Settings never show a status badge. */
const NO_BADGE = new Set(["diagnostics", "assistant", "settings"]);
const RANK: Record<Finding["severity"], number> = { info: 1, warning: 2, critical: 3 };
const VARIANT: Record<Finding["severity"], StatusVariant> = { info: "info", warning: "warn", critical: "crit" };

export function severityVariant(severity: Finding["severity"]): StatusVariant {
  return VARIANT[severity];
}

/** The page a finding belongs to: the category before the dot in its rule ID. */
export function pageOf(finding: Finding): string {
  return finding.rule.split(".")[0];
}

export function findingsFor(pageId: string, report: StatusReport | null): Finding[] {
  if (!report) return [];
  return pageId === "overview" ? report.findings : report.findings.filter((f) => pageOf(f) === pageId);
}

/** `null` means "show no badge". */
export function pageStatus(pageId: string, report: StatusReport | null, error: AppError | null): StatusVariant | null {
  if (NO_BADGE.has(pageId)) return null;
  if (error || !report?.checked_at) return "off";
  const findings = findingsFor(pageId, report);
  if (findings.length === 0) {
    const checked = pageId === "overview" ? (report.checked ?? []).length > 0 : (report.checked ?? []).includes(pageId);
    return checked ? "ok" : "off";
  }
  const worst = findings.reduce((a, b) => (RANK[b.severity] > RANK[a.severity] ? b : a));
  return VARIANT[worst.severity];
}
