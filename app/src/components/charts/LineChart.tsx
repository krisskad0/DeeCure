// SPDX-License-Identifier: Apache-2.0
// 12 Line / area chart (docs/03-ui-design/03-components/12-line-area-chart.md).

import { useState } from "react";
import type { Range } from "../../ui";
import type { SeriesSlot } from "../Sparkline";
import { formatValue, labelArrayIndices, niceCeil, timeLabel, timeLabelIndices } from "./scale";
import { useWidth } from "./useWidth";

export interface LineSeries {
  name: string;
  data: number[];
  slot?: SeriesSlot;
}

export type XAxis = { kind: "time"; times: number[]; range: Range } | { kind: "labels"; labels: string[] };

export function LineChart(props: {
  series: LineSeries[];
  x: XAxis;
  height?: number;
  min?: number;
  /** A fixed maximum, or "auto" for the nice ceiling of the highest value × 1.15. */
  max?: number | "auto";
  unit?: string;
  stacked?: boolean;
  threshold?: { value: number; label: string };
  /** Extends the line `ahead` points to `value`, dashed, with a label. */
  forecast?: { ahead: number; value: number; label: string };
}) {
  const [wrap, width] = useWidth<HTMLDivElement>();
  const [hover, setHover] = useState<number | null>(null);
  const { series, x, height = 200, min = 0, unit = "", stacked = false, threshold, forecast } = props;
  const n = Math.max(0, ...series.map((s) => s.data.length));
  if (n < 2) return <div className="chart-empty muted">Not enough data yet</div>;

  const left = 46;
  const right = forecast ? 74 : 12;
  const top = 8;
  const bottom = 22;
  const plotW = Math.max(10, width - left - right);
  const plotH = height - top - bottom;
  const domain = n - 1 + (forecast?.ahead ?? 0);

  // Stacked series add up; each area fills down to the series below it.
  const tops: number[][] = [];
  series.forEach((s, k) => tops.push(s.data.map((v, i) => v + (stacked && k > 0 ? tops[k - 1][i] : 0))));
  // A missing point (NaN, such as before a reading existed) leaves a gap in its line.
  const highest = Math.max(...tops.flat().filter(Number.isFinite), forecast?.value ?? -Infinity, threshold?.value ?? -Infinity);
  const max = typeof props.max === "number" ? props.max : niceCeil(highest * 1.15);

  const xOf = (i: number) => left + (i / domain) * plotW;
  const yOf = (v: number) => top + plotH - ((Math.min(max, Math.max(min, v)) - min) / (max - min || 1)) * plotH;
  const path = (values: number[]) => values.map((v, i) => `${i && Number.isFinite(values[i - 1]) ? "L" : "M"}${xOf(i).toFixed(1)} ${yOf(v).toFixed(1)}`).filter((_, i) => Number.isFinite(values[i])).join("");
  /** The filled area under a line, one piece per run of points without a gap. */
  const area = (values: number[]) => {
    const runs: number[][] = [];
    values.forEach((v, i) => {
      if (!Number.isFinite(v)) return;
      if (i > 0 && Number.isFinite(values[i - 1])) runs[runs.length - 1].push(i);
      else runs.push([i]);
    });
    return runs.map((r) => `${r.map((i, j) => `${j ? "L" : "M"}${xOf(i).toFixed(1)} ${yOf(values[i]).toFixed(1)}`).join("")}L${xOf(r[r.length - 1])} ${yOf(min)}L${xOf(r[0])} ${yOf(min)}Z`).join("");
  };

  const grid = [0, 1, 2, 3, 4].map((k) => min + ((max - min) * k) / 4);
  const labelIdx = x.kind === "time" ? timeLabelIndices(n) : labelArrayIndices(n);
  const xLabel = (i: number) => (x.kind === "labels" ? x.labels[i] : i === n - 1 ? "Now" : timeLabel(x.times[i], x.range, false));
  const anchor = (px: number) => (px < left + 24 ? "start" : px > left + plotW - 24 ? "end" : "middle");
  const color = (s: LineSeries, k: number) => `var(--s${s.slot ?? k + 1})`;

  const onMove = (event: React.PointerEvent<SVGSVGElement>) => {
    const box = event.currentTarget.getBoundingClientRect();
    const i = Math.round(((event.clientX - box.left - left) / plotW) * domain);
    setHover(Math.max(0, Math.min(n - 1, i)));
  };

  const last = n - 1;
  return (
    <div className="chart" ref={wrap}>
      {series.length > 1 && (
        <div className="legend">
          {series.map((s, k) => (
            <span key={s.name}>
              <i style={{ background: color(s, k) }} />
              {s.name}
            </span>
          ))}
        </div>
      )}
      <svg width={width} height={height} onPointerMove={onMove} onPointerLeave={() => setHover(null)} role="img"
        aria-label={series.filter((s) => Number.isFinite(s.data[s.data.length - 1])).map((s) => `${s.name}: ${formatValue(s.data[s.data.length - 1], unit)}`).join(", ")}>
        {grid.map((v, k) => (
          <g key={k}>
            <line x1={left} x2={left + plotW} y1={yOf(v)} y2={yOf(v)} stroke={k === 0 ? "var(--axis)" : "var(--grid)"} />
            <text className="axis" x={left - 7} y={yOf(v) + 4} textAnchor="end">
              {formatValue(v, unit)}
            </text>
          </g>
        ))}
        {labelIdx.map((i) => (
          <text key={i} className="axis" x={xOf(i)} y={height - 6} textAnchor={anchor(xOf(i))}>
            {xLabel(i)}
          </text>
        ))}
        {/* Drawn last series first, so the first series sits on top. */}
        {[...series.keys()].reverse().map((k) => {
          const below = stacked && k > 0 ? tops[k - 1] : null;
          const fill = below
            ? `${path(tops[k])}${below.map((_, i) => `L${xOf(last - i).toFixed(1)} ${yOf(below[last - i]).toFixed(1)}`).join("")}Z`
            : area(tops[k]);
          return (
            <g key={series[k].name}>
              <path d={fill} fill={color(series[k], k)} fillOpacity={stacked ? 0.2 : 0.1} />
              <path d={path(tops[k])} fill="none" stroke={color(series[k], k)} strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" />
            </g>
          );
        })}
        {threshold && (
          <g>
            <line x1={left} x2={left + plotW} y1={yOf(threshold.value)} y2={yOf(threshold.value)} stroke="var(--muted)" strokeDasharray="4 4" />
            <text className="axis" x={left + plotW} y={yOf(threshold.value) - 5} textAnchor="end">
              {threshold.label}
            </text>
          </g>
        )}
        {forecast && (
          <g>
            <line x1={xOf(last)} y1={yOf(tops[0][last])} x2={xOf(last + forecast.ahead)} y2={yOf(forecast.value)}
              stroke="var(--crit)" strokeWidth={2} strokeDasharray="5 4" />
            <circle cx={xOf(last + forecast.ahead)} cy={yOf(forecast.value)} r={4.5} fill="var(--crit)" stroke="var(--panel)" strokeWidth={2} />
            <text className="forecast-label" x={xOf(last + forecast.ahead) + 8} y={yOf(forecast.value) + 4}>
              {forecast.label}
            </text>
          </g>
        )}
        {series.map((s, k) =>
          Number.isFinite(tops[k][last]) ? <circle key={s.name} cx={xOf(last)} cy={yOf(tops[k][last])} r={4} fill={color(s, k)} stroke="var(--panel)" strokeWidth={2} /> : null,
        )}
        {hover !== null && (
          <g>
            <line x1={xOf(hover)} x2={xOf(hover)} y1={top} y2={top + plotH} stroke="var(--line-2)" />
            {series.map((s, k) =>
              Number.isFinite(tops[k][hover]) ? <circle key={s.name} cx={xOf(hover)} cy={yOf(tops[k][hover])} r={4} fill="var(--panel)" stroke={color(s, k)} strokeWidth={2} /> : null,
            )}
          </g>
        )}
      </svg>
      {hover !== null && (
        <div
          className="chart-tip"
          style={{
            left: Math.max(70, Math.min(width - 70, xOf(hover))),
            top: yOf(Math.max(min, ...tops.map((t) => t[hover]).filter(Number.isFinite))) + (series.length > 1 ? 26 : 0) - 10,
          }}
        >
          <div className="tip-head">{x.kind === "labels" ? x.labels[hover] : timeLabel(x.times[hover], x.range, true)}</div>
          {series.map((s, k) => (
            <div key={s.name} className="tip-row">
              <i style={{ background: color(s, k) }} />
              {series.length > 1 ? `${s.name}: ` : ""}
              {s.data[hover] === undefined || !Number.isFinite(s.data[hover]) ? "-" : formatValue(s.data[hover], unit)}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
