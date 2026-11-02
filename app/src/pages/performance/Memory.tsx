// SPDX-License-Identifier: Apache-2.0
// Performance › Memory (docs/03-ui-design/05-pages/02-performance.md), with real data.
// The Expert view adds the detailed memory map, apps and services, and real memory per process
// (MemoryExpert.tsx), and leaves out the simple process list they replace.

import { Guard } from "../../components/ErrorBoundary";
import { getInventory } from "../../api";
import { type Kpi, KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { BarList } from "../../components/charts/BarList";
import { bytes, pick } from "../../metrics";
import { atLeast, useUi } from "../../ui";
import { useLive } from "../../useLive";
import { MemoryExpertSections } from "./MemoryExpert";
import { MemoryMap } from "./MemoryMap";
import { GB, TimeChart, spark, useHistory, useLatest } from "./shared";

const loadProcesses = () => getInventory("processes");

export function PerformanceMemory() {
  const { mode } = useUi();
  const latest = useLatest();
  const expert = atLeast(mode, "expert");
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
      <Panel title="What fills memory" subtitle={total ? `${bytes(total)} in all` : "now"} info="Striped parts are cache: the kernel gives that memory back at once when an app needs it, so a full-looking memory is normal." span={4}>
        {total === undefined || used === undefined || free === undefined || reclaimable === undefined ? (
          <div className="empty muted">Measuring…</div>
        ) : (
          <MemoryMap
            total={total}
            parts={[
              { name: "Apps and the kernel", bytes: used, slot: 1 },
              { name: "Cache", bytes: reclaimable, slot: 2, freeable: true },
              { name: "Free", bytes: free, slot: "free" },
            ]}
          />
        )}
      </Panel>
      {!expert && (
        <Panel title="Biggest memory users" subtitle="now" info="Resident memory of each process: what it holds in RAM, counting memory it shares with others." source="/proc/[pid]/status" span={12}>
          {processes ? (
            <BarList rows={top.map((p) => ({ id: String(p.pid), label: p.name, hint: `${p.user}${total ? ` · ${((p.memory_bytes / total) * 100).toFixed(1)}% of memory` : ""}`, value: p.memory_bytes, display: bytes(p.memory_bytes) }))} />
          ) : (
            <div className="empty muted">Loading…</div>
          )}
        </Panel>
      )}
      {expert && <Guard><MemoryExpertSections /></Guard>}
    </>
  );
}
