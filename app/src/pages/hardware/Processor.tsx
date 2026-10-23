// SPDX-License-Identifier: Apache-2.0
// Hardware › Processor (docs/03-ui-design/05-pages/06-hardware.md), with real data.

import { temp as tempText } from "../../units";
import type { HardwareInfo } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { InlineMeter } from "../../components/InlineMeter";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { GaugeGroup } from "../../components/charts/Gauge";
import { bytes, pick } from "../../metrics";
import { useLatest } from "../performance/shared";
import { NoData } from "../NoData";
import { GHZ, useHardware } from "./shared";

interface Row {
  label: string;
  type: string;
  mhz?: number;
  usage?: number;
  temp?: number;
}

const COLUMNS: Column<Row>[] = [
  { key: "core", label: "CPU", text: (r) => r.label },
  { key: "type", label: "Type", text: (r) => r.type },
  { key: "mhz", label: "Frequency", text: (r) => (r.mhz === undefined ? "—" : `${Math.round(r.mhz)} MHz`), numeric: true },
  { key: "usage", label: "Utilization", text: (r) => (r.usage === undefined ? "—" : `${Math.round(r.usage)}%`), render: (r) => (r.usage === undefined ? "—" : <InlineMeter percent={r.usage} />) },
  { key: "temp", label: "Temperature", text: (r) => (r.temp === undefined ? "—" : tempText(r.temp)), numeric: true },
];

/** P0, P1, … for performance threads and E0, E1, … for efficient ones; plain numbers otherwise. */
function rows(p: HardwareInfo["processor"], latest: ReturnType<typeof useLatest>): Row[] {
  const seen = { Performance: 0, Efficient: 0 };
  return p.cpus.map((c) => {
    const kind = c.kind as keyof typeof seen | undefined;
    const label = kind ? `${kind[0]}${seen[kind]++}` : String(c.cpu);
    return {
      label,
      type: kind ? `${kind} · core ${c.core_id}` : `Core ${c.core_id}`,
      mhz: c.mhz,
      usage: pick(latest, "cpu.core", String(c.cpu)),
      temp: pick(latest, "temp", `coretemp/Core ${c.core_id}`),
    };
  });
}

export function HardwareProcessor() {
  const { data: hw, error } = useHardware();
  const latest = useLatest();
  if (!hw) {
    return (
      <Panel title="Processor" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const p = hw.processor;
  const usage = pick(latest, "cpu.usage");
  const temp = pick(latest, "temp.cpu");
  const freq = pick(latest, "cpu.frequency");
  return (
    <>
      <Panel title="Live" span={6}>
        <GaugeGroup
          gauges={[
            { value: usage ?? 0, display: usage === undefined ? "—" : `${Math.round(usage)}%`, subtitle: "utilization", caption: "Load" },
            { value: temp ?? 0, display: temp === undefined ? "N/A" : tempText(temp), subtitle: "package", caption: "Temperature", status: temp === undefined ? undefined : temp >= 90 ? "crit" : temp >= 80 ? "warn" : "ok" },
            { value: freq ?? 0, max: p.max_hz ?? 1, display: freq === undefined ? "—" : GHZ(freq), subtitle: p.max_hz ? `of ${GHZ(p.max_hz)}` : "", caption: "Frequency" },
          ]}
        />
      </Panel>
      <Panel title="Specifications" source="/proc/cpuinfo · /sys/devices/system/cpu" span={6} flush>
        <KeyValueList
          rows={[
            ["Model", p.model ?? "—"],
            ["Cores", p.hybrid ? `${p.cores} (${p.hybrid[0]} performance + ${p.hybrid[1]} efficient)` : String(p.cores)],
            ["Threads", String(p.threads)],
            ["Base / max frequency", p.base_hz || p.max_hz ? `${GHZ(p.base_hz).replace(" GHz", "")} / ${GHZ(p.max_hz)}` : "—"],
            ["L3 cache", p.l3_bytes ? bytes(p.l3_bytes, 0) : "—"],
            ["Microcode", p.microcode ? <code key="m">{p.microcode}</code> : "—"],
            ["Features", p.features.length ? <span key="f" className="tags">{p.features.map((f) => <span key={f} className="tag">{f}</span>)}</span> : "—"],
          ]}
        />
      </Panel>
      <Panel title="Cores" subtitle="utilization over the last minute" source="/sys/devices/system/cpu" span={12} flush>
        <DataTable rows={rows(p, latest)} columns={COLUMNS} rowKey={(r) => r.label} pageSize={16} />
      </Panel>
    </>
  );
}
