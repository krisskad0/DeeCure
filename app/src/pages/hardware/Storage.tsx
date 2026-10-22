// SPDX-License-Identifier: Apache-2.0
// Hardware › Storage Devices (docs/03-ui-design/05-pages/06-hardware.md), with real data.
// SMART health needs Full access (Phase 2); drive temperature comes from hwmon.

import type { HardwareInfo } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { bytes, pick, pickAll } from "../../metrics";
import { TimeChart, useHistory, useLatest } from "../performance/shared";
import { NoData } from "../NoData";
import { useHardware } from "./shared";

type DiskRow = HardwareInfo["disks"][number] & { temp?: number };

const COLUMNS: Column<DiskRow>[] = [
  { key: "name", label: "Device", text: (d) => d.name, render: (d) => <code>{d.name}</code> },
  { key: "model", label: "Model", text: (d) => d.model ?? "—" },
  { key: "kind", label: "Type", text: (d) => d.kind },
  { key: "size", label: "Size", text: (d) => bytes(d.size, 0), numeric: true },
  { key: "fw", label: "Firmware", text: (d) => d.firmware ?? "—", expert: true },
  { key: "smart", label: "SMART", text: () => "Full access", render: () => <StatusBadge variant="off" label="Needs Full access" /> },
  { key: "temp", label: "Temperature", text: (d) => (d.temp === undefined ? "—" : `${Math.round(d.temp)} °C`), numeric: true },
];

export function HardwareStorage() {
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
      <Panel title="Drive health (SMART)" span={12}>
        <div className="empty muted">Life left, data written, power-on hours, and errors need Full access · Settings › Access</div>
      </Panel>
      {nvme && (
        <Panel title="Drive temperature" source="/sys/class/hwmon (nvme)" span={12}>
          <TimeChart history={history} names={["NVMe"]} unit="°C" height={180} />
        </Panel>
      )}
    </>
  );
}
