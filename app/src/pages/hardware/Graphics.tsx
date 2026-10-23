// SPDX-License-Identifier: Apache-2.0
// Hardware › Graphics (docs/03-ui-design/05-pages/06-hardware.md), with real data.

import { temp as tempText } from "../../units";
import { type Check, Checklist } from "../../components/Checklist";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { NoData } from "../NoData";
import { useGpus } from "./shared";

/** Kernel drivers that give hardware-accelerated graphics. */
const ACCELERATED = new Set(["i915", "xe", "amdgpu", "radeon", "nouveau", "nvidia"]);

export function HardwareGraphics() {
  const gpus = useGpus();
  if (!gpus?.length) {
    return (
      <Panel title="Adapter" span={12}>
        <NoData error={null} waiting={gpus ? "No graphics card was found." : "Loading…"} />
      </Panel>
    );
  }
  return (
    <>
      {gpus.map((g) => {
        const checks: Check[] = [
          g.driver ? { state: "pass", title: "Kernel driver loaded", detail: g.driver } : { state: "fail", title: "No kernel driver loaded", detail: "The card works only in basic mode" },
          g.driver && ACCELERATED.has(g.driver) ? { state: "pass", title: "Hardware acceleration available" } : { state: "warn", title: "Hardware acceleration not available", detail: "Basic display driver only" },
          g.temperature === undefined ? { state: "na", title: "GPU temperature", detail: `Not exposed by ${g.driver ?? "the driver"}` } : { state: "pass", title: "GPU temperature reported", detail: tempText(g.temperature) },
        ];
        return [
          <Panel key={`${g.card}-a`} title={gpus.length > 1 ? `Adapter · ${g.card}` : "Adapter"} source="/sys/class/drm · pci.ids" span={6} flush>
            <KeyValueList
              rows={[
                ["GPU", g.model ?? "Unknown"],
                ["Maker", g.vendor ?? "-"],
                ["Kernel driver", g.driver ? <code key="d">{g.driver}</code> : "None"],
                ["PCI ID", g.pci_id ? <code key="p">{g.pci_id}</code> : "-"],
                ["Main card", g.boot_vga ? "Yes" : "No"],
              ]}
            />
          </Panel>,
          <Panel key={`${g.card}-c`} title="Driver checks" span={6} flush>
            <Checklist checks={checks} />
          </Panel>,
        ];
      })}
    </>
  );
}
