// SPDX-License-Identifier: Apache-2.0
// Expert sections for services and boot (docs/01-product/expert-coverage.md, section 10): the cost
// of each service, its unit file facts (local changes, who masked it, restarts), how locked down
// it is, what the desktop really waited for at boot, and every unit's start time.

import { type Exposure, type Groups, type UnitFile, getInventory } from "../../api";
import { DataTable } from "../../components/DataTable";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { bytes } from "../../metrics";
import { perSecond, useLive, useLiveWithPrevious } from "../../useLive";
import { NoData } from "../NoData";

const loadGroups = () => getInventory("cgroups");
const loadUnits = () => getInventory("unit_files");
const loadSecurity = () => getInventory("service_security");
const loadBoot = () => getInventory("boot_detail");

const ms = (v?: number) => (v === undefined ? "-" : v >= 1000 ? `${(v / 1000).toFixed(v >= 10_000 ? 0 : 1)} s` : `${v} ms`);
const name = (unit: string) => unit.replace(/\.service$/, "");

/** A badge for an exposure level from systemd-analyze security. */
export function exposureBadge(e: Exposure): { variant: StatusVariant; text: string } {
  const level = e.level.toUpperCase();
  if (level === "OK") return { variant: "ok", text: `${e.score.toFixed(1)} locked down` };
  if (level === "MEDIUM") return { variant: "info", text: `${e.score.toFixed(1)} some rights` };
  if (level === "EXPOSED") return { variant: "warn", text: `${e.score.toFixed(1)} many rights` };
  return { variant: "crit", text: `${e.score.toFixed(1)} almost every right` };
}

/** The state of a unit file in plain words. */
export function stateText(u: UnitFile): string {
  if (u.state === "masked") return u.masked_by === "local" ? "Blocked on this computer" : u.masked_by === "runtime" ? "Blocked until restart" : "Blocked by its package";
  return (
    { enabled: "Starts at boot", disabled: "Off", static: "Started by others", indirect: "Started by others", generated: "Made by the system", "enabled-runtime": "On until restart", alias: "Another name" } as Record<string, string>
  )[u.state ?? ""] ?? u.state ?? "-";
}

// ---------- Services › System Services ----------

export function ServicesExpertSections() {
  const groups = useLiveWithPrevious<Groups>(loadGroups, 10_000);
  const units = useLive(loadUnits, 120_000);
  const security = useLive(loadSecurity, 600_000);
  const g = groups.data;
  const seconds = g && groups.before ? (g.at - groups.before.at) / 1000 : 0;
  const costs = (g?.groups ?? [])
    .filter((x) => x.kind === "service")
    .map((x) => {
      const b = groups.before?.groups.find((y) => y.unit === x.unit);
      const cpuNow = perSecond(x.cpu_usec, b?.cpu_usec, seconds);
      return { ...x, cpuNow: cpuNow === undefined ? undefined : cpuNow / 1e4 };
    })
    .sort((a, b) => (groups.before ? (b.cpuNow ?? 0) - (a.cpuNow ?? 0) : (b.cpu_usec ?? 0) - (a.cpu_usec ?? 0)));
  const changed = (units.data ?? []).filter((u) => u.local_overrides.length || u.masked_by === "local" || u.masked_by === "runtime");
  const looping = (units.data ?? []).filter((u) => u.restarts >= 3);
  const unsafe = (security.data ?? []).filter((e) => e.level.toUpperCase() === "UNSAFE").length;
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Changed on this computer", value: units.data ? String(changed.length) : "-", subtext: "local overrides or blocks" },
          { label: "Restarting again and again", value: units.data ? String(looping.length) : "-", subtext: looping.length ? looping.slice(0, 3).map((u) => name(u.unit)).join(", ") : "3 or more restarts", status: looping.length ? "warn" : "ok" },
          { label: "Services with almost every right", value: security.data ? String(unsafe) : "-", subtext: "exposure 9 or more of 10" },
        ]}
      />
      <Panel title="Cost per service" subtitle={groups.before ? "CPU now, busiest first" : "since each started (CPU now after the next reading)"} source="/sys/fs/cgroup/system.slice" span={12} flush>
        {g ? (
          <DataTable
            rows={costs}
            columns={[
              { key: "name", label: "Service", text: (x) => x.name },
              { key: "cpuNow", label: "CPU now", text: (x) => (x.cpuNow === undefined ? "-" : `${x.cpuNow.toFixed(1)}%`), numeric: true },
              { key: "cpu", label: "CPU time", text: (x) => (x.cpu_usec === undefined ? "-" : `${(x.cpu_usec / 1e6).toFixed(0)} s`), numeric: true },
              { key: "mem", label: "Own memory", text: (x) => (x.memory_own ?? x.memory) === undefined ? "-" : bytes(x.memory_own ?? x.memory ?? 0), numeric: true },
              { key: "peak", label: "Peak", text: (x) => (x.memory_peak === undefined ? "-" : bytes(x.memory_peak)), numeric: true },
              { key: "disk", label: "Disk read · written", text: (x) => `${bytes(x.disk_read ?? 0)} · ${bytes(x.disk_write ?? 0)}`, numeric: true },
              { key: "tasks", label: "Tasks", text: (x) => (x.tasks === undefined ? "-" : String(x.tasks)), numeric: true },
            ]}
            rowKey={(x) => x.unit}
            pageSize={12}
            search={{ placeholder: "Filter services" }}
          />
        ) : (
          <NoData error={groups.error} waiting="Loading…" />
        )}
      </Panel>
      <Panel
        title="Service setup"
        info="Changed here means a local override file or a block made on this computer. Services blocked by their own package are normal on Ubuntu."
        source="systemctl show"
        span={12}
        flush
      >
        {units.data ? (
          <DataTable
            rows={units.data}
            columns={[
              { key: "unit", label: "Service", text: (u) => name(u.unit) },
              { key: "state", label: "State", text: (u) => stateText(u) },
              { key: "by", label: "Turned on by", text: (u) => u.wanted_by.join(", ") || "-", wrap: true },
              {
                key: "local",
                label: "Changed here",
                text: (u) => (u.local_overrides.length ? u.local_overrides.join(", ") : u.masked_by === "local" ? "Blocked" : "-"),
                render: (u) => (u.local_overrides.length || u.masked_by === "local" ? <StatusBadge variant="info" label={u.local_overrides.length ? `${u.local_overrides.length} override${u.local_overrides.length === 1 ? "" : "s"}` : "Blocked"} /> : "-"),
              },
              { key: "restart", label: "Restarts", text: (u) => `${u.restarts}${u.restart ? ` (${u.restart})` : ""}`, render: (u) => (u.restarts >= 3 ? <StatusBadge variant="warn" label={`${u.restarts} (${u.restart ?? "-"})`} /> : `${u.restarts}${u.restart && u.restart !== "no" ? ` (${u.restart})` : ""}`) },
              { key: "file", label: "Unit file", text: (u) => [u.file, ...u.local_overrides, ...u.package_overrides].filter(Boolean).join(", ") || "-", wrap: true, expert: true },
            ]}
            rowKey={(u) => u.unit}
            pageSize={15}
            search={{ placeholder: "Filter services" }}
          />
        ) : (
          <NoData error={units.error} waiting="Loading…" />
        )}
      </Panel>
      <Panel
        title="How locked down each service is"
        info="systemd's exposure score: 0 means locked down, 10 means the service may do almost anything. It is not a security hole by itself; it shows how much damage a flaw in that service could do."
        source="systemd-analyze security"
        span={12}
        flush
      >
        {security.data ? (
          <DataTable
            rows={[...security.data].sort((a, b) => b.score - a.score)}
            columns={[
              { key: "unit", label: "Service", text: (e) => name(e.unit) },
              { key: "score", label: "Exposure", text: (e) => exposureBadge(e).text, render: (e) => <StatusBadge variant={exposureBadge(e).variant} label={exposureBadge(e).text} /> },
            ]}
            rowKey={(e) => e.unit}
            pageSize={12}
            empty="systemd-analyze did not answer"
          />
        ) : (
          <NoData error={security.error} waiting="Loading…" />
        )}
      </Panel>
    </>
  );
}

// ---------- System › Boot ----------

export function BootExpertSections() {
  const { data: d, error } = useLive(loadBoot, 600_000);
  if (!d) {
    return (
      <Panel title="Boot in detail" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const slowest = Math.max(0, ...d.critical_chain.map((c) => c.took_ms ?? 0));
  return (
    <>
      <Panel
        title="What the desktop waited for"
        subtitle="the boot's critical chain"
        info="Each step waited for the one below it. The step that took longest here is what really made the boot slow; units outside this chain started in parallel."
        source="systemd-analyze critical-chain"
        span={6}
        flush
      >
        {d.critical_chain.length ? (
          <ul className="chain panel-pad">
            {d.critical_chain.map((c) => (
              <li key={`${c.depth}|${c.unit}`} style={{ paddingLeft: Math.min(c.depth, 12) * 12 }}>
                <code>{c.unit}</code>
                <span className="muted"> ready at {ms(c.at_ms)}</span>
                {c.took_ms !== undefined && (c.took_ms === slowest && slowest >= 1000 ? <> <StatusBadge variant="warn" label={`took ${ms(c.took_ms)}`} /></> : <span className="muted"> · took {ms(c.took_ms)}</span>)}
              </li>
            ))}
          </ul>
        ) : (
          <div className="empty muted">systemd-analyze did not answer (the boot may still be running).</div>
        )}
      </Panel>
      <Panel title="Every unit's start time" subtitle={`${d.blame.length} units, slowest first`} info="Time each unit took to start. Many start at the same time, so these do not add up to the boot time." source="systemd-analyze blame" span={6} flush>
        <DataTable
          rows={d.blame.map(([unit, t]) => ({ unit, t }))}
          columns={[
            { key: "unit", label: "Unit", text: (b) => b.unit, wrap: true },
            { key: "t", label: "Took", text: (b) => ms(b.t), numeric: true },
          ]}
          rowKey={(b) => b.unit}
          pageSize={14}
          search={{ placeholder: "Filter units" }}
        />
      </Panel>
    </>
  );
}
