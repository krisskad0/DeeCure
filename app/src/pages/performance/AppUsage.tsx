// SPDX-License-Identifier: Apache-2.0
// Use per app over time (CPU time, memory, traffic): the biggest apps over the last 24 hours,
// 7 days, or 30 days, with the top 5 as lines, from the agent's history (`app_usage`).

import { useState } from "react";
import { type AppUsageEntry, type AppUsageInfo, getInventory, getMetrics } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { Select } from "../../components/FormControls";
import { Panel } from "../../components/Panel";
import { ShowAll } from "../../components/ShowAll";
import type { SeriesSlot } from "../../components/Sparkline";
import { BarList } from "../../components/charts/BarList";
import { LineChart } from "../../components/charts/LineChart";
import { RANGE_MS, bytes, duration } from "../../metrics";
import type { Range } from "../../ui";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadUsage = () => getInventory("app_usage");
/** One answer for every panel on the page. */
export const useAppUsage = () => useLive(loadUsage, 300_000);

type Window = "24h" | "7d" | "30d";
const WINDOWS = [
  { value: "24h", label: "Last 24 hours" },
  { value: "7d", label: "Last 7 days" },
  { value: "30d", label: "Last 30 days" },
] as const;
const WINDOW_TEXT: Record<Window, string> = { "24h": "in the last 24 hours", "7d": "in the last 7 days", "30d": "in the last 30 days" };

export const KIND: Record<string, string> = {
  app: "App",
  service: "System service",
  "user-service": "Background service",
  other: "Many small ones together",
};
export const usageLabel = (e: AppUsageEntry) => (e.kind === "other" ? "Other apps and services" : e.name);

/** The entries of one metric in one window. */
export function entries(info: AppUsageInfo | undefined, window: Window, metric: string): AppUsageEntry[] {
  return info?.windows.find((w) => w.range === window)?.metrics[metric] ?? [];
}

/** Lines that share one time axis: every time any of them has, 0 where one has no point. */
export function union(series: { at: number; value: number }[][]): { times: number[]; data: number[][] } {
  const times = [...new Set(series.flatMap((s) => s.map((p) => p.at)))].sort((a, b) => a - b);
  const maps = series.map((s) => new Map(s.map((p) => [p.at, p.value])));
  return { times, data: maps.map((m) => times.map((t) => m.get(t) ?? 0)) };
}

function TopLines(props: { metric: string; top: AppUsageEntry[]; window: Window; unit: string; scale: (v: number) => number }) {
  const key = `${props.metric}|${props.window}|${props.top.map((e) => e.subject).join(",")}`;
  const { data } = useLive(
    async () => {
      const until = Date.now();
      const since = until - RANGE_MS[props.window];
      return Promise.all(props.top.map((e) => getMetrics(props.metric, since, until, { subject: e.subject })));
    },
    300_000,
    key,
  );
  if (!data) return <div className="chart-empty muted">Loading…</div>;
  const { times, data: values } = union(data);
  return (
    <LineChart
      series={props.top.map((e, k) => ({ name: usageLabel(e), data: (values[k] ?? []).map(props.scale), slot: (k + 1) as SeriesSlot }))}
      x={{ kind: "time", times, range: props.window as Range }}
      unit={props.unit}
      max="auto"
      height={200}
    />
  );
}

export function AppUsagePanel(props: {
  title: string;
  info: string;
  source: string;
  metric: string;
  /** `total` ranks by the sum (CPU time, traffic), `average` by the average (memory). */
  rank: "total" | "average";
  /** The text on each bar, such as "2 h 10 m · 4% on average". */
  display: (e: AppUsageEntry, window: Window) => string;
  /** The chart: its unit and how a stored point becomes a shown value. */
  unit: string;
  scale: (value: number) => number;
  /** Table columns after the name and kind. */
  columns: Column<AppUsageEntry>[];
  usage: ReturnType<typeof useAppUsage>;
  /** A note under the chart, such as "TCP only". */
  note?: string;
  span?: 6 | 12;
}) {
  const [window, setWindow] = useState<Window>("24h");
  const { data, error } = props.usage;
  const list = entries(data ?? undefined, window, props.metric);
  const top = list.filter((e) => e.kind !== "other").slice(0, 5);
  const columns: Column<AppUsageEntry>[] = [{ key: "name", label: "App or service", text: usageLabel }, { key: "kind", label: "Kind", text: (e) => KIND[e.kind] ?? e.kind }, ...props.columns];
  return (
    <Panel title={props.title} info={props.info} source={props.source} actions={<Select value={window} options={WINDOWS} onChange={setWindow} label="Time" />} span={props.span ?? 12}>
      {!data ? (
        <NoData error={error} waiting="Loading…" />
      ) : list.length === 0 ? (
        <div className="empty muted">Nothing recorded {WINDOW_TEXT[window]} yet. DeeCure saves use per app every 5 minutes while it runs.</div>
      ) : (
        <>
          <TopLines metric={props.metric} top={top} window={window} unit={props.unit} scale={props.scale} />
          {props.note && <p className="muted">{props.note}</p>}
          <div className="kn-head svc-head">{props.rank === "total" ? "Most in all" : "Most on average"}</div>
          <BarList rows={list.slice(0, 8).map((e) => ({ id: e.subject, label: usageLabel(e), hint: KIND[e.kind] ?? e.kind, value: props.rank === "total" ? e.total : e.average, display: props.display(e, window) }))} />
          {list.length > 8 && (
            <ShowAll label={`Show all ${list.length}`}>
              <DataTable rows={list} columns={columns} rowKey={(e) => e.subject} pageSize={12} />
            </ShowAll>
          )}
        </>
      )}
    </Panel>
  );
}

const WINDOW_SECONDS: Record<Window, number> = { "24h": 86_400, "7d": 7 * 86_400, "30d": 30 * 86_400 };
/** One stored CPU point is the CPU time of 5 minutes. */
const CPU_STEP_SECONDS = 300;
const cores = () => (typeof navigator !== "undefined" && navigator.hardwareConcurrency) || 1;
/** CPU seconds over `seconds` as a share of the whole CPU: "3%", or "<1%". */
const cpuShare = (cpu: number, seconds: number) => {
  const pct = (cpu / seconds / cores()) * 100;
  return pct < 1 ? "<1%" : `${Math.round(pct)}%`;
};

export function CpuByAppPanel() {
  const usage = useAppUsage();
  return (
    <AppUsagePanel
      title="CPU use by app over time"
      info="CPU time each app and service used, saved every 5 minutes. The lines show the 5 busiest as a share of the whole CPU; the bars add up all their CPU time."
      source="cgroup cpu.stat"
      metric="app.cpu_seconds"
      rank="total"
      display={(e, w) => `${duration(e.total)} · ${cpuShare(e.total, WINDOW_SECONDS[w])} avg`}
      unit="%"
      scale={(v) => (v / CPU_STEP_SECONDS / cores()) * 100}
      columns={[{ key: "time", label: "CPU time", text: (e) => duration(e.total), numeric: true }]}
      usage={usage}
    />
  );
}

export function MemoryByAppPanel() {
  const usage = useAppUsage();
  return (
    <AppUsagePanel
      title="Memory use by app over time"
      info="Each app's own memory (without file cache the kernel can give back at once), saved every 5 minutes while it is open. The average counts only the times it was open."
      source="cgroup memory.stat"
      metric="app.memory"
      rank="average"
      display={(e) => `${bytes(e.average)} avg · ${bytes(e.peak)} peak`}
      unit="GB"
      scale={(v) => v / 1024 ** 3}
      columns={[
        { key: "avg", label: "Average", text: (e) => bytes(e.average), numeric: true },
        { key: "peak", label: "Peak", text: (e) => bytes(e.peak), numeric: true },
      ]}
      usage={usage}
    />
  );
}
