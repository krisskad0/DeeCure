// SPDX-License-Identifier: Apache-2.0
// 07 Key-value list (docs/03-ui-design/03-components/07-key-value-list.md).

import type { ReactNode } from "react";

export function KeyValueList({ rows }: { rows: [string, ReactNode][] }) {
  return (
    <dl className="kv">
      {rows.map(([key, value]) => (
        <div className="kv-row" key={key}>
          <dt>{key}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
}
