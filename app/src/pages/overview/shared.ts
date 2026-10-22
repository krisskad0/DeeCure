// SPDX-License-Identifier: Apache-2.0
// Small helpers the Overview pages share.

import type { AppError, Finding, StatusReport } from "../../api";
import type { StatusVariant } from "../../components/StatusBadge";
import { PAGES } from "../../spec/pages";
import { severityVariant } from "../../status";

export interface PageProps {
  report: StatusReport | null;
  error: AppError | null;
  now: number;
}

const RANK: Record<Finding["severity"], number> = { info: 1, warning: 2, critical: 3 };

/** The badge for the worst of these findings; `ok` when there are none. */
export function worst(findings: Finding[]): StatusVariant {
  if (findings.length === 0) return "ok";
  return severityVariant(findings.reduce((a, b) => (RANK[b.severity] > RANK[a.severity] ? b : a)).severity);
}

/** "storage" → "Storage", "jobs" → "Scheduled Jobs". */
export function areaLabel(id: string): string {
  return PAGES.find((p) => p.id === id)?.label ?? id;
}

export function plural(n: number, word: string): string {
  return `${n} ${word}${n === 1 ? "" : "s"}`;
}
