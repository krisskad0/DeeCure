// SPDX-License-Identifier: Apache-2.0
// 17 Horizontal bar list (docs/03-ui-design/03-components/17-horizontal-bar-list.md).

export interface BarListRow {
  label: string;
  value: number;
  /** Shown on the right instead of the value, such as "52 GB". */
  display?: string;
  tone?: "warn";
  mono?: boolean;
}

export function BarList({ rows }: { rows: BarListRow[] }) {
  const max = Math.max(1e-9, ...rows.map((r) => r.value));
  return (
    <ul className="barlist">
      {rows.map((row) => (
        <li key={row.label}>
          <span className={`bl-label ${row.mono ? "mono" : ""}`} title={row.label}>
            {row.label}
          </span>
          <span className="bl-track">
            <span className={`bl-fill ${row.tone === "warn" ? "warn" : ""}`} style={{ width: `${(Math.max(0, row.value) / max) * 100}%` }} />
          </span>
          <span className="bl-value">{row.display ?? row.value}</span>
        </li>
      ))}
    </ul>
  );
}
