// SPDX-License-Identifier: Apache-2.0
// Privacy › Device Access, App Permissions, Privacy Settings
// (docs/03-ui-design/05-pages/19-privacy.md), with real data. Changing settings comes later.

import { type PrivacyInfo, getInventory } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { FieldRow, StateTiles } from "../../components/FormControls";
import { Icon } from "../../components/Icon";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

type App = PrivacyInfo["apps"][number];

const loadPrivacy = () => getInventory("privacy");

function Waiting({ title, error }: { title: string; error: Parameters<typeof NoData>[0]["error"] }) {
  return (
    <Panel title={title} span={12}>
      <NoData error={error} waiting="Loading…" />
    </Panel>
  );
}

const state = (apps: string[], off: string) => (apps.length ? `In use by ${apps.join(", ")}` : off);

export function PrivacyAccess() {
  const { data: p, error } = useLive(loadPrivacy, 5_000);
  if (!p) return <Waiting title="Right now" error={error} />;
  return (
    <>
      <Panel title="Right now" source="PipeWire" span={12}>
        <StateTiles
          tiles={[
            { icon: "mic", title: "Microphone", state: state(p.in_use.microphone, "Not in use") },
            { icon: "cam", title: "Camera", state: state(p.in_use.camera, "Not in use") },
            { icon: "screen", title: "Screen sharing", state: state(p.in_use.screen, "Off") },
          ]}
        />
      </Panel>
    </>
  );
}

const yes = (on: boolean) => (on ? <Icon name="check" size={16} /> : <span className="muted">-</span>);
const PERMS: Column<App>[] = [
  { key: "app", label: "App", text: (a) => a.app },
  { key: "kind", label: "Kind", text: (a) => a.kind },
  { key: "camera", label: "Camera", text: (a) => (a.camera ? "Yes" : "No"), render: (a) => yes(a.camera) },
  { key: "microphone", label: "Microphone", text: (a) => (a.microphone ? "Yes" : "No"), render: (a) => yes(a.microphone) },
  { key: "location", label: "Location", text: (a) => (a.location ? "Yes" : "No"), render: (a) => yes(a.location) },
  { key: "files", label: "Files", text: (a) => (a.files ? "Yes" : "No"), render: (a) => yes(a.files) },
  { key: "network", label: "Network", text: (a) => (a.network ? "Yes" : "No"), render: (a) => yes(a.network) },
];

export function PrivacyPermissions() {
  const { data: p, error } = useLive(loadPrivacy, 60_000);
  if (!p) return <Waiting title="Permissions by app" error={error} />;
  return (
    <Panel title="Permissions by app" subtitle="snap and Flatpak apps; other apps can use everything you can" source="snap connections · flatpak info" span={12} flush>
      <DataTable rows={p.apps} columns={PERMS} rowKey={(a) => `${a.kind}|${a.app}`} search={{ placeholder: "Filter apps" }} filter={{ column: "kind", options: ["snap", "flatpak"] }} pageSize={15} empty="No snap or Flatpak apps" />
    </Panel>
  );
}

export function PrivacySettings() {
  const { data: p, error } = useLive(loadPrivacy, 60_000);
  if (!p) return <Waiting title="Settings" error={error} />;
  return (
    <Panel title="Settings" source="GNOME settings (gsettings)" span={8}>
      {p.settings.length ? (
        p.settings.map((s) => (
          <FieldRow key={s.label} label={s.label} description={s.description}>
            <StatusBadge variant={s.on ? "info" : "off"} label={s.on ? "On" : "Off"} />
          </FieldRow>
        ))
      ) : (
        <div className="empty muted">GNOME settings are not readable (not a GNOME session).</div>
      )}
    </Panel>
  );
}
