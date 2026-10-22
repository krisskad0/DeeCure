// SPDX-License-Identifier: Apache-2.0
// 15 Gauge and gauge group (docs/03-ui-design/03-components/15-gauge.md).

export interface GaugeProps {
  value: number;
  max?: number;
  /** The text in the middle; defaults to the rounded value. */
  display?: string;
  subtitle?: string;
  status?: "ok" | "warn" | "crit";
  caption?: string;
}

const COLOR = { ok: "var(--good)", warn: "var(--warn)", crit: "var(--crit)" };

export function Gauge({ value, max = 100, display, subtitle, status }: GaugeProps) {
  const f = Math.max(0.001, Math.min(1, value / max));
  const angle = Math.PI - f * Math.PI;
  const end = `${(100 + 80 * Math.cos(angle)).toFixed(2)} ${(96 - 80 * Math.sin(angle)).toFixed(2)}`;
  return (
    <svg className="gauge" viewBox="0 0 200 112" role="img" aria-label={`${display ?? Math.round(value)} ${subtitle ?? ""}`.trim()}>
      <path d="M20 96A80 80 0 0 1 180 96" fill="none" stroke="var(--track)" strokeWidth={14} strokeLinecap="round" />
      <path d={`M20 96A80 80 0 0 1 ${end}`} fill="none" stroke={status ? COLOR[status] : "var(--s1)"} strokeWidth={14} strokeLinecap="round" />
      <text x={100} y={86} textAnchor="middle" className="gauge-value">
        {display ?? Math.round(value)}
      </text>
      {subtitle && (
        <text x={100} y={106} textAnchor="middle" className="gauge-sub">
          {subtitle}
        </text>
      )}
    </svg>
  );
}

export function GaugeGroup({ gauges }: { gauges: GaugeProps[] }) {
  return (
    <div className="gauge-group">
      {gauges.map((g) => (
        <figure key={g.caption ?? g.subtitle}>
          <Gauge {...g} />
          {g.caption && <figcaption>{g.caption}</figcaption>}
        </figure>
      ))}
    </div>
  );
}
