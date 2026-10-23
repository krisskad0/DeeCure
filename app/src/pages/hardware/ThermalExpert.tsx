// SPDX-License-Identifier: Apache-2.0
// Expert sections for temperature (docs/01-product/expert-coverage.md, section 8): every thermal
// zone with the temperatures where the system acts, the cooling devices and their level, and
// heat slowdowns over time next to the CPU temperature.

import { type ThermalInfo, getInventory } from "../../api";
import { DataTable } from "../../components/DataTable";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { tempUnit, tempValue } from "../../units";
import { useLive } from "../../useLive";
import { type History, TimeChart, useHistory } from "../performance/shared";
import { NoData } from "../NoData";

const loadThermal = () => getInventory("thermal");

/** Firmware zone names, in plain words. Unknown ones keep their name. */
const ZONES: Record<string, string> = {
  TCPU: "Processor",
  x86_pkg_temp: "Processor package",
  TCPU_PCI: "Processor (PCI sensor)",
  TSKN: "Case surface (skin)",
  TMEM: "Memory area",
  NGFF: "M.2 slot area",
  INT3400: "Thermal manager (DPTF)",
  acpitz: "Board (ACPI)",
  iwlwifi_1: "Wi-Fi card",
  pch_skylake: "Chipset",
  pch_cannonlake: "Chipset",
  pch_tigerlake: "Chipset",
  pch_alderlake: "Chipset",
  "INT3400 Thermal": "Thermal manager (DPTF)",
};

/** Cooling device kinds, in plain words, with what their level means. */
export function coolingText(c: ThermalInfo["cooling"][number]): { name: string; now: string } {
  if (c.kind === "Processor") return { name: `CPU speed limit (${c.count} CPU${c.count === 1 ? "" : "s"})`, now: c.level === 0 ? "Not limiting" : `Limited, step ${c.level} of ${c.max_level}` };
  if (c.kind === "TCC Offset") return { name: "CPU heat limit lowered", now: c.level === 0 ? "Not lowered" : `${c.level} °C below the chip's maximum` };
  if (c.kind === "intel_powerclamp") return { name: "Forced idle time", now: c.level === 0 ? "None" : `${c.level}% of the time` };
  if (/fan/i.test(c.kind)) return { name: `Fan (${c.kind})`, now: `${c.level} of ${c.max_level}${c.level === c.max_level && c.max_level > 0 ? " (full speed)" : ""}` };
  if (c.kind.startsWith("PCIe_Port_Link_Speed")) return { name: `PCIe link speed for cooling (${c.kind.replace("PCIe_Port_Link_Speed_", "")})`, now: c.level === c.max_level ? "Full speed" : `Slowed, step ${c.level} of ${c.max_level}` };
  return { name: c.kind, now: `${c.level} of ${c.max_level}` };
}

/** Slowdowns per interval from the history of a counter since start (a restart resets it). */
export function perInterval(history: History | undefined): History | undefined {
  const points = history?.series[0];
  if (!history || !points) return undefined;
  const out = points.slice(1).map((p, i) => ({ ...p, value: Math.max(0, p.value - points[i].value) }));
  return { range: history.range, series: [out] };
}

const temp = (c?: number) => (c === undefined ? "-" : `${Math.round(tempValue(c))} ${tempUnit()}`);

export function ThermalExpertSections() {
  const { data: t, error } = useLive(loadThermal, 15_000);
  const throttle = perInterval(useHistory(["cpu.throttle_total"]));
  const cpuTemp = useHistory(["temp.cpu"]);
  if (!t) {
    return (
      <Panel title="Thermal zones" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const first = (z: ThermalInfo["zones"][number], kind: string) => z.trips.find((x) => x.kind === kind)?.celsius;
  return (
    <>
      <Panel
        title="Thermal zones"
        info="Each zone is a place the firmware watches. Slows down: the system lowers speed. Fans: fans speed up. Shuts down: the computer turns off to protect itself."
        source="/sys/class/thermal"
        span={12}
        flush
      >
        <DataTable
          rows={t.zones}
          columns={[
            { key: "zone", label: "Zone", text: (z) => ZONES[z.kind] ?? z.kind },
            { key: "now", label: "Now", text: (z) => temp(z.celsius), numeric: true },
            { key: "passive", label: "Slows down at", text: (z) => temp(first(z, "passive")), numeric: true },
            { key: "active", label: "Fans at", text: (z) => temp(first(z, "active")), numeric: true },
            { key: "critical", label: "Shuts down at", text: (z) => temp(first(z, "critical")), numeric: true },
            {
              key: "margin",
              label: "Room before slowing",
              text: (z) => {
                const p = first(z, "passive");
                return z.celsius === undefined || p === undefined ? "-" : `${Math.round(p - z.celsius)} °C`;
              },
              render: (z) => {
                const p = first(z, "passive");
                if (z.celsius === undefined || p === undefined) return "-";
                const room = Math.round(p - z.celsius);
                return room <= 5 ? <StatusBadge variant="warn" label={`${room} °C`} /> : `${room} °C`;
              },
              numeric: true,
            },
            { key: "kind", label: "Firmware name", text: (z) => `${z.kind} (${z.zone})`, expert: true },
          ]}
          rowKey={(z) => z.zone}
          pageSize={20}
          empty="This computer reports no thermal zones"
        />
      </Panel>
      <Panel title="Cooling" info="What the system does to cool down right now." source="/sys/class/thermal/cooling_device*" span={6} flush>
        <DataTable
          rows={t.cooling.map((c) => ({ ...c, ...coolingText(c) }))}
          columns={[
            { key: "name", label: "Cooling", text: (c) => c.name, wrap: true },
            { key: "now", label: "Now", text: (c) => c.now },
          ]}
          rowKey={(c) => c.kind}
          empty="No cooling devices reported"
        />
      </Panel>
      <Panel title="Heat slowdowns" subtitle="times heat slowed the whole chip" info="Counted by the processor. Slowdowns that rise with the temperature mean the cooling is not enough." source="cpu.throttle_total history" span={6}>
        <TimeChart history={throttle} names={["Heat slowdowns"]} unit="" height={150} />
      </Panel>
      <Panel title="CPU temperature" subtitle="next to the slowdowns" source="temp.cpu history" span={12}>
        <TimeChart history={cpuTemp} names={["CPU"]} unit={tempUnit()} scale={tempValue} height={150} />
      </Panel>
    </>
  );
}
