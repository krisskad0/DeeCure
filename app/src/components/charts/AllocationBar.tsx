// SPDX-License-Identifier: Apache-2.0
// 18 Allocation bar (docs/03-ui-design/03-components/18-allocation-bar.md).

export interface Allocation {
  name: string;
  gb: number;
  kind: "s1" | "s2" | "s3" | "s4" | "reclaimable" | "free";
}

export function AllocationBar({ segments, totalGb }: { segments: Allocation[]; totalGb: number }) {
  const fmt = (gb: number) => `${gb % 1 === 0 ? gb.toFixed(0) : gb.toFixed(1)} GB`;
  return (
    <div className="alloc">
      <div className="alloc-bar" role="img" aria-label={segments.map((s) => `${s.name} ${fmt(s.gb)}`).join(", ")}>
        {segments.map((s) => (
          <span key={s.name} className={`alloc-${s.kind}`} style={{ width: `${(s.gb / totalGb) * 100}%` }} title={`${s.name}: ${fmt(s.gb)}`} />
        ))}
      </div>
      <div className="alloc-legend">
        {segments.map((s) => (
          <span key={s.name}>
            <i className={`alloc-${s.kind}`} />
            {s.name} <b>{fmt(s.gb)}</b>
          </span>
        ))}
      </div>
    </div>
  );
}
