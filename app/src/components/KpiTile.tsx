// SPDX-License-Identifier: Apache-2.0
// 04 KPI tile and KPI row (docs/03-ui-design/03-components/04-kpi-tile.md).

import type { ReactNode } from "react";
import { Sparkline, type SeriesSlot } from "./Sparkline";
import { StatusBadge, type StatusVariant } from "./StatusBadge";

export type KpiVisual = { kind: "spark"; data: number[]; slot?: SeriesSlot } | { kind: "bar"; percent: number };

export interface Kpi {
  label: string;
  value: string;
  unit?: string;
  status?: StatusVariant;
  subtext?: string;
  visual?: KpiVisual;
  /** Where the tile leads, such as `#/storage/overview`. */
  href?: string;
}

export function KpiRow({ tiles }: { tiles: Kpi[] }) {
  return (
    <div className="kpi-row span-12">
      {tiles.map((tile) => (
        <KpiTile key={tile.label} {...tile} />
      ))}
    </div>
  );
}

export function KpiTile({ label, value, unit, status = "ok", subtext, visual, href }: Kpi) {
  const body: ReactNode = (
    <>
      <div className="kpi-label">
        <span>{label}</span>
        {status !== "ok" && <StatusBadge variant={status} />}
      </div>
      <div className="kpi-value">
        {value}
        {unit && <span className="kpi-unit">{unit}</span>}
      </div>
      <div className="kpi-sub">{subtext || " "}</div>
      {visual?.kind === "spark" && <Sparkline data={visual.data} slot={visual.slot} />}
      {visual?.kind === "bar" && (
        <div className="usage" aria-hidden="true">
          <span
            className={status === "crit" ? "fill-crit" : status === "warn" ? "fill-warn" : ""}
            style={{ width: `${Math.max(0, Math.min(100, visual.percent))}%` }}
          />
        </div>
      )}
    </>
  );
  return href ? (
    <a className="kpi kpi-link" href={href}>
      {body}
    </a>
  ) : (
    <div className="kpi">{body}</div>
  );
}
