// SPDX-License-Identifier: Apache-2.0
// Overview › Health Score (docs/03-ui-design/05-pages/01-overview.md), with real data.
// Rule (docs/02-architecture/health-score.md): open critical −15, warning −5, info 0;
// each area loses at most 15 points. The agent saves the score after every check.

import { type Report, getMetrics, getWeeklyReports } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { Gauge } from "../../components/charts/Gauge";
import { LineChart } from "../../components/charts/LineChart";
import { RANGE_MS, scoreTone } from "../../metrics";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";
import { type PageProps, areaLabel, worst } from "./shared";

/** The most points one area can take off. Mirrors `AREA_SHARE` in deecure-core. */
const AREA_SHARE = 15;

const loadWeekly = () => getWeeklyReports();

function WeeklySummaries() {
  const { data, error } = useLive(loadWeekly, 600_000);
  return (
    <Panel title="Weekly summaries" subtitle="Sunday 18:00" info="DeeCure saves one each week, and shows it as a notification when Settings › Notifications › Weekly summary is on." span={12} flush>
      {!data ? (
        <NoData error={error} waiting="Loading…" />
      ) : data.length ? (
        <DataTable rows={data} columns={WEEKLY} rowKey={(r) => String(r.created_at)} empty="" />
      ) : (
        <div className="empty muted">The first weekly summary comes on Sunday, after DeeCure has watched this computer for a few days.</div>
      )}
    </Panel>
  );
}

const total = (r: Report) => Object.values(r.changes).reduce((a, b) => a + b, 0);
const WEEKLY: Column<Report>[] = [
  { key: "week", label: "Week", text: (r) => String(r.since), render: (r) => `${new Date(r.since).toLocaleDateString(undefined, { day: "numeric", month: "short" })} - ${new Date(r.until).toLocaleDateString(undefined, { day: "numeric", month: "short" })}` },
  { key: "score", label: "Health", text: (r) => (r.score === undefined ? "-" : String(r.score)) },
  { key: "changes", label: "Changes", text: (r) => String(total(r)) },
  { key: "problems", label: "Problems", text: (r) => String(r.findings.filter((f) => f.severity !== "info").length) },
  { key: "tip", label: "Tip", text: (r) => r.tip, wrap: true },
];

const loadTrend = () => {
  const until = Date.now();
  return getMetrics("health.score", until - RANGE_MS["30d"], until, { maxPoints: 30 });
};

interface Row {
  area: string;
  deduction: number;
  findings: string;
  status: StatusVariant;
}

const COLUMNS: Column<Row>[] = [
  { key: "area", label: "Area", text: (r) => r.area },
  { key: "max", label: "Most it can take", text: () => String(AREA_SHARE), numeric: true },
  { key: "deduction", label: "Deduction", text: (r) => (r.deduction ? `−${r.deduction}` : "0"), numeric: true },
  { key: "findings", label: "Open findings", text: (r) => r.findings, wrap: true },
  { key: "status", label: "Status", text: (r) => r.status, render: (r) => <StatusBadge variant={r.status} /> },
];

export function OverviewScore({ report, error }: PageProps) {
  const trend = useLive(loadTrend, 300_000).data;
  if (error || !report || report.score === undefined) {
    return (
      <Panel title="Current score" span={12}>
        <NoData error={error} waiting="The score appears after DeeCure's first check." />
      </Panel>
    );
  }
  const score = report.score;
  // Areas that have rules, plus any area that lost points.
  const areas = [...new Set([...(report.checked ?? []), ...(report.deductions ?? []).map((d) => d.category)])];
  const rows: Row[] = areas
    .map((id) => {
      const open = report.findings.filter((f) => f.rule.split(".")[0] === id);
      return {
        area: areaLabel(id),
        deduction: report.deductions?.find((d) => d.category === id)?.points ?? 0,
        findings: open.map((f) => f.title).join(" · ") || "-",
        status: worst(open),
      };
    })
    .sort((a, b) => b.deduction - a.deduction || a.area.localeCompare(b.area));

  return (
    <>
      <Panel title="Health score trend" subtitle="30 days, daily average" span={8}>
        {trend && trend.length >= 2 ? (
          <LineChart series={[{ name: "Score", data: trend.map((p) => p.value), slot: 1 }]} x={{ kind: "time", times: trend.map((p) => p.at), range: "30d" }} height={240} min={0} max={100} />
        ) : (
          <div className="chart-empty muted">The trend fills in as DeeCure keeps checking, one point a day.</div>
        )}
      </Panel>
      <Panel title="Current score" span={4}>
        <Gauge value={score} subtitle="out of 100" status={scoreTone(score)} />
        <p className="muted center">90+ Healthy · 75-89 Fair · below 75 At risk</p>
      </Panel>
      <Panel title="Score breakdown" subtitle="Areas DeeCure checks today" span={12} flush>
        <DataTable rows={rows} columns={COLUMNS} rowKey={(r) => r.area} empty="No areas are checked yet" />
      </Panel>
      <WeeklySummaries />
    </>
  );
}
