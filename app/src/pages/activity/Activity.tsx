// SPDX-License-Identifier: Apache-2.0
// Activity › Timeline, Package History, Config Changes, Device Events, DeeCure Actions
// (docs/03-ui-design/05-pages/04-activity.md), with real data.

import { type ConfigChange, type DeviceEvent, type PackageChange, getInventory } from "../../api";
import { ActivityTimeline, type TimelineTone } from "../../components/ActivityTimeline";
import { type Column, DataTable } from "../../components/DataTable";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { Donut } from "../../components/charts/Donut";
import { aptTime, when } from "../../metrics";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadPackages = () => getInventory("package_history");
const loadConfig = () => getInventory("config_changes");
const loadDevices = () => getInventory("device_events");
const loadBoots = () => getInventory("boots");
const loadCrashes = () => getInventory("crashes");
const WEEK = 7 * 86_400_000;

export interface Change {
  at: number;
  type: "Packages" | "Configuration" | "Devices" | "System" | "Crashes";
  title: string;
  detail: string;
  tone: TimelineTone;
}

/** All changes of the last 7 days, newest first. Package changes of one apt run become one row. */
export function useChanges(): Change[] | undefined {
  const packages = useLive(loadPackages, 300_000).data;
  const config = useLive(loadConfig, 300_000).data;
  const devices = useLive(loadDevices, 300_000).data;
  const boots = useLive(loadBoots, 300_000).data;
  const crashes = useLive(loadCrashes, 300_000).data;
  if (!packages || !config || !devices || !boots || !crashes) return undefined;
  const since = Date.now() - WEEK;
  const out: Change[] = [];
  const runs = new Map<string, PackageChange[]>();
  for (const p of packages) runs.set(p.start, [...(runs.get(p.start) ?? []), p]);
  for (const [start, list] of runs) {
    const at = aptTime(start);
    if (at < since) continue;
    const names = list.map((p) => p.name);
    const verb = list.every((p) => p.action === "upgrade") ? "Upgraded" : list.every((p) => p.action === "install") ? "Installed" : list.every((p) => p.action === "remove" || p.action === "purge") ? "Removed" : "Changed";
    out.push({ at, type: "Packages", title: `${verb} ${names.slice(0, 3).join(", ")}${names.length > 3 ? ` +${names.length - 3} more` : ""}`, detail: `Packages · ${list[0].source}`, tone: "info" });
  }
  // Config files changed together within a minute count as one change.
  let last: Change | null = null;
  for (const c of config.filter((c) => c.modified >= since)) {
    if (last && last.type === "Configuration" && Math.abs(last.at - c.modified) < 60_000) {
      last.detail = `Configuration · ${Number(last.detail.split(" · ")[1]?.split(" ")[0] ?? 1) + 1} files`;
      continue;
    }
    last = { at: c.modified, type: "Configuration", title: `Changed ${c.path}`, detail: "Configuration · 1 files", tone: "default" };
    out.push(last);
  }
  for (const d of devices.filter((d) => d.at >= since)) out.push({ at: d.at, type: "Devices", title: `${d.device} ${d.event}`, detail: `Devices · USB${d.id ? ` · ${d.id}` : ""}`, tone: d.event === "connected" ? "ok" : "default" });
  for (const b of boots.filter((b) => b.started >= since)) {
    out.push({ at: b.started, type: "System", title: "Computer started", detail: `System · kernel ${b.kernel ?? "unknown"}`, tone: "ok" });
    if (b.shutdown === "unclean") out.push({ at: b.ended, type: "System", title: "Unclean shutdown", detail: "System · power loss, freeze, or forced power-off", tone: "warn" });
  }
  for (const c of crashes) out.push({ at: c.at, type: "Crashes", title: `${c.app} crashed${c.count > 1 ? ` (${c.count} times)` : ""}`, detail: `Crashes · ${c.signal}`, tone: "crit" });
  return out.sort((a, b) => b.at - a.at);
}

export function ActivityTimelinePage() {
  const changes = useChanges();
  if (!changes) {
    return (
      <Panel title="Change timeline" span={12}>
        <NoData error={null} waiting="Collecting changes…" />
      </Panel>
    );
  }
  const by = (t: Change["type"]) => changes.filter((c) => c.type === t).length;
  const types: Change["type"][] = ["Packages", "Configuration", "Devices", "System", "Crashes"];
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Changes", value: String(changes.length), subtext: "last 7 days" },
          { label: "Package changes", value: String(by("Packages")) },
          { label: "Configuration changes", value: String(by("Configuration")) },
          { label: "Crashes", value: String(by("Crashes")), status: by("Crashes") ? "warn" : "ok" },
        ]}
      />
      <Panel title="Change timeline" span={8} flush>
        {changes.length ? <ActivityTimeline rows={changes.slice(0, 40).map((c) => ({ time: when(c.at), title: c.title, detail: c.detail, tone: c.tone }))} /> : <div className="empty muted">No changes in the last 7 days.</div>}
      </Panel>
      <Panel title="Changes by type" subtitle="7 days" span={4}>
        <Donut segments={types.map((t, i) => ({ name: t, value: by(t), slot: ((i % 5) + 1) as 1 | 2 | 3 | 4 | 5 }))} center={{ value: String(changes.length), subtitle: "changes" }} />
      </Panel>
    </>
  );
}

const ACTION: Record<PackageChange["action"], StatusVariant> = { install: "ok", upgrade: "info", remove: "warn", purge: "warn", downgrade: "warn", reinstall: "off" };

const PACKAGES: Column<PackageChange>[] = [
  { key: "date", label: "Date", text: (p) => p.start, render: (p) => when(aptTime(p.start)) },
  { key: "action", label: "Action", text: (p) => p.action[0].toUpperCase() + p.action.slice(1), render: (p) => <StatusBadge variant={ACTION[p.action]} label={p.action[0].toUpperCase() + p.action.slice(1)} /> },
  { key: "name", label: "Package", text: (p) => p.name },
  { key: "from", label: "From", text: (p) => p.from ?? "—", render: (p) => (p.from ? <code>{p.from}</code> : "—"), expert: true },
  { key: "to", label: "To", text: (p) => p.to ?? "—", render: (p) => (p.to ? <code>{p.to}</code> : "—") },
  { key: "source", label: "Source", text: (p) => p.source },
];

export function ActivityPackages() {
  const { data: list, error } = useLive(loadPackages, 300_000);
  return (
    <Panel title="Package history" source="/var/log/apt/history.log" span={12} flush>
      {list ? (
        <DataTable rows={list} columns={PACKAGES} rowKey={(p) => `${p.start}|${p.action}|${p.name}`} search={{ placeholder: "Search packages" }} filter={{ column: "action", options: ["Install", "Upgrade", "Remove"] }} pageSize={14} empty="No package changes in the current apt log" />
      ) : (
        <NoData error={error} waiting="Loading…" />
      )}
    </Panel>
  );
}

const CONFIG: Column<ConfigChange>[] = [
  { key: "time", label: "Time", text: (c) => String(c.modified), render: (c) => when(c.modified) },
  { key: "file", label: "File", text: (c) => c.path, render: (c) => <code className="wrap-anywhere">{c.path}</code>, wrap: true },
  { key: "owner", label: "Owner", text: (c) => c.owner },
];

export function ActivityConfig() {
  const { data: list, error } = useLive(loadConfig, 300_000);
  return (
    <Panel title="Configuration changes" subtitle="files in /etc changed in the last 30 days" source="/etc (file times)" span={12} flush>
      {list ? <DataTable rows={list} columns={CONFIG} rowKey={(c) => c.path} search={{ placeholder: "Search files" }} pageSize={14} empty="No files in /etc changed in the last 30 days" /> : <NoData error={error} waiting="Loading…" />}
    </Panel>
  );
}

const DEVICES: Column<DeviceEvent>[] = [
  { key: "time", label: "Time", text: (d) => String(d.at), render: (d) => when(d.at) },
  { key: "event", label: "Event", text: (d) => d.event, render: (d) => <StatusBadge variant={d.event === "connected" ? "ok" : "off"} label={d.event === "connected" ? "Connected" : "Disconnected"} /> },
  { key: "device", label: "Device", text: (d) => d.device },
  { key: "bus", label: "Bus", text: () => "USB" },
  { key: "id", label: "ID", text: (d) => d.id ?? "—", render: (d) => (d.id ? <code>{d.id}</code> : "—") },
];

export function ActivityDevices() {
  const { data: list, error } = useLive(loadDevices, 120_000);
  return (
    <Panel title="Device events" subtitle="last 30 days" source="kernel log" span={12} flush>
      {list ? <DataTable rows={list} columns={DEVICES} rowKey={(d) => `${d.at}|${d.device}|${d.event}`} empty="No USB devices were plugged in or out" /> : <NoData error={error} waiting="Loading…" />}
    </Panel>
  );
}

export function ActivityDeeCure() {
  return (
    <Panel title="Remediation log" info="Every change DeeCure made, with undo where possible." span={12}>
      <div className="empty muted">DeeCure has not changed anything on this computer. Fixes, and this log of them, come with the fix engine.</div>
    </Panel>
  );
}
