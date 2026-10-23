// SPDX-License-Identifier: Apache-2.0
// The list of open findings, with honest states for "agent not running" and "not checked yet".

import type { ReactNode } from "react";
import type { AppError, Finding, StatusReport } from "../api";
import { Panel, type Span } from "../components/Panel";
import { StatusBadge } from "../components/StatusBadge";
import { NoData } from "./NoData";
import { severityVariant } from "../status";
import { ago } from "../time";
import { kbTopic, useKb } from "../kb";

export function IssuesPanel(props: {
  title: string;
  subtitle: string;
  findings: Finding[];
  report: StatusReport | null;
  error: AppError | null;
  now: number;
  span?: Span;
  actions?: ReactNode;
}) {
  return (
    <Panel title={props.title} subtitle={props.subtitle} span={props.span} actions={props.actions} flush>
      <Body {...props} />
    </Panel>
  );
}

function Body({ findings, report, error, now }: { findings: Finding[]; report: StatusReport | null; error: AppError | null; now: number }) {
  const kb = useKb();
  if (error) return <NoData error={error} waiting="" />;
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
            <HowTo topic={kbTopic(kb, f.rule)} />
          </div>
        </li>
      ))}
    </ul>
  );
}

/** The knowledge base article for a finding, folded until opened. */
function HowTo({ topic }: { topic?: { title: string; summary: string; steps: string[]; notes?: string } }) {
  if (!topic) return null;
  return (
    <details className="howto">
      <summary>How to fix: {topic.title}</summary>
      <p>{topic.summary}</p>
      <ol>
        {topic.steps.map((step) => (
          <li key={step}>{step}</li>
        ))}
      </ol>
      {topic.notes && <p className="muted">{topic.notes}</p>}
    </details>
  );
}

