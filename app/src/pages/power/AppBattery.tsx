// SPDX-License-Identifier: Apache-2.0
// Power › Battery: "Battery use by app", an estimate per local day from the agent (`app_battery`).

import { useState } from "react";
import { type AppBatteryInfo, type AppBatteryUse, getInventory } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { Select } from "../../components/FormControls";
import { Panel } from "../../components/Panel";
import { ShowAll } from "../../components/ShowAll";
import { BarList } from "../../components/charts/BarList";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const load = () => getInventory("app_battery");

type Range = "today" | "yesterday" | "week";
const RANGES = [
  { value: "today", label: "Today" },
  { value: "yesterday", label: "Yesterday" },
  { value: "week", label: "Last 7 days" },
] as const;
const RANGE_TEXT: Record<Range, string> = { today: "today", yesterday: "yesterday", week: "in the last 7 days" };

const KIND: Record<string, string> = {
  app: "App",
  service: "System service",
  "user-service": "Background service",
  other: "Many small ones together",
  rest: "Screen, Wi-Fi, and the computer idle",
};

/** The uses of the chosen days, the same app on several days added up, biggest first. */
export function usesIn(info: AppBatteryInfo, range: Range): AppBatteryUse[] {
  const days = range === "today" ? info.days.slice(0, 1) : range === "yesterday" ? info.days.slice(1, 2) : info.days.slice(0, 7);
  const sum = new Map<string, AppBatteryUse>();
  for (const day of days) {
    for (const u of day.uses) {
      const key = `${u.kind}:${u.name}`;
      const had = sum.get(key);
      sum.set(key, had ? { ...had, mwh: had.mwh + u.mwh } : { ...u });
    }
  }
  return [...sum.values()].sort((a, b) => b.mwh - a.mwh);
}

const label = (u: AppBatteryUse) => (u.kind === "rest" ? "Screen and system at rest" : u.kind === "other" ? "Other apps and services" : u.name);

/** "12%" of a full charge (or "<1%"), and the watt hours. */
function amount(mwh: number, fullWh?: number): string {
  const wh = `${(mwh / 1000).toFixed(mwh < 1000 ? 2 : 1)} Wh`;
  if (!fullWh) return wh;
  const pct = mwh / (fullWh * 10);
  return `${pct < 1 ? "<1" : Math.round(pct)}% · ${wh}`;
}

export function AppBatteryPanel({ fullWh }: { fullWh?: number }) {
  const { data, error } = useLive(load, 60_000);
  const [range, setRange] = useState<Range>("today");
  const uses = data ? usesIn(data, range) : [];
  const total = uses.reduce((s, u) => s + u.mwh, 0);
  const columns: Column<AppBatteryUse>[] = [
    { key: "name", label: "App or part", text: label },
    { key: "kind", label: "Kind", text: (u) => KIND[u.kind] ?? u.kind },
    { key: "used", label: "Used", text: (u) => `${(u.mwh / 1000).toFixed(2)} Wh`, numeric: true },
    { key: "share", label: "Share", text: (u) => `${Math.round((u.mwh / total) * 100)}%`, numeric: true },
  ];
  return (
    <Panel
      title="Battery use by app"
      subtitle="estimate"
      info="Linux has no power meter per app, so this is an estimate. Each minute on battery, DeeCure takes the drain it measures. The usual low drain goes to the screen and the system at rest. The drain above it is split between apps by the CPU and graphics time each one used."
      source="cgroup cpu.stat · DRM fdinfo · /sys/class/power_supply"
      actions={<Select value={range} options={RANGES} onChange={setRange} label="Time" />}
      span={12}
    >
      {!data ? (
        <NoData error={error} waiting="Loading…" />
      ) : uses.length === 0 ? (
        <div className="empty muted">Nothing counted {RANGE_TEXT[range]}. DeeCure counts battery use each minute while the laptop runs on battery, not while it charges.</div>
      ) : (
        <>
          <p className="muted">
            On battery {RANGE_TEXT[range]}: used <b>{amount(total, fullWh)}</b>
            {fullWh ? " of a full charge" : ""}.
          </p>
          <BarList rows={uses.slice(0, 8).map((u) => ({ id: `${u.kind}:${u.name}`, label: label(u), hint: KIND[u.kind] ?? u.kind, value: u.mwh, display: amount(u.mwh, fullWh) }))} />
          {uses.length > 8 && (
            <ShowAll label={`Show all ${uses.length}`}>
              <DataTable rows={uses} columns={columns} rowKey={(u) => `${u.kind}:${u.name}`} pageSize={12} />
            </ShowAll>
          )}
        </>
      )}
    </Panel>
  );
}
