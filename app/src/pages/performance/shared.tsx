// SPDX-License-Identifier: Apache-2.0
// Data hooks and a time chart the Performance pages share.

import { type Metric, getLatest, getMetrics } from "../../api";
import { LineChart, type LineSeries } from "../../components/charts/LineChart";
import { RANGE_MS, align } from "../../metrics";
import { type Range, useUi } from "../../ui";
import { useLive } from "../../useLive";

const loadLatest = () => getLatest();

/** The newest value of every metric, refreshed every 10 seconds. */
export function useLatest(): Metric[] | undefined {
  return useLive(loadLatest, 10_000).data;
}

export interface History {
  range: Range;
  /** One list of points per name, oldest first. */
  series: Metric[][];
}

/** A metric name, or a name with its own subject (such as one sensor). */
export type SeriesSpec = string | { name: string; subject?: string };

/** Points for some metrics over the chosen time range (or a fixed one), reloaded every minute. */
export function useHistory(specs: SeriesSpec[], subject?: string, fixed?: Range): History | undefined {
  const ui = useUi();
  const range = fixed ?? ui.range;
  const list = specs.map((s) => (typeof s === "string" ? { name: s, subject } : s));
  const key = `${range}|${list.map((s) => `${s.name}@${s.subject ?? ""}`).join(",")}`;
  return useLive(
    async () => {
      const until = Date.now();
      const since = until - RANGE_MS[range];
      const series = await Promise.all(list.map((s) => getMetrics(s.name, since, until, { subject: s.subject })));
      return { range, series };
    },
    60_000,
    key,
  ).data;
}

/** The values of one series, for a sparkline. */
export function spark(history: History | undefined, index: number): number[] | undefined {
  const values = history?.series[index]?.map((p) => p.value);
  return values && values.length > 1 ? values : undefined;
}

/** A line chart over time from a `useHistory` result. `scale` turns stored values into shown ones. */
export function TimeChart(props: {
  history: History | undefined;
  names: string[];
  unit: string;
  scale?: (value: number) => number;
  stacked?: boolean;
  max?: number | "auto";
  height?: number;
  threshold?: { value: number; label: string };
}) {
  const { history, scale = (v) => v } = props;
  if (!history) return <div className="chart-empty muted">Loading…</div>;
  const { times, data } = align(history.series);
  // While a reload is on its way, `names` can be longer than the data; leave those out.
  const series: LineSeries[] = props.names
    .map((name, k) => ({ name, data: (data[k] ?? []).map(scale), slot: (k + 1) as LineSeries["slot"] }))
    .filter((s) => s.data.length > 0);
  return (
    <LineChart
      series={series}
      x={{ kind: "time", times, range: history.range }}
      unit={props.unit}
      stacked={props.stacked}
      max={props.max ?? "auto"}
      height={props.height ?? 220}
      threshold={props.threshold}
    />
  );
}

export const GB = 1024 ** 3;
export const MB = 1024 ** 2;

/** 6200 → "6.2K". */
export function compact(value: number): string {
  if (value >= 1e6) return `${(value / 1e6).toFixed(1)}M`;
  if (value >= 1e3) return `${(value / 1e3).toFixed(1)}K`;
  return String(Math.round(value));
}
