// SPDX-License-Identifier: Apache-2.0
// Expert sections for the processor (docs/01-product/expert-coverage.md, sections 1 and 7b):
// each core's speed policy, heat slowdowns, interrupts, power limits, caches, microcode, CPU
// flaws, and sleep states. The data is read only while one of these sections is open. Counters
// are totals since start; two readings give the "now" rates.

import { useEffect, useRef, useState } from "react";
import { type CpuDetail, getInventory } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { bytes } from "../../metrics";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadDetail = () => getInventory("cpu_detail");

/** The newest reading, and the one before it (for rates). */
export function useCpuDetail() {
  const { data, error } = useLive(loadDetail, 5000);
  const last = useRef<CpuDetail | undefined>(undefined);
  const [before, setBefore] = useState<CpuDetail | undefined>(undefined);
  useEffect(() => {
    if (data && data !== last.current) {
      setBefore(last.current);
      last.current = data;
    }
  }, [data]);
  return { data, before: before && data && data.at > before.at ? before : undefined, error };
}

/** Per second between two readings, or undefined. */
function perSecond(now: number | undefined, then: number | undefined, seconds: number): number | undefined {
  if (now === undefined || then === undefined || seconds <= 0 || now < then) return undefined;
  return (now - then) / seconds;
}

const mhz = (khz?: number) => (khz === undefined ? "-" : `${Math.round(khz / 1000).toLocaleString()} MHz`);
const rateText = (r?: number) => (r === undefined ? "-" : r >= 100 ? Math.round(r).toLocaleString() : r.toFixed(1));

// ---------- Performance › CPU ----------

interface CoreRow {
  cpu: number;
  now?: number;
  min?: number;
  max?: number;
  hwMax?: number;
  policy: string;
  preferred: boolean;
  throttles?: number;
  throttlesNow?: number;
}

const CORE_COLUMNS: Column<CoreRow>[] = [
  { key: "cpu", label: "CPU", text: (r) => String(r.cpu), numeric: true },
  { key: "now", label: "Speed now", text: (r) => mhz(r.now), numeric: true },
  { key: "range", label: "Allowed", text: (r) => (r.min === undefined || r.max === undefined ? "-" : `${mhz(r.min)} to ${mhz(r.max)}`) },
  { key: "hw", label: "Hardware max", text: (r) => mhz(r.hwMax), numeric: true },
  { key: "policy", label: "Speed policy", text: (r) => r.policy },
  { key: "preferred", label: "Preferred", text: (r) => (r.preferred ? "Yes" : "-") },
  { key: "throttles", label: "Heat slowdowns", text: (r) => (r.throttles === undefined ? "-" : r.throttles.toLocaleString()), numeric: true },
  { key: "throttlesNow", label: "Per second now", text: (r) => rateText(r.throttlesNow), numeric: true },
];

/** The device at the end of an interrupt line, after its trigger type (`0-edge iwlwifi`). */
export function irqDevice(text: string): string {
  const parts = text.split(/\s+/);
  const trigger = parts.findIndex((p) => /-(edge|fasteoi|level)$/.test(p));
  return trigger >= 0 && trigger < parts.length - 1 ? parts.slice(trigger + 1).join(" ") : text;
}

/** `waitingForDisk`: names of processes in state D, from the process list the CPU tab reads. */
export function CpuExpertSections({ waitingForDisk }: { waitingForDisk?: string[] }) {
  const { data: d, before, error } = useCpuDetail();
  if (!d) {
    return (
      <Panel title="Processor details" subtitle="Expert" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const seconds = before ? (d.at - before.at) / 1000 : 0;
  const ranks = d.cores.map((c) => c.highest_perf).filter((p): p is number => p !== undefined);
  const top = ranks.length && Math.max(...ranks) !== Math.min(...ranks) ? Math.max(...ranks) : undefined;
  const rows: CoreRow[] = d.cores.map((c) => ({
    cpu: c.cpu,
    now: c.now_khz,
    min: c.min_khz,
    max: c.max_khz,
    hwMax: c.hardware_max_khz,
    policy: [c.governor, c.energy_preference?.replaceAll("_", " ")].filter(Boolean).join(" · ") || "-",
    preferred: top !== undefined && c.highest_perf === top,
    throttles: c.throttle_count,
    throttlesNow: perSecond(c.throttle_count, before?.cores.find((b) => b.cpu === c.cpu)?.throttle_count, seconds),
  }));
  const chipNow = perSecond(d.package_throttle_count, before?.package_throttle_count, seconds);
  const irqs = d.interrupts
    .map((i) => ({ ...i, rate: perSecond(i.count, before?.interrupts.find((b) => b.irq === i.irq)?.count, seconds) }))
    .filter((i) => (before ? (i.rate ?? 0) > 0 : i.count > 0))
    .sort((a, b) => (before ? (b.rate ?? 0) - (a.rate ?? 0) : b.count - a.count))
    .slice(0, 12);
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Speed driver", value: d.driver ?? "-", subtext: d.driver_mode ? `mode: ${d.driver_mode}` : "" },
          { label: "Turbo", value: d.turbo === undefined ? "N/A" : d.turbo ? "On" : "Off", subtext: d.turbo === undefined ? "No turbo switch on this CPU" : "" },
          {
            label: "Chip heat slowdowns",
            value: d.package_throttle_count === undefined ? "N/A" : d.package_throttle_count.toLocaleString(),
            subtext: d.package_throttle_count === undefined ? "Not reported by this CPU" : chipNow !== undefined ? `since start · ${rateText(chipNow)} per second now` : "since start",
            status: chipNow ? "warn" : undefined,
          },
          { label: "Longest slowdown", value: d.package_throttle_max_ms === undefined ? "N/A" : String(d.package_throttle_max_ms), unit: d.package_throttle_max_ms === undefined ? undefined : "ms" },
          {
            label: "Waiting for the disk",
            value: waitingForDisk === undefined ? "-" : String(waitingForDisk.length),
            subtext: waitingForDisk?.length ? [...new Set(waitingForDisk)].slice(0, 3).join(", ") : "processes; they add to the load without using the CPU",
            status: waitingForDisk && waitingForDisk.length > 2 ? "warn" : undefined,
          },
        ]}
      />
      <Panel
        title="Speed policy per core"
        info="Speed policy and energy preference are set by the power mode. Preferred cores are the ones the firmware rates fastest; heavy work goes there first. Heat slowdowns count the times heat forced the core to slow down."
        source="/sys/devices/system/cpu/cpu*/cpufreq · thermal_throttle · acpi_cppc"
        span={12}
        flush
      >
        <DataTable rows={rows} columns={CORE_COLUMNS} rowKey={(r) => String(r.cpu)} pageSize={16} />
      </Panel>
      <Panel
        title="Interrupts"
        subtitle={before ? "per second now, busiest first" : "since start, busiest first (rates after the next reading)"}
        info="Each interrupt wakes a CPU. A device with many interrupts per second keeps the CPU from resting."
        source="/proc/interrupts"
        span={12}
        flush
      >
        <DataTable
          rows={irqs}
          columns={[
            { key: "irq", label: "Line", text: (i) => i.irq },
            { key: "device", label: "Device", text: (i) => irqDevice(i.device), wrap: true },
            { key: "rate", label: "Per second", text: (i) => rateText(i.rate), numeric: true },
            { key: "count", label: "Since start", text: (i) => i.count.toLocaleString(), numeric: true },
          ]}
          rowKey={(i) => i.irq}
          empty="No interrupts reported"
        />
      </Panel>
    </>
  );
}

// ---------- Hardware › Processor ----------

const LIMIT_NAMES: Record<string, string> = { long_term: "Long-term (sustained)", short_term: "Short-term (boost)", peak_power: "Peak" };

function windowText(s?: number): string {
  if (s === undefined) return "-";
  return s >= 1 ? `${s.toFixed(s >= 10 ? 0 : 1)} s` : `${(s * 1000).toFixed(s * 1000 >= 10 ? 0 : 1)} ms`;
}

const FLAW_BADGE = {
  not_affected: ["ok", "Not affected"],
  mitigated: ["info", "Protected"],
  vulnerable: ["crit", "Open"],
  unknown: ["off", "Unknown"],
} as const;

export function ProcessorExpertSections() {
  const { data: d, error } = useCpuDetail();
  if (!d) {
    return (
      <Panel title="Processor details" subtitle="Expert" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const caches = Object.values(
    d.caches.reduce<Record<string, { level: number; kind: string; bytes: number; copies: number; sharedBy: Set<number> }>>((all, c) => {
      const key = `${c.level}|${c.kind}|${c.bytes}`;
      const row = (all[key] ??= { level: c.level, kind: c.kind, bytes: c.bytes, copies: 0, sharedBy: new Set() });
      row.copies += 1;
      row.sharedBy.add(c.copies);
      return all;
    }, {}),
  );
  const loaded = d.microcode_loaded ? parseInt(d.microcode_loaded, 16) : undefined;
  const onDisk = d.microcode_on_disk ? parseInt(d.microcode_on_disk, 16) : undefined;
  const open = d.flaws.filter((f) => f.state === "vulnerable").length;
  return (
    <>
      <Panel
        title="Power limits"
        info="Set by the firmware. The CPU may use the short-term limit for a short time, then drops to the long-term limit. When two zones set the same kind of limit, the lower one applies."
        source="/sys/class/powercap"
        span={6}
        flush
      >
        <DataTable
          rows={d.power_limits}
          columns={[
            { key: "zone", label: "Zone", text: (l) => l.zone },
            { key: "name", label: "Limit", text: (l) => LIMIT_NAMES[l.name] ?? l.name },
            { key: "watts", label: "Watts", text: (l) => (l.watts ? `${l.watts.toFixed(l.watts >= 10 ? 0 : 1)} W` : "-"), numeric: true },
            { key: "window", label: "Averaged over", text: (l) => windowText(l.window_s), numeric: true },
            { key: "on", label: "On", text: (l) => (l.enabled === undefined ? "-" : l.enabled ? "Yes" : "No") },
          ]}
          rowKey={(l) => `${l.zone}|${l.name}`}
          empty="This computer does not report power limits"
        />
      </Panel>
      <Panel title="Chip revision and microcode" source="/proc/cpuinfo · /lib/firmware/intel-ucode" span={6} flush>
        <KeyValueList
          rows={[
            ["Maker", d.vendor ?? "-"],
            ["Family · model · stepping", d.family === undefined ? "-" : `${d.family} · ${d.model ?? "-"}${d.model !== undefined ? ` (0x${d.model.toString(16)})` : ""} · ${d.stepping ?? "-"}`],
            ["Microcode running", d.microcode_loaded ? <code key="l">{d.microcode_loaded}</code> : "-"],
            [
              "Microcode installed",
              onDisk === undefined ? (
                d.vendor === "GenuineIntel" ? "No microcode file for this CPU" : "Not checked for this maker"
              ) : loaded !== undefined && onDisk > loaded ? (
                <span key="d"><code>{d.microcode_on_disk}</code> · newer: restart to load it</span>
              ) : (
                <span key="d"><code>{d.microcode_on_disk}</code> · the running one is the newest</span>
              ),
            ],
          ]}
        />
      </Panel>
      <Panel title="Caches" source="/sys/devices/system/cpu/cpu*/cache" span={6} flush>
        <DataTable
          rows={caches}
          columns={[
            { key: "cache", label: "Cache", text: (c) => `L${c.level} ${c.kind.toLowerCase()}` },
            { key: "size", label: "Size", text: (c) => bytes(c.bytes, 0), numeric: true },
            { key: "copies", label: "Copies", text: (c) => String(c.copies), numeric: true },
            { key: "shared", label: "Each shared by", text: (c) => [...c.sharedBy].sort((a, b) => a - b).map((n) => (n === 1 ? "1 CPU" : `${n} CPUs`)).join(" or ") },
          ]}
          rowKey={(c) => `${c.level}|${c.kind}|${c.bytes}`}
          empty="This computer does not report its caches"
        />
      </Panel>
      <Panel title="CPU security flaws" subtitle={`${d.flaws.length} checked · ${open} open`} info="Known flaws in the processor design, as the kernel sees them on this computer. Protected means the kernel or microcode works around it." source="/sys/devices/system/cpu/vulnerabilities" span={6} flush>
        <DataTable
          rows={d.flaws}
          columns={[
            { key: "name", label: "Flaw", text: (f) => f.name.replaceAll("_", " ") },
            { key: "state", label: "State", text: (f) => FLAW_BADGE[f.state][1], render: (f) => <StatusBadge variant={FLAW_BADGE[f.state][0]} label={FLAW_BADGE[f.state][1]} /> },
            { key: "detail", label: "Kernel says", text: (f) => f.detail, wrap: true, expert: true },
          ]}
          rowKey={(f) => f.name}
          pageSize={20}
          empty="The kernel does not report CPU flaws"
        />
      </Panel>
    </>
  );
}

// ---------- Power › Sleep ----------

export function CpuSleepStates() {
  const { data: d, before, error } = useCpuDetail();
  if (!d) {
    return (
      <Panel title="CPU sleep states" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const sum = (x: CpuDetail | undefined, name: string) =>
    x?.cores.reduce((n, c) => n + (c.idle.find((s) => s.name === name)?.time_us ?? 0), 0) ?? 0;
  const names = [...new Set(d.cores.flatMap((c) => c.idle.map((s) => s.name)))];
  const info = (name: string) => d.cores.flatMap((c) => c.idle).find((s) => s.name === name);
  const totalNow = names.reduce((n, s) => n + sum(d, s), 0);
  const deltas = names.map((s) => Math.max(0, sum(d, s) - sum(before, s)));
  const totalDelta = deltas.reduce((a, b) => a + b, 0);
  const rows = names.map((name, i) => ({
    name,
    latency: info(name)?.latency_us,
    disabled: d.cores.some((c) => c.idle.find((s) => s.name === name)?.disabled),
    since: totalNow ? (sum(d, name) / totalNow) * 100 : undefined,
    now: before && totalDelta ? (deltas[i] / totalDelta) * 100 : undefined,
  }));
  const pct = (v?: number) => (v === undefined ? "-" : `${v.toFixed(1)}%`);
  return (
    <Panel
      title="CPU sleep states"
      subtitle="share of the time the cores rest"
      info="Deeper states save more power but take longer to wake up. Much time in the deepest state means the CPU rests well."
      source="/sys/devices/system/cpu/cpu*/cpuidle"
      span={12}
      flush
    >
      <DataTable
        rows={rows}
        columns={[
          { key: "name", label: "State", text: (r) => r.name },
          { key: "latency", label: "Wake-up time", text: (r) => (r.latency === undefined ? "-" : `${r.latency} µs`), numeric: true },
          { key: "now", label: "Now", text: (r) => pct(r.now), numeric: true },
          { key: "since", label: "Since start", text: (r) => pct(r.since), numeric: true },
          { key: "off", label: "Turned off", text: (r) => (r.disabled ? "Yes" : "-") },
        ]}
        rowKey={(r) => r.name}
        empty="This computer does not report CPU sleep states"
      />
    </Panel>
  );
}
