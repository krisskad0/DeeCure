// SPDX-License-Identifier: Apache-2.0
// Hardware › Peripherals (docs/03-ui-design/05-pages/06-hardware.md), with real data.

import { PortsExpertSections } from "./PlatformExpert";
import { BuiltInDevicesExpert } from "./DevicesExpert";
import { PcieLinks } from "../performance/DiskExpert";
import { atLeast, useUi } from "../../ui";
import { type Column, DataTable } from "../../components/DataTable";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { NoData } from "../NoData";
import { NEEDS_DRIVER, useBluetooth, useHardware } from "./shared";

interface Row {
  key: string;
  name: string;
  bus: string;
  vendor: string;
  driver: string;
  status: [StatusVariant, string];
}

const COLUMNS: Column<Row>[] = [
  { key: "name", label: "Device", text: (r) => r.name, wrap: true },
  { key: "bus", label: "Bus", text: (r) => r.bus },
  { key: "vendor", label: "Vendor", text: (r) => r.vendor },
  { key: "driver", label: "Driver", text: (r) => r.driver, render: (r) => (r.driver === "-" ? "-" : <code>{r.driver}</code>) },
  { key: "status", label: "Status", text: (r) => r.status[1], render: (r) => <StatusBadge variant={r.status[0]} label={r.status[1]} /> },
];

export function HardwarePeripherals() {
  const { mode } = useUi();
  const { data: hw, error } = useHardware();
  const bt = useBluetooth();
  if (!hw) {
    return (
      <Panel title="Devices" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const rows: Row[] = hw.devices.map((d, i) => {
    const needs = d.class ? NEEDS_DRIVER.has(d.class) : d.bus === "USB";
    const status: [StatusVariant, string] = d.drivers.length ? ["ok", "Active"] : d.bus === "Input" ? ["ok", "Active"] : needs ? ["warn", "No driver"] : ["off", "No driver needed"];
    return { key: `${d.bus}-${i}`, name: d.name, bus: d.bus, vendor: d.vendor ?? "-", driver: d.drivers.join(", ") || (d.bus === "Input" ? "input" : "-"), status };
  });
  for (const [i, d] of (bt?.devices ?? []).entries()) {
    rows.push({
      key: `bt-${i}`,
      name: d.battery === undefined ? d.name : `${d.name} · battery ${d.battery}%`,
      bus: "Bluetooth",
      vendor: "-",
      driver: "bluez",
      status: d.connected ? ["ok", "Connected"] : ["off", "Paired"],
    });
  }
  const missing = rows.filter((r) => r.status[1] === "No driver").length;
  const connected = bt?.devices.filter((d) => d.connected).length ?? 0;
  return (
    <>
      <KpiRow
        tiles={[
          { label: "USB devices", value: String(rows.filter((r) => r.bus === "USB").length) },
          { label: "PCI devices", value: String(rows.filter((r) => r.bus === "PCI").length) },
          {
            label: "Bluetooth",
            value: bt?.available ? String(bt.devices.length) : "-",
            subtext: !bt ? "" : !bt.available ? "Bluetooth service not running" : bt.adapters === 0 ? "No adapter found" : !bt.powered ? "Turned off" : `${connected} connected`,
          },
          { label: "Missing drivers", value: String(missing), status: missing ? "warn" : "ok" },
        ]}
      />
      <Panel title="Devices" source="/sys/bus/{pci,usb} · /proc/bus/input · BlueZ" span={12} flush>
        <DataTable rows={rows} columns={COLUMNS} rowKey={(r) => r.key} search={{ placeholder: "Search devices" }} filter={{ column: "bus", options: ["USB", "PCI", "Bluetooth", "Input"] }} pageSize={14} />
      </Panel>
      {atLeast(mode, "expert") && (
        <>
          <BuiltInDevicesExpert />
          <PortsExpertSections />
          <PcieLinks />
        </>
      )}
    </>
  );
}
