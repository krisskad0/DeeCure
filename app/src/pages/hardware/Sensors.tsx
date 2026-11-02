// SPDX-License-Identifier: Apache-2.0
// Hardware › Sensors (docs/03-ui-design/05-pages/06-hardware.md), with real data.

import { Guard } from "../../components/ErrorBoundary";
import { atLeast, useUi } from "../../ui";
import { ThermalExpertSections } from "./ThermalExpert";
import { temp as tempText, tempUnit, tempValue } from "../../units";
import type { Metric } from "../../api";
import { AlertBanner } from "../../components/AlertBanner";
import { Panel } from "../../components/Panel";
import { GaugeGroup } from "../../components/charts/Gauge";
import { heatStatus, pick, pickAll } from "../../metrics";
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

const fanText = (f: Metric) => (f.value ? `${Math.round(f.value)} RPM` : "0 RPM (stopped)");

export function HardwareSensors() {
  const { mode } = useUi();
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
              display: tempText(t.value),
              subtitle: t.subject?.split("/")[1] ?? "",
              caption: t.subject?.split("/")[0] ?? "sensor",
              status: heatStatus(t.value, pick(latest, "temp.limit", t.subject), pick(latest, "temp.critical", t.subject)),
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
          unit={tempUnit()}
          scale={tempValue}
          max={cpu !== undefined && cpu > 85 ? "auto" : tempValue(100)}
          threshold={{ value: tempValue(95), label: `Throttle ~${tempText(95)}` }}
        />
      </Panel>
      {fans.length ? (
        <Panel title="Fans" subtitle={fans.every((f) => !f.value) ? "all stopped: the computer is cool enough to stay quiet" : undefined} span={12}>
          <div className="kn-chips">
            {fans.map((f) => (
              <span key={f.subject ?? "fan"} className="kn-chip">
                {f.subject ?? "fan"} <b>{fanText(f)}</b>
              </span>
            ))}
          </div>
        </Panel>
      ) : (
        latest && <AlertBanner tone="info" title="Fan speed not reported" text="This computer's embedded controller does not expose fan speed to Linux." />
      )}
      {atLeast(mode, "expert") && <Guard><ThermalExpertSections /></Guard>}
    </>
  );
}
