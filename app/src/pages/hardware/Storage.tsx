// SPDX-License-Identifier: Apache-2.0
// Hardware › Storage Devices (docs/03-ui-design/05-pages/06-hardware.md), with real data.
// SMART health needs Full access (Phase 2); drive temperature comes from hwmon.

import { DriveExpertSections } from "../performance/DiskExpert";
import { atLeast, useUi } from "../../ui";
import { temp as tempText, tempUnit, tempValue } from "../../units";
import type { HardwareInfo } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { bytes, pick, pickAll } from "../../metrics";
import { TimeChart, useHistory, useLatest } from "../performance/shared";
import { NoData } from "../NoData";
import { AdminRead } from "../../components/AdminRead";
import { KpiRow } from "../../components/KpiTile";
import { useHardware } from "./shared";

type DiskRow = HardwareInfo["disks"][number] & { temp?: number };

const COLUMNS: Column<DiskRow>[] = [
  { key: "name", label: "Device", text: (d) => d.name, render: (d) => <code>{d.name}</code> },
  { key: "model", label: "Model", text: (d) => d.model ?? "-" },
  { key: "kind", label: "Type", text: (d) => d.kind },
  { key: "size", label: "Size", text: (d) => bytes(d.size, 0), numeric: true },
  { key: "fw", label: "Firmware", text: (d) => d.firmware ?? "-", expert: true },
  { key: "smart", label: "SMART", text: () => "Full access", render: () => <StatusBadge variant="off" label="Needs Full access" /> },
  { key: "temp", label: "Temperature", text: (d) => (d.temp === undefined ? "-" : tempText(d.temp)), numeric: true },
];

export function HardwareStorage() {
  const { mode } = useUi();
  const { data: hw, error } = useHardware();
  const latest = useLatest();
  // NVMe drives report their temperature as hwmon "nvme/Composite".
  const nvme = pickAll(latest, "temp").find((t) => t.subject?.startsWith("nvme/"))?.subject;
  const history = useHistory(nvme ? [{ name: "temp", subject: nvme }] : []);
  if (!hw) {
    return (
      <Panel title="Disks" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const rows: DiskRow[] = hw.disks.map((d) => ({ ...d, temp: d.name.startsWith("nvme") && nvme ? pick(latest, "temp", nvme) : undefined }));
  return (
    <>
      <Panel title="Disks" source="/sys/block" span={12} flush>
        <DataTable rows={rows} columns={COLUMNS} rowKey={(d) => d.name} empty="No disks found" />
      </Panel>
      {hw.disks.map((d) => (
        <Panel key={d.name} title={`Drive health (SMART) · ${d.name}`} source="smartctl -j -a" span={12}>
          <AdminRead what="smart" arg={d.name} label="Read drive health (asks for your password)">
            {(text) => <Smart json={text} />}
          </AdminRead>
        </Panel>
      ))}
      {nvme && (
        <Panel title="Drive temperature" source="/sys/class/hwmon (nvme)" span={12}>
          <TimeChart history={history} names={["NVMe"]} unit={tempUnit()} scale={tempValue} height={180} />
        </Panel>
      )}
      {atLeast(mode, "expert") && <DriveExpertSections />}
    </>
  );
}

/** The parts of `smartctl -j` output a person needs. */
export function Smart({ json }: { json: string }) {
  let v: Record<string, unknown>;
  try {
    v = JSON.parse(json) as Record<string, unknown>;
  } catch {
    return <div className="empty muted">smartctl gave no readable answer.</div>;
  }
  const get = (path: string): unknown => path.split(".").reduce<unknown>((o, k) => (o && typeof o === "object" ? (o as Record<string, unknown>)[k] : undefined), v);
  const passed = get("smart_status.passed");
  const nvme = get("nvme_smart_health_information_log") as Record<string, number> | undefined;
  const hours = (get("power_on_time.hours") as number | undefined) ?? nvme?.power_on_hours;
  const tiles = [
    { label: "Overall", value: passed === undefined ? "Unknown" : passed ? "Passed" : "FAILING", status: passed === false ? ("crit" as const) : ("ok" as const) },
    ...(nvme
      ? [
          { label: "Life used", value: String(nvme.percentage_used ?? "-"), unit: "%", status: (nvme.percentage_used ?? 0) >= 90 ? ("warn" as const) : ("ok" as const), subtext: "of rated endurance" },
          { label: "Data written", value: nvme.data_units_written ? ((nvme.data_units_written * 512_000) / 1e12).toFixed(1) : "-", unit: "TB" },
          { label: "Unsafe shutdowns", value: String(nvme.unsafe_shutdowns ?? "-") },
          { label: "Media errors", value: String(nvme.media_errors ?? "-"), status: (nvme.media_errors ?? 0) > 0 ? ("crit" as const) : ("ok" as const) },
          { label: "Spare capacity", value: String(nvme.available_spare ?? "-"), unit: "%" },
        ]
      : []),
    { label: "Power-on hours", value: hours === undefined ? "-" : hours.toLocaleString(), unit: "h" },
  ];
  return <KpiRow tiles={tiles} />;
}

