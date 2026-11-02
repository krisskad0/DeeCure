// SPDX-License-Identifier: Apache-2.0
// Overview › Health Score (docs/03-ui-design/05-pages/01-overview.md), with real data.
// Rule (docs/02-architecture/health-score.md): open critical −15, warning −5, info 0;
// each area loses at most 15 points. The agent saves the score after every check.

import { type Finding, type Report, getMetrics, getWeeklyReports } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { Icon } from "../../components/Icon";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { Gauge } from "../../components/charts/Gauge";
import { LineChart } from "../../components/charts/LineChart";
import { RANGE_MS, scoreTone } from "../../metrics";
import { href } from "../../route";
import { PAGES } from "../../spec/pages";
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

export interface AreaScore {
  id: string;
  label: string;
  deduction: number;
  findings: Finding[];
  status: StatusVariant;
}

/** Every checked area, plus any area that lost points: the ones losing most first. */
export function areaScores(report: NonNullable<PageProps["report"]>): AreaScore[] {
  const ids = [...new Set([...(report.checked ?? []), ...(report.deductions ?? []).map((d) => d.category)])];
  return ids
    .map((id) => {
      const findings = report.findings.filter((f) => f.rule.split(".")[0] === id);
      return { id, label: areaLabel(id), deduction: report.deductions?.find((d) => d.category === id)?.points ?? 0, findings, status: worst(findings) };
    })
    .sort((a, b) => b.deduction - a.deduction || a.label.localeCompare(b.label));
}

const SEVERITY_ORDER: Record<Finding["severity"], number> = { critical: 0, warning: 1, info: 2 };

/** How the score is reached: 100, minus each area's points, equals the score. */
function ScoreSum({ areas, score }: { areas: AreaScore[]; score: number }) {
  const losing = areas.filter((a) => a.deduction > 0);
  return (
    <div className="score-sum" aria-label={`Score: 100${losing.map((a) => ` minus ${a.deduction} for ${a.label}`).join("")} equals ${score}`}>
      <span className="score-start">100</span>
      {losing.length === 0 ? (
        <span className="muted">Nothing takes points off.</span>
      ) : (
        losing.map((a) => (
          <span key={a.id} className={`score-chip score-chip-${a.status}`}>
            −{a.deduction} {a.label}
          </span>
        ))
      )}
      <span className="score-equals">=</span>
      <span className={`score-total score-total-${scoreTone(score)}`}>{score}</span>
    </div>
  );
}

function AreaCard({ area }: { area: AreaScore }) {
  const page = PAGES.find((p) => p.id === area.id);
  const kept = AREA_SHARE - area.deduction;
  const open = [...area.findings].sort((a, b) => SEVERITY_ORDER[a.severity] - SEVERITY_ORDER[b.severity]);
  return (
    <a className={`area-card area-card-${area.status}`} href={href(area.id, page?.tabs[0]?.id ?? "")}>
      <div className="area-card-head">
        {page && <Icon name={page.icon} size={16} />}
        <span className="area-card-name">{area.label}</span>
        <StatusBadge variant={area.status} />
      </div>
      <div className="area-card-points">
        <span className="area-card-kept">{kept}</span>
        <span className="muted">of {AREA_SHARE} points kept</span>
        {area.deduction > 0 && <span className="area-card-lost">−{area.deduction}</span>}
      </div>
      <div className="area-card-bar" role="meter" aria-valuemin={0} aria-valuemax={AREA_SHARE} aria-valuenow={kept} aria-label={`${area.label}: ${kept} of ${AREA_SHARE} points kept`}>
        <span style={{ width: `${(kept / AREA_SHARE) * 100}%` }} />
      </div>
      {open.length ? (
        <ul className="area-card-findings">
          {open.slice(0, 3).map((f) => (
            <li key={`${f.rule}|${f.subject ?? ""}`}>
              <span className={`sev-dot sev-dot-${f.severity}`} aria-hidden="true" />
              {f.title}
            </li>
          ))}
          {open.length > 3 && <li className="muted">+{open.length - 3} more</li>}
        </ul>
      ) : (
        <p className="area-card-none muted">No open problems</p>
      )}
    </a>
  );
}

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
  const areas = areaScores(report);

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
      <Panel title="Score breakdown" subtitle="Areas DeeCure checks today" info={`Each area can take at most ${AREA_SHARE} points off: a critical problem takes 15, a warning 5, and info nothing. Select an area to open it.`} span={12}>
        {areas.length ? (
          <>
            <ScoreSum areas={areas} score={score} />
            <div className="area-grid">
              {areas.map((a) => (
                <AreaCard key={a.id} area={a} />
              ))}
            </div>
          </>
        ) : (
          <div className="empty muted">No areas are checked yet.</div>
        )}
      </Panel>
      <WeeklySummaries />
    </>
  );
}
