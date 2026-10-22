// SPDX-License-Identifier: Apache-2.0
// Performance › Pressure (PSI) (docs/03-ui-design/05-pages/02-performance.md), with real data.

import { KeyValueList } from "../../components/KeyValueList";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { pick, when } from "../../metrics";
import { type History, TimeChart, useHistory, useLatest } from "./shared";

const NAMES = ["CPU", "Memory", "I/O"];

/** The highest point in the range, with its series and time. */
function peak(history: History | undefined): { name: string; value: number; at: number } | null {
  let best: { name: string; value: number; at: number } | null = null;
  history?.series.forEach((points, k) => {
    for (const p of points) if (!best || p.value > best.value) best = { name: NAMES[k], value: p.value, at: p.at };
  });
  return best;
}

export function PerformancePressure() {
  const latest = useLatest();
  const history = useHistory(["pressure.cpu", "pressure.memory", "pressure.io"]);
  const tile = (label: string, name: string) => {
    const v = pick(latest, name);
    return { label, value: v === undefined ? "N/A" : v.toFixed(1), unit: v === undefined ? undefined : "%" };
  };
  const top = peak(history);
  const tone = !top ? "off" : top.value > 20 ? "crit" : top.value >= 5 ? "warn" : "ok";
  if (latest && pick(latest, "pressure.cpu") === undefined) {
    return (
      <Panel title="Pressure stall information" span={12}>
        <div className="empty muted">This kernel does not report pressure (PSI). It needs Linux 4.20 or newer with PSI turned on.</div>
      </Panel>
    );
  }
  return (
    <>
      <KpiRow
        tiles={[
          tile("CPU some", "pressure.cpu"),
          tile("Memory some", "pressure.memory"),
          tile("Memory full", "pressure.memory_full"),
          tile("I/O some", "pressure.io"),
          tile("I/O full", "pressure.io_full"),
        ]}
      />
      <Panel
        title="Pressure stall information"
        info="Share of time tasks waited for CPU, memory or disk. The most reliable sign of a slow system."
        source="/proc/pressure/{cpu,memory,io}"
        span={12}
      >
        <TimeChart history={history} names={NAMES} unit="%" height={240} />
      </Panel>
      <Panel title="Thresholds" span={12} flush>
        <KeyValueList
          rows={[
            ["Healthy", "below 5% (avg10)"],
            ["Noticeable slowdown", "5–20%"],
            ["Severe", "above 20%"],
            ["Peak in range", top ? <StatusBadge variant={tone} label={`${top.name} ${top.value.toFixed(1)}% · ${when(top.at)}`} /> : "—"],
          ]}
        />
      </Panel>
    </>
  );
}
