// SPDX-License-Identifier: Apache-2.0
// Performance › Disk I/O (docs/03-ui-design/05-pages/02-performance.md), with real data.
// Queue depth is not collected yet. Per-process disk use needs admin rights (Phase 2).

import type { Metric } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { pick, pickAll, rate } from "../../metrics";
import { MB, TimeChart, spark, useHistory, useLatest } from "./shared";

interface Device {
  name: string;
  read?: number;
  write?: number;
  busy?: number;
  wait?: number;
}

const COLUMNS: Column<Device>[] = [
  { key: "name", label: "Device", text: (d) => d.name, render: (d) => <code>{d.name}</code> },
  { key: "read", label: "Read/s", text: (d) => (d.read === undefined ? "—" : rate(d.read)), numeric: true },
  { key: "write", label: "Write/s", text: (d) => (d.write === undefined ? "—" : rate(d.write)), numeric: true },
  { key: "util", label: "Util", text: (d) => (d.busy === undefined ? "—" : `${Math.round(d.busy)}%`), numeric: true },
  { key: "await", label: "Await", text: (d) => (d.wait === undefined ? "—" : `${(d.wait * 1000).toFixed(1)} ms`), numeric: true },
];

function devices(latest: Metric[] | undefined): Device[] {
  const names = [...new Set(pickAll(latest, "disk.read").map((m) => m.subject ?? ""))].sort();
  return names.map((name) => ({
    name,
    read: pick(latest, "disk.read", name),
    write: pick(latest, "disk.write", name),
    busy: pick(latest, "disk.busy", name),
    wait: pick(latest, "disk.await", name),
  }));
}

const split = (bytesPerSecond: number | undefined) => {
  if (bytesPerSecond === undefined) return { value: "—" };
  const [value, unit] = rate(bytesPerSecond).split(" ");
  return { value, unit };
};

export function PerformanceDisk() {
  const latest = useLatest();
  const history = useHistory(["disk.read", "disk.write"]);
  const read = pick(latest, "disk.read");
  const write = pick(latest, "disk.write");
  const iops = pick(latest, "disk.iops");
  const busy = pick(latest, "disk.busy");
  const wait = pick(latest, "disk.await");
  const r = spark(history, 0);
  const w = spark(history, 1);
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Read", ...split(read), visual: r && { kind: "spark", data: r, slot: 1 } },
          { label: "Write", ...split(write), visual: w && { kind: "spark", data: w, slot: 2 } },
          { label: "IOPS", value: iops === undefined ? "—" : String(Math.round(iops)), subtext: "reads and writes per second" },
          { label: "Utilization", value: busy === undefined ? "—" : String(Math.round(busy)), unit: busy === undefined ? undefined : "%", subtext: "busiest disk" },
          { label: "Avg latency", value: wait === undefined ? "—" : (wait * 1000).toFixed(1), unit: wait === undefined ? undefined : "ms" },
        ]}
      />
      <Panel title="Throughput" source="/proc/diskstats" span={12}>
        <TimeChart history={history} names={["Read", "Write"]} unit="MB/s" scale={(v) => v / MB} />
      </Panel>
      <Panel title="Devices" subtitle="last minute" source="/proc/diskstats" span={6} flush>
        <DataTable rows={devices(latest)} columns={COLUMNS} rowKey={(d) => d.name} empty="Measuring… (needs two readings, one minute apart)" />
      </Panel>
      <Panel title="Top I/O processes" source="/proc/[pid]/io" span={6} flush>
        <div className="empty muted">Requires Full access · Settings › Access</div>
      </Panel>
    </>
  );
}
