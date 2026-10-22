// SPDX-License-Identifier: Apache-2.0
// Overview › Issues (docs/03-ui-design/05-pages/01-overview.md), with real data.
// "Resolved" and "Mean time to resolve" need the agent to report closed findings; they come later.

import { KpiRow } from "../../components/KpiTile";
import { IssuesPanel } from "../IssuesPanel";
import type { PageProps } from "./shared";

export function OverviewIssues({ report, error, now }: PageProps) {
  const findings = report?.findings ?? [];
  const count = (severity: string) => findings.filter((f) => f.severity === severity).length;
  const critical = count("critical");
  const warning = count("warning");
  return (
    <>
      {report?.checked_at && (
        <KpiRow
          tiles={[
            { label: "Critical", value: String(critical), status: critical ? "crit" : "ok" },
            { label: "Warning", value: String(warning), status: warning ? "warn" : "ok" },
            { label: "Info", value: String(count("info")) },
          ]}
        />
      )}
      <IssuesPanel title="All open issues" subtitle="Everything DeeCure found, most serious first" findings={findings} report={report} error={error} now={now} />
    </>
  );
}
