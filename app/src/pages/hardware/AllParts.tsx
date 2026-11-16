// SPDX-License-Identifier: Apache-2.0
// Hardware › All Parts (docs/03-ui-design/05-pages/06-hardware.md): every physical part in one
// table, each with a link to its details, then the slots and sockets. The parts come from
// answers the agent already gives; the slots and sockets the maker lists need the password once.

import { useState } from "react";
import { type BluetoothInfo, type DevicesDetail, type HardwareInfo, type PlatformDetail, type PowerDetail, type PowerInfo, getInventory, helperRead, toAppError } from "../../api";
import { useHelperInstalled } from "../../components/AdminRead";
import { Button, LinkButton } from "../../components/Button";
import { type Column, DataTable } from "../../components/DataTable";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { bytes, pickAll } from "../../metrics";
import { usePref } from "../../prefs";
import { href } from "../../route";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";
import { useLatest } from "../performance/shared";
import { type MemorySlots, MemorySlotsView } from "./AdminDetails";
import { NEEDS_DRIVER, shortCpu, useBluetooth, useGpus, useHardware } from "./shared";

const loadDevices = () => getInventory("devices_detail");
const loadPower = () => getInventory("power");
const loadPowerDetail = () => getInventory("power_detail");
const loadPlatform = () => getInventory("platform_detail");

/** Where a part lives in the app. */
type Place = [page: string, tab: string];

export interface Part {
  key: string;
  /** "Inside", "Input", "Screens and sound", "Network", or "Connected". */
  group: string;
  kind: string;
  name: string;
  detail: string;
  status: [StatusVariant, string];
  place?: Place;
}

const GROUPS = ["Inside", "Input", "Screens and sound", "Network", "Connected"];
const OK: [StatusVariant, string] = ["ok", "Working"];

/** A touchpad, touchscreen, or pen from its name; anything else that points is a mouse. */
export function pointerKind(name: string): string {
  const n = name.toLowerCase();
  if (n.includes("touchpad") || n.includes("trackpoint")) return "Touchpad";
  if (n.includes("touchscreen") || n.includes("touch screen")) return "Touchscreen";
  if (n.includes("pen") || n.includes("stylus") || n.includes("wacom")) return "Pen";
  return "Mouse";
}

/** A network chip's kind from its interface name (`wlp45s0`, `enp44s0`, `wwan0`). */
function networkKind(name: string): string {
  if (name.startsWith("wl")) return "Wi-Fi";
  if (name.startsWith("ww")) return "Mobile data";
  if (name.startsWith("en") || name.startsWith("eth")) return "Ethernet";
  return "Network";
}

/** Every part the answers name, in the order of GROUPS. Each answer may still be missing. */
export function partsList(a: {
  hw: HardwareInfo;
  gpus?: { model?: string; driver?: string; card: string }[];
  fans: { subject?: string; value: number }[];
  power?: PowerInfo;
  devices?: DevicesDetail;
  powerDetail?: PowerDetail;
  platform?: PlatformDetail;
  bluetooth?: BluetoothInfo;
}): Part[] {
  const { hw } = a;
  const out: Part[] = [];
  const add = (p: Omit<Part, "key">) => out.push({ ...p, key: `${p.group}|${p.kind}|${p.name}|${out.length}` });

  // Inside the computer.
  add({ group: "Inside", kind: "Processor", name: shortCpu(hw.processor.model) || "Processor", detail: `${hw.processor.cores} cores · ${hw.processor.threads} threads`, status: OK, place: ["hardware", "processor"] });
  add({ group: "Inside", kind: "Memory", name: hw.memory_total ? bytes(hw.memory_total) : "Memory", detail: "Each slot is under Slots and sockets", status: OK, place: ["hardware", "memory"] });
  for (const g of a.gpus ?? []) {
    add({ group: "Inside", kind: "Graphics", name: g.model?.match(/\[(.+)\]/)?.[1] ?? g.model ?? g.card, detail: g.driver ? `Driver ${g.driver}` : "No driver", status: g.driver ? OK : ["warn", "No driver"], place: ["hardware", "graphics"] });
  }
  for (const d of hw.disks) {
    add({ group: "Inside", kind: d.removable || d.kind === "USB drive" ? "Removable drive" : "Drive", name: d.model ?? d.name, detail: `${d.kind} · ${bytes(d.size, 0)} · ${d.name}`, status: OK, place: ["hardware", "storage"] });
  }
  for (const b of a.power?.batteries ?? []) {
    add({ group: "Inside", kind: "Battery", name: [b.manufacturer, b.model].filter(Boolean).join(" ") || b.name, detail: [b.charge !== undefined && `${Math.round(b.charge)}% charged`, b.technology].filter(Boolean).join(" · "), status: OK, place: ["power", "battery"] });
  }
  a.fans.forEach((f, i) => add({ group: "Inside", kind: "Fan", name: `Fan ${i + 1}`, detail: [`${Math.round(f.value)} RPM`, f.subject].filter(Boolean).join(" · "), status: f.value > 0 ? OK : ["off", "Stopped"], place: ["hardware", "sensors"] }));
  if (a.devices?.fingerprint?.device) add({ group: "Inside", kind: "Fingerprint reader", name: a.devices.fingerprint.device, detail: a.devices.fingerprint.enrolled.length ? `${a.devices.fingerprint.enrolled.length} fingers saved` : "No fingers saved", status: OK, place: ["devices", "ports"] });
  for (const c of a.devices?.cameras ?? []) add({ group: "Inside", kind: "Camera", name: c.name, detail: c.driver ? `Driver ${c.driver}` : "", status: OK, place: ["devices", "ports"] });

  // Keyboards, the keyboard light, touchpads, and mice.
  for (const d of hw.devices.filter((d) => d.bus === "Input")) {
    if (d.class === "Keyboard") {
      const builtIn = d.name === "AT Translated Set 2 keyboard";
      add({ group: "Input", kind: "Keyboard", name: builtIn ? "Built-in keyboard" : d.name, detail: builtIn ? d.name : "", status: OK, place: ["devices", "ports"] });
    } else {
      add({ group: "Input", kind: pointerKind(d.name), name: d.name, detail: "", status: OK, place: ["devices", "ports"] });
    }
  }
  const light = a.powerDetail?.keyboard_backlight;
  if (light) add({ group: "Input", kind: "Keyboard light", name: "Keyboard backlight", detail: light.level ? `Level ${light.level} of ${light.max}` : `Off (levels 1 to ${light.max})`, status: light.level ? ["ok", "On"] : ["off", "Off"], place: ["power", "profile"] });

  // Screens and sound.
  for (const s of a.devices?.screens ?? []) {
    const builtIn = s.connector.startsWith("eDP") || s.connector.startsWith("LVDS");
    add({ group: "Screens and sound", kind: builtIn ? "Built-in screen" : "Screen", name: [s.maker, s.model].filter(Boolean).join(" ") || s.connector, detail: [s.native && `${s.native[0]} × ${s.native[1]}`, s.refresh_max && `up to ${s.refresh_max} Hz`, s.connector].filter(Boolean).join(" · "), status: OK, place: ["devices", "displays"] });
  }
  for (const s of a.devices?.sound ?? []) add({ group: "Screens and sound", kind: "Sound card", name: s.name, detail: s.codecs.join(", "), status: OK, place: ["devices", "audio"] });

  // Network chips.
  for (const n of a.devices?.network ?? []) add({ group: "Network", kind: networkKind(n.interface), name: n.interface, detail: [n.driver && `Driver ${n.driver}`, n.bus].filter(Boolean).join(" · "), status: n.driver ? OK : ["warn", "No driver"], place: ["network", "interfaces"] });
  if (a.bluetooth?.adapters) add({ group: "Network", kind: "Bluetooth", name: `${a.bluetooth.adapters} adapter${a.bluetooth.adapters === 1 ? "" : "s"}`, detail: a.bluetooth.powered ? "Turned on" : "Turned off", status: a.bluetooth.powered ? OK : ["off", "Off"], place: ["devices", "ports"] });

  // What is plugged in: USB devices, USB-C ports, Thunderbolt, and Bluetooth devices.
  for (const d of hw.devices.filter((d) => d.bus === "USB")) {
    const needs = d.class ? NEEDS_DRIVER.has(d.class) : true;
    add({ group: "Connected", kind: "USB device", name: d.name, detail: [d.vendor, d.drivers.join(", ")].filter(Boolean).join(" · "), status: d.drivers.length ? OK : needs ? ["warn", "No driver"] : ["off", "No driver needed"], place: ["devices", "ports"] });
  }
  for (const p of a.platform?.typec ?? []) add({ group: "Connected", kind: "USB-C port", name: p.name, detail: p.connected ? [p.partner_type, p.power_role && `power ${p.power_role}`].filter(Boolean).join(" · ") || "Something is plugged in" : "Nothing plugged in", status: p.connected ? ["ok", "In use"] : ["off", "Free"], place: ["devices", "ports"] });
  for (const t of a.platform?.thunderbolt ?? []) add({ group: "Connected", kind: "Thunderbolt device", name: [t.vendor, t.name].filter(Boolean).join(" ") || t.id, detail: "", status: t.authorized ? OK : ["warn", "Not allowed yet"], place: ["devices", "ports"] });
  for (const d of a.bluetooth?.devices ?? []) add({ group: "Connected", kind: "Bluetooth device", name: d.name, detail: [d.kind, d.battery !== undefined && `battery ${d.battery}%`].filter(Boolean).join(" · "), status: d.connected ? ["ok", "Connected"] : ["off", "Paired"], place: ["devices", "ports"] });
  return out;
}

const COLUMNS: Column<Part>[] = [
  { key: "group", label: "Where", text: (p) => p.group },
  { key: "kind", label: "Part", text: (p) => p.kind },
  { key: "name", label: "Name", text: (p) => p.name, render: (p) => (p.detail ? <span className="cell-two"><b>{p.name}</b><span>{p.detail}</span></span> : <b>{p.name}</b>), wrap: true },
  { key: "status", label: "Status", text: (p) => p.status[1], render: (p) => <StatusBadge variant={p.status[0]} label={p.status[1]} /> },
  { key: "open", label: "", text: () => "", render: (p) => (p.place ? <LinkButton to={href(...p.place)}>Details</LinkButton> : null) },
];

type Fields = Record<string, string | undefined>;
interface SlotsPorts {
  ports: Fields[];
  slots: Fields[];
}

/** Memory slots, expansion slots (PCIe, M.2), and sockets: one password for all three. */
function MakerSlots() {
  const helper = useHelperInstalled();
  const [access] = usePref<"basic" | "full">("access", "basic");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [data, setData] = useState<{ memory?: MemorySlots; board?: SlotsPorts } | null>(null);
  if (helper === false) return <div className="empty muted">Only the admin can read the memory slots, the expansion slots, and the sockets. It needs Full access, which needs the DeeCure admin helper (Settings › Access).</div>;
  if (helper && access !== "full")
    return (
      <div className="empty muted">
        Only the admin can read the memory slots, the expansion slots, and the sockets. Turn on Full access in <a href={href("settings", "access")}>Settings › Access</a> to read them with your password.
      </div>
    );
  if (!data) {
    const read = async () => {
      setBusy(true);
      setError(null);
      try {
        // The password is asked once: polkit keeps the approval for a few minutes.
        const memory = JSON.parse(await helperRead("memory_slots")) as MemorySlots;
        const board = JSON.parse(await helperRead("slots_ports")) as SlotsPorts;
        setData({ memory, board });
      } catch (e) {
        setError(e instanceof SyntaxError ? "The helper gave no readable answer." : toAppError(e).message);
      } finally {
        setBusy(false);
      }
    };
    return (
      <div className="empty">
        <p className="muted">{error ?? "The maker's table lists every memory slot, expansion slot, and socket, used or free. Only the admin can read it."}</p>
        <Button small disabled={busy || helper === null} onClick={() => void read()}>
          {busy ? "Waiting for approval…" : "Read slots and sockets (asks for your password)"}
        </Button>
      </div>
    );
  }
  const slots = data.board?.slots ?? [];
  const ports = data.board?.ports ?? [];
  return (
    <>
      <div className="kn-head">Memory slots</div>
      {data.memory ? <MemorySlotsView data={data.memory} /> : null}
      <div className="kn-head" style={{ marginTop: 12 }}>Expansion slots (PCIe and M.2)</div>
      <DataTable
        rows={slots}
        columns={[
          { key: "name", label: "Slot", text: (s) => s.Designation ?? "-" },
          { key: "type", label: "Type", text: (s) => s.Type ?? "-", wrap: true },
          { key: "use", label: "In use", text: (s) => s["Current Usage"] ?? "-", render: (s) => <StatusBadge variant={s["Current Usage"] === "In Use" ? "ok" : "off"} label={s["Current Usage"] === "In Use" ? "In use" : s["Current Usage"] === "Available" ? "Free" : (s["Current Usage"] ?? "Unknown")} /> },
          { key: "bus", label: "Bus address", text: (s) => s["Bus Address"] ?? "-" },
        ]}
        rowKey={(s) => `${s.Designation}|${s["Bus Address"]}`}
        empty="The maker did not list the expansion slots for this computer."
      />
      <div className="kn-head" style={{ marginTop: 12 }}>Sockets</div>
      <DataTable
        rows={ports}
        columns={[
          { key: "name", label: "Socket", text: (p) => known(p["External Reference Designator"]) ?? known(p["Internal Reference Designator"]) ?? "-" },
          { key: "type", label: "Type", text: (p) => known(p["Port Type"]) ?? "-" },
          { key: "conn", label: "Connector", text: (p) => known(p["External Connector Type"]) ?? known(p["Internal Connector Type"]) ?? "-" },
          { key: "side", label: "Outside or inside", text: (p) => (known(p["External Reference Designator"]) ? "Outside" : "Inside the case") },
        ]}
        rowKey={(p) => `${p["Internal Reference Designator"]}|${p["External Reference Designator"]}`}
        empty="The maker did not list the sockets for this computer."
      />
      <p className="muted" style={{ marginTop: 8 }}>The maker writes this list. On some computers, laptops most of all, it is missing or not complete.</p>
    </>
  );
}

const known = (v?: string) => (v && !/^(none|other|not specified|unknown)$/i.test(v) ? v : undefined);

export function HardwareAllParts() {
  const { data: hw, error } = useHardware();
  const gpus = useGpus();
  const bluetooth = useBluetooth();
  const latest = useLatest();
  const power = useLive(loadPower, 60_000).data;
  const devices = useLive(loadDevices, 60_000).data;
  const powerDetail = useLive(loadPowerDetail, 60_000).data;
  const platform = useLive(loadPlatform, 60_000).data;
  if (!hw) {
    return (
      <Panel title="Every part" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const parts = partsList({ hw, gpus, fans: pickAll(latest, "fan"), power, devices, powerDetail, platform, bluetooth });
  const sata = hw.sata_ports ?? [];
  const needDriver = parts.filter((p) => p.status[1] === "No driver").length;
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Parts found", value: String(parts.length) },
          { label: "Need a driver", value: String(needDriver), status: needDriver ? "warn" : "ok" },
          { label: "Connected now", value: String(parts.filter((p) => p.group === "Connected" && p.status[0] === "ok").length), subtext: "USB, USB-C, Thunderbolt, Bluetooth" },
          { label: "SATA ports", value: sata.length ? `${sata.filter((p) => p.disk).length} of ${sata.length}` : "None", subtext: sata.length ? "in use" : "this computer has no SATA controller" },
        ]}
      />
      <Panel title="Every part" info="Every part DeeCure can see, inside the computer and plugged in. Details opens the page with more about it." source="/sys · /proc · PipeWire · BlueZ · UPower" span={12} flush>
        <DataTable rows={parts} columns={COLUMNS} rowKey={(p) => p.key} search={{ placeholder: "Search parts" }} filter={{ column: "group", options: GROUPS }} preview={false} pageSize={60} empty="No parts found" />
      </Panel>
      <Panel title="Slots and sockets" info="Every memory slot, expansion slot (PCIe and M.2), and socket, used or free. SATA ports need no password; the rest comes from the maker's table." source="/sys/class/ata_port · dmidecode -t 8 -t 9 -t 16 -t 17" span={12}>
        <div className="kn-head">SATA ports (for a hard disk or a SATA SSD)</div>
        {sata.length ? (
          <DataTable
            rows={sata}
            columns={[
              { key: "port", label: "Port", text: (p) => `SATA ${p.port}` },
              { key: "disk", label: "Drive", text: (p) => p.disk ?? "-", render: (p) => (p.disk ? <code>{p.disk}</code> : <span className="muted">Nothing connected</span>) },
              { key: "speed", label: "Link speed", text: (p) => p.speed ?? "-" },
            ]}
            rowKey={(p) => String(p.port)}
          />
        ) : (
          <div className="empty muted">{hw.sata_ports ? "This computer has no SATA ports. Its drives use NVMe (M.2) or USB." : "Restart the agent to read the SATA ports."}</div>
        )}
        {sata.some((p) => !p.disk) && <p className="muted">A controller can have more ports than the case has sockets, so an empty port is not always a free slot.</p>}
        <div style={{ marginTop: 12 }}>
          <MakerSlots />
        </div>
      </Panel>
    </>
  );
}
