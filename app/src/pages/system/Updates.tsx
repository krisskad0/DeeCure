// SPDX-License-Identifier: Apache-2.0
// System › Updates (docs/03-ui-design/05-pages/05-system.md), with real data.
// Installing updates needs the helper (Phase 2); until then the page only shows them.

import { type AptRun, type UpdatesInfo, getInventory, getSnapshot } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { useLive } from "../../useLive";
import { Button } from "../../components/Button";
import { useFix } from "../../shell/FixDrawer";
import { ago } from "../../time";
import { NoData } from "../NoData";

type Pending = UpdatesInfo["pending"][number];

const loadUpdates = () => getSnapshot<UpdatesInfo>("updates");
const loadHistory = () => getInventory("update_history");
const loadSystem = () => getInventory("system");

const PENDING: Column<Pending>[] = [
  { key: "name", label: "Package", text: (p) => p.name },
  { key: "installed", label: "Installed", text: (p) => p.installed, render: (p) => <code>{p.installed}</code>, expert: true },
  { key: "candidate", label: "Candidate", text: (p) => p.candidate, render: (p) => <code>{p.candidate}</code> },
  { key: "priority", label: "Priority", text: (p) => (p.security ? "Security" : "Regular"), render: (p) => <StatusBadge variant={p.security ? "warn" : "off"} label={p.security ? "Security" : "Regular"} /> },
  { key: "origin", label: "From", text: (p) => p.origin, expert: true },
];

/** apt writes local time as "2026-10-23  00:34:58". */
const aptTime = (start: string) => new Date(start.replace(/\s+/, "T")).getTime();

const HISTORY: Column<AptRun>[] = [
  { key: "date", label: "Date", text: (r) => r.start, render: (r) => new Date(aptTime(r.start)).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" }) },
  { key: "packages", label: "Packages", text: (r) => String(r.packages), numeric: true },
  { key: "changes", label: "Changes", text: (r) => r.changes.join(", "), render: (r) => r.changes.slice(0, 4).join(", ") + (r.changes.length > 4 ? ` +${r.changes.length - 4} more` : ""), wrap: true },
  { key: "source", label: "Source", text: (r) => r.source },
  { key: "result", label: "Result", text: (r) => (r.failed ? "Failed" : "Success"), render: (r) => <StatusBadge variant={r.failed ? "crit" : "ok"} label={r.failed ? "Failed" : "Success"} /> },
];

export function SystemUpdates() {
  const { data: snapshot, error } = useLive(loadUpdates, 60_000);
  const history = useLive(loadHistory, 300_000).data;
  const system = useLive(loadSystem, 60_000).data;
  const openFix = useFix();
  const info = snapshot?.data;
  if (!info) {
    return (
      <Panel title="Pending updates" span={12}>
        <NoData error={error} waiting={snapshot === null ? "DeeCure has not asked apt yet. It does so within a minute of starting." : "Loading…"} />
      </Panel>
    );
  }
  const security = info.pending.filter((p) => p.security).length;
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Pending", value: String(info.pending.length), status: info.pending.length ? "warn" : "ok" },
          { label: "Security", value: String(security), status: security ? "warn" : "ok" },
          { label: "Last check", value: info.lists_updated_at ? ago(info.lists_updated_at) : "Never", subtext: "package lists refreshed" },
          { label: "Automatic security updates", value: info.automatic_security ? "On" : "Off", status: info.automatic_security ? "ok" : "warn" },
          { label: "Reboot required", value: system?.reboot_required ? "Yes" : "No", status: system?.reboot_required ? "info" : "ok" },
        ]}
      />
      <Panel
        title="Pending updates"
        subtitle={`${info.pending.length} packages`}
        source="apt list --upgradable"
        actions={
          info.pending.length > 0 && (
            <span className="btn-row">
              {security > 0 && <Button small variant="primary" onClick={() => openFix("security.install_security_updates")}>Install security updates…</Button>}
              <Button small onClick={() => openFix("system.install_updates")}>Install all…</Button>
            </span>
          )
        }
        span={12}
        flush
      >
        <DataTable rows={info.pending} columns={PENDING} rowKey={(p) => p.name} search={{ placeholder: "Search packages" }} empty="Everything is up to date" />
      </Panel>
      <Panel title="Update history" source="/var/log/apt/history.log" span={12} flush>
        {history ? <DataTable rows={history} columns={HISTORY} rowKey={(r) => r.start} pageSize={8} empty="No apt runs in the current log" /> : <div className="empty muted">Loading…</div>}
      </Panel>
    </>
  );
}
