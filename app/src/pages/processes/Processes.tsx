// SPDX-License-Identifier: Apache-2.0
// Processes › All Processes and Top Consumers (docs/03-ui-design/05-pages/09-processes.md),
// with real data. Ending a process goes through the fix engine.

import { Guard } from "../../components/ErrorBoundary";
import { atLeast, useUi } from "../../ui";
import { ProcessExpertDetail } from "./ProcessExpert";
import { useState } from "react";
import { type Process, getInventory } from "../../api";
import { LinkButton } from "../../components/Button";
import { LogLine } from "../../components/CodeBlock";
import { type Column, DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { BarList } from "../../components/charts/BarList";
import { bytes, when } from "../../metrics";
import { href } from "../../route";
import { Drawer } from "../../shell/Drawer";
import { Button } from "../../components/Button";
import { useFix } from "../../shell/FixDrawer";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadProcesses = () => getInventory("processes");

/** The kernel's one-letter states, in words. */
export const STATES: Record<string, string> = { R: "Running", S: "Sleeping", D: "Waiting for disk", Z: "Zombie", T: "Stopped", t: "Stopped (traced)", I: "Idle", X: "Dead" };
const stateName = (s: string) => STATES[s] ?? s;
const time = (ms?: number) => (ms ? new Date(ms).toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" }) : "-");

const COLUMNS: Column<Process>[] = [
  { key: "name", label: "Name", text: (p) => p.name },
  { key: "pid", label: "PID", text: (p) => String(p.pid), numeric: true },
  { key: "user", label: "User", text: (p) => p.user },
  { key: "state", label: "State", text: (p) => stateName(p.state), render: (p) => (p.state === "R" ? <StatusBadge variant="ok" label="Running" /> : p.state === "Z" ? <StatusBadge variant="warn" label="Zombie" /> : stateName(p.state)) },
  { key: "cpu", label: "CPU", text: (p) => `${p.cpu_percent.toFixed(1)}%`, numeric: true },
  { key: "memory", label: "Memory", text: (p) => bytes(p.memory_bytes), numeric: true },
  { key: "threads", label: "Threads", text: (p) => String(p.threads), numeric: true, expert: true },
  { key: "started", label: "Started", text: (p) => time(p.started_at), expert: true },
];

function useProcesses() {
  return useLive(loadProcesses, 5_000);
}

export function ProcessesAll() {
  const { mode } = useUi();
  const { data: processes, error } = useProcesses();
  const [inspect, setInspect] = useState<Process | null>(null);
  const openFix = useFix();
  if (!processes) {
    return (
      <Panel title="Processes" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const count = (states: string) => processes.filter((p) => states.includes(p.state)).length;
  // Filter chips: the two users with the most processes.
  const byUser = new Map<string, number>();
  for (const p of processes) byUser.set(p.user, (byUser.get(p.user) ?? 0) + 1);
  const users = [...byUser.entries()].sort((a, b) => b[1] - a[1]).slice(0, 3).map(([u]) => u);
  const live = inspect && (processes.find((p) => p.pid === inspect.pid) ?? null);
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Total", value: processes.length.toLocaleString() },
          { label: "Running", value: String(count("R")) },
          { label: "Sleeping", value: String(count("SI")) },
          { label: "Waiting for disk", value: String(count("D")), status: count("D") > 5 ? "warn" : "ok" },
          { label: "Zombie", value: String(count("Z")), status: count("Z") ? "warn" : "ok" },
          { label: "Threads", value: processes.reduce((s, p) => s + p.threads, 0).toLocaleString() },
        ]}
      />
      <Panel title="Processes" source="/proc/[pid]" span={12} flush>
        <DataTable
          rows={processes}
          columns={COLUMNS}
          rowKey={(p) => String(p.pid)}
          search={{ placeholder: "Filter by name, PID or user" }}
          filter={{ column: "user", options: users }}
          pageSize={14}
          total={processes.length}
          onRowClick={setInspect}
        />
      </Panel>
      {inspect && (
        <Drawer
          title={inspect.name}
          subtitle={`PID ${inspect.pid} · ${inspect.user}`}
          note={live ? "Live: updates every 5 seconds" : "This process has ended"}
          actions={
            <span className="btn-row">
              <LinkButton to={href("performance", "cpu")}>Open Performance</LinkButton>
              {live && (
                <Button
                  small
                  onClick={() => {
                    setInspect(null);
                    openFix("processes.end", `${live.pid}:${live.name}`);
                  }}
                >
                  End process…
                </Button>
              )}
            </span>
          }
          onClose={() => setInspect(null)}
        >
          <ProcessDetails process={live ?? inspect} ended={!live} />
          {live && atLeast(mode, "expert") && <Guard><ProcessExpertDetail pid={live.pid} /></Guard>}
        </Drawer>
      )}
    </>
  );
}

function ProcessDetails({ process: p, ended }: { process: Process; ended: boolean }) {
  return (
    <>
      <KeyValueList
        rows={[
          ["State", ended ? <StatusBadge variant="off" label="Ended" /> : stateName(p.state)],
          ["CPU", `${p.cpu_percent.toFixed(1)}% of one CPU`],
          ["Memory", bytes(p.memory_bytes)],
          ["Swap", p.swap_bytes ? bytes(p.swap_bytes) : "None"],
          ["Threads", String(p.threads)],
          ["Started", p.started_at ? when(p.started_at) : "-"],
        ]}
      />
      {p.command && (
        <div>
          <div className="field-label">Command line</div>
          <LogLine>{p.command}</LogLine>
        </div>
      )}
    </>
  );
}

export function ProcessesTop() {
  const { data: processes, error } = useProcesses();
  if (!processes) {
    return (
      <Panel title="Top consumers" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const byCpu = processes.slice(0, 8);
  const byMemory = [...processes].sort((a, b) => b.memory_bytes - a.memory_bytes).slice(0, 8);
  return (
    <>
      <Panel title="By CPU" subtitle="% of one CPU, now" span={6}>
        <BarList rows={byCpu.map((p) => ({ label: `${p.name} (${p.pid})`, value: p.cpu_percent, display: `${p.cpu_percent.toFixed(1)}%` }))} />
      </Panel>
      <Panel title="By memory" subtitle="resident" span={6}>
        <BarList rows={byMemory.map((p) => ({ label: `${p.name} (${p.pid})`, value: p.memory_bytes, display: bytes(p.memory_bytes) }))} />
      </Panel>
    </>
  );
}
