// SPDX-License-Identifier: Apache-2.0
// Services › System Services (docs/03-ui-design/05-pages/10-services.md), with real data.
// Memory per service is not collected yet, so that column is left out rather than faked.

import { type UnitState, getSnapshot } from "../../api";
import { AlertBanner } from "../../components/AlertBanner";
import { type Column, DataTable } from "../../components/DataTable";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadServices = () => getSnapshot<UnitState[]>("services");

const ACTIVE: Record<string, StatusVariant> = { active: "ok", failed: "crit", inactive: "off", activating: "info", deactivating: "info" };

const COLUMNS: Column<UnitState>[] = [
  {
    key: "unit",
    label: "Unit",
    text: (u) => `${u.name} ${u.description}`,
    render: (u) => (
      <span className="cell-two">
        <b>{u.name}</b>
        <span>{u.description}</span>
      </span>
    ),
  },
  { key: "active", label: "Active", text: (u) => u.active, render: (u) => <StatusBadge variant={ACTIVE[u.active] ?? "off"} label={u.active} /> },
  { key: "sub", label: "Sub", text: (u) => u.sub },
  {
    key: "enabled",
    label: "Enabled",
    text: (u) => u.enabled ?? "—",
    render: (u) => (u.enabled === "enabled" ? <StatusBadge variant="info" label="enabled" /> : (u.enabled ?? "—")),
  },
  { key: "restarts", label: "Restarts", text: (u) => (u.restarts === undefined ? "—" : String(u.restarts)), numeric: true },
];

/** Failed first, then by name. */
function byConcern(a: UnitState, b: UnitState): number {
  const failed = Number(b.active === "failed") - Number(a.active === "failed");
  return failed || a.name.localeCompare(b.name);
}

export function SystemServices() {
  const { data: snapshot, error } = useLive(loadServices);
  if (!snapshot) {
    return (
      <Panel title="Units" span={12} flush>
        <NoData error={error} waiting={snapshot === null ? "DeeCure has not read the services yet. It does so within a few minutes of starting." : "Loading…"} />
      </Panel>
    );
  }
  const units = [...snapshot.data].sort(byConcern);
  const count = (test: (u: UnitState) => boolean) => units.filter(test).length;
  const failed = units.filter((u) => u.active === "failed");
  const restarts = units.reduce((sum, u) => sum + (u.restarts ?? 0), 0);

  return (
    <>
      <KpiRow
        tiles={[
          { label: "Running", value: String(count((u) => u.active === "active" && u.sub === "running")) },
          {
            label: "Failed",
            value: String(failed.length),
            status: failed.length ? "crit" : "ok",
            subtext: failed.map((u) => u.name).join(", "),
          },
          { label: "Inactive", value: String(count((u) => u.active === "inactive")) },
          { label: "Enabled at boot", value: String(count((u) => u.enabled === "enabled")) },
          { label: "Restarts since boot", value: String(restarts), status: restarts ? "warn" : "ok" },
        ]}
      />
      {failed.slice(0, 3).map((u) => (
        <AlertBanner
          key={u.name}
          tone="crit"
          title={`${u.name} has stopped working`}
          text={`${u.description}${u.restarts ? ` · restarted ${u.restarts} times since boot` : ""}`}
        />
      ))}
      <Panel title="Units" subtitle={`${units.length} services`} source="systemctl list-units --type=service --all" span={12} flush>
        <DataTable
          rows={units}
          columns={COLUMNS}
          rowKey={(u) => u.name}
          search={{ placeholder: "Filter units" }}
          filter={{ column: "active", options: ["active", "failed", "inactive"] }}
          pageSize={14}
          empty="No units match"
        />
      </Panel>
    </>
  );
}
