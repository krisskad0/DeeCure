// SPDX-License-Identifier: Apache-2.0
// Shown on sub-pages that are not built yet: says so plainly, and points to the spec.

import type { Finding, StatusReport, AppError } from "../api";
import type { Route } from "../route";
import { IssuesPanel } from "./IssuesPanel";

export function NotBuilt(props: { route: Route; findings: Finding[]; report: StatusReport | null; error: AppError | null; now: number }) {
  const { page, tab } = props.route;
  return (
    <>
      <section className="panel span-12">
        <div className="panel-head">
          <h2>
            {page.label} › {tab.label}
          </h2>
          <span className="panel-sub">Not built yet</span>
        </div>
        <div className="empty">
          <p>This page comes later in Phase 1. Its full design is in the spec:</p>
          <pre className="path">{page.spec}</pre>
        </div>
      </section>
      {props.findings.length > 0 && (
        <IssuesPanel
          title={`${page.label} issues`}
          subtitle="Problems DeeCure already found in this area"
          findings={props.findings}
          report={props.report}
          error={props.error}
          now={props.now}
        />
      )}
    </>
  );
}
