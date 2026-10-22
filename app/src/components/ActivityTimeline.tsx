// SPDX-License-Identifier: Apache-2.0
// 10 Activity timeline (docs/03-ui-design/03-components/10-activity-timeline.md).

export type TimelineTone = "crit" | "warn" | "ok" | "info" | "default";

export interface TimelineRow {
  time: string;
  tone?: TimelineTone;
  title: string;
  detail?: string;
}

export function ActivityTimeline({ rows }: { rows: TimelineRow[] }) {
  return (
    <ol className="timeline">
      {rows.map((row, i) => (
        <li className="tl-row" key={`${row.time}|${row.title}|${i}`}>
          <span className="tl-time">{row.time}</span>
          <span className={`tl-dot tl-${row.tone ?? "default"}`} aria-hidden="true" />
          <span>
            <span className="tl-title">{row.title}</span>
            {row.detail && <span className="tl-detail">{row.detail}</span>}
          </span>
        </li>
      ))}
    </ol>
  );
}
