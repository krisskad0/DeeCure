// SPDX-License-Identifier: Apache-2.0
// Hardware details only the admin can read, through the helper (docs/01-product/expert-coverage.md):
// memory slots and the board's limits, BIOS settings, NVMe controller details, and drive
// self-tests. Each stays empty until the person asks with their password.

import type { ReactNode } from "react";
import { AdminRead } from "../../components/AdminRead";
import { type Column, DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { bytes } from "../../metrics";

type Fields = Record<string, string | undefined>;
export interface MemorySlots {
  arrays: Fields[];
  modules: Fields[];
}

const filled = (m: Fields) => m.Filled === "true";
const known = (v?: string) => (v && !/^(unknown|not specified|none)$/i.test(v) ? v : undefined);

/** Memory channels in use, from bank names such as "P0 CHANNEL A", "ChannelA-DIMM0", or "BANK 1". */
export function channelsInUse(modules: Fields[]): number | undefined {
  const ids = new Set<string>();
  for (const m of modules.filter(filled)) {
    const text = `${m["Bank Locator"] ?? ""} ${m.Locator ?? ""}`;
    const ch = /channel[\s_-]*([a-z0-9]+)/i.exec(text) ?? /bank[\s_-]*(\d+)/i.exec(text);
    if (!ch) return undefined;
    ids.add(ch[1].toUpperCase());
  }
  return ids.size || undefined;
}

/** The speed a module runs at against its rated speed, such as "4800 of 5600 MT/s". */
export function moduleSpeed(m: Fields): { text: string; slower: boolean } {
  const rated = known(m.Speed);
  const now = known(m["Configured Memory Speed"]);
  if (!rated || !now || rated === now) return { text: now ?? rated ?? "-", slower: false };
  const slower = parseFloat(now) < parseFloat(rated);
  return { text: `${now} (rated ${rated})`, slower };
}

const SLOT_COLUMNS: Column<Fields>[] = [
  { key: "slot", label: "Slot", text: (m) => [m.Locator, known(m["Bank Locator"])].filter(Boolean).join(" · ") || "-" },
  { key: "size", label: "Size", text: (m) => (filled(m) ? (m.Size ?? "-") : "Empty"), render: (m) => (filled(m) ? (m.Size ?? "-") : <span className="muted">Empty</span>) },
  { key: "type", label: "Type", text: (m) => (filled(m) ? [known(m.Type), known(m["Form Factor"])].filter(Boolean).join(" · ") || "-" : "-") },
  {
    key: "speed",
    label: "Speed",
    text: (m) => (filled(m) ? moduleSpeed(m).text : "-"),
    render: (m) => {
      if (!filled(m)) return "-";
      const s = moduleSpeed(m);
      return s.slower ? <StatusBadge variant="info" label={s.text} /> : s.text;
    },
  },
  { key: "maker", label: "Maker and part", text: (m) => (filled(m) ? [known(m.Manufacturer), known(m["Part Number"])].filter(Boolean).join(" · ") || "-" : "-"), wrap: true },
  { key: "rank", label: "Rank", text: (m) => (filled(m) ? (known(m.Rank) ?? "-") : "-"), expert: true },
];

export function MemorySlotsView({ data }: { data: MemorySlots }) {
  const used = data.modules.filter(filled);
  const free = data.modules.length - used.length;
  const max = data.arrays.map((a) => known(a["Maximum Capacity"])).filter(Boolean);
  const ecc = data.arrays.map((a) => known(a["Error Correction Type"])).find(Boolean);
  const soldered = used.length > 0 && used.every((m) => /row of chips|chip/i.test(m["Form Factor"] ?? ""));
  const channels = channelsInUse(data.modules);
  const slower = used.some((m) => moduleSpeed(m).slower);
  const rows: [string, ReactNode][] = [
    ["Most memory the board takes", max.length ? max.join(" + ") : "Not reported"],
    ["Slots", soldered ? "Soldered to the board: it cannot be upgraded" : `${data.modules.length} (${free} free)`],
    [
      "Channels in use",
      channels === undefined ? (
        "Not reported"
      ) : channels === 1 && !soldered ? (
        <StatusBadge key="c" variant="warn" label="1: single-channel, about half the memory speed. A module in another channel doubles it." />
      ) : (
        `${channels}`
      ),
    ],
    ["Error correction (ECC)", ecc ?? "Not reported"],
  ];
  if (slower) rows.push(["Speed", "Some memory runs below its rated speed. The processor or a slower second module usually sets the limit."]);
  return (
    <>
      <KeyValueList rows={rows} />
      <DataTable rows={data.modules} columns={SLOT_COLUMNS} rowKey={(m) => `${m.Locator}-${m["Bank Locator"]}`} empty="The firmware lists no memory slots" />
    </>
  );
}

export interface BiosInterface {
  name: string;
  admin_password?: boolean;
  system_password?: boolean;
  settings: { name: string; label?: string; kind?: string; value?: string; default?: string; options?: string[]; min?: string; max?: string }[];
}
type BiosSetting = BiosInterface["settings"][number];

const allowed = (s: BiosSetting) => (s.options?.length ? s.options.join(", ") : s.min !== undefined || s.max !== undefined ? `${s.min ?? "?"} to ${s.max ?? "?"}` : "-");

const BIOS_COLUMNS: Column<BiosSetting>[] = [
  { key: "name", label: "Setting", text: (s) => s.label ?? s.name, render: (s) => <span title={s.name}>{s.label ?? s.name}</span>, wrap: true },
  {
    key: "value",
    label: "Value",
    text: (s) => s.value ?? "-",
    render: (s) => (
      <>
        <code>{s.value ?? "-"}</code>
        {s.default !== undefined && s.value !== undefined && s.value !== s.default && <> <StatusBadge variant="info" label="changed" /></>}
      </>
    ),
  },
  { key: "default", label: "Default", text: (s) => s.default ?? "-" },
  { key: "allowed", label: "Allowed", text: allowed, wrap: true },
];

const passwordText = (v?: boolean) => (v === undefined ? "Not reported" : v ? "Set" : "Not set");

export function BiosSettingsView({ data }: { data: BiosInterface[] }) {
  if (data.length === 0) return <div className="empty muted">This computer offers no BIOS settings to Linux. Dell, Lenovo, and HP laptops usually do.</div>;
  return (
    <>
      {data.map((i) => (
        <div key={i.name}>
          <KeyValueList
            rows={[
              ["Interface", <code key="i">{i.name}</code>],
              ["BIOS admin password", passwordText(i.admin_password)],
              ["Power-on password", passwordText(i.system_password)],
              ["Settings", `${i.settings.length} (${i.settings.filter((s) => s.default !== undefined && s.value !== s.default).length} changed from the default)`],
            ]}
          />
          <DataTable rows={i.settings} columns={BIOS_COLUMNS} rowKey={(s) => s.name} search={{ placeholder: "Find a setting" }} empty="No settings" />
        </div>
      ))}
    </>
  );
}

export interface NvmeDetails {
  controller?: {
    model?: string;
    firmware?: string;
    firmware_slots?: number;
    slot1_read_only?: boolean;
    activate_without_reset?: boolean;
    power_states?: number;
    autonomous_power_saving?: boolean;
    warning_celsius?: number;
    critical_celsius?: number;
    namespaces?: number;
    write_cache?: boolean;
    self_test?: boolean;
  } | null;
  namespace?: { size_bytes?: number; formats: { block_size: number; metadata: number; performance: string; in_use: boolean }[] } | null;
  firmware?: { active: number; next?: number | null; slots: { slot: number; version: string }[] } | null;
}

const yesNo = (v?: boolean) => (v === undefined ? "-" : v ? "Yes" : "No");

export function NvmeDetailsView({ data }: { data: NvmeDetails }) {
  const c = data.controller;
  const ns = data.namespace;
  const fw = data.firmware;
  const inUse = ns?.formats.find((f) => f.in_use);
  const better = ns?.formats.find((f) => f.metadata === 0 && inUse && f.block_size > inUse.block_size && f.performance === "best" && inUse.performance !== "best");
  const rows: [string, ReactNode][] = [];
  if (c) {
    rows.push(
      ["Firmware slots", c.firmware_slots ? `${c.firmware_slots}${c.slot1_read_only ? " (slot 1 is read-only, a factory copy)" : ""}` : "-"],
      ["New firmware starts without a restart", yesNo(c.activate_without_reset)],
      ["Namespaces", c.namespaces === undefined ? "-" : String(c.namespaces)],
      ["Power states", c.power_states === undefined ? "-" : `${c.power_states}${c.autonomous_power_saving ? " · saves power by itself (APST)" : ""}`],
      ["Write cache", yesNo(c.write_cache)],
      ["Self-tests", c.self_test === undefined ? "-" : c.self_test ? "Supported" : "Not supported"],
    );
  }
  if (fw) {
    rows.push(["Firmware in each slot", fw.slots.length ? fw.slots.map((s) => `${s.slot}: ${s.version}${s.slot === fw.active ? " (running)" : ""}`).join(" · ") : "-"]);
    if (fw.next) rows.push(["At the next restart", `slot ${fw.next} starts`]);
  }
  if (ns) {
    rows.push(
      ["Size", ns.size_bytes ? bytes(ns.size_bytes, 1) : "-"],
      ["Block formats", ns.formats.map((f) => `${f.block_size} bytes${f.metadata ? ` + ${f.metadata}` : ""} (${f.performance}${f.in_use ? ", in use" : ""})`).join(" · ") || "-"],
    );
    if (better) rows.push(["Note", `The drive says ${better.block_size}-byte blocks are faster. Changing it erases the whole drive, so it is only worth it on a new install.`]);
  }
  return rows.length ? <KeyValueList rows={rows} /> : <div className="empty muted">The drive gave no NVMe details.</div>;
}

export interface SelfTests {
  supported: boolean;
  running?: string | null;
  tests: { kind?: string; result?: string; passed?: boolean | null; hours?: number }[];
}

export function SelfTestsView({ data }: { data: SelfTests }) {
  if (!data.supported) return <div className="empty muted">This drive keeps no self-test log.</div>;
  return (
    <>
      {data.running && <div className="empty">{data.running}</div>}
      <DataTable
        rows={data.tests}
        columns={[
          { key: "kind", label: "Test", text: (t) => t.kind ?? "-" },
          {
            key: "result",
            label: "Result",
            text: (t) => t.result ?? "-",
            render: (t) => (t.passed === undefined || t.passed === null ? (t.result ?? "-") : <StatusBadge variant={t.passed ? "ok" : "crit"} label={t.result ?? (t.passed ? "Passed" : "Failed")} />),
            wrap: true,
          },
          { key: "hours", label: "At drive hour", text: (t) => (t.hours === undefined ? "-" : t.hours.toLocaleString()), numeric: true },
        ]}
        rowKey={(t) => `${t.hours}-${t.kind}-${t.result}`}
        empty="No self-test has run on this drive yet"
      />
    </>
  );
}

/** JSON from the helper, or a short note when it is not readable. */
function parsed<T>(text: string, view: (v: T) => ReactNode): ReactNode {
  try {
    return view(JSON.parse(text) as T);
  } catch {
    return <div className="empty muted">The helper gave no readable answer.</div>;
  }
}

/** Expert panels per drive: NVMe details and self-test results. */
export function DriveAdminSections({ disks }: { disks: { name: string }[] }) {
  return (
    <>
      {disks.map((d) => (
        <Panel key={`st-${d.name}`} title={`Self-tests · ${d.name}`} info="The drive checks itself: a short test takes about 2 minutes, a long one reads every block." source="smartctl -j -l selftest" span={6}>
          <AdminRead what="self_tests" arg={d.name} label="Read the self-test log (asks for your password)">
            {(t) => parsed<SelfTests>(t, (v) => <SelfTestsView data={v} />)}
          </AdminRead>
        </Panel>
      ))}
      {disks
        .filter((d) => /^nvme\d+n\d+$/.test(d.name))
        .map((d) => (
          <Panel key={`nv-${d.name}`} title={`NVMe controller · ${d.name}`} source="nvme id-ctrl · id-ns · fw-log" span={6} flush>
            <AdminRead what="nvme" arg={d.name} label="Read the NVMe details (asks for your password)">
              {(t) => parsed<NvmeDetails>(t, (v) => <NvmeDetailsView data={v} />)}
            </AdminRead>
          </Panel>
        ))}
    </>
  );
}

export function MemorySlotsPanel() {
  return (
    <Panel title="Memory slots" info="Each slot, the most memory the board takes, and how many memory channels are in use. One channel halves memory speed and slows built-in graphics." source="dmidecode -t 16 -t 17" span={12}>
      <AdminRead what="memory_slots" label="Read the memory slots (asks for your password)">
        {(t) => parsed<MemorySlots>(t, (v) => <MemorySlotsView data={v} />)}
      </AdminRead>
    </Panel>
  );
}

export function BiosSettingsPanel() {
  return (
    <Panel title="BIOS settings" info="Every setting the firmware shares with Linux, read only. Changing them still happens in the BIOS setup." source="/sys/class/firmware-attributes" span={12}>
      <AdminRead what="bios_settings" label="Read the BIOS settings (asks for your password)">
        {(t) => parsed<BiosInterface[]>(t, (v) => <BiosSettingsView data={v} />)}
      </AdminRead>
    </Panel>
  );
}
