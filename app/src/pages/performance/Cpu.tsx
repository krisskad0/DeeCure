// SPDX-License-Identifier: Apache-2.0
// Performance › CPU (docs/03-ui-design/05-pages/02-performance.md), with real data.

import { tempUnit, tempValue } from "../../units";
import { type Process, type SystemInfo, getInventory } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { InlineMeter } from "../../components/InlineMeter";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { BarChart } from "../../components/charts/BarChart";
import { pick, pickAll } from "../../metrics";
import { useLive } from "../../useLive";
import { TimeChart, compact, spark, useHistory, useLatest } from "./shared";

const loadSystem = () => getInventory("system");
const loadProcesses = () => getInventory("processes");

const COLUMNS: Column<Process>[] = [
  { key: "name", label: "Process", text: (p) => p.name },
  { key: "pid", label: "PID", text: (p) => String(p.pid), numeric: true },
  { key: "user", label: "User", text: (p) => p.user },
  { key: "cpu", label: "CPU", text: (p) => `${p.cpu_percent.toFixed(1)}%`, render: (p) => <InlineMeter percent={Math.min(100, p.cpu_percent)} label={`${p.cpu_percent.toFixed(1)}%`} /> },
];

const ghz = (hz: number) => (hz / 1e9).toFixed(1);

export function PerformanceCpu() {
  const latest = useLatest();
  const system = useLive<SystemInfo>(loadSystem, 300_000).data;
  const processes = useLive(loadProcesses, 10_000).data;
  const types = useHistory(["cpu.user", "cpu.system", "cpu.iowait"]);
  const trends = useHistory(["cpu.usage", "temp.cpu"]);

  const usage = pick(latest, "cpu.usage");
  const load = [pick(latest, "cpu.load_1m"), pick(latest, "cpu.load_5m"), pick(latest, "cpu.load_15m")];
  const freq = pick(latest, "cpu.frequency");
  const temp = pick(latest, "temp.cpu");
  const ctxt = pick(latest, "cpu.context_switches");
  const intr = pick(latest, "cpu.interrupts");
  const threads = processes?.reduce((sum, p) => sum + p.threads, 0);
  const cores = pickAll(latest, "cpu.core").sort((a, b) => Number(a.subject) - Number(b.subject));
  const sparks = { usage: spark(trends, 0), temp: spark(trends, 1) };

  return (
    <>
      <KpiRow
        tiles={[
          { label: "Utilization", value: usage === undefined ? "-" : String(Math.round(usage)), unit: usage === undefined ? undefined : "%", visual: sparks.usage && { kind: "spark", data: sparks.usage, slot: 1 } },
          {
            label: "Load average",
            value: load[0] === undefined ? "-" : load[0].toFixed(2),
            subtext: load[1] !== undefined && load[2] !== undefined ? `5 min ${load[1].toFixed(2)} · 15 min ${load[2].toFixed(2)}` : "",
          },
          { label: "Frequency", value: freq === undefined ? "-" : ghz(freq), unit: freq === undefined ? undefined : "GHz", subtext: system?.cpu_max_hz ? `Max ${ghz(system.cpu_max_hz)} GHz` : "" },
          {
            label: "Temperature",
            value: temp === undefined ? "N/A" : String(Math.round(tempValue(temp))),
            unit: temp === undefined ? undefined : tempUnit(),
            subtext: temp === undefined ? "No CPU sensor found" : "",
            visual: sparks.temp && { kind: "spark", data: sparks.temp, slot: 2 },
          },
          { label: "Processes", value: processes ? String(processes.length) : "-", subtext: threads ? `${threads.toLocaleString()} threads` : "" },
          { label: "Context switches", value: ctxt === undefined ? "-" : compact(ctxt), unit: ctxt === undefined ? undefined : "/s", subtext: intr === undefined ? "" : `Interrupts ${compact(intr)}/s` },
        ]}
      />
      <Panel title="CPU utilization by type" info="User = your apps. System = the kernel. I/O wait = waiting for the disk." source="/proc/stat" span={12}>
        <TimeChart history={types} names={["User", "System", "I/O wait"]} unit="%" stacked height={240} />
      </Panel>
      <Panel title="Per-core utilization" subtitle="last minute" source="/proc/stat" span={6}>
        {cores.length ? (
          <BarChart bars={cores.map((c) => ({ label: c.subject ?? "", long: `CPU ${c.subject}`, value: c.value }))} max={100} unit="%" />
        ) : (
          <div className="chart-empty muted">Measuring… (needs two readings, one minute apart)</div>
        )}
      </Panel>
      <Panel title="Top CPU consumers" subtitle="% of one CPU" source="/proc/[pid]/stat" span={6} flush>
        {processes ? <DataTable rows={processes.slice(0, 7)} columns={COLUMNS} rowKey={(p) => String(p.pid)} pageSize={7} /> : <div className="empty muted">Loading…</div>}
      </Panel>
    </>
  );
}
