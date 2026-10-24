// SPDX-License-Identifier: Apache-2.0
// Expert power controls (docs/01-product/expert-coverage.md, section 7c). Every control opens the
// fix preview: it shows the exact change and the command, asks for the password when the change
// needs the admin helper, and records how to undo it. Controls are shown only when the hardware
// offers them.

import { useState } from "react";
import { type CpuDetail, type KeptSetting, type NetDeep, type PciDevice, type PowerDetail, getInventory } from "../../api";
import { Button } from "../../components/Button";
import { FieldRow } from "../../components/FormControls";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { useFix } from "../../shell/FixDrawer";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadPower = () => getInventory("power_detail");
const loadCpu = () => getInventory("cpu_detail");
const loadNet = () => getInventory("net_deep");

/** Charge limit presets: the stop limit, and a start limit 5 points below it. */
export const PRESETS = [
  { name: "Full", stop: 100 },
  { name: "Balanced", stop: 90 },
  { name: "Long life", stop: 80 },
] as const;

/** The subject for a charge limit change: `BAT0:75:80`, or `BAT0:-:80` without a start limit. */
export function limitsSubject(battery: string, hasStart: boolean, stop: number, start?: number): string {
  return `${battery}:${hasStart ? (start ?? Math.max(0, stop - 5)) : "-"}:${stop}`;
}

/** Radios of one kind together: on unless every one is blocked in software. */
export function radioKinds(radios: PowerDetail["radios"]) {
  const kinds = new Map<string, { kind: string; on: boolean; hard: boolean }>();
  for (const r of radios) {
    const k = kinds.get(r.kind) ?? { kind: r.kind, on: false, hard: false };
    k.on ||= !r.soft_blocked && !r.hard_blocked;
    k.hard ||= r.hard_blocked;
    kinds.set(r.kind, k);
  }
  return [...kinds.values()].filter((k) => ["wlan", "bluetooth", "wwan"].includes(k.kind));
}

const RADIO_NAMES: Record<string, string> = { wlan: "Wi-Fi", bluetooth: "Bluetooth", wwan: "Mobile broadband" };

const ASPM_NAMES: Record<string, string> = { default: "Firmware default", performance: "Off (performance)", powersave: "Save power", powersupersave: "Save most power" };
const SUSPEND_NAMES: Record<string, string> = { s2idle: "s2idle (modern standby)", shallow: "shallow (standby)", deep: "deep (suspend to RAM)" };

/** Battery CPU limits offered, in percent of the top speed. */
export const CPU_LIMITS = [80, 60, 40] as const;

/** A kept setting in words, with the device's product name when the agent knows it. */
export function describeKept(k: KeptSetting, p: Pick<PowerDetail, "usb" | "wake_sources">): string {
  const c = k.call;
  const device = c.device ?? "";
  const name =
    p.usb.find((u) => u.sys_path === device)?.product ??
    p.wake_sources.find((w) => w.device === device)?.product ??
    (device.split("/").pop() || device);
  switch (c.op) {
    case "set_charge_limits":
      return `Charge limits · ${c.battery}: ${c.start != null ? `start below ${c.start}%, ` : ""}stop at ${c.stop}%`;
    case "set_charge_mode":
      return `Charge mode · ${c.battery}: ${c.mode}`;
    case "set_turbo":
      return `CPU turbo ${c.on ? "on" : "off"}`;
    case "set_device_wakeup":
      return `${name}: ${c.enabled ? "may wake the computer" : "does not wake the computer"}`;
    case "set_device_power_saving":
      return `${name}: power saving ${c.on ? "on" : "off"}`;
    case "set_battery_cpu_limit":
      return `Highest CPU speed on battery: ${c.percent}%`;
    case "set_aspm_policy":
      return `PCIe power saving: ${ASPM_NAMES[c.policy ?? ""] ?? c.policy}`;
    case "set_suspend_mode":
      return `Suspend type: ${SUSPEND_NAMES[c.mode ?? ""] ?? c.mode}`;
    default:
      return k.setting;
  }
}

/**
 * A setting that is tried for this session first and kept only on request: one button per other
 * choice, then "Keep" for the one in use when it is not kept yet.
 */
function TryThenKeep(props: { label: string; about: string; action: string; choice: { current?: string; offered: string[] }; kept?: string; names: Record<string, string>; base: string }) {
  const openFix = useFix();
  const { current, offered } = props.choice;
  const name = (v: string) => props.names[v] ?? v;
  const kept = props.kept === current;
  return (
    <FieldRow label={props.label} description={`Now: ${current ? name(current) : "not reported"}${current && current !== props.base ? (kept ? ", kept after restart" : ", for this session only") : ""}. ${props.about}`}>
      <span className="btn-row">
        {offered
          .filter((v) => v !== current)
          .map((v) => (
            <Button key={v} small onClick={() => openFix(props.action, `${v}|try`)}>
              Try {name(v)}
            </Button>
          ))}
        {current && current !== props.base && !kept && (
          <Button small variant="primary" onClick={() => openFix(props.action, `${current}|keep`)}>
            Keep {name(current)}
          </Button>
        )}
        {kept && <StatusBadge variant="ok" label="Kept" />}
      </span>
    </FieldRow>
  );
}

/** The kept value of a try-then-keep setting. */
function keptValue(p: PowerDetail, setting: string): string | undefined {
  const c = p.kept?.find((k) => k.setting === setting)?.call;
  return c?.policy ?? c?.mode;
}

function KeptList({ p }: { p: PowerDetail }) {
  if (!p.kept?.length) return null;
  return (
    <FieldRow label="Kept after restart" description="DeeCure sets these again at each start, after the same checks. A device setting is skipped when a different device is in its place.">
      <ul className="plain-list">
        {p.kept.map((k) => (
          <li key={k.setting}>
            {describeKept(k, p)}{" "}
            {k.boot_ok === undefined ? (
              <StatusBadge variant="info" label="From the next start" />
            ) : k.boot_ok ? (
              <StatusBadge variant="ok" label="Set at this start" />
            ) : (
              <>
                <StatusBadge variant="warn" label="Not set at this start" /> <span className="muted">{k.boot_message}</span>
              </>
            )}
          </li>
        ))}
      </ul>
    </FieldRow>
  );
}

function BatteryControls({ b }: { b: PowerDetail["batteries"][number] }) {
  const openFix = useFix();
  const hasStart = b.charge_start !== undefined;
  const [stop, setStop] = useState(String(b.charge_stop ?? 80));
  const [start, setStart] = useState(String(b.charge_start ?? 75));
  const [mode, setMode] = useState(b.charge_mode?.current ?? "");
  const s = Number(stop);
  const st = Number(start);
  const valid = Number.isInteger(s) && s >= 20 && s <= 100 && (!hasStart || (Number.isInteger(st) && st >= 0 && st < s));
  return (
    <>
      {b.charge_stop !== undefined && (
        <FieldRow label={`Charge limits · ${b.name}`} description={`Now: ${hasStart ? `starts below ${b.charge_start}%, ` : ""}stops at ${b.charge_stop}%. Keeping the battery below full slows its wear.`}>
          <span className="btn-row">
            {PRESETS.map((p) => (
              <Button key={p.name} small onClick={() => openFix("power.set_charge_limits", limitsSubject(b.name, hasStart, p.stop))} disabled={b.charge_stop === p.stop}>
                {p.name} ({p.stop}%)
              </Button>
            ))}
          </span>
          <span className="btn-row">
            {hasStart && (
              <label>
                Start below <input className="input-num" type="number" min={0} max={99} value={start} onChange={(e) => setStart(e.target.value)} aria-label="Start charging below" />%
              </label>
            )}
            <label>
              Stop at <input className="input-num" type="number" min={20} max={100} value={stop} onChange={(e) => setStop(e.target.value)} aria-label="Stop charging at" />%
            </label>
            <Button small disabled={!valid} onClick={() => openFix("power.set_charge_limits", limitsSubject(b.name, hasStart, s, st))}>
              Set
            </Button>
          </span>
        </FieldRow>
      )}
      {b.charge_mode && b.charge_mode.offered.length > 1 && (
        <FieldRow label={`Charge mode · ${b.name}`} description="Adaptive learns your use; Fast charges quickly but wears the battery more; Custom uses the limits above.">
          <span className="btn-row">
            <select className="select" value={mode} onChange={(e) => setMode(e.target.value)} aria-label="Charge mode">
              {b.charge_mode.offered.map((m) => (
                <option key={m} value={m}>
                  {m}
                </option>
              ))}
            </select>
            <Button small disabled={!mode || mode === b.charge_mode.current} onClick={() => openFix("power.set_charge_mode", `${b.name}:${mode}`)}>
              Change
            </Button>
          </span>
        </FieldRow>
      )}
    </>
  );
}

export function PowerControls() {
  const openFix = useFix();
  const { data: p, error } = useLive<PowerDetail>(loadPower, 15_000);
  const cpu = useLive<CpuDetail>(loadCpu, 30_000).data;
  const net = useLive<NetDeep>(loadNet, 30_000).data;
  if (!p) {
    return (
      <Panel title="Power controls" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const wifi = net?.wifi.find((w) => w.uuid);
  const topKhz = Math.max(0, ...(cpu?.cores ?? []).map((c) => c.hardware_max_khz ?? c.max_khz ?? 0));
  const cpuLimit = p.kept?.find((k) => k.setting === "battery_cpu_limit")?.call.percent ?? undefined;
  return (
    <Panel
      title="Power controls"
      subtitle="each change shows a preview and can be undone"
      info="Changes to the battery, turbo, and devices need your password. Linux forgets these at each start, so DeeCure saves each change and sets it again at boot. Setting a value back to what it was stops keeping it."
      span={12}
    >
      <KeptList p={p} />
      {p.batteries.map((b) => (
        <BatteryControls key={b.name} b={b} />
      ))}
      {cpu?.turbo !== undefined && (
        <FieldRow label="CPU turbo" description={`Now ${cpu.turbo ? "on" : "off"}. Off: lower top speed, less heat, longer battery. Kept after restart.`}>
          <Button small onClick={() => openFix("power.set_turbo", cpu.turbo ? "off" : "on")}>
            Turn turbo {cpu.turbo ? "off" : "on"}
          </Button>
        </FieldRow>
      )}
      {p.batteries.length > 0 && topKhz > 0 && (
        <FieldRow label="Highest CPU speed on battery" description={`Now: ${cpuLimit ? `${cpuLimit}% on battery` : "no limit"}. On mains the CPU keeps its full speed. Lower: less heat and longer battery; heavy work takes longer.`}>
          <span className="btn-row">
            <Button small disabled={!cpuLimit} onClick={() => openFix("power.set_battery_cpu_limit", "off")}>
              No limit
            </Button>
            {CPU_LIMITS.map((pct) => (
              <Button key={pct} small disabled={cpuLimit === pct} onClick={() => openFix("power.set_battery_cpu_limit", String(pct))}>
                {pct}% ({((topKhz * pct) / 100 / 1e6).toFixed(1)} GHz)
              </Button>
            ))}
          </span>
        </FieldRow>
      )}
      {p.aspm_policy && p.aspm_policy.offered.length > 1 && (
        <TryThenKeep
          label="PCIe power saving"
          about="Lets Wi-Fi, drives, and graphics links sleep when idle. If a device acts up, restart: a setting that is not kept goes back. Some firmware does not let Linux change it."
          action="power.set_aspm_policy"
          choice={p.aspm_policy}
          kept={keptValue(p, "aspm_policy")}
          names={ASPM_NAMES}
          base="default"
        />
      )}
      {p.suspend && p.suspend.offered.length > 1 && (
        <TryThenKeep
          label="Suspend type"
          about="deep uses less battery while suspended, but some laptops do not wake from it. Try it, suspend and wake once, then keep it. If it does not wake, hold the power button: after the restart it is back."
          action="power.set_suspend_mode"
          choice={p.suspend}
          kept={keptValue(p, "suspend_mode")}
          names={SUSPEND_NAMES}
          base="s2idle"
        />
      )}
      {p.keyboard_backlight && p.keyboard_backlight.max > 0 && (
        <FieldRow label="Keyboard backlight" description={`Level ${p.keyboard_backlight.level} of ${p.keyboard_backlight.max}`}>
          <span className="btn-row">
            {Array.from({ length: p.keyboard_backlight.max + 1 }, (_, level) => (
              <Button key={level} small disabled={level === p.keyboard_backlight?.level} onClick={() => openFix("power.set_keyboard_backlight", String(level))}>
                {level === 0 ? "Off" : `Level ${level}`}
              </Button>
            ))}
          </span>
        </FieldRow>
      )}
      {radioKinds(p.radios).map((r) => (
        <FieldRow key={r.kind} label={RADIO_NAMES[r.kind] ?? r.kind} description={r.hard ? "Switched off by a hardware switch or key" : `Now ${r.on ? "on" : "off"}`}>
          {r.hard ? (
            <StatusBadge variant="off" label="Hardware switch" />
          ) : (
            <Button small onClick={() => openFix("power.set_radio", `${r.kind}|${r.on ? "off" : "on"}`)}>
              Turn {r.on ? "off" : "on"}
            </Button>
          )}
        </FieldRow>
      ))}
      {wifi?.uuid && (
        <FieldRow label={`Wi-Fi power saving · ${wifi.ssid}`} description={`Now ${wifi.power_save === undefined ? "set by the driver" : wifi.power_save ? "on" : "off"}. Saves battery, can add a little delay. Applies the next time Wi-Fi connects.`}>
          <span className="btn-row">
            <Button small disabled={wifi.power_save === true} onClick={() => openFix("power.set_wifi_power_save", `${wifi.uuid}|on`)}>
              On
            </Button>
            <Button small disabled={wifi.power_save === false} onClick={() => openFix("power.set_wifi_power_save", `${wifi.uuid}|off`)}>
              Off
            </Button>
          </span>
        </FieldRow>
      )}
    </Panel>
  );
}

/** A PCI device worth offering power saving: it has a driver, and is not storage (NVMe has its
 * own power states), a bridge, graphics, audio, or a system peripheral. */
export function pciPowerSavingSafe(d: PciDevice): boolean {
  return !!d.driver && !!d.sys_path && d.power_control === "on" && !["Storage", "Bridge", "Display", "Audio and video", "System", "Memory"].includes(d.class ?? "");
}
