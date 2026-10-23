// SPDX-License-Identifier: Apache-2.0
// Expert sections for firmware, platform, and ports (docs/01-product/expert-coverage.md, sections
// 12 and 16): the firmware security level and its checks, UEFI boot entries, Secure Boot keys, the
// management engine, the TPM, port protection, the hardware clock, virtualization, the USB tree,
// USB-C ports, and Thunderbolt devices.

import { type PlatformDetail, getInventory } from "../../api";
import { DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadPlatform = () => getInventory("platform_detail");
const usePlatform = () => useLive(loadPlatform, 300_000);

type Usb = PlatformDetail["usb"][number];

/** USB speed in plain words, and whether a USB 3 device is held back to USB 2 speed. */
export function usbSpeed(u: Usb): { text: string; variant?: StatusVariant } {
  const s = u.speed_mbps;
  if (s === undefined) return { text: "-" };
  const name = s >= 20000 ? "20 Gbit/s (USB 3.2)" : s >= 10000 ? "10 Gbit/s (USB 3.1)" : s >= 5000 ? "5 Gbit/s (USB 3)" : s >= 480 ? "480 Mbit/s (USB 2)" : s >= 12 ? "12 Mbit/s (USB 1)" : `${s} Mbit/s (USB 1)`;
  const v = parseFloat(u.version ?? "0");
  if (v >= 3 && s <= 480) return { text: `${name}: a USB 3 device on a USB 2 link`, variant: "warn" };
  return { text: name };
}

/** What the HSI level means, from fwupd's definitions. */
export function hsiMeaning(hsi?: string): string {
  const level = hsi?.match(/HSI:(\d)/)?.[1];
  const runtime = hsi?.includes("!") ? " Some protections are off in the running system." : "";
  const words: Record<string, string> = {
    "0": "Not protected: critical firmware protections are missing.",
    "1": "Critical protections are in place (such as Secure Boot and a locked firmware chip).",
    "2": "Important protections too (such as BIOS rollback protection and device memory protection).",
    "3": "Also protected against attacks during boot and sleep.",
    "4": "Also encrypted memory.",
  };
  return level ? `${words[level] ?? `Level ${level}.`}${runtime}` : "fwupd did not report a level.";
}

// ---------- Hardware › Firmware ----------

export function FirmwareExpertSections() {
  const { data: d, error } = usePlatform();
  if (!d) {
    return (
      <Panel title="Firmware security" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const failed = d.security_checks.filter((c) => !c.passed);
  return (
    <>
      <Panel title="Firmware security" subtitle={d.hsi ?? undefined} info="fwupd's Host Security ID (HSI): one level for many firmware and platform protections. Each check below is one of them." source="fwupdmgr security" span={6} flush>
        <KeyValueList
          rows={[
            ["Level", <span key="l">{d.hsi ?? "-"} · {hsiMeaning(d.hsi)}</span>],
            ["Checks passed", `${d.security_checks.length - failed.length} of ${d.security_checks.length}`],
            ["Secure Boot", d.secure_boot ?? "-"],
            ["Keys you enrolled (MOK)", d.mok_keys ? (d.mok_keys.length ? d.mok_keys.join("; ") : "None") : "mokutil not available"],
            ["Management engine firmware", d.management_engine ?? "Not reported"],
            ["TPM", d.tpm_version ? `TPM ${d.tpm_version}.0${d.tpm_driver ? ` (${d.tpm_driver})` : ""}` : "None found"],
            ["Disk encryption", d.disk_encrypted ? (d.disk_unlocked_by_tpm ? "On, unlocked by the TPM" : "On, unlocked with a password") : "No encrypted disk"],
            ["Memory protection from devices (IOMMU)", d.iommu_units ? `On (${d.iommu_units} units)` : "Off"],
            ...(d.thunderbolt_security
              ? [["Thunderbolt and USB4 ports", `${d.thunderbolt_security === "user" ? "Ask before new devices" : d.thunderbolt_security === "none" ? "Every device allowed" : d.thunderbolt_security}${d.thunderbolt_dma_protection ? " · DMA protection on" : " · no DMA protection"}`] as [string, string]]
              : []),
            ["Hardware clock", d.rtc_drift_s === undefined ? "-" : Math.abs(d.rtc_drift_s) <= 2 ? "Matches the system time" : `${d.rtc_drift_s > 0 ? "+" : ""}${d.rtc_drift_s} s from the system time (or set to local time)`],
            ["Virtual machines", d.virtualization ? `${d.virtualization === "vmx" ? "Intel VT-x" : "AMD-V"} on${d.kvm_ready ? " · KVM ready" : " · KVM not loaded"}` : "Not available: off in the firmware, or not supported"],
          ]}
        />
      </Panel>
      <Panel title="Security checks" subtitle={`${failed.length} not passed`} source="fwupdmgr security" span={6} flush>
        <DataTable
          rows={[...d.security_checks].sort((a, b) => Number(a.passed) - Number(b.passed))}
          columns={[
            { key: "name", label: "Check", text: (c) => c.name, wrap: true },
            {
              key: "result",
              label: "Result",
              text: (c) => c.result,
              render: (c) => (c.passed ? c.result : <StatusBadge variant={c.runtime ? "info" : "warn"} label={`${c.result}${c.runtime ? " (running system)" : ""}`} />),
            },
            { key: "id", label: "ID", text: (c) => c.id, expert: true },
          ]}
          rowKey={(c) => c.id}
          pageSize={12}
          empty="fwupd is not installed or did not answer"
        />
      </Panel>
      <Panel title="UEFI boot entries" info="What the firmware can start, in the order it tries them. The current entry is the one this boot used." source="/sys/firmware/efi/efivars" span={12} flush>
        <DataTable
          rows={[...d.boot_entries].sort((a, b) => (a.order ?? 99) - (b.order ?? 99))}
          columns={[
            { key: "order", label: "Order", text: (b) => (b.order === undefined ? "Not in order" : String(b.order + 1)), numeric: true },
            { key: "name", label: "Entry", text: (b) => b.name, render: (b) => (b.current ? <span>{b.name} <StatusBadge variant="ok" label="this boot" /></span> : b.name), wrap: true },
            { key: "file", label: "Starts", text: (b) => b.file ?? "-", wrap: true },
            { key: "active", label: "On", text: (b) => (b.active ? "Yes" : "No") },
            { key: "number", label: "Boot number", text: (b) => b.number, expert: true },
          ]}
          rowKey={(b) => b.number}
          empty="Not started with UEFI, or the boot entries cannot be read"
        />
      </Panel>
    </>
  );
}

// ---------- Hardware › Peripherals ----------

export function PortsExpertSections() {
  const { data: d, error } = usePlatform();
  if (!d) {
    return (
      <Panel title="Ports" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  return (
    <>
      <Panel title="USB devices" info="Speed is the link speed now. A USB 3 device on a USB 2 link is slowed down by the port, hub, or cable. Power is the most current the device asked for." source="/sys/bus/usb/devices" span={12} flush>
        <DataTable
          rows={d.usb}
          columns={[
            { key: "product", label: "Device", text: (u) => u.product ?? u.port, wrap: true },
            { key: "speed", label: "Speed", text: (u) => usbSpeed(u).text, render: (u) => (usbSpeed(u).variant ? <StatusBadge variant="warn" label={usbSpeed(u).text} /> : usbSpeed(u).text) },
            { key: "power", label: "Power asked", text: (u) => (u.max_power_ma === undefined ? "-" : `${u.max_power_ma} mA`), numeric: true },
            { key: "drivers", label: "Driver", text: (u) => u.drivers.join(", ") || (u.hub ? "hub" : "-") },
            { key: "version", label: "USB version", text: (u) => u.version ?? "-", expert: true },
            { key: "port", label: "Port", text: (u) => u.port, expert: true },
          ]}
          rowKey={(u) => u.port}
          pageSize={15}
          empty="No USB devices"
        />
      </Panel>
      {d.typec.length > 0 && (
        <Panel title="USB-C ports" info="Data role: host (the computer leads) or device. Power role: source (gives power) or sink (takes power). Alternate modes carry a screen (DisplayPort) or Thunderbolt over USB-C." source="/sys/class/typec" span={6} flush>
          <DataTable
            rows={d.typec}
            columns={[
              { key: "name", label: "Port", text: (p) => p.name },
              { key: "conn", label: "Plugged in", text: (p) => (p.connected ? `Yes${p.partner_type ? ` (${p.partner_type})` : ""}${p.partner_power_delivery ? " · Power Delivery" : ""}` : "No") },
              { key: "roles", label: "Data · power", text: (p) => `${p.data_role ?? "-"} · ${p.power_role ?? "-"}` },
              { key: "power", label: "Power mode", text: (p) => (p.power_mode === "usb_power_delivery" ? "Power Delivery" : (p.power_mode ?? "-")) },
              { key: "alt", label: "Modes in use", text: (p) => p.alt_modes.filter((m) => m.active).map((m) => m.name).join(", ") || "-" },
            ]}
            rowKey={(p) => p.name}
          />
        </Panel>
      )}
      {d.thunderbolt_security && (
        <Panel title="Thunderbolt and USB4 devices" source="/sys/bus/thunderbolt" span={6} flush>
          <DataTable
            rows={d.thunderbolt}
            columns={[
              { key: "name", label: "Device", text: (t) => [t.vendor, t.name].filter(Boolean).join(" ") || t.id },
              { key: "auth", label: "Approved", text: (t) => (t.authorized ? "Yes" : "No"), render: (t) => (t.authorized ? "Yes" : <StatusBadge variant="warn" label="Not approved: it does not work until approved" />) },
            ]}
            rowKey={(t) => t.id}
            empty="Nothing plugged in"
          />
        </Panel>
      )}
    </>
  );
}
