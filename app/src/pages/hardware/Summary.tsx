// SPDX-License-Identifier: Apache-2.0
// Hardware › Summary (docs/03-ui-design/05-pages/06-hardware.md), with real data.
// Badges come from simple, stated thresholds; DeeCure has no hardware rules yet.

import { temp as tempText } from "../../units";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { type StatusRow, StatusList } from "../../components/StatusList";
import { bytes, pick, pickAll } from "../../metrics";
import { href } from "../../route";
import { useLatest } from "../performance/shared";
import { NoData } from "../NoData";
import { AdminRead } from "../../components/AdminRead";
import { NEEDS_DRIVER, biosDate, shortCpu, useBluetooth, useFirmware, useGpus, useHardware } from "./shared";

export function HardwareSummary() {
  const { data: hw, error } = useHardware();
  const latest = useLatest();
  const gpus = useGpus();
  const firmware = useFirmware();
  const bluetooth = useBluetooth();
  if (!hw) {
    return (
      <Panel title="Machine" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const m = hw.machine;
  const temp = pick(latest, "temp.cpu");
  const health = pickAll(latest, "battery.health")[0]?.value;
  const fans = pickAll(latest, "fan");
  const disk = hw.disks[0];
  const missing = hw.devices.filter((d) => d.bus === "PCI" && d.class && NEEDS_DRIVER.has(d.class) && d.drivers.length === 0).length;
  const updates = firmware?.devices.filter((d) => d.available).length ?? 0;
  const gpu = gpus?.[0];
  const rows: StatusRow[] = [
    { icon: "chip", label: "Processor", metric: [shortCpu(hw.processor.model), temp !== undefined && tempText(temp)].filter(Boolean).join(" · "), status: temp === undefined ? "off" : temp >= 90 ? "crit" : temp >= 80 ? "warn" : "ok", href: href("hardware", "processor") },
    { icon: "layers", label: "Memory", metric: hw.memory_total ? bytes(hw.memory_total) : "—", status: "ok", href: href("hardware", "memory") },
    { icon: "db", label: "Storage", metric: disk ? `${disk.kind} ${bytes(disk.size, 0)}` : "No disks found", status: "off", href: href("hardware", "storage") },
    { icon: "bat", label: "Battery", metric: health === undefined ? "No battery" : `${Math.round(health)}% of design`, status: health === undefined ? "off" : health < 60 ? "warn" : health < 80 ? "info" : "ok", href: href("power", "battery") },
    { icon: "mon", label: "Graphics", metric: gpu ? [gpu.model?.match(/\[(.+)\]/)?.[1] ?? gpu.model, gpu.driver].filter(Boolean).join(" · ") : "—", status: gpu ? (gpu.driver ? "ok" : "warn") : "off", href: href("hardware", "graphics") },
    { icon: "fan", label: "Fans", metric: fans.length ? fans.map((f) => `${Math.round(f.value)} RPM`).join(", ") : "Not reported", status: fans.length ? "ok" : "off", href: href("hardware", "sensors") },
    {
      icon: "usb",
      label: "Peripherals",
      metric: `${hw.devices.filter((d) => d.bus === "USB").length + (bluetooth?.devices.length ?? 0)} devices · ${missing} missing drivers`,
      status: missing ? "warn" : "ok",
      href: href("hardware", "peripherals"),
    },
    { icon: "bolt", label: "Firmware", metric: !firmware ? "Loading…" : !firmware.available ? "fwupd not running" : updates ? `${updates} update${updates === 1 ? "" : "s"} available` : "Up to date", status: !firmware?.available ? "off" : updates ? "info" : "ok", href: href("hardware", "firmware") },
  ];
  return (
    <>
      <Panel title="Machine" source="/sys/class/dmi/id" span={6} flush>
        <KeyValueList
          rows={[
            ["Vendor", m.vendor ?? "—"],
            ["Model", m.model ?? "—"],
            ["Chassis", m.chassis ?? "—"],
            ["Serial number", <AdminRead key="s" what="serial" label="Show (asks for your password)">{(t) => <code>{t || "Not set by the maker"}</code>}</AdminRead>],
            ["BIOS", [m.bios_version, m.bios_date && biosDate(m.bios_date)].filter(Boolean).join(" · ") || "—"],
            ["Secure Boot", m.secure_boot === undefined ? "Not available (legacy BIOS)" : <StatusBadge key="sb" variant={m.secure_boot ? "ok" : "warn"} label={m.secure_boot ? "Enabled" : "Disabled"} />],
          ]}
        />
      </Panel>
      <Panel title="Components" span={6} flush>
        <StatusList rows={rows} />
      </Panel>
    </>
  );
}
