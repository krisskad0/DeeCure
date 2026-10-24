// SPDX-License-Identifier: Apache-2.0
// Services › System Services (docs/03-ui-design/05-pages/10-services.md), with real data.
// The unit list comes from the 5-minute snapshot; memory is read live.

import { Guard } from "../../components/ErrorBoundary";
import { ServicesExpertSections } from "./ServicesExpert";
import { atLeast, useUi } from "../../ui";
import { useState } from "react";
import { type UnitState, getInventory, getSnapshot } from "../../api";
import { KeyValueList } from "../../components/KeyValueList";
import { kbService, useKb } from "../../kb";
import { Drawer } from "../../shell/Drawer";
import { Button } from "../../components/Button";
import { useFix } from "../../shell/FixDrawer";
import { bytes } from "../../metrics";
import { AlertBanner } from "../../components/AlertBanner";
import { type Column, DataTable } from "../../components/DataTable";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadServices = () => getSnapshot<UnitState[]>("services");
const loadMemory = () => getInventory("service_memory");

type Row = UnitState & { memory?: number };

const ACTIVE: Record<string, StatusVariant> = { active: "ok", failed: "crit", inactive: "off", activating: "info", deactivating: "info" };

const COLUMNS: Column<Row>[] = [
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
    text: (u) => u.enabled ?? "-",
    render: (u) => (u.enabled === "enabled" ? <StatusBadge variant="info" label="enabled" /> : (u.enabled ?? "-")),
  },
  { key: "memory", label: "Memory", text: (u) => (u.memory === undefined ? "-" : bytes(u.memory)), numeric: true },
  { key: "restarts", label: "Restarts", text: (u) => (u.restarts === undefined ? "-" : String(u.restarts)), numeric: true, expert: true },
];

/** Failed first, then by name. */
function byConcern(a: UnitState, b: UnitState): number {
  const failed = Number(b.active === "failed") - Number(a.active === "failed");
  return failed || a.name.localeCompare(b.name);
}

export function SystemServices() {
  const { mode } = useUi();
  const { data: snapshot, error } = useLive(loadServices);
  const memory = useLive(loadMemory, 30_000).data;
  const kb = useKb();
  const [inspect, setInspect] = useState<Row | null>(null);
  if (!snapshot) {
    return (
      <Panel title="Units" span={12} flush>
        <NoData error={error} waiting={snapshot === null ? "DeeCure has not read the services yet. It does so within a few minutes of starting." : "Loading…"} />
      </Panel>
    );
  }
  const units: Row[] = [...snapshot.data].sort(byConcern).map((u) => ({ ...u, memory: memory?.[u.name] }));
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
          onRowClick={setInspect}
        />
      </Panel>
      {inspect && <UnitDrawer unit={inspect} info={kbService(kb, inspect.name)} onClose={() => setInspect(null)} />}
      {atLeast(mode, "expert") && <Guard><ServicesExpertSections /></Guard>}
    </>
  );
}

/** One service: its state, and what the knowledge base says about it. */
export function UnitDrawer({ unit, info, onClose }: { unit: Row; info?: { plain: string; safe_to_disable?: string; if_failed?: string }; onClose: () => void }) {
  const openFix = useFix();
  const act = (action: string) => {
    onClose();
    openFix(action, unit.name);
  };
  return (
    <Drawer
      title={unit.name}
      subtitle={unit.description}
      onClose={onClose}
      actions={
        <span className="btn-row">
          <Button small onClick={() => act("services.restart")}>Restart…</Button>
          {unit.active === "active" || unit.enabled === "enabled" ? (
            <Button small onClick={() => act("services.disable")}>Turn off…</Button>
          ) : (
            <Button small onClick={() => act("services.enable")}>Turn on…</Button>
          )}
        </span>
      }
    >
      <KeyValueList
        rows={[
          ["State", <StatusBadge key="s" variant={ACTIVE[unit.active] ?? "off"} label={`${unit.active} (${unit.sub})`} />],
          ["Starts at boot", unit.enabled ?? "-"],
          ["Memory", unit.memory === undefined ? "-" : bytes(unit.memory)],
          ["Restarts since boot", unit.restarts === undefined ? "-" : String(unit.restarts)],
        ]}
      />
      {info ? (
        <KeyValueList
          rows={[
            ["What it does", info.plain],
            ["Safe to turn off?", info.safe_to_disable ?? "-"],
            ["If it fails", info.if_failed ?? "-"],
          ]}
        />
      ) : (
        <p className="muted">The knowledge base has no entry for this service yet.</p>
      )}
    </Drawer>
  );
}

