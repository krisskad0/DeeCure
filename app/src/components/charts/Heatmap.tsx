// SPDX-License-Identifier: Apache-2.0
// 16 Heatmap, day × hour (docs/03-ui-design/03-components/16-heatmap.md).

import { heatLevel } from "./scale";

export function Heatmap({ rows, unit }: { rows: { label: string; values: number[] }[]; unit: string }) {
  const max = Math.max(1, ...rows.flatMap((r) => r.values));
  const hours = Array.from({ length: 24 }, (_, h) => h);
  return (
    <div className="heatmap">
      <div className="heat-grid">
        <span />
        {hours.map((h) => (
          <span key={h} className="heat-hour">
            {h % 3 === 0 ? String(h).padStart(2, "0") : ""}
          </span>
        ))}
        {rows.map((row, r) => [
          // Labels can repeat (two jobs with one name), so the row number keeps keys unique.
          <span key={`${r}-label`} className="heat-label">
            {row.label}
          </span>,
          ...hours.map((h) => {
            const v = row.values[h] ?? 0;
            const tip = `${row.label} ${String(h).padStart(2, "0")}:00 · ${v} ${unit}`;
            return <span key={`${r}-${h}`} className={`heat-cell h${heatLevel(v, max)}`} title={tip} aria-label={tip} />;
          }),
        ])}
      </div>
      <div className="heat-legend">
        Less
        {[1, 2, 3, 4, 5].map((l) => (
          <i key={l} className={`h${l}`} />
        ))}
        More
      </div>
    </div>
  );
}
