// SPDX-License-Identifier: Apache-2.0
// Hardware › Memory (docs/03-ui-design/05-pages/06-hardware.md), with real data.
// Module type, speed, and slots come from dmidecode through the admin helper (Full access).

import { Guard } from "../../components/ErrorBoundary";
import { atLeast, useUi } from "../../ui";
import { MemoryErrors } from "../performance/MemoryExpert";
import { LinkButton } from "../../components/Button";
import { type Check, Checklist } from "../../components/Checklist";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { bytes, pick } from "../../metrics";
import { href } from "../../route";
import { useHistory, useLatest } from "../performance/shared";
import { NoData } from "../NoData";
import { useHardware } from "./shared";
import { AdminRead } from "../../components/AdminRead";

const FULL = <span className="muted">See below</span>;

export function HardwareMemory() {
  const { mode } = useUi();
  const { data: hw, error } = useHardware();
  const latest = useLatest();
  const pressure = useHistory(["pressure.memory"], undefined, "24h");
  if (!hw) {
    return (
      <Panel title="Modules" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const usable = pick(latest, "memory.total");
  const peak = pressure?.series[0]?.reduce((m, p) => Math.max(m, p.value), 0);
  const checks: Check[] = [
    peak === undefined
      ? { state: "na", title: "Memory pressure today", detail: "Not measured yet" }
      : peak > 20
        ? { state: "warn", title: `Memory pressure reached ${peak.toFixed(1)}% today`, detail: "Apps waited for memory. Close some, or add memory." }
        : { state: "pass", title: "No memory pressure stalls above 20% today" },
    { state: "na", title: "Out-of-memory kills", detail: "See the kernel log", action: <LinkButton to={href("logs", "kernel")}>View</LinkButton> },
    hw.ecc_reporting ? { state: "pass", title: "Error correction (ECC) reporting", detail: "The kernel reports memory errors (EDAC)" } : { state: "na", title: "Error correction (ECC) reporting", detail: "Not available on this memory type" },
    { state: "na", title: "Full memory test", detail: "Restart, hold Shift, and choose Memory test (memtest86+)" },
  ];
  return (
    <>
      <Panel title="Modules" source="/proc/meminfo" span={6} flush>
        <KeyValueList
          rows={[
            ["Total", hw.memory_total ? `${bytes(hw.memory_total)}${usable ? ` (${bytes(usable)} usable)` : ""}` : "-"],
            ["Type and speed", FULL],
            ["Form factor", FULL],
            ["Slots", FULL],
            ["ECC", hw.ecc_reporting ? "Yes" : "No"],
          ]}
        />
      </Panel>
      <Panel title="Health" span={6} flush>
        <Checklist checks={checks} />
      </Panel>
      <Panel title="Memory modules" source="dmidecode -t memory" span={12}>
        <AdminRead what="memory_modules" label="Read the memory modules (asks for your password)">
          {(text) => {
            const mods = JSON.parse(text) as Record<string, string>[];
            const filled = mods.filter((m) => m.Filled === "true");
            return (
              <KeyValueList
                rows={[
                  ["Slots", `${mods.length} (${mods.length - filled.length} free)`],
                  ...filled.map((m, i): [string, string] => [m.Locator || `Module ${i + 1}`, [m.Size, m.Type, m["Configured Memory Speed"] || m.Speed, m["Form Factor"], m.Manufacturer].filter((x) => x && x !== "Unknown").join(" · ")]),
                ]}
              />
            );
          }}
        </AdminRead>
      </Panel>
      {atLeast(mode, "expert") && <Guard><MemoryErrors /></Guard>}
    </>
  );
}
