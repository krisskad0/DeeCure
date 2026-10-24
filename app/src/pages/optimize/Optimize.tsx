// SPDX-License-Identifier: Apache-2.0
// Optimize (docs/03-ui-design/05-pages/23-optimize.md): every improvement a measurement on this
// computer shows is needed, each with its numbers. Fixes open the fix preview; advice says what to
// do by hand. Everything checked without a result is listed too, with what was measured.

import { type OptimizeCheck, getInventory } from "../../api";
import { AlertBanner } from "../../components/AlertBanner";
import { Button } from "../../components/Button";
import { type Column, DataTable } from "../../components/DataTable";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { useFix } from "../../shell/FixDrawer";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const load = () => getInventory("optimize");

const CHECKED: Column<OptimizeCheck>[] = [
  { key: "title", label: "Check", text: (c) => c.title },
  { key: "measured", label: "What DeeCure measured", text: (c) => c.measured },
];

/** Offered items, fixes first, in the agent's order within each kind. */
export function offered(checks: OptimizeCheck[]): OptimizeCheck[] {
  return [...checks.filter((c) => c.offered && c.kind === "fix"), ...checks.filter((c) => c.offered && c.kind === "advice")];
}

function Item({ c }: { c: OptimizeCheck }) {
  const openFix = useFix();
  return (
    <li className="optimize-item">
      <div className="optimize-text">
        <strong>{c.title}</strong>
        <span className="muted">{c.measured}</span>
        {c.advice && <span>{c.advice}</span>}
      </div>
      <span className="btn-row">
        {c.kind === "advice" ? (
          <StatusBadge variant="warn" label="Advice" />
        ) : (
          <>
            {c.access === "admin" && <StatusBadge variant="info" label="Needs your password" />}
            {c.action && (
              <Button small variant="primary" onClick={() => openFix(c.action as string, c.subject)}>
                Preview
              </Button>
            )}
          </>
        )}
      </span>
    </li>
  );
}

export function OptimizeAll() {
  // The checks read boot times and several system lists, so once a minute is enough.
  const { data, error } = useLive<OptimizeCheck[]>(load, 60_000);
  if (!data) {
    return (
      <Panel title="Improvements" span={12}>
        <NoData error={error} waiting="Checking this computer…" />
      </Panel>
    );
  }
  const now = offered(data);
  const fine = data.filter((c) => !c.offered);
  return (
    <>
      <Panel title="Improvements" info="Offered only when a measurement on this computer shows it is needed. Each fix shows a preview first, and can be undone where undo is possible." span={12}>
        {now.length === 0 ? (
          <AlertBanner tone="ok" title="Nothing to improve now" text="Every check below passed." />
        ) : (
          <ul className="optimize-list">
            {now.map((c) => (
              <Item key={c.id} c={c} />
            ))}
          </ul>
        )}
      </Panel>
      <Panel title={`Checked, nothing needed (${fine.length})`} span={12} flush>
        <details>
          <summary className="optimize-summary">Show what was checked and measured</summary>
          <DataTable rows={fine} columns={CHECKED} rowKey={(c) => c.id} empty="No other checks" />
        </details>
      </Panel>
    </>
  );
}
