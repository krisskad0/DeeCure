// SPDX-License-Identifier: Apache-2.0
// 06 Inline meter (docs/03-ui-design/03-components/06-inline-meter.md).

export function InlineMeter({ percent, label, tone }: { percent: number; label?: string; tone?: "ok" | "warn" | "crit" }) {
  const auto = percent >= 90 ? "crit" : percent >= 80 ? "warn" : "ok";
  const color = tone ?? auto;
  return (
    <span className="meter">
      <span className="meter-track">
        <span className={`meter-fill meter-${color}`} style={{ width: `${Math.max(0, Math.min(100, percent))}%` }} />
      </span>
      <span className="meter-label">{label ?? `${Math.round(percent)}%`}</span>
    </span>
  );
}
