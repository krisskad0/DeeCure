// SPDX-License-Identifier: Apache-2.0
// Expert sections for power (docs/01-product/expert-coverage.md, sections 7a and 7b): the battery
// in depth, chargers and USB-C Power Delivery, battery drain while suspended, what may wake the
// computer and what last did, and devices that could save power but do not.

import { type PciDevice, type PdOffer, type PowerDetail, type SleepEvent, getInventory, getMetrics } from "../../api";
import { DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadPower = () => getInventory("power_detail");
const loadPci = () => getInventory("pci_detail");
const loadSleep = () => getInventory("sleep");

/** ACPI and kernel wake-source names, in plain words. */
const WAKE_NAMES: [RegExp, string][] = [
  [/^PNP0C0D/, "Lid"],
  [/^(PNP0C0C|LNXPWRBN)/, "Power button"],
  [/^PNP0C0E/, "Sleep button"],
  [/^PNP0C0A/, "Battery"],
  [/^(AC|ACPI0003)/, "Power adapter"],
  [/^PNP0C09/, "Embedded controller"],
  [/^LNXVIDEO/, "Display"],
  [/^PNP0C14/, "Firmware events (WMI)"],
  [/^INT33D5/, "Intel HID events (keys and buttons)"],
  [/^serio/, "Built-in keyboard"],
  [/^i2c-/, "Touchpad or touch screen"],
  [/^ucsi-source-psy/, "USB-C charger"],
  [/^rtc/, "Wake-up alarm (clock)"],
];

/** A name for people: the USB product, the PCI device, or the ACPI or kernel name. */
export function wakeLabel(w: PowerDetail["wake_sources"][number], pci: PciDevice[] | undefined): string {
  if (w.product) return w.product;
  const plain = WAKE_NAMES.find(([re]) => re.test(w.name))?.[1];
  if (plain) return plain;
  const device = w.pci_address ? pci?.find((p) => p.address === w.pci_address) : undefined;
  if (device) return device.model ?? device.class ?? device.address;
  return w.name;
}

function offerText(o: PdOffer): string {
  const v = o.millivolts !== undefined ? `${(o.millivolts / 1000).toFixed(o.millivolts % 1000 ? 1 : 0)} V` : "";
  const range = o.max_millivolts !== undefined ? ` to ${(o.max_millivolts / 1000).toFixed(1)} V` : "";
  const a = o.milliamps !== undefined ? ` · ${(o.milliamps / 1000).toFixed(2).replace(/\.?0+$/, "")} A` : "";
  const w = o.milliwatts !== undefined ? ` · ${(o.milliwatts / 1000).toFixed(0)} W` : "";
  return `${v}${range}${a}${w}${o.kind === "programmable_supply" ? " (PPS)" : ""}`;
}

const maxWatts = (offers: PdOffer[]) =>
  Math.max(0, ...offers.map((o) => o.milliwatts ?? ((o.max_millivolts ?? o.millivolts ?? 0) * (o.milliamps ?? 0)) / 1000)) / 1000;

// ---------- Power › Battery ----------

export function BatteryExpertSections() {
  const { data: d, error } = useLive(loadPower, 15_000);
  if (!d) {
    return (
      <Panel title="Battery details" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  return (
    <>
      {d.batteries.map((b) => (
        <Panel key={b.name} title={`Battery details · ${b.name}`} info="Read from the battery's own controller. Voltage close to the design minimum under load warns of a worn battery." source="/sys/class/power_supply" span={6} flush>
          <KeyValueList
            rows={[
              ["Now", `${b.status ?? "-"}${b.watts !== undefined ? ` · ${b.watts.toFixed(1)} W` : ""}`],
              ["Voltage", b.volts !== undefined ? `${b.volts.toFixed(2)} V${b.volts_min_design ? ` (design minimum ${b.volts_min_design.toFixed(2)} V)` : ""}` : "-"],
              ["Current", b.amps !== undefined ? `${b.amps.toFixed(2)} A` : "-"],
              ["Temperature", b.temp_c !== undefined ? `${b.temp_c.toFixed(1)} °C` : "Not reported"],
              ["The battery says", b.health ?? "Not reported"],
              ["Made", b.made ?? "Not reported"],
              ["Charge limits", b.charge_stop !== undefined ? `starts below ${b.charge_start ?? "-"}%, stops at ${b.charge_stop}%` : "This battery has no charge limits"],
              ["Charge mode", b.charge_mode ? `${b.charge_mode.current ?? "-"} (offered: ${b.charge_mode.offered.join(", ")})` : "Not offered"],
              ...(b.charge_behaviour ? [["Charging behaviour", `${b.charge_behaviour.current ?? "-"} (offered: ${b.charge_behaviour.offered.join(", ")})`] as [string, string]] : []),
              ["Serial number", b.serial ?? "-"],
            ]}
          />
        </Panel>
      ))}
      <Panel title="Chargers" info="A USB-C charger tells the laptop what it can give (Power Delivery). A charger weaker than what the laptop draws charges slowly or not at all." source="/sys/class/power_supply · /sys/class/usb_power_delivery" span={6} flush>
        <KeyValueList
          rows={[
            ...d.chargers.map((c): [string, React.ReactNode] => [
              c.kind === "Mains" ? "Power adapter" : `USB-C (${c.name})`,
              c.online ? `Connected${c.usb_type?.current ? ` · ${c.usb_type.current}` : ""}${c.max_watts ? ` · up to ${c.max_watts.toFixed(0)} W` : ""}` : "Not connected",
            ]),
            ...d.power_delivery.map((p): [string, React.ReactNode] => [
              `${p.name}: charger offers`,
              p.offered.length ? `${p.offered.map(offerText).join("; ")} (most ${maxWatts(p.offered).toFixed(0)} W)` : "Nothing connected",
            ]),
            ...d.power_delivery.map((p): [string, React.ReactNode] => [`${p.name}: laptop accepts`, p.accepted.length ? p.accepted.map(offerText).join("; ") : "-"]),
          ]}
        />
      </Panel>
      <SuspendDrain battery={d.batteries[0]?.name} />
    </>
  );
}

/** Battery drain per hour during each recent suspend, from the charge just before and after. */
function SuspendDrain({ battery }: { battery?: string }) {
  const sleep = useLive(loadSleep, 300_000).data;
  const events = (sleep ?? []).filter((e: SleepEvent) => !e.failed && (e.duration_ms ?? 0) >= 10 * 60_000).slice(0, 10);
  const key = events.map((e) => e.at).join(",");
  const drains = useLive(
    () =>
      Promise.all(
        events.map(async (e) => {
          const end = e.at + (e.duration_ms ?? 0);
          const [before, after] = await Promise.all([
            getMetrics("battery.charge", e.at - 5 * 60_000, e.at, { subject: battery, maxPoints: 2000 }),
            getMetrics("battery.charge", end, end + 5 * 60_000, { subject: battery, maxPoints: 2000 }),
          ]);
          const a = before.at(-1)?.value;
          const b = after[0]?.value;
          const hours = (e.duration_ms ?? 0) / 3_600_000;
          return { at: e.at, hours, from: a, to: b, perHour: a !== undefined && b !== undefined && b <= a ? (a - b) / hours : undefined };
        }),
      ),
    300_000,
    key,
  ).data;
  if (!battery) return null;
  return (
    <Panel title="Battery used while suspended" subtitle="suspends of 10 minutes or more" info="From the charge level just before and just after each suspend. A sleeping laptop should lose about 1% per hour or less." source="journal: systemd-sleep · battery.charge history" span={12} flush>
      <DataTable
        rows={drains ?? []}
        columns={[
          { key: "at", label: "When", text: (r) => new Date(r.at).toLocaleString() },
          { key: "hours", label: "Asleep", text: (r) => `${r.hours.toFixed(1)} h`, numeric: true },
          { key: "levels", label: "Charge", text: (r) => (r.from !== undefined && r.to !== undefined ? `${r.from.toFixed(0)}% to ${r.to.toFixed(0)}%` : "No reading"), numeric: true },
          {
            key: "rate",
            label: "Per hour",
            text: (r) => (r.perHour === undefined ? (r.to !== undefined && r.from !== undefined ? "Charging" : "-") : `${r.perHour.toFixed(1)}%`),
            render: (r) => (r.perHour === undefined ? (r.to !== undefined && r.from !== undefined ? "Charging" : "-") : r.perHour > 2 ? <StatusBadge variant="warn" label={`${r.perHour.toFixed(1)}%`} /> : `${r.perHour.toFixed(1)}%`),
            numeric: true,
          },
        ]}
        rowKey={(r) => String(r.at)}
        empty={sleep ? "No suspend of 10 minutes or more in the last 30 days" : "Loading…"}
      />
    </Panel>
  );
}

// ---------- Power › Sleep ----------

export function WakeExpertSections() {
  const { data: d, error } = useLive(loadPower, 30_000);
  const pci = useLive(loadPci, 60_000).data;
  if (!d) {
    return (
      <Panel title="Wake-ups from suspend" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const allowed = d.wake_sources.filter((w) => w.may_wake);
  const savers = [
    ...(pci ?? []).filter((p) => p.power_control === "on").map((p) => ({ key: p.address, name: p.model ?? p.address, bus: "PCI", state: p.power_state })),
    ...d.usb.filter((u) => u.control === "on" && !u.keep_awake).map((u) => ({ key: u.port, name: u.product ?? u.port, bus: "USB", state: u.state })),
  ];
  const keptAwake = d.usb.filter((u) => u.keep_awake && u.control === "on").length;
  return (
    <>
      <Panel title="Suspend" source="/sys/power" span={6} flush>
        <KeyValueList
          rows={[
            ["Suspend type", d.suspend?.current ? `${d.suspend.current === "s2idle" ? "Light (s2idle)" : d.suspend.current === "deep" ? "Deep (S3)" : d.suspend.current}` : "-"],
            ["Types this computer offers", d.suspend ? d.suspend.offered.join(", ") : "-"],
            ["Last woken by", d.last_wake_irq ? `interrupt ${d.last_wake_irq}${d.last_wake_device ? ` (${d.last_wake_device.split(/\s+/).at(-1)})` : ""}` : "Not reported since start"],
          ]}
        />
      </Panel>
      <Panel
        title="Devices that could save power but do not"
        subtitle={`${savers.length} found`}
        info="These devices support power saving, but it is turned off, so they stay awake and use power. Keyboards, mice, and sound devices are left out on purpose."
        source="/sys/bus/pci · /sys/bus/usb (power/control)"
        span={6}
        flush
      >
        <DataTable
          rows={savers}
          columns={[
            { key: "name", label: "Device", text: (s) => s.name, wrap: true },
            { key: "bus", label: "Bus", text: (s) => s.bus },
            { key: "state", label: "Now", text: (s) => (s.state === "suspended" ? "Asleep" : s.state === "active" ? "Awake" : (s.state ?? "-")) },
          ]}
          rowKey={(s) => s.key}
          empty="Every device that supports power saving uses it"
        />
        {keptAwake > 0 && <p className="muted panel-pad">{keptAwake} input or sound device{keptAwake === 1 ? " stays" : "s stay"} awake on purpose.</p>}
      </Panel>
      <Panel
        title="May wake the computer"
        subtitle={`${allowed.length} sources`}
        info="Devices allowed to wake the computer from suspend. Signalled counts every event since start; stopped a suspend counts the times it kept the computer from going to sleep or woke it at once."
        source="/sys/class/wakeup · power/wakeup"
        span={12}
        flush
      >
        <DataTable
          rows={allowed}
          columns={[
            { key: "label", label: "Device", text: (w) => wakeLabel(w, pci), wrap: true },
            { key: "events", label: "Signalled", text: (w) => w.events.toLocaleString(), numeric: true },
            { key: "wakeups", label: "Stopped a suspend", text: (w) => w.wakeups.toLocaleString(), numeric: true },
            { key: "name", label: "Kernel name", text: (w) => w.name, expert: true },
          ]}
          rowKey={(w) => w.name}
          pageSize={12}
          empty="No device may wake the computer"
        />
      </Panel>
    </>
  );
}
