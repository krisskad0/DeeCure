// SPDX-License-Identifier: Apache-2.0
// Services › User Services and Startup Applications (docs/03-ui-design/05-pages/10-services.md),
// with real data. Turning startup apps on and off comes with the fix engine (Phase 2).

import { type StartupApp, type UserService, getInventory, getMachine } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { bytes } from "../../metrics";
import { useLive } from "../../useLive";
import { Button } from "../../components/Button";
import { useFix } from "../../shell/FixDrawer";
import { NoData } from "../NoData";

const loadUser = () => getInventory("user_services");
const loadStartup = () => getInventory("startup");
const loadMachine = () => getMachine();
const ACTIVE: Record<string, StatusVariant> = { active: "ok", failed: "crit", inactive: "off", activating: "info", deactivating: "info" };

const USER: Column<UserService>[] = [
  { key: "unit", label: "Unit", text: (u) => `${u.name} ${u.description}`, render: (u) => (<span className="cell-two"><b>{u.name}</b><span>{u.description}</span></span>) },
  { key: "active", label: "Active", text: (u) => u.active, render: (u) => <StatusBadge variant={ACTIVE[u.active] ?? "off"} label={u.active} /> },
  { key: "sub", label: "Sub", text: (u) => u.sub },
  { key: "memory", label: "Memory", text: (u) => (u.memory === undefined ? "-" : bytes(u.memory)), numeric: true },
  { key: "enabled", label: "Enabled", text: (u) => u.enabled ?? "-", expert: true },
];

export function ServicesUser() {
  const { data: units, error } = useLive(loadUser, 15_000);
  const user = useLive(loadMachine, 3_600_000).data?.user;
  return (
    <Panel title="User units" subtitle={user} source="systemctl --user" span={12} flush>
      {units ? (
        <DataTable rows={units} columns={USER} rowKey={(u) => u.name} search={{ placeholder: "Filter units" }} filter={{ column: "active", options: ["active", "failed", "inactive"] }} pageSize={14} />
      ) : (
        <NoData error={error} waiting="Loading…" />
      )}
    </Panel>
  );
}

const STARTUP: Column<StartupApp>[] = [
  { key: "name", label: "Name", text: (a) => a.name },
  { key: "scope", label: "Scope", text: (a) => a.scope },
  { key: "command", label: "Command", text: (a) => a.command, render: (a) => (a.command ? <code>{a.command}</code> : "-"), expert: true, wrap: true },
  { key: "enabled", label: "Enabled", text: (a) => (a.enabled ? "On" : "Off"), render: (a) => <StatusBadge variant={a.enabled ? "ok" : "off"} label={a.enabled ? "On" : "Off"} /> },
];

export function ServicesStartup() {
  const { data: apps, error } = useLive(loadStartup, 60_000);
  const openFix = useFix();
  const columns: Column<StartupApp>[] = [
    ...STARTUP,
    { key: "act", label: "", text: () => "", render: (a) => <Button small onClick={() => openFix(a.enabled ? "services.disable_startup_app" : "services.enable_startup_app", a.id)}>{a.enabled ? "Turn off…" : "Turn on…"}</Button> },
  ];
  return (
    <Panel title="Started at login" source="~/.config/autostart · /etc/xdg/autostart" span={12} flush>
      {apps ? <DataTable rows={apps} columns={columns} rowKey={(a) => a.id} filter={{ column: "scope", options: ["User", "System"] }} empty="No startup applications" /> : <NoData error={error} waiting="Loading…" />}
    </Panel>
  );
}
