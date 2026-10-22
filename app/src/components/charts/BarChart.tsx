// SPDX-License-Identifier: Apache-2.0
// 13 Bar chart (docs/03-ui-design/03-components/13-bar-chart.md).

import { useState } from "react";
import { barLabelEvery, formatValue, niceCeil } from "./scale";
import { useWidth } from "./useWidth";

export interface Bar {
  label: string;
  /** A longer label for the tooltip, such as a full date. */
  long?: string;
  value: number;
  /** `track` for empty or skipped values. */
  color?: "track" | "warn";
}

export function BarChart(props: { bars: Bar[]; height?: number; max?: number; unit?: string; highlight?: number[] }) {
  const [wrap, width] = useWidth<HTMLDivElement>();
  const [hover, setHover] = useState<number | null>(null);
  const { bars, height = 200, unit = "", highlight = [] } = props;
  const n = bars.length;
  if (n === 0) return <div className="chart-empty muted">No data yet</div>;

  const left = 42;
  const right = 8;
  const top = 18;
  const bottom = 22;
  const plotW = Math.max(10, width - left - right);
  const plotH = height - top - bottom;
  const max = props.max ?? niceCeil(Math.max(...bars.map((b) => b.value)) * 1.1);
  const slot = plotW / n;
  const barW = Math.max(1, Math.min(24, slot - 2));
  const yOf = (v: number) => top + plotH - (Math.min(max, Math.max(0, v)) / max) * plotH;
  const every = barLabelEvery(n, width);
  const fill = (bar: Bar, i: number) =>
    bar.color === "track" ? "var(--track)" : bar.color === "warn" || highlight.includes(i) ? "var(--warn)" : "var(--s1)";

  return (
    <div className="chart" ref={wrap}>
      <svg width={width} height={height} role="img" aria-label={bars.map((b) => `${b.label}: ${formatValue(b.value, unit)}`).join(", ")}
        onPointerLeave={() => setHover(null)}>
        {[0, 1, 2, 3, 4].map((k) => {
          const v = (max * k) / 4;
          return (
            <g key={k}>
              <line x1={left} x2={left + plotW} y1={yOf(v)} y2={yOf(v)} stroke={k === 0 ? "var(--axis)" : "var(--grid)"} />
              <text className="axis" x={left - 7} y={yOf(v) + 4} textAnchor="end">
                {formatValue(v, unit)}
              </text>
            </g>
          );
        })}
        {bars.map((bar, i) => {
          const x = left + slot * i + (slot - barW) / 2;
          const y = yOf(bar.value);
          const h = top + plotH - y;
          const r = Math.min(4, barW / 2, h);
          // Rounded top corners, square base.
          const d = `M${x} ${top + plotH}V${y + r}Q${x} ${y} ${x + r} ${y}H${x + barW - r}Q${x + barW} ${y} ${x + barW} ${y + r}V${top + plotH}Z`;
          const showLabel = i % every === 0 || i === n - 1;
          return (
            <g key={`${bar.label}|${i}`} onPointerEnter={() => setHover(i)}>
              <rect x={left + slot * i} y={top} width={slot} height={plotH} fill="transparent" />
              {h > 0 && <path d={d} fill={fill(bar, i)} />}
              {highlight.includes(i) && (
                <text className="bar-value" x={x + barW / 2} y={y - 5} textAnchor="middle">
                  {formatValue(bar.value, unit)}
                </text>
              )}
              {showLabel && (
                <text className="axis" x={x + barW / 2} y={height - 6} textAnchor="middle">
                  {bar.label}
                </text>
              )}
            </g>
          );
        })}
      </svg>
      {hover !== null && (
        <div className="chart-tip" style={{ left: Math.max(60, Math.min(width - 60, left + slot * hover + slot / 2)), top: yOf(bars[hover].value) - 10 }}>
          <div className="tip-head">{bars[hover].long ?? bars[hover].label}</div>
          <div className="tip-row">{formatValue(bars[hover].value, unit)}</div>
        </div>
      )}
    </div>
  );
}
