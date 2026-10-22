// SPDX-License-Identifier: Apache-2.0
// The list of open findings, with honest states for "agent not running" and "not checked yet".

import type { AppError, Finding, StatusReport } from "../api";
import { Panel, type Span } from "../components/Panel";
import { StatusBadge } from "../components/StatusBadge";
import { severityVariant } from "../status";
import { ago } from "../time";

export function IssuesPanel(props: {
  title: string;
  subtitle: string;
  findings: Finding[];
  report: StatusReport | null;
  error: AppError | null;
  now: number;
  span?: Span;
}) {
  return (
    <Panel title={props.title} subtitle={props.subtitle} span={props.span} flush>
      <Body {...props} />
    </Panel>
  );
}

function Body({ findings, report, error, now }: { findings: Finding[]; report: StatusReport | null; error: AppError | null; now: number }) {
  if (error?.kind === "agent_not_running") {
    return (
      <div className="empty">
        <StatusBadge variant="off" label="Agent not running" />
        <p>DeeCure's background agent is not running, so there is nothing to show yet.</p>
        <p>Start it with:</p>
        <pre className="cmd">systemctl --user start deecure-agent.socket</pre>
        <p>Working from a source checkout? Run:</p>
        <pre className="cmd">make agent</pre>
      </div>
    );
  }
  if (error) {
    return (
      <div className="empty">
        <StatusBadge variant="crit" label="Error" />
        <p>{error.message}</p>
      </div>
    );
  }
  if (!report) return <div className="empty muted">Loading…</div>;
  if (!report.checked_at) {
    return <div className="empty muted">DeeCure is checking this computer for the first time. This takes a few seconds.</div>;
  }
  if (findings.length === 0) {
    return (
      <div className="empty">
        <StatusBadge variant="ok" label="No problems found" />
        <p className="muted">Everything DeeCure checks here looks fine.</p>
      </div>
    );
  }
  return (
    <ul className="issues">
      {findings.map((f) => (
        <li key={`${f.rule}|${f.subject ?? ""}`} className="issue">
          <StatusBadge variant={severityVariant(f.severity)} />
          <div className="issue-body">
            <div className="issue-title">{f.title}</div>
            <p className="issue-explain">{f.explain}</p>
            <div className="issue-meta">
              <code>{f.rule}</code>
              {f.subject && <> · {f.subject}</>} · first seen {ago(f.first_seen, now)}
            </div>
          </div>
        </li>
      ))}
    </ul>
  );
}
