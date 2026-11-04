// SPDX-License-Identifier: Apache-2.0
// The "Health score" card on Overview › Summary: the score with its level and how it moved
// this week, problem counts by level, and what takes points off, each opening its area.
// `children` (the Optimize summary in Expert) sits at the bottom of the card.

import type { ReactNode } from "react";
import { type Finding, getMetrics } from "../../api";
import { LinkButton } from "../../components/Button";
import { Icon } from "../../components/Icon";
import { Panel } from "../../components/Panel";
import { Sparkline } from "../../components/Sparkline";
import { Gauge } from "../../components/charts/Gauge";
import { scoreTone } from "../../metrics";
import { href } from "../../route";
import { PAGES } from "../../spec/pages";
import { useLive } from "../../useLive";
import { type PageProps, areaLabel, plural } from "./shared";

const WEEK = 7 * 86_400_000;
const loadWeek = () => getMetrics("health.score", Date.now() - WEEK, Date.now(), { maxPoints: 28 });

/** The level a score is in: 90+ Healthy, 75-89 Fair, below 75 At risk. */
export function scoreLevel(score: number): string {
  return score >= 90 ? "Healthy" : score >= 75 ? "Fair" : "At risk";
}

/** How the score moved since the oldest reading of the week, or nothing without one. */
export function weekChange(points: { value: number }[], score: number): number | undefined {
  if (points.length < 2) return undefined;
  return Math.round(score - points[0].value);
}

const RANK: Record<Finding["severity"], number> = { critical: 0, warning: 1, info: 2 };

export function ScoreCard({ report, children }: { report: PageProps["report"]; children?: ReactNode }) {
  const week = useLive(loadWeek, 300_000).data ?? [];
  const score = report?.score;
  const action = <LinkButton to={href("overview", "score")}>Details</LinkButton>;
  if (score === undefined || !report) {
    return (
      <Panel title="Health score" actions={action} span={4}>
        <div className="sc">
          <div className="empty muted">The score appears after DeeCure's first check.</div>
          {children}
        </div>
      </Panel>
    );
  }
  const tone = scoreTone(score);
  const change = weekChange(week, score);
  const count = (s: Finding["severity"]) => report.findings.filter((f) => f.severity === s).length;
  const deductions = [...(report.deductions ?? [])].sort((a, b) => b.points - a.points);
  const lost = deductions.reduce((n, d) => n + d.points, 0);
  return (
    <Panel title="Health score" info="Starts at 100. Each open problem takes points off its area: critical 15, warning 5, info 0, at most 15 per area." actions={action} span={4}>
      <div className="sc">
        <Gauge value={score} subtitle="out of 100" status={tone} />
        <div className="sc-level">
          <span className={`sc-word sc-${tone}`}>{scoreLevel(score)}</span>
          {change !== undefined && (
            <span className={`sc-change ${change > 0 ? "sc-up" : change < 0 ? "sc-down" : ""}`} title="Change since the oldest reading of the last 7 days">
              {change > 0 ? `▲ ${change}` : change < 0 ? `▼ ${-change}` : "No change"} since last week
            </span>
          )}
        </div>
        {week.length >= 2 && (
          <div className="sc-trend" aria-hidden="true">
            <Sparkline data={week.map((p) => p.value)} slot={1} height={26} domain={[70, 100]} />
          </div>
        )}
        <div className="sc-counts">
          <span className={`sc-count ${count("critical") ? "sc-count-crit" : ""}`}>
            <b>{count("critical")}</b> critical
          </span>
          <span className={`sc-count ${count("warning") ? "sc-count-warn" : ""}`}>
            <b>{count("warning")}</b> {count("warning") === 1 ? "warning" : "warnings"}
          </span>
          <span className="sc-count">
            <b>{count("info")}</b> info
          </span>
        </div>
        {deductions.length ? (
          <>
            <div className="sc-head">What takes points off</div>
            <ul className="sc-list">
              {deductions.map((d) => {
                const page = PAGES.find((p) => p.id === d.category);
                const top = report.findings.filter((f) => f.rule.split(".")[0] === d.category).sort((a, b) => RANK[a.severity] - RANK[b.severity])[0];
                return (
                  <li key={d.category}>
                    <a href={href(d.category, page?.tabs[0]?.id ?? "")}>
                      {page && <Icon name={page.icon} size={14} />}
                      <span className="sc-area">
                        <span>{areaLabel(d.category)}</span>
                        {top && <span className="sc-why muted">{top.title}</span>}
                      </span>
                      <span className={`sc-points sc-points-${top?.severity === "critical" ? "crit" : "warn"}`}>−{d.points}</span>
                    </a>
                  </li>
                );
              })}
            </ul>
            <p className="sc-hint muted">Fix {deductions.length === 1 ? "this" : "these"} to get back {plural(lost, "point")}.</p>
          </>
        ) : (
          <p className="sc-hint muted">Nothing takes points off. DeeCure keeps checking every minute.</p>
        )}
        {children}
      </div>
    </Panel>
  );
}
