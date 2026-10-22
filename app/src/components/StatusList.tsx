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
  href?: string;
}

export function StatusList({ rows }: { rows: StatusRow[] }) {
  return (
    <ul className="status-list">
      {rows.map((row) => {
        const content = (
          <>
            <span className="sl-label">
              <Icon name={row.icon} size={16} />
              {row.label}
            </span>
            <span className="sl-metric">{row.metric}</span>
            <StatusBadge variant={row.status} />
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
