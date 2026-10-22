// SPDX-License-Identifier: Apache-2.0
// Hardware › Sensors (docs/03-ui-design/05-pages/06-hardware.md), with real data.

import type { Metric } from "../../api";
import { AlertBanner } from "../../components/AlertBanner";
import { type Column, DataTable } from "../../components/DataTable";
import { Panel } from "../../components/Panel";
import { GaugeGroup } from "../../components/charts/Gauge";
import { pick, pickAll } from "../../metrics";
import { TimeChart, useHistory, useLatest } from "../performance/shared";

/** The hottest sensor of each chip, hottest chip first. */
export function hottestPerChip(temps: Metric[]): Metric[] {
  const best = new Map<string, Metric>();
  for (const t of temps) {
    const chip = t.subject?.split("/")[0] ?? "";
    const old = best.get(chip);
    if (!old || t.value > old.value) best.set(chip, t);
  }
  return [...best.values()].sort((a, b) => b.value - a.value);
}

const FANS: Column<Metric>[] = [
  { key: "fan", label: "Fan", text: (f) => f.subject ?? "fan" },
  { key: "rpm", label: "Speed", text: (f) => (f.value ? `${Math.round(f.value)} RPM` : "0 RPM (stopped)"), numeric: true },
];

export function HardwareSensors() {
  const latest = useLatest();
  const temps = pickAll(latest, "temp");
  const fans = pickAll(latest, "fan");
  const nvme = temps.find((t) => t.subject?.startsWith("nvme/"))?.subject;
  const battery = temps.find((t) => t.subject?.startsWith("BAT"))?.subject;
  const series = [{ name: "temp.cpu" }, ...(nvme ? [{ name: "temp", subject: nvme }] : []), ...(battery ? [{ name: "temp", subject: battery }] : [])];
  const history = useHistory(series);
  const cpu = pick(latest, "temp.cpu");
  const chips = hottestPerChip(temps).slice(0, 8);
  return (
    <>
      <Panel title="Temperatures" subtitle="hottest sensor of each chip" source="/sys/class/hwmon" span={12}>
        {chips.length ? (
          <GaugeGroup
            gauges={chips.map((t) => ({
              value: t.value,
              display: `${Math.round(t.value)} °C`,
              subtitle: t.subject?.split("/")[1] ?? "",
              caption: t.subject?.split("/")[0] ?? "sensor",
              status: t.value >= 90 ? "crit" : t.value >= 80 ? "warn" : "ok",
            }))}
          />
        ) : (
          <div className="empty muted">{latest ? "This computer exposes no temperature sensors to Linux." : "Loading…"}</div>
        )}
      </Panel>
      <Panel title="Temperature history" source="/sys/class/hwmon" span={12}>
        <TimeChart
          history={history}
          names={["CPU", ...(nvme ? ["NVMe"] : []), ...(battery ? ["Battery"] : [])]}
          unit="°C"
          max={cpu !== undefined && cpu > 85 ? "auto" : 100}
          threshold={{ value: 95, label: "Throttle ~95 °C" }}
        />
      </Panel>
      {fans.length ? (
        <Panel title="Fans" span={12} flush>
          <DataTable rows={fans} columns={FANS} rowKey={(f) => f.subject ?? "fan"} />
        </Panel>
      ) : (
        latest && <AlertBanner tone="info" title="Fan speed not reported" text="This computer's embedded controller does not expose fan speed to Linux." />
      )}
    </>
  );
}
