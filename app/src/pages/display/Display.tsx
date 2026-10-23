// SPDX-License-Identifier: Apache-2.0
// Display & Audio (docs/03-ui-design/05-pages/08-display.md), with real data.

import { type AudioDevice, type DisplayInfo, getInventory, getMachine } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { Icon } from "../../components/Icon";
import { InlineMeter } from "../../components/InlineMeter";
import { KeyValueList } from "../../components/KeyValueList";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

type Output = DisplayInfo["outputs"][number];

const loadDisplay = () => getInventory("display");
const loadAudio = () => getInventory("audio");
const loadMachine = () => getMachine();

const yes = (on: boolean) => (on ? <Icon name="check" size={16} /> : <span className="muted">-</span>);

const OUTPUTS: Column<Output>[] = [
  { key: "out", label: "Output", text: (o) => o.connector, render: (o) => <code>{o.connector}</code> },
  { key: "name", label: "Screen", text: (o) => o.name ?? (o.connected ? "Unknown" : "-") },
  { key: "res", label: "Resolution", text: (o) => (o.width && o.height ? `${o.width} × ${o.height}` : "-") },
  { key: "hz", label: "Refresh", text: (o) => (o.refresh_hz ? `${Math.round(o.refresh_hz)} Hz` : "-"), numeric: true },
  { key: "scale", label: "Scale", text: (o) => (o.scale ? `${Math.round(o.scale * 100)}%` : "-"), numeric: true },
  { key: "primary", label: "Primary", text: (o) => (o.primary ? "Yes" : "No"), render: (o) => yes(o.primary) },
  { key: "status", label: "Status", text: (o) => (o.connected ? "Connected" : "Disconnected"), render: (o) => <StatusBadge variant={o.connected ? "ok" : "off"} label={o.connected ? "Connected" : "Disconnected"} /> },
];

export function DisplayDisplays() {
  const { data: display, error } = useLive(loadDisplay, 15_000);
  const machine = useLive(loadMachine, 3_600_000).data;
  if (!display) {
    return (
      <Panel title="Connected displays" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const session = machine?.session_type;
  return (
    <>
      <Panel title="Session" source="$XDG_SESSION_TYPE · GNOME" span={4} flush>
        <KeyValueList
          rows={[
            ["Display server", session ? session[0].toUpperCase() + session.slice(1) : "-"],
            ["Compositor", display.compositor ? `${display.compositor}${display.shell_version ? ` (GNOME ${display.shell_version})` : ""}` : "Not GNOME, or not reachable"],
            ["Fractional scaling", display.fractional_scaling ? "On" : "Off"],
          ]}
        />
      </Panel>
      <Panel title="Connected displays" source="/sys/class/drm · Mutter" span={8} flush>
        <DataTable rows={display.outputs} columns={OUTPUTS} rowKey={(o) => o.connector} empty="No display outputs found" />
      </Panel>
    </>
  );
}

const devices = (label: string): Column<AudioDevice>[] => [
  { key: "name", label: "Device", text: (d) => d.description, wrap: true },
  { key: "vol", label: label, text: (d) => (d.volume === undefined ? "-" : `${d.volume}%`), render: (d) => (d.muted ? <StatusBadge variant="off" label="Muted" /> : d.volume === undefined ? "-" : <InlineMeter percent={d.volume} tone="ok" />) },
  { key: "default", label: "Default", text: (d) => (d.default ? "Yes" : "No"), render: (d) => yes(d.default) },
];

export function DisplayAudio() {
  const { data: audio, error } = useLive(loadAudio, 10_000);
  if (!audio) {
    return (
      <Panel title="Audio" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  if (!audio.server) {
    return (
      <Panel title="Audio" span={12}>
        <div className="empty muted">No sound server answered (PipeWire is not running for this user).</div>
      </Panel>
    );
  }
  const out = audio.outputs.find((d) => d.default);
  const mic = audio.inputs.find((d) => d.default);
  const short = (d?: AudioDevice) => d?.description.split(/\s+(?=Speaker|Headphones|HDMI|Digital|Headset|Microphone)/).pop() ?? "None";
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Output", value: short(out), subtext: out ? (out.muted ? "Muted" : `Volume ${out.volume ?? "-"}%`) : "" },
          { label: "Input", value: short(mic), subtext: mic ? (mic.muted ? "Muted" : "Not muted") : "" },
          { label: "Sound server", value: audio.server.split(" ")[0], subtext: audio.server.split(" ")[1] ?? "" },
          { label: "Active streams", value: String(audio.streams.length), subtext: audio.streams.slice(0, 3).join(", ") },
        ]}
      />
      <Panel title="Output devices" source="pw-dump" span={6} flush>
        <DataTable rows={audio.outputs} columns={devices("Volume")} rowKey={(d) => d.name} empty="No output devices" />
      </Panel>
      <Panel title="Input devices" source="pw-dump" span={6} flush>
        <DataTable rows={audio.inputs} columns={devices("Level")} rowKey={(d) => d.name} empty="No input devices" />
      </Panel>
    </>
  );
}
