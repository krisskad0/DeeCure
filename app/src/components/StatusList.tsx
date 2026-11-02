// SPDX-License-Identifier: Apache-2.0
// 08 Status list (docs/03-ui-design/03-components/08-status-list.md).

import type { IconName } from "../spec/icons";
import { Icon } from "./Icon";
import { StatusBadge, type StatusVariant } from "./StatusBadge";

export interface StatusRow {
  icon: IconName;
  label: string;
  metric?: string;
  status: StatusVariant;
  /** A word for the badge instead of the usual one, such as "Not checked". */
  statusLabel?: string;
  href?: string;
}

/** `grid` shows the rows as small tiles side by side, for a full-width panel. */
export function StatusList({ rows, layout = "list" }: { rows: StatusRow[]; layout?: "list" | "grid" }) {
  return (
    <ul className={layout === "grid" ? "status-list status-grid" : "status-list"}>
      {rows.map((row) => {
        const content = (
          <>
            <span className="sl-label">
              <Icon name={row.icon} size={16} />
              {row.label}
            </span>
            <span className="sl-metric">{row.metric}</span>
            <StatusBadge variant={row.status} label={row.statusLabel} />
          </>
        );
        return (
          <li key={row.label}>
            {row.href ? (
              <a className="sl-row sl-link" href={row.href}>
                {content}
              </a>
            ) : (
              <div className="sl-row">{content}</div>
            )}
          </li>
        );
      })}
    </ul>
  );
}
