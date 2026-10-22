// SPDX-License-Identifier: Apache-2.0
// 05 Data table (docs/03-ui-design/03-components/05-data-table.md).

import { type ReactNode, useMemo, useState } from "react";
import { useUi } from "../ui";
import { Button } from "./Button";
import { Icon } from "./Icon";

export interface Column<R> {
  key: string;
  /** Header text. Columns with a label can be sorted. */
  label: string;
  /** Plain text of the cell, used for search, filters, and sorting. */
  text: (row: R) => string;
  /** How the cell looks; defaults to `text`. */
  render?: (row: R) => ReactNode;
  /** `n`: right-aligned, tabular numbers. */
  numeric?: boolean;
  /** `x`: shown in Expert mode only. */
  expert?: boolean;
  /** `w`: wraps instead of staying on one line. */
  wrap?: boolean;
}

const UNITS: Record<string, number> = { KB: 0.001, kB: 0.001, K: 0.001, MB: 1, GB: 1000, TB: 1_000_000, ms: 1, s: 1, "%": 1, h: 1, m: 1, W: 1 };

/** Reads "22 MB", "1.5 GB", "94%", "1,240" as numbers (sizes in MB); `null` for anything else. */
export function sortNumber(text: string): number | null {
  const match = /^(-?[\d,]*\.?\d+)\s*(KB|kB|K|MB|GB|TB|ms|s|%|h|m|W)?$/.exec(text.trim());
  if (!match) return null;
  const value = Number(match[1].replace(/,/g, ""));
  return Number.isFinite(value) ? value * (match[2] ? UNITS[match[2]] : 1) : null;
}

export function compareCells(a: string, b: string): number {
  const x = sortNumber(a);
  const y = sortNumber(b);
  if (x !== null && y !== null) return x - y;
  return a.localeCompare(b, undefined, { numeric: true, sensitivity: "base" });
}

export function DataTable<R>(props: {
  rows: R[];
  columns: Column<R>[];
  rowKey: (row: R) => string;
  search?: { placeholder: string };
  /** Filter chips: a row matches when this column's text starts with the chosen option. */
  filter?: { column: string; options: string[] };
  /** Row selection with one bulk action. */
  selection?: { action: string; onAction: (rows: R[]) => void };
  pageSize?: number;
  /** The full count, shown as "of N" while nothing is filtered. */
  total?: number;
  empty?: string;
  onRowClick?: (row: R) => void;
}) {
  const { mode } = useUi();
  const { rows, rowKey, pageSize = 12 } = props;
  const columns = props.columns.filter((c) => !c.expert || mode === "expert");
  const [query, setQuery] = useState("");
  const [chip, setChip] = useState<string | null>(null);
  const [sort, setSort] = useState<{ key: string; desc: boolean } | null>(null);
  const [page, setPage] = useState(0);
  const [selected, setSelected] = useState<Set<string>>(new Set());

  const filterColumn = props.filter && props.columns.find((c) => c.key === props.filter?.column);
  const shown = useMemo(() => {
    const needle = query.trim().toLowerCase();
    let out = rows.filter((row) => {
      if (needle && !props.columns.some((c) => c.text(row).toLowerCase().includes(needle))) return false;
      if (chip && filterColumn && !filterColumn.text(row).toLowerCase().startsWith(chip.toLowerCase())) return false;
      return true;
    });
    const sortColumn = sort && props.columns.find((c) => c.key === sort.key);
    if (sort && sortColumn) {
      out = [...out].sort((a, b) => compareCells(sortColumn.text(a), sortColumn.text(b)) * (sort.desc ? -1 : 1));
    }
    return out;
  }, [rows, query, chip, sort, props.columns, filterColumn]);

  const pages = Math.max(1, Math.ceil(shown.length / pageSize));
  const current = Math.min(page, pages - 1);
  const visible = shown.slice(current * pageSize, current * pageSize + pageSize);
  const filtered = query.trim() !== "" || chip !== null;
  const count = !filtered && props.total !== undefined ? props.total : shown.length;
  const pageKeys = visible.map(rowKey);
  const allOnPage = pageKeys.length > 0 && pageKeys.every((k) => selected.has(k));

  const toggleSort = (key: string) => {
    setSort((s) => (s?.key === key ? { key, desc: !s.desc } : { key, desc: false }));
    setPage(0);
  };
  const toggle = (key: string) => {
    const next = new Set(selected);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    setSelected(next);
  };
  const togglePage = () => {
    const next = new Set(selected);
    for (const k of pageKeys) {
      if (allOnPage) next.delete(k);
      else next.add(k);
    }
    setSelected(next);
  };

  const tools = props.search || props.filter || props.selection;
  return (
    <div className="dt">
      {tools && (
        <div className="dt-tools">
          {props.search && (
            <label className="dt-search">
              <Icon name="search" size={14} />
              <input
                value={query}
                placeholder={props.search.placeholder}
                aria-label={props.search.placeholder}
                onChange={(e) => {
                  setQuery(e.target.value);
                  setPage(0);
                }}
              />
            </label>
          )}
          {props.filter && (
            <div className="chips" role="group" aria-label="Filter">
              {[null, ...props.filter.options].map((option) => (
                <button
                  key={option ?? "All"}
                  type="button"
                  className="chip"
                  aria-pressed={chip === option}
                  onClick={() => {
                    setChip(option);
                    setPage(0);
                  }}
                >
                  {option ?? "All"}
                </button>
              ))}
            </div>
          )}
          {props.selection && (
            <div className="dt-selection">
              <span className="muted">{selected.size ? `${selected.size} selected` : "Select rows"}</span>
              <Button
                small
                variant={selected.size ? "primary" : "default"}
                disabled={!selected.size}
                onClick={() => props.selection?.onAction(rows.filter((r) => selected.has(rowKey(r))))}
              >
                {props.selection.action}
              </Button>
            </div>
          )}
        </div>
      )}
      <div className="dt-scroll">
        <table>
          <thead>
            <tr>
              {props.selection && (
                <th className="dt-check">
                  <input type="checkbox" aria-label="Select all rows on this page" checked={allOnPage} onChange={togglePage} />
                </th>
              )}
              {columns.map((c) => (
                <th key={c.key} className={c.numeric ? "num" : ""} aria-sort={sort?.key === c.key ? (sort.desc ? "descending" : "ascending") : undefined}>
                  {c.label ? (
                    <button type="button" className="dt-sort" onClick={() => toggleSort(c.key)}>
                      {c.label}
                      {sort?.key === c.key && <span className="dt-arrow">{sort.desc ? "↓" : "↑"}</span>}
                    </button>
                  ) : null}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {visible.length === 0 ? (
              <tr>
                <td className="dt-empty" colSpan={columns.length + (props.selection ? 1 : 0)}>
                  {props.empty ?? "No matching rows"}
                </td>
              </tr>
            ) : (
              visible.map((row) => {
                const key = rowKey(row);
                return (
                  <tr key={key} className={props.onRowClick ? "clickable" : ""} onClick={props.onRowClick ? () => props.onRowClick?.(row) : undefined}>
                    {props.selection && (
                      <td className="dt-check" onClick={(e) => e.stopPropagation()}>
                        <input type="checkbox" aria-label={`Select ${key}`} checked={selected.has(key)} onChange={() => toggle(key)} />
                      </td>
                    )}
                    {columns.map((c) => (
                      <td key={c.key} className={`${c.numeric ? "num" : ""} ${c.wrap ? "wrap" : ""}`}>
                        {c.render ? c.render(row) : c.text(row)}
                      </td>
                    ))}
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>
      {pages > 1 && (
        <div className="dt-foot">
          <span>
            {current * pageSize + 1}–{current * pageSize + visible.length} of {count}
          </span>
          <span className="dt-pager">
            <Button small disabled={current === 0} onClick={() => setPage(current - 1)}>
              Previous
            </Button>
            <Button small disabled={current >= pages - 1} onClick={() => setPage(current + 1)}>
              Next
            </Button>
          </span>
        </div>
      )}
    </div>
  );
}
