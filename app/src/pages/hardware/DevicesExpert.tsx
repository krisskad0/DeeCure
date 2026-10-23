// SPDX-License-Identifier: Apache-2.0
// Expert sections for built-in devices, screens and sound, and hardware errors
// (docs/01-product/expert-coverage.md, sections 17, 18, and 19).

import { type DevicesDetail, type HwErrors, getInventory } from "../../api";
import { DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { ago } from "../../time";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadDevices = () => getInventory("devices_detail");
const loadErrors = () => getInventory("hw_errors");
const loadPci = () => getInventory("pci_detail");
const loadMemory = () => getInventory("memory_detail");
const useDevices = () => useLive(loadDevices, 120_000);

/** Screen makers by their EDID code. Unknown codes are shown as they are. */
const MAKERS: Record<string, string> = {
  LGD: "LG Display", AUO: "AU Optronics", BOE: "BOE", SHP: "Sharp", SDC: "Samsung Display", CMN: "Innolux", IVO: "InfoVision", CSO: "CSOT",
  DEL: "Dell", SAM: "Samsung", GSM: "LG", ACR: "Acer", AUS: "ASUS", HWP: "HP", LEN: "Lenovo", BNQ: "BenQ", VSC: "ViewSonic", PHL: "Philips",
  AOC: "AOC", MSI: "MSI", APP: "Apple", HPN: "HP", NCP: "Najing CEC", TMX: "Tianma",
};

type Screen = DevicesDetail["screens"][number];

/** Diagonal in inches and pixel density, from the screen's own size and native mode. */
export function screenSize(s: Screen): { inches?: number; ppi?: number } {
  if (!s.width_cm || !s.height_cm) return {};
  const inches = Math.sqrt(s.width_cm ** 2 + s.height_cm ** 2) / 2.54;
  const ppi = s.native ? Math.sqrt(s.native[0] ** 2 + s.native[1] ** 2) / inches : undefined;
  return { inches, ppi };
}

// ---------- Hardware › Peripherals ----------

export function BuiltInDevicesExpert() {
  const { data: d, error } = useDevices();
  if (!d) {
    return (
      <Panel title="Built-in devices" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  return (
    <>
      <Panel title="Built-in devices" source="/sys/class/video4linux · fprintd · /proc/bus/input · upower · CUPS" span={6} flush>
        <KeyValueList
          rows={[
            ["Cameras", d.cameras.length ? d.cameras.map((c) => `${c.name}${c.driver ? ` (${c.driver})` : ""}`).join(", ") : "None"],
            ["Fingerprint reader", !d.fingerprint ? "fprintd is not installed" : d.fingerprint.device ? `${d.fingerprint.device} · ${d.fingerprint.enrolled.length ? `${d.fingerprint.enrolled.length} finger${d.fingerprint.enrolled.length === 1 ? "" : "s"} enrolled` : "no finger enrolled"}` : "None supported by Ubuntu"],
            ["Touchpad, touch screen, pen", d.pointing.length ? d.pointing.join(", ") : "None found"],
            ["Batteries of devices", d.batteries.length ? d.batteries.map((b) => `${b.model ?? b.kind}: ${b.percent === undefined ? "-" : `${b.percent}%`}`).join(", ") : "None reported"],
            ["Printers", !d.printers ? "CUPS is not installed" : d.printers.length ? d.printers.map((p) => `${p.name}${p.default ? " (default)" : ""}: ${p.state}${p.jobs ? `, ${p.jobs} waiting` : ""}`).join("; ") : "None set up"],
          ]}
        />
      </Panel>
      <Panel title="Network chips" info="For Ethernet: the speed agreed with the other end, against the fastest the chip supports. A gigabit chip at 100 Mb/s often means a worn cable." source="ethtool" span={6} flush>
        <DataTable
          rows={d.network}
          columns={[
            { key: "if", label: "Interface", text: (n) => n.interface },
            { key: "driver", label: "Driver · firmware", text: (n) => [n.driver, n.firmware].filter(Boolean).join(" · ") || "-", wrap: true },
            {
              key: "speed",
              label: "Speed",
              text: (n) => (n.link === false ? "No cable" : n.speed ? `${n.speed}${n.duplex ? ` ${n.duplex.toLowerCase()}` : ""}` : "-"),
              render: (n) => {
                if (n.link === false) return "No cable";
                const now = parseInt(n.speed ?? "");
                const best = n.fastest ? parseFloat(n.fastest) * (n.fastest.includes("Gb") ? 1000 : 1) : undefined;
                const text = n.speed ? `${n.speed}${n.duplex ? ` ${n.duplex.toLowerCase()}` : ""}` : "-";
                return best && now && now < best ? <StatusBadge variant="warn" label={`${text} (chip supports ${n.fastest})`} /> : text;
              },
            },
            { key: "eee", label: "Energy saving", text: (n) => n.energy_saving ?? "-" },
          ]}
          rowKey={(n) => n.interface}
        />
      </Panel>
    </>
  );
}

// ---------- Display & Audio ----------

export function ScreensExpert() {
  const { data: d, error } = useDevices();
  if (!d) {
    return (
      <Panel title="Screens in detail" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  return (
    <>
      <Panel title="Screens in detail" info="Read from each screen's own description (EDID). Colour depth is per channel; 6 bits usually means the panel adds dithering to show more colours." source="/sys/class/drm/*/edid" span={12} flush>
        <DataTable
          rows={d.screens}
          columns={[
            { key: "screen", label: "Screen", text: (s) => `${s.model ?? "-"} (${s.connector})`, wrap: true },
            { key: "maker", label: "Maker", text: (s) => (s.maker ? (MAKERS[s.maker] ?? s.maker) : "-") },
            { key: "year", label: "Made", text: (s) => (s.year ? String(s.year) : "-") },
            { key: "size", label: "Size", text: (s) => (screenSize(s).inches ? `${screenSize(s).inches?.toFixed(1)}″ (${s.width_cm} × ${s.height_cm} cm)` : "-") },
            { key: "native", label: "Native", text: (s) => (s.native ? `${s.native[0]} × ${s.native[1]}${screenSize(s).ppi ? ` · ${Math.round(screenSize(s).ppi ?? 0)} ppi` : ""}` : "-") },
            { key: "refresh", label: "Refresh range", text: (s) => (s.refresh_min && s.refresh_max ? `${s.refresh_min} to ${s.refresh_max} Hz` : "Not stated") },
            { key: "bits", label: "Colour depth", text: (s) => (s.bits ? `${s.bits} bits` : "-") },
            { key: "hdr", label: "HDR", text: (s) => s.hdr.join(", ") || "No" },
            { key: "code", label: "Product code", text: (s) => `${s.maker ?? ""} ${s.product_code.toString(16).padStart(4, "0")}`, expert: true },
          ]}
          rowKey={(s) => s.connector}
          empty="No connected screen gives its description"
        />
      </Panel>
      {d.backlights.length > 0 && (
        <Panel title="Backlight" source="/sys/class/backlight" span={6} flush>
          <KeyValueList rows={d.backlights.map((b): [string, string] => [b.name, `${b.max ? Math.round((b.level / b.max) * 100) : 0}% (${b.level.toLocaleString()} of ${b.max.toLocaleString()} steps, ${b.kind ?? "-"})`])} />
        </Panel>
      )}
    </>
  );
}

export function SoundHardwareExpert() {
  const { data: d, error } = useDevices();
  if (!d) {
    return (
      <Panel title="Sound hardware" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  return (
    <Panel title="Sound hardware" info="The sound card's driver and the chips (codecs) behind its outputs. sof-… cards use Intel's Sound Open Firmware." source="/proc/asound" span={12} flush>
      <DataTable
        rows={d.sound}
        columns={[
          { key: "card", label: "Card", text: (c) => `${c.index}: ${c.name}`, wrap: true },
          { key: "codecs", label: "Codecs", text: (c) => c.codecs.join(", ") || "-", wrap: true },
        ]}
        rowKey={(c) => String(c.index)}
        empty="No sound card found"
      />
    </Panel>
  );
}

// ---------- Hardware › Summary ----------

const KIND_NAMES: Record<string, string> = {
  cpu_memory: "CPU or memory hardware faults",
  firmware_missing: "Firmware that failed to load",
  firmware_crash: "Firmware crashes",
  usb: "USB errors",
  disk: "Disk link errors",
  pcie: "PCIe link errors (logged)",
  gpu: "Graphics hangs",
  heat: "Heat events (logged)",
};

export function HardwareErrorsPanel() {
  const { data: e, error } = useLive<HwErrors>(loadErrors, 600_000);
  const pci = useLive(loadPci, 600_000).data;
  const mem = useLive(loadMemory, 600_000).data;
  if (!e) {
    return (
      <Panel title="Hardware errors" span={12}>
        <NoData error={error} waiting="Checking the kernel log…" />
      </Panel>
    );
  }
  const pcieCounted = (pci ?? []).reduce((n, d) => n + (d.errors ? d.errors.corrected + d.errors.nonfatal + d.errors.fatal : 0), 0);
  const memErrors = (mem?.controllers ?? []).reduce((n, c) => n + c.corrected + c.uncorrected, 0);
  const rows = [
    ...(e.kinds ?? []).map((k) => ({ key: k.kind, name: KIND_NAMES[k.kind] ?? k.kind, count: k.count, last: k.last_at, example: k.examples[0] })),
    ...(pci ? [{ key: "pcie_counted", name: "PCIe link errors (counted by the hardware since start)", count: pcieCounted, last: undefined, example: undefined }] : []),
    ...(mem?.controllers.length ? [{ key: "ecc", name: "Memory errors (counted by the memory controller)", count: memErrors, last: undefined, example: undefined }] : []),
    { key: "dumps", name: "Driver crash dumps since start", count: e.crash_dumps.length, last: undefined, example: e.crash_dumps.join(", ") || undefined },
  ];
  const total = rows.reduce((n, r) => n + r.count, 0);
  const now = Date.now();
  return (
    <>
      <Panel
        title="Hardware errors"
        subtitle={total ? `${total} in the last 7 days or since start` : "none found"}
        info="Every error the hardware itself reported: the kernel log of the last 7 days, the error counters of PCIe links and memory, and crash dumps of drivers."
        source="journalctl -k · /sys/bus/pci · /sys/devices/system/edac · /sys/class/devcoredump"
        span={12}
        flush
      >
        {e.kinds === undefined && <p className="muted panel-pad">The kernel log cannot be read, so only the hardware counters are shown.</p>}
        <DataTable
          rows={rows}
          columns={[
            { key: "name", label: "Kind", text: (r) => r.name, wrap: true },
            { key: "count", label: "Count", text: (r) => String(r.count), render: (r) => (r.count ? <StatusBadge variant="warn" label={String(r.count)} /> : "0"), numeric: true },
            { key: "last", label: "Last", text: (r) => (r.last ? ago(r.last, now) : "-") },
            { key: "example", label: "Example", text: (r) => r.example ?? "-", wrap: true },
          ]}
          rowKey={(r) => r.key}
          pageSize={12}
        />
      </Panel>
      {e.driver_offers && (
        <Panel title="Drivers Ubuntu offers" info="Devices for which Ubuntu has a driver package, such as NVIDIA's. The recommended one is Ubuntu's pick." source="ubuntu-drivers devices" span={12} flush>
          <DataTable
            rows={e.driver_offers}
            columns={[
              { key: "device", label: "Device", text: (o) => [o.vendor, o.model].filter(Boolean).join(" ") || "-", wrap: true },
              { key: "drivers", label: "Drivers", text: (o) => o.drivers.map(([n, rec]) => `${n}${rec ? " (recommended)" : ""}`).join(", "), wrap: true },
            ]}
            rowKey={(o) => `${o.vendor}|${o.model}`}
            empty="No device needs an extra driver"
          />
        </Panel>
      )}
    </>
  );
}
