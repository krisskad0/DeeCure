// SPDX-License-Identifier: Apache-2.0
// Logs › Journal, Errors, Crashes, Kernel, Boots (docs/03-ui-design/05-pages/03-logs.md), with real data.

import { type BootRecord, type Crash, type ErrorsInfo, type KernelLine, type LogEntry, getInventory, getMetrics } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { BarChart } from "../../components/charts/BarChart";
import { Heatmap } from "../../components/charts/Heatmap";
import { duration, heatRows, unescapeUnit, when } from "../../metrics";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadJournal = () => getInventory("journal");
const loadErrors = () => getInventory("errors");
const loadKernel = () => getInventory("kernel_log");
const loadCrashes = () => getInventory("crashes");
const loadBoots = () => getInventory("boots");
const loadBootTimes = () => {
  const until = Date.now();
  return getMetrics("boot.time", until - 365 * 86_400_000, until, { maxPoints: 2000 });
};

/** Journal priorities in words, with a badge color. */
export function level(priority: number): [StatusVariant, string] {
  if (priority <= 2) return ["crit", "critical"];
  if (priority === 3) return ["crit", "error"];
  if (priority === 4) return ["warn", "warning"];
  if (priority === 5) return ["off", "notice"];
  return ["off", priority === 7 ? "debug" : "info"];
}

const time = (at: number) => {
  const d = new Date(at);
  return new Date().toDateString() === d.toDateString() ? d.toLocaleTimeString(undefined, { hour12: false }) : when(at);
};

const ENTRIES: Column<LogEntry>[] = [
  { key: "time", label: "Time", text: (e) => time(e.at), render: (e) => <code>{time(e.at)}</code> },
  { key: "priority", label: "Priority", text: (e) => level(e.priority)[1], render: (e) => <StatusBadge variant={level(e.priority)[0]} label={level(e.priority)[1]} /> },
  { key: "unit", label: "Unit", text: (e) => unescapeUnit(e.unit), render: (e) => <code>{unescapeUnit(e.unit)}</code> },
  { key: "message", label: "Message", text: (e) => e.message, render: (e) => <code className="wrap-anywhere">{e.message}</code>, wrap: true },
];

function Waiting({ title, error }: { title: string; error: Parameters<typeof NoData>[0]["error"] }) {
  return (
    <Panel title={title} span={12}>
      <NoData error={error} waiting="Reading the journal…" />
    </Panel>
  );
}

export function LogsJournal() {
  const { data: j, error } = useLive(loadJournal, 30_000);
  if (!j) return <Waiting title="Journal entries" error={error} />;
  const now = new Date();
  const max = Math.max(...j.volume);
  const peak = j.volume.indexOf(max);
  const average = j.volume.reduce((a, b) => a + b, 0) / 24;
  return (
    <>
      <Panel title="Log volume" subtitle="events per hour, last 24 h" source="journalctl --since -24h" span={12}>
        <BarChart
          bars={j.volume.map((v, i) => {
            const h = (now.getHours() - 23 + i + 24) % 24;
            return { label: String(h).padStart(2, "0"), long: `${String(h).padStart(2, "0")}:00`, value: v };
          })}
          height={150}
          highlight={max > average * 3 ? [peak] : []}
        />
      </Panel>
      <Panel title="Journal entries" subtitle={`newest ${j.entries.length} of ${j.total.toLocaleString()}`} span={12} flush>
        <DataTable rows={j.entries} columns={ENTRIES} rowKey={(e) => `${e.at}|${e.unit}|${e.message.slice(0, 40)}`} search={{ placeholder: "Filter messages" }} filter={{ column: "priority", options: ["error", "warning", "info"] }} pageSize={12} />
      </Panel>
    </>
  );
}

type Group = ErrorsInfo["groups"][number];
const CLASS: Record<Group["class"], [StatusVariant, string]> = { attention: ["crit", "Needs attention"], review: ["warn", "Worth a look"], harmless: ["off", "Harmless"] };

const GROUPS: Column<Group>[] = [
  { key: "message", label: "Message", text: (g) => g.message, render: (g) => <span className="cell-two"><b className="wrap-anywhere">{unescapeUnit(g.message)}</b></span>, wrap: true },
  { key: "unit", label: "Unit", text: (g) => g.unit, render: (g) => <code>{unescapeUnit(g.unit)}</code> },
  { key: "count", label: "Count", text: (g) => String(g.count), numeric: true },
  { key: "last", label: "Last seen", text: (g) => String(g.last_seen), render: (g) => when(g.last_seen) },
  { key: "class", label: "Classification", text: (g) => CLASS[g.class][1], render: (g) => <StatusBadge variant={CLASS[g.class][0]} label={CLASS[g.class][1]} /> },
];

export function LogsErrors() {
  const { data: e, error } = useLive(loadErrors, 60_000);
  if (!e) return <Waiting title="Grouped errors" error={error} />;
  const attention = e.groups.filter((g) => g.class === "attention").length;
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Errors", value: e.total.toLocaleString(), status: e.total ? "warn" : "ok", subtext: "last 7 days" },
          { label: "Unique messages", value: String(e.groups.length) },
          { label: "Needs attention", value: String(attention), status: attention ? "crit" : "ok" },
        ]}
      />
      <Panel title="Grouped errors" subtitle="same message, numbers ignored" source="journalctl -p err --since -7d" span={12} flush>
        <DataTable rows={e.groups} columns={GROUPS} rowKey={(g) => `${g.unit}|${g.message}`} filter={{ column: "class", options: ["Needs attention", "Worth a look", "Harmless"] }} empty="No errors in the last 7 days" />
      </Panel>
      <Panel title="Errors by day and hour" source="journalctl -p err" span={12}>
        <Heatmap rows={heatRows(e.times)} unit="errors" />
      </Panel>
    </>
  );
}

const SOURCE: Record<Crash["source"], string> = { kernel: "kernel", systemd: "systemd", apport: "apport (crash report)" };

const CRASHES: Column<Crash>[] = [
  { key: "time", label: "Time", text: (c) => String(c.at), render: (c) => when(c.at) },
  { key: "app", label: "Application", text: (c) => c.app, render: (c) => <code>{c.app}</code> },
  { key: "signal", label: "Signal", text: (c) => c.signal },
  { key: "source", label: "Source", text: (c) => SOURCE[c.source] },
  { key: "count", label: "Count", text: (c) => String(c.count), numeric: true },
  { key: "status", label: "Status", text: (c) => (c.count > 2 ? "Recurring" : "Once"), render: (c) => (c.count > 2 ? <StatusBadge variant="crit" label="Recurring" /> : <StatusBadge variant="off" label={c.count === 1 ? "Once" : "Twice"} />) },
];

export function LogsCrashes() {
  const { data: crashes, error } = useLive(loadCrashes, 120_000);
  if (!crashes) return <Waiting title="Crash reports" error={error} />;
  const total = crashes.reduce((s, c) => s + c.count, 0);
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Crashes", value: String(total), status: total ? "warn" : "ok", subtext: "last 7 days" },
          { label: "Applications", value: String(new Set(crashes.map((c) => c.app)).size) },
          { label: "Recurring", value: String(crashes.filter((c) => c.count > 2).length), status: crashes.some((c) => c.count > 2) ? "crit" : "ok" },
        ]}
      />
      <Panel title="Crash reports" source="journal · /var/crash" span={12} flush>
        <DataTable rows={crashes} columns={CRASHES} rowKey={(c) => `${c.app}|${c.signal}`} empty="No crashes in the last 7 days" />
      </Panel>
    </>
  );
}

const KERNEL: Column<KernelLine>[] = [
  { key: "time", label: "Time", text: (k) => time(k.at), render: (k) => <code>{time(k.at)}</code> },
  { key: "level", label: "Level", text: (k) => level(k.priority)[1], render: (k) => <StatusBadge variant={level(k.priority)[0]} label={level(k.priority)[1]} /> },
  { key: "subsystem", label: "Subsystem", text: (k) => k.subsystem },
  { key: "message", label: "Message", text: (k) => k.message, render: (k) => <code className="wrap-anywhere">{k.message}</code>, wrap: true },
];

export function LogsKernel() {
  const { data: lines, error } = useLive(loadKernel, 60_000);
  if (!lines) return <Waiting title="Kernel messages" error={error} />;
  return (
    <Panel title="Kernel messages" subtitle="last 7 days, newest first" source="journalctl -k" span={12} flush>
      <DataTable rows={lines} columns={KERNEL} rowKey={(k) => `${k.at}|${k.message.slice(0, 60)}`} search={{ placeholder: "Filter kernel log" }} filter={{ column: "level", options: ["error", "warning", "info"] }} pageSize={14} />
    </Panel>
  );
}

const SHUTDOWN: Record<BootRecord["shutdown"], [StatusVariant, string]> = { running: ["info", "Running"], clean: ["ok", "Clean"], unclean: ["crit", "Unclean"] };

const BOOTS: Column<BootRecord>[] = [
  { key: "started", label: "Started", text: (b) => String(b.started), render: (b) => when(b.started) },
  { key: "ended", label: "Ended", text: (b) => (b.shutdown === "running" ? "—" : when(b.ended)) },
  { key: "duration", label: "Duration", text: (b) => duration((b.ended - b.started) / 1000), numeric: true },
  { key: "shutdown", label: "Shutdown", text: (b) => SHUTDOWN[b.shutdown][1], render: (b) => <StatusBadge variant={SHUTDOWN[b.shutdown][0]} label={SHUTDOWN[b.shutdown][1]} /> },
  { key: "kernel", label: "Kernel", text: (b) => b.kernel ?? "—", render: (b) => (b.kernel ? <code>{b.kernel.replace(/-generic$/, "")}</code> : "—") },
];

export function LogsBoots() {
  const { data: boots, error } = useLive(loadBoots, 300_000);
  const times = useLive(loadBootTimes, 300_000).data ?? [];
  if (!boots) return <Waiting title="Boot history" error={error} />;
  const recent = times.slice(-10);
  const today = new Date().toDateString();
  return (
    <>
      <Panel title="Boot duration" subtitle={`last ${recent.length} boots DeeCure measured`} source="systemd (like systemd-analyze)" span={6}>
        {recent.length ? (
          <BarChart bars={recent.map((p) => ({ label: new Date(p.at).toDateString() === today ? "Today" : new Date(p.at).toLocaleDateString(undefined, { day: "numeric", month: "short" }), value: p.value }))} unit="s" />
        ) : (
          <div className="chart-empty muted">Boot times appear after the next boot.</div>
        )}
      </Panel>
      <Panel title="Boot history" source="journalctl --list-boots" span={6} flush>
        <DataTable rows={boots} columns={BOOTS} rowKey={(b) => String(b.started)} empty="The journal keeps no earlier boots" />
      </Panel>
    </>
  );
}
