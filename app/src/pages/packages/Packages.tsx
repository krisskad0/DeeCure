// SPDX-License-Identifier: Apache-2.0
// Packages › Installed, Sources, Cleanup, Permissions (docs/03-ui-design/05-pages/13-packages.md),
// with real data. Cleanup removes packages no longer needed and leftover settings through
// the fix engine; removing any one package is not offered.

import { Guard } from "../../components/ErrorBoundary";
import { atLeast, useUi } from "../../ui";
import { PackageExpertDetail, SoftwareHealthExpert } from "./SoftwareExpert";
import { useState } from "react";
import { type PackageInfo, type PackagesInfo, getInventory } from "../../api";
import { AlertBanner } from "../../components/AlertBanner";
import { type Column, DataTable } from "../../components/DataTable";
import { Icon } from "../../components/Icon";
import { KeyValueList } from "../../components/KeyValueList";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { Donut } from "../../components/charts/Donut";
import { bytes } from "../../metrics";
import { Drawer } from "../../shell/Drawer";
import { kbPackage, useKb } from "../../kb";
import { useFix } from "../../shell/FixDrawer";
import { Button } from "../../components/Button";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";
import { PrivacyPermissions } from "../privacy/Privacy";

const loadPackages = () => getInventory("packages");
const usePackages = () => useLive(loadPackages, 300_000);

function Waiting({ title, error }: { title: string; error: Parameters<typeof NoData>[0]["error"] }) {
  return (
    <Panel title={title} span={12}>
      <NoData error={error} waiting="Reading the package lists…" />
    </Panel>
  );
}

const SOURCES = ["apt", "snap", "flatpak", "appimage"] as const;
const LABEL: Record<PackageInfo["source"], string> = { apt: "APT", snap: "Snap", flatpak: "Flatpak", appimage: "AppImage" };

const COLUMNS: Column<PackageInfo>[] = [
  { key: "name", label: "Name", text: (p) => `${p.name} ${p.summary ?? ""}`, render: (p) => (p.summary ? <span className="cell-two"><b>{p.name}</b><span>{p.summary}</span></span> : <b>{p.name}</b>) },
  { key: "version", label: "Version", text: (p) => p.version, render: (p) => (p.version ? <code>{p.version}</code> : "-") },
  { key: "source", label: "Source", text: (p) => p.source, render: (p) => <StatusBadge variant="info" label={p.source} /> },
  { key: "size", label: "Size", text: (p) => bytes(p.size), numeric: true },
  { key: "by", label: "Installed by", text: (p) => (p.manual ? "You" : "System"), render: (p) => (p.manual ? "You" : <span className="muted">System</span>) },
];

export function PackagesInstalled() {
  const { mode } = useUi();
  const { data: info, error } = usePackages();
  const [inspect, setInspect] = useState<PackageInfo | null>(null);
  const kb = useKb();
  if (!info) return <Waiting title="Packages" error={error} />;
  const of = (src: PackageInfo["source"]) => info.packages.filter((p) => p.source === src);
  const size = (src: PackageInfo["source"]) => of(src).reduce((s, p) => s + p.size, 0);
  const total = info.packages.reduce((s, p) => s + p.size, 0);
  return (
    <>
      <KpiRow
        tiles={SOURCES.filter((s) => s === "apt" || of(s).length).map((s) => ({
          label: LABEL[s],
          value: of(s).length.toLocaleString(),
          subtext: s === "apt" ? `${of(s).filter((p) => p.manual).length} installed by you` : bytes(size(s)),
        }))}
      />
      <Panel title="Disk usage by source" span={4}>
        <Donut
          segments={SOURCES.filter((s) => of(s).length).map((s, i) => ({ name: LABEL[s], value: size(s), slot: ((i % 5) + 1) as 1 | 2 | 3 | 4 | 5, display: bytes(size(s)) }))}
          center={{ value: (total / 1024 ** 3).toFixed(1), subtitle: "GB" }}
        />
      </Panel>
      <Panel title="Packages" source="dpkg · snap · flatpak" span={8} flush>
        <DataTable
          rows={info.packages}
          columns={COLUMNS}
          rowKey={(p) => `${p.source}|${p.name}`}
          search={{ placeholder: "Search packages" }}
          filter={{ column: "source", options: ["apt", "snap", "flatpak", "appimage"] }}
          pageSize={10}
          total={info.packages.length}
          onRowClick={setInspect}
        />
      </Panel>
      {inspect && (
        <Drawer title={inspect.name} subtitle={`${LABEL[inspect.source]} package`} note="To remove packages no longer needed, use Packages › Cleanup" onClose={() => setInspect(null)}>
          <KeyValueList
            rows={[
              ["Version", inspect.version ? <code key="v">{inspect.version}</code> : "-"],
              ["Size", bytes(inspect.size)],
              ["Installed by", inspect.manual ? "You" : "The system (as a dependency)"],
              ["Section", inspect.section || "-"],
              ["About", kbPackage(kb, inspect.name)?.plain ?? (inspect.summary || "-")],
            ]}
          />
          {atLeast(mode, "expert") && <Guard><PackageExpertDetail name={inspect.name} /></Guard>}
        </Drawer>
      )}
    </>
  );
}

type SourceRow = PackagesInfo["sources"][number];
const SOURCE_COLUMNS: Column<SourceRow>[] = [
  { key: "line", label: "Repository", text: (s) => s.line, render: (s) => <code className="wrap-anywhere">{s.line}</code>, wrap: true },
  { key: "kind", label: "Type", text: (s) => s.kind, render: (s) => <StatusBadge variant={s.kind === "Official" ? "ok" : s.kind === "PPA" ? "info" : "off"} label={s.kind} /> },
  { key: "signed", label: "Signed", text: (s) => (s.signed ? "Yes" : "No"), render: (s) => (s.signed ? <Icon name="check" size={16} /> : <StatusBadge variant="crit" label="Not checked" />) },
  { key: "key", label: "Own key", text: (s) => (s.own_key ? "Yes" : "No"), render: (s) => (s.own_key ? <Icon name="check" size={16} /> : <span className="muted">Shared keyring</span>), expert: true },
  { key: "status", label: "Status", text: (s) => (s.enabled ? "On" : "Off"), render: (s) => <StatusBadge variant={s.enabled ? "ok" : "off"} label={s.enabled ? "On" : "Off"} /> },
  { key: "file", label: "File", text: (s) => s.file, expert: true },
];

export function PackagesSources() {
  const { mode } = useUi();
  const { data: info, error } = usePackages();
  if (!info) return <Waiting title="Software sources" error={error} />;
  const unchecked = info.sources.filter((s) => s.enabled && !s.signed).length;
  return (
    <>
      {unchecked > 0 && <AlertBanner tone="crit" title={`${unchecked} source${unchecked === 1 ? " skips" : "s skip"} signature checks`} text="Sources marked trusted=yes can install anything. Remove the option, or the source." />}
      <Panel title="Software sources" source="/etc/apt/sources.list · sources.list.d" span={12} flush>
        <DataTable rows={info.sources} columns={SOURCE_COLUMNS} rowKey={(s) => `${s.file}|${s.line}`} filter={{ column: "kind", options: ["Official", "PPA", "Third-party"] }} />
      </Panel>
      {atLeast(mode, "expert") && <Guard><SoftwareHealthExpert /></Guard>}
    </>
  );
}

type Removable = PackagesInfo["removable"][number];
const REMOVABLE: Column<Removable>[] = [
  { key: "title", label: "Item", text: (r) => r.title, render: (r) => (<span className="cell-two"><b>{r.title}</b><span>{r.names.slice(0, 4).join(", ")}{r.names.length > 4 ? ` +${r.names.length - 4} more` : ""}</span></span>) },
  { key: "count", label: "Count", text: (r) => String(r.count), numeric: true },
  { key: "size", label: "Size", text: (r) => (r.size ? bytes(r.size) : "-"), numeric: true },
  { key: "undo", label: "Undo", text: (r) => r.undo, render: (r) => <StatusBadge variant={r.undo === "reinstallable" ? "info" : "warn"} label={r.undo === "reinstallable" ? "Reinstallable" : "Irreversible"} /> },
];

export function PackagesCleanup() {
  const { data: info, error } = usePackages();
  const openFix = useFix();
  if (!info) return <Waiting title="Reclaimable" error={error} />;
  return (
    <>
      <Panel title="Reclaimable" source="apt-get -s autoremove (simulation) · snap · dpkg" span={12} flush>
        <DataTable
          rows={info.removable}
          columns={[...REMOVABLE, { key: "act", label: "", text: () => "", render: (r) => <Button small onClick={() => openFix(r.action)}>Remove…</Button> }]}
          rowKey={(r) => r.action}
          empty="Nothing to remove"
        />
      </Panel>
    </>
  );
}

export const PackagesPermissions = PrivacyPermissions;
