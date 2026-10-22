// SPDX-License-Identifier: Apache-2.0
// Phase 1, step 1: one live screen that proves the path window → Tauri → agent → database.
// The full shell (sidebar, pages, tabs) replaces this in the next steps.

import { useCallback, useEffect, useState } from "react";
import { type AppError, type Finding, type StatusReport, getStatus, toAppError } from "./api";
import { StatusBadge, type StatusVariant } from "./components/StatusBadge";
import { ago } from "./time";

const REFRESH_MS = 10_000;

const VARIANT: Record<Finding["severity"], StatusVariant> = {
  critical: "crit",
  warning: "warn",
  info: "info",
};

export default function App() {
  const [report, setReport] = useState<StatusReport | null>(null);
  const [error, setError] = useState<AppError | null>(null);
  const [now, setNow] = useState(Date.now());

  const refresh = useCallback(async () => {
    try {
      setReport(await getStatus());
      setError(null);
    } catch (e) {
      setError(toAppError(e));
    }
    setNow(Date.now());
  }, []);

  useEffect(() => {
    void refresh();
    const timer = setInterval(() => void refresh(), REFRESH_MS);
    return () => clearInterval(timer);
  }, [refresh]);

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <span className="logo" aria-hidden="true">
            <svg viewBox="0 0 24 24" width="18" height="18">
              <path d="M2.5 13h4l2.2-6 3.6 11 3-8 1.5 3H21.5" />
            </svg>
          </span>
          DeeCure
        </div>
        <AgentState report={report} error={error} now={now} />
      </header>

      <main className="main">
        <div className="page-header">
          <div>
            <div className="crumbs">Monitoring › Overview</div>
            <h1>Overview</h1>
          </div>
          <button className="btn" onClick={() => void refresh()}>
            Refresh
          </button>
        </div>

        <section className="panel">
          <div className="panel-head">
            <h2>Active issues</h2>
            <span className="panel-sub">Problems DeeCure found on this computer, most serious first</span>
          </div>
          <Issues report={report} error={error} now={now} />
        </section>
      </main>
    </div>
  );
}

function AgentState({ report, error, now }: { report: StatusReport | null; error: AppError | null; now: number }) {
  if (error) {
    return (
      <span className="live live-off">
        <span className="dot" /> Agent not reachable
      </span>
    );
  }
  if (!report) return <span className="live">Connecting…</span>;
  return (
    <span className="live">
      <span className="dot" /> Live · {report.checked_at ? `checked ${ago(report.checked_at, now)}` : "first check running"}
    </span>
  );
}

function Issues({ report, error, now }: { report: StatusReport | null; error: AppError | null; now: number }) {
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
  if (report.findings.length === 0) {
    return (
      <div className="empty">
        <StatusBadge variant="ok" label="No problems found" />
        <p className="muted">Everything DeeCure checks looks fine.</p>
      </div>
    );
  }
  return (
    <ul className="issues">
      {report.findings.map((f) => (
        <li key={`${f.rule}|${f.subject ?? ""}`} className="issue">
          <StatusBadge variant={VARIANT[f.severity]} />
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
