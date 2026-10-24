// SPDX-License-Identifier: Apache-2.0
// Performance › Memory (docs/03-ui-design/05-pages/02-performance.md), with real data.
// Not shown yet: OOM kills, which come from the journal with the Logs page.

import { Guard } from "../../components/ErrorBoundary";
import { type Process, getInventory } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { InlineMeter } from "../../components/InlineMeter";
import { type Kpi, KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { Donut } from "../../components/charts/Donut";
import { bytes, pick } from "../../metrics";
import { atLeast, useUi } from "../../ui";
import { useLive } from "../../useLive";
import { MemoryExpertSections } from "./MemoryExpert";
import { GB, TimeChart, spark, useHistory, useLatest } from "./shared";

const loadProcesses = () => getInventory("processes");

function columns(total: number | undefined): Column<Process>[] {
  return [
    { key: "name", label: "Process", text: (p) => p.name },
    { key: "pid", label: "PID", text: (p) => String(p.pid), numeric: true },
    { key: "user", label: "User", text: (p) => p.user },
    { key: "rss", label: "Resident", text: (p) => bytes(p.memory_bytes), numeric: true },
    {
      key: "share",
      label: "Share of total",
      text: (p) => (total ? `${((p.memory_bytes / total) * 100).toFixed(1)}%` : "-"),
      render: (p) => (total ? <InlineMeter percent={(p.memory_bytes / total) * 100 * 4} label={`${((p.memory_bytes / total) * 100).toFixed(1)}%`} tone="ok" /> : "-"),
    },
    { key: "swap", label: "Swap", text: (p) => (p.swap_bytes ? bytes(p.swap_bytes) : "0"), numeric: true, expert: true },
  ];
}

export function PerformanceMemory() {
  const { mode } = useUi();
  const latest = useLatest();
  const processes = useLive(loadProcesses, 10_000).data;
  const history = useHistory(["memory.used", "memory.cache"]);
  const percent = useHistory(["memory.used_percent"]);

  const used = pick(latest, "memory.used");
  const total = pick(latest, "memory.total");
  const free = pick(latest, "memory.free");
  const cache = pick(latest, "memory.cache");
  const buffers = pick(latest, "memory.buffers");
  const swapUsed = pick(latest, "swap.used");
  const swapTotal = pick(latest, "swap.total");
  const pressure = pick(latest, "pressure.memory");
  const top = [...(processes ?? [])].sort((a, b) => b.memory_bytes - a.memory_bytes).slice(0, 8);
  const sparkUsed = spark(percent, 0);

  const split = (value: number | undefined) => (value === undefined ? { value: "-" } : { value: bytes(value).split(" ")[0], unit: bytes(value).split(" ")[1] });
  const tiles: Kpi[] = [
    { label: "Used", ...split(used), subtext: used !== undefined && total ? `${Math.round((used / total) * 100)}% of ${bytes(total)}` : "", visual: sparkUsed && { kind: "spark", data: sparkUsed, slot: 2 } },
    { label: "Available", ...split(used !== undefined && total !== undefined ? total - used : undefined) },
    { label: "Cache and buffers", ...split(cache !== undefined ? cache + (buffers ?? 0) : undefined) },
    swapTotal
      ? { label: "Swap", ...split(swapUsed ?? 0), subtext: `of ${bytes(swapTotal)}`, visual: { kind: "bar", percent: ((swapUsed ?? 0) / swapTotal) * 100 } }
      : { label: "Swap", value: "None", subtext: "This computer has no swap" },
    { label: "Pressure", value: pressure === undefined ? "N/A" : pressure.toFixed(1), unit: pressure === undefined ? undefined : "%", subtext: pressure === undefined ? "Not reported by this kernel" : "some avg10" },
  ];

  // Used + reclaimable cache + free adds up to the total exactly.
  const reclaimable = used !== undefined && total !== undefined && free !== undefined ? Math.max(0, total - used - free) : undefined;
  return (
    <>
      <KpiRow tiles={tiles} />
      <Panel title="Memory usage" source="/proc/meminfo" span={8}>
        <TimeChart
          history={history}
          names={["Used", "Cache"]}
          unit="GB"
          scale={(v) => v / GB}
          stacked
          height={240}
          max={total ? Math.ceil(total / GB) : "auto"}
          threshold={total ? { value: total / GB, label: `Total ${bytes(total)}` } : undefined}
        />
      </Panel>
      <Panel title="Composition" subtitle="now" span={4}>
        {total === undefined || used === undefined || free === undefined || reclaimable === undefined ? (
          <div className="empty muted">Measuring…</div>
        ) : (
          <Donut
            segments={[
              { name: "Used", value: used, slot: 1, display: bytes(used) },
              { name: "Cache (can be freed)", value: reclaimable, slot: 2, display: bytes(reclaimable) },
              { name: "Free", value: free, tone: "neutral", display: bytes(free) },
            ]}
            center={{ value: (total / GB).toFixed(1), subtitle: "GB total" }}
          />
        )}
      </Panel>
      <Panel title="Top memory consumers" source="/proc/[pid]/status" span={12} flush>
        {processes ? <DataTable rows={top} columns={columns(total)} rowKey={(p) => String(p.pid)} pageSize={8} /> : <div className="empty muted">Loading…</div>}
      </Panel>
      {atLeast(mode, "expert") && <Guard><MemoryExpertSections /></Guard>}
    </>
  );
}
