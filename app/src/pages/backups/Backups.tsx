// SPDX-License-Identifier: Apache-2.0
// Backups › Backup Jobs, System Snapshots, Restore Points (docs/03-ui-design/05-pages/15-backups.md),
// with real data from Déjà Dup and Timeshift. Read-only.

import { type BackupsInfo, getInventory } from "../../api";
import { AlertBanner } from "../../components/AlertBanner";
import { type Column, DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { when } from "../../metrics";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";
import { FixHistory } from "../../shell/FixHistory";

type Snap = BackupsInfo["timeshift"]["snapshots"][number];

const loadBackups = () => getInventory("backups");
const useBackups = () => useLive(loadBackups, 300_000);
const days = (ms: number) => Math.floor((Date.now() - ms) / 86_400_000);

export function BackupsJobs() {
  const { data: b, error } = useBackups();
  if (!b) return <Panel title="Backups" span={12}><NoData error={error} waiting="Loading…" /></Panel>;
  const d = b.deja_dup;
  const newest = [d.last_backup, b.timeshift.snapshots[0]?.created].filter((x): x is number => x !== undefined).sort((x, y) => y - x)[0];
  const age = newest === undefined ? undefined : days(newest);
  const tool = d.installed && d.scheduled ? "Déjà Dup" : b.timeshift.configured ? "Timeshift" : "None";
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Last backup", value: age === undefined ? "Never" : String(age), unit: age === undefined ? undefined : "days ago", status: age === undefined ? "warn" : age > 35 ? "warn" : "ok", subtext: newest ? when(newest) : "" },
          { label: "Tool", value: tool, subtext: tool === "Déjà Dup" ? "Ubuntu Backups" : tool === "Timeshift" ? "system snapshots" : "nothing scheduled", status: tool === "None" ? "warn" : "ok" },
          { label: "Personal files", value: d.installed ? (d.scheduled ? "Backed up" : "Not scheduled") : "Not protected", status: d.scheduled ? "ok" : "warn", subtext: d.scheduled && d.every_days ? `every ${d.every_days} days` : "" },
        ]}
      />
      <Panel title="Ubuntu Backups (Déjà Dup)" source="gsettings org.gnome.DejaDup" span={6} flush>
        {d.installed ? (
          <KeyValueList
            rows={[
              ["Automatic backups", d.scheduled ? `On, every ${d.every_days ?? "?"} days` : "Off"],
              ["Last backup", d.last_backup ? when(d.last_backup) : "Never"],
              ["Stored on", d.backend ?? "-"],
            ]}
          />
        ) : (
          <div className="empty muted">Déjà Dup (Ubuntu Backups) is not installed, so personal files are not backed up by Ubuntu's own tool.</div>
        )}
      </Panel>
      <Panel title="Timeshift" source="/etc/timeshift/timeshift.json" span={6} flush>
        {b.timeshift.installed ? (
          <KeyValueList
            rows={[
              ["Set up", b.timeshift.configured ? "Yes" : "No"],
              ["Mode", b.timeshift.mode ?? "-"],
              ["Schedule", b.timeshift.schedule.join(", ") || "Off"],
              ["Snapshots found", String(b.timeshift.snapshots.length)],
            ]}
          />
        ) : (
          <div className="empty muted">Timeshift is not installed.</div>
        )}
      </Panel>
      {!d.installed && <AlertBanner tone="warn" title="Personal files have no backup" text="Timeshift protects the system, not your documents and photos. Install Ubuntu Backups (deja-dup) to copy your home folder to another drive or a cloud account." />}
    </>
  );
}

const SNAPS: Column<Snap>[] = [
  { key: "created", label: "Created", text: (s) => String(s.created), render: (s) => when(s.created) },
  { key: "kind", label: "Type", text: (s) => s.kind },
  { key: "tool", label: "Tool", text: () => "Timeshift" },
  { key: "comment", label: "Comment", text: (s) => s.comment ?? "-" },
];

export function BackupsSnapshots() {
  const { data: b, error } = useBackups();
  if (!b) return <Panel title="Snapshots" span={12}><NoData error={error} waiting="Loading…" /></Panel>;
  return (
    <>
      {!b.timeshift.installed && <AlertBanner tone="warn" title="No system snapshots" text="Timeshift is not installed. Snapshots let you roll back the system after a bad update." />}
      {b.timeshift.installed && !b.timeshift.snapshots.length && <AlertBanner tone="info" title="No snapshots found" text="The backup drive may not be connected, or no snapshot was taken yet." />}
      <Panel title="Snapshots" source="/timeshift/snapshots" span={12} flush>
        <DataTable rows={b.timeshift.snapshots} columns={SNAPS} rowKey={(s) => String(s.created)} empty="No snapshots yet" />
      </Panel>
    </>
  );
}

export function BackupsRestore() {
  return (
    <Panel title="DeeCure restore points" info="Fixes that can be taken back. Undo restores the earlier state." span={12} flush>
      <FixHistory restorePoints />
    </Panel>
  );
}

