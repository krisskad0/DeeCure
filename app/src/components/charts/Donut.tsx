// SPDX-License-Identifier: Apache-2.0
// 14 Donut chart (docs/03-ui-design/03-components/14-donut-chart.md).

import type { SeriesSlot } from "../Sparkline";
import { formatValue } from "./scale";

export interface DonutSegment {
  name: string;
  value: number;
  slot?: SeriesSlot;
  /** Overrides the colour: warn, crit, or neutral (track grey). */
  tone?: "warn" | "crit" | "neutral";
  /** Shown instead of the value in the legend. */
  display?: string;
}

const TONE = { warn: "var(--warn)", crit: "var(--crit)", neutral: "var(--track)" };
const GAP = 0.02;

function arc(start: number, end: number, outer: number, inner: number): string {
  const p = (r: number, a: number) => `${(75 + r * Math.cos(a)).toFixed(2)} ${(75 + r * Math.sin(a)).toFixed(2)}`;
  const large = end - start > Math.PI ? 1 : 0;
  return `M${p(outer, start)}A${outer} ${outer} 0 ${large} 1 ${p(outer, end)}L${p(inner, end)}A${inner} ${inner} 0 ${large} 0 ${p(inner, start)}Z`;
}

export function Donut(props: { segments: DonutSegment[]; center: { value: string; subtitle?: string }; unit?: string }) {
  const { segments, unit = "" } = props;
  const total = segments.reduce((sum, s) => sum + Math.max(0, s.value), 0) || 1;
  let angle = -Math.PI / 2;
  let slotIndex = 0;
  const parts = segments.map((s) => {
    const sweep = (Math.max(0, s.value) / total) * 2 * Math.PI;
    const start = angle + GAP / 2;
    const end = angle + sweep - GAP / 2;
    angle += sweep;
    const color = s.tone ? TONE[s.tone] : `var(--s${s.slot ?? ((slotIndex++ % 5) + 1)})`;
    return { ...s, color, d: end > start ? arc(start, end, 60, 42) : "" };
  });
  return (
    <div className="donut">
      <svg viewBox="0 0 150 150" width={150} height={150} role="img"
        aria-label={`${props.center.value} ${props.center.subtitle ?? ""}`.trim()}>
        {parts.map((p) => (
          <path key={p.name} d={p.d} fill={p.color}>
            <title>{`${p.name}: ${p.display ?? formatValue(p.value, unit)}`}</title>
          </path>
        ))}
        <text x={75} y={78} textAnchor="middle" className="donut-value">
          {props.center.value}
        </text>
        {props.center.subtitle && (
          <text x={75} y={95} textAnchor="middle" className="axis">
            {props.center.subtitle}
          </text>
        )}
      </svg>
      <table className="donut-legend">
        <tbody>
          {parts.map((p) => (
            <tr key={p.name}>
              <td><i style={{ background: p.color }} /></td>
              <td>{p.name}</td>
              <td className="num">{p.display ?? formatValue(p.value, unit)}</td>
              <td className="num muted">{Math.round((Math.max(0, p.value) / total) * 100)}%</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
