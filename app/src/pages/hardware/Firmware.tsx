// SPDX-License-Identifier: Apache-2.0
// Hardware › Firmware (docs/03-ui-design/05-pages/06-hardware.md), with real data.
// Installing firmware needs the helper (Phase 2).

import type { FirmwareInfo } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { NoData } from "../NoData";
import { biosDate, useFirmware, useHardware } from "./shared";

type Device = FirmwareInfo["devices"][number];

const COLUMNS: Column<Device>[] = [
  { key: "name", label: "Device", text: (d) => d.name, wrap: true },
  { key: "current", label: "Current", text: (d) => d.version ?? "-", render: (d) => <code>{d.version ?? "-"}</code> },
  { key: "available", label: "Available", text: (d) => d.available ?? "-", render: (d) => (d.available ? <code>{d.available}</code> : "-") },
  { key: "vendor", label: "Vendor", text: (d) => d.vendor ?? "-" },
  {
    key: "status",
    label: "Status",
    text: (d) => (d.available ? "Update available" : d.updatable ? "Up to date" : "Not on LVFS"),
    render: (d) => (d.available ? <StatusBadge variant="info" label="Update available" /> : d.updatable ? <StatusBadge variant="ok" label="Up to date" /> : <StatusBadge variant="off" label="Not on LVFS" />),
  },
];

export function HardwareFirmware() {
  const { data: hw, error } = useHardware();
  const fw = useFirmware();
  if (!hw) {
    return (
      <Panel title="Platform firmware" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const m = hw.machine;
  return (
    <>
      <Panel title="Platform firmware" source="/sys/class/dmi/id · /sys/firmware" span={5} flush>
        <KeyValueList
          rows={[
            ["BIOS vendor", m.bios_vendor ?? "-"],
            ["Version", m.bios_version ? <code key="v">{m.bios_version}</code> : "-"],
            ["Release date", biosDate(m.bios_date)],
            ["Boot mode", m.boot_mode],
            ["Secure Boot", m.secure_boot === undefined ? "Not available" : <StatusBadge key="s" variant={m.secure_boot ? "ok" : "warn"} label={m.secure_boot ? "Enabled" : "Disabled"} />],
            ["TPM", m.tpm ? `${m.tpm} · present` : "Not found"],
          ]}
        />
      </Panel>
      <Panel title="Firmware updates" source="fwupd (local metadata)" span={7} flush>
        {!fw ? (
          <div className="empty muted">Loading…</div>
        ) : !fw.available ? (
          <div className="empty muted">The firmware update service (fwupd) is not running, so updates are unknown. DeeCure does not start it.</div>
        ) : (
          <DataTable rows={fw.devices} columns={COLUMNS} rowKey={(d) => `${d.name}-${d.version}`} empty="fwupd knows no devices" />
        )}
      </Panel>
    </>
  );
}
