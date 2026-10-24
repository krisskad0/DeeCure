// SPDX-License-Identifier: Apache-2.0
// The agent's latest status report (findings), for pages that show a warning's own decision
// instead of judging the numbers again, so the page and Overview always agree and the user's
// own limits apply (docs/01-product/health-warnings.md).

import { createContext, useContext } from "react";
import type { Finding, StatusReport } from "./api";

export const ReportContext = createContext<StatusReport | null>(null);

/** The open finding of a rule (and subject), if any. */
export function useFinding(rule: string, subject?: string): Finding | undefined {
  const report = useContext(ReportContext);
  return report?.findings.find((f) => f.rule === rule && (subject === undefined || f.subject === subject));
}
