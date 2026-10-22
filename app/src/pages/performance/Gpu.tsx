// SPDX-License-Identifier: Apache-2.0
// Performance › GPU (docs/03-ui-design/05-pages/02-performance.md), with real data.
// Load history is not stored yet: drivers that report load without admin rights are rare.

import { type Gpu, getInventory } from "../../api";
import { AlertBanner } from "../../components/AlertBanner";
import { LinkButton } from "../../components/Button";
import { type Column, DataTable } from "../../components/DataTable";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { bytes } from "../../metrics";
import { href } from "../../route";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadGpus = () => getInventory("gpu");

/** "Raptor Lake-P [Iris Xe Graphics]" → "Iris Xe Graphics". */
function shortName(gpu: Gpu): string {
  const bracket = gpu.model?.match(/\[(.+)\]/)?.[1];
  return bracket ?? gpu.model ?? gpu.pci_id ?? gpu.card;
}

const COLUMNS: Column<Gpu>[] = [
  { key: "card", label: "Card", text: (g) => g.card, render: (g) => <code>{g.card}</code> },
  { key: "model", label: "Model", text: (g) => g.model ?? "Unknown", wrap: true },
  { key: "vendor", label: "Maker", text: (g) => g.vendor ?? "—" },
  { key: "driver", label: "Driver", text: (g) => g.driver ?? "none" },
  { key: "boot", label: "Main", text: (g) => (g.boot_vga ? "Yes" : "No") },
  { key: "pci", label: "PCI ID", text: (g) => g.pci_id ?? "—", expert: true },
];

export function PerformanceGpu() {
  const { data: gpus, error } = useLive(loadGpus, 10_000);
  if (!gpus || gpus.length === 0) {
    return (
      <Panel title="Graphics" span={12}>
        <NoData error={error} waiting={gpus ? "No graphics card was found." : "Loading…"} />
      </Panel>
    );
  }
  const gpu = gpus[0];
  const codeName = gpu.model?.replace(/\s*\[.*\]/, "");
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Device", value: shortName(gpu), subtext: [gpu.vendor?.replace(" Corporation", ""), codeName !== shortName(gpu) ? codeName : undefined].filter(Boolean).join(" · ") },
          gpu.busy_percent === undefined
            ? { label: "Utilization", value: "N/A", subtext: `Not reported by ${gpu.driver ?? "the driver"} without admin rights` }
            : { label: "Utilization", value: String(Math.round(gpu.busy_percent)), unit: "%", visual: { kind: "bar", percent: gpu.busy_percent } },
          gpu.vram_total
            ? { label: "Memory", value: bytes(gpu.vram_used ?? 0).split(" ")[0], unit: bytes(gpu.vram_used ?? 0).split(" ")[1], subtext: `of ${bytes(gpu.vram_total)}`, visual: { kind: "bar", percent: ((gpu.vram_used ?? 0) / gpu.vram_total) * 100 } }
            : { label: "Memory", value: "Shared", subtext: "Uses system RAM" },
          gpu.temperature === undefined
            ? { label: "Temperature", value: "N/A", subtext: `Not reported by ${gpu.driver ?? "the driver"}` }
            : { label: "Temperature", value: String(Math.round(gpu.temperature)), unit: "°C" },
          { label: "Driver", value: gpu.driver ?? "None", subtext: gpu.driver ? "" : "No driver is loaded for this card" },
        ]}
      />
      {gpu.temperature === undefined && (
        <AlertBanner
          tone="info"
          title="GPU temperature is not exposed"
          text={`The ${gpu.driver ?? "graphics"} driver does not report its own sensor. On most laptops the GPU shares the CPU package sensor.`}
          actions={<LinkButton to={href("hardware", "sensors")}>Open sensors</LinkButton>}
        />
      )}
      <Panel title="Graphics cards" subtitle={`${gpus.length} found`} source="/sys/class/drm · pci.ids" span={12} flush>
        <DataTable rows={gpus} columns={COLUMNS} rowKey={(g) => g.card} />
      </Panel>
    </>
  );
}
