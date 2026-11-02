// SPDX-License-Identifier: Apache-2.0
// Measurements only the admin can take, through the helper (docs/01-product/expert-coverage.md):
// measured chip power, the chip's low-power counters, and wake-ups per process. Each runs only
// when the person asks, with their password.

import type { ReactNode } from "react";
import { AdminRead } from "../../components/AdminRead";
import { DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";

export interface PowerMeter {
  zone: string;
  name: string;
  watts: number;
}
export interface Counter {
  source: string;
  name: string;
  value: number;
}
export interface Wakeups {
  pid: number;
  name: string;
  per_second: number;
  kernel: boolean;
}

const ZONE_NAMES: Record<string, string> = {
  core: "Processor cores",
  uncore: "Built-in graphics",
  dram: "Memory",
  psys: "Whole platform",
};

/** A powercap zone's name in plain words: "package-0" is the whole chip. */
export function zoneName(name: string): string {
  if (name.startsWith("package")) return name === "package-0" ? "Whole chip" : `Whole chip ${name.replace("package-", "")}`;
  return ZONE_NAMES[name] ?? name;
}

/** Microseconds as a short duration, such as "2 h 5 min" or "40 s". */
export function duration(us: number): string {
  const s = Math.round(us / 1e6);
  if (s < 60) return `${s} s`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m} min`;
  return `${Math.floor(m / 60)} h ${m % 60} min`;
}

function parsed<T>(text: string, view: (v: T) => ReactNode): ReactNode {
  try {
    return view(JSON.parse(text) as T);
  } catch {
    return <div className="empty muted">The helper gave no readable answer.</div>;
  }
}

export function PowerMetersView({ data }: { data: PowerMeter[] }) {
  if (data.length === 0) return <div className="empty muted">This processor has no power meters (RAPL) the kernel can read.</div>;
  return <KeyValueList rows={data.map((m): [string, string] => [zoneName(m.name), `${m.watts.toFixed(2)} W`])} />;
}

export function LowPowerView({ data }: { data: Counter[] }) {
  if (data.length === 0) return <div className="empty muted">This chip reports no low-power counters (Intel pmc_core or AMD amd_pmc).</div>;
  const s0ix = data.find((c) => c.name === "slp_s0_residency_usec");
  const rest = data.filter((c) => c !== s0ix);
  return (
    <>
      {s0ix && (
        <KeyValueList
          rows={[
            [
              "Deepest whole-chip sleep (S0ix) since start",
              s0ix.value === 0 ? <StatusBadge key="z" variant="warn" label="Never reached: suspend and idle drain more battery than they need to" /> : duration(s0ix.value),
            ],
          ]}
        />
      )}
      {rest.length > 0 && (
        <DataTable
          rows={rest}
          columns={[
            { key: "name", label: "Counter", text: (c) => c.name },
            { key: "value", label: "Value", text: (c) => c.value.toLocaleString(), numeric: true },
            { key: "source", label: "Source", text: (c) => c.source, expert: true },
          ]}
          rowKey={(c) => `${c.source}-${c.name}`}
        />
      )}
    </>
  );
}

export function WakeupsView({ data }: { data: Wakeups[] }) {
  const total = data.reduce((n, w) => n + w.per_second, 0);
  return (
    <>
      <KeyValueList rows={[["Wake-ups per second, all processes listed", total.toFixed(0)]]} />
      <DataTable
        rows={data}
        columns={[
          { key: "name", label: "Process", text: (w) => w.name, render: (w) => (w.kernel ? <>{w.name} <StatusBadge variant="off" label="kernel" /></> : w.name) },
          { key: "pid", label: "PID", text: (w) => String(w.pid), numeric: true },
          { key: "rate", label: "Per second", text: (w) => w.per_second.toFixed(1), numeric: true },
        ]}
        rowKey={(w) => String(w.pid)}
        pageSize={15}
        empty="Nothing woke up while measuring"
      />
    </>
  );
}

export function PowerMetersPanel() {
  return (
    <Panel title="Measured chip power" info="The chip's own energy counters (RAPL) over one second: measured, not a model. The whole chip includes the cores and the built-in graphics." source="/sys/class/powercap/intel-rapl*/energy_uj" span={6} flush>
      <AdminRead what="power_meters" label="Measure for 1 second (asks for your password)">
        {(t) => parsed<PowerMeter[]>(t, (v) => <PowerMetersView data={v} />)}
      </AdminRead>
    </Panel>
  );
}

export function LowPowerPanel() {
  return (
    <Panel title="Whole-chip low power" info="The chip, not one core, decides the battery drain at rest. These counters show how long the whole chip reached its low-power states since the computer started." source="/sys/kernel/debug/pmc_core · amd_pmc" span={6} flush>
      <AdminRead what="low_power" label="Read the low-power counters (asks for your password)">
        {(t) => parsed<Counter[]>(t, (v) => <LowPowerView data={v} />)}
      </AdminRead>
    </Panel>
  );
}

export function WakeupsPanel() {
  return (
    <Panel title="Wake-ups per process" info="How often each process woke the processor, counted by the kernel for 5 seconds. Fewer wake-ups let the chip sleep longer and save battery." source="tracefs: sched_wakeup" span={6} flush>
      <AdminRead what="wakeups" label="Count for 5 seconds (asks for your password)">
        {(t) => parsed<Wakeups[]>(t, (v) => <WakeupsView data={v} />)}
      </AdminRead>
    </Panel>
  );
}
