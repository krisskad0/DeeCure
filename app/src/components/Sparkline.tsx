// SPDX-License-Identifier: Apache-2.0
// Sparkline for KPI tiles (docs/03-ui-design/03-components/04-kpi-tile.md):
// min-max normalized line with a faint area, stretched to the tile's width.

export type SeriesSlot = 1 | 2 | 3 | 4 | 5;

export function Sparkline({ data, slot = 1, height = 28 }: { data: number[]; slot?: SeriesSlot; height?: number }) {
  if (data.length < 2) return <div style={{ height }} />;
  const min = Math.min(...data);
  const max = Math.max(...data);
  const spread = max - min || 1;
  const points = data.map((v, i) => [(i / (data.length - 1)) * 100, height - 1 - ((v - min) / spread) * (height - 2)]);
  const line = points.map(([x, y], i) => `${i ? "L" : "M"}${x.toFixed(2)} ${y.toFixed(2)}`).join("");
  const color = `var(--s${slot})`;
  return (
    <svg className="spark" viewBox={`0 0 100 ${height}`} preserveAspectRatio="none" height={height} aria-hidden="true">
      <path d={`${line}L100 ${height}L0 ${height}Z`} fill={color} fillOpacity={0.12} />
      <path d={line} fill="none" stroke={color} strokeWidth={1.6} vectorEffect="non-scaling-stroke" strokeLinejoin="round" />
    </svg>
  );
}
