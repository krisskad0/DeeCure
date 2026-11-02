// SPDX-License-Identifier: Apache-2.0
// Page header (docs/03-ui-design/02-shell/page-header.md): breadcrumb, title, status, actions, tabs.
// Beginner mode shows fewer tabs (see menuPages in visible.ts).

import { useEffect, useState } from "react";
import { Icon } from "../components/Icon";
import { StatusBadge, type StatusVariant } from "../components/StatusBadge";
import { href, type Route } from "../route";
import { useUi } from "../ui";
import { menuPages } from "../visible";

export function PageHeader(props: {
  route: Route;
  status: StatusVariant | null;
  tabCounts: Record<string, { count: number; critical: boolean }>;
  /** Reads again; the button spins until it is done. */
  onRefresh: () => Promise<void> | void;
}) {
  const { page, tab } = props.route;
  const [busy, setBusy] = useState(false);
  const [updated, setUpdated] = useState<number | null>(null);
  // A new page starts without an "Updated" time.
  useEffect(() => setUpdated(null), [page.id, tab.id]);
  const refresh = async () => {
    if (busy) return;
    setBusy(true);
    try {
      await props.onRefresh();
    } catch {
      // Each panel shows its own error; the button only stops spinning.
    } finally {
      setBusy(false);
      setUpdated(Date.now());
    }
  };
  const tabs = menuPages(useUi().mode, { pageId: page.id, tabId: tab.id }).find((p) => p.id === page.id)?.tabs ?? [tab];
  const single = tabs.length === 1;
  const crumbs = page.group === "Footer" ? [page.label] : [page.group, page.label];
  return (
    <div className="page-header">
      <div className="crumbs">
        {crumbs.map((crumb, i) => (
          // A page can have its group's name (System › System), so the place is the key.
          <span key={i} className="crumb">
            {i > 0 && <Icon name="chev" size={12} />}
            {crumb === page.label ? <a href={href(page.id, page.tabs[0].id)}>{crumb}</a> : crumb}
          </span>
        ))}
        {!single && (
          <span className="crumb">
            <Icon name="chev" size={12} />
            {tab.label}
          </span>
        )}
      </div>
      <div className="title-row">
        <span className="title-icon">
          <Icon name={page.icon} size={18} />
        </span>
        <h1>{page.label}</h1>
        {props.status && <StatusBadge variant={props.status} />}
        <div className="actions">
          {updated !== null && (
            <span className="refresh-note" role="status" title="Numbers are read now. Charts and warnings follow the background reading, taken once a minute.">
              Updated {new Date(updated).toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit", second: "2-digit" })}
            </span>
          )}
          <button className="btn" type="button" onClick={() => void refresh()} disabled={busy} aria-busy={busy} title="Read the numbers again now. Charts, the health score, and warnings use the background reading, taken once a minute.">
            <span className={busy ? "refresh-icon spinning" : "refresh-icon"}>
              <Icon name="ref" size={15} />
            </span>
            <span className="btn-label">{busy ? "Refreshing…" : "Refresh"}</span>
          </button>
        </div>
      </div>
      {single ? (
        <div className="tabs-spacer" />
      ) : (
        <div className="tabs" role="tablist">
          {tabs.map((t) => {
            const count = props.tabCounts[t.id];
            return (
              <a
                key={t.id}
                role="tab"
                aria-selected={t.id === tab.id}
                className={`tab ${t.id === tab.id ? "active" : ""}`}
                href={href(page.id, t.id)}
              >
                {t.label}
                {count && count.count > 0 && <span className={`tab-count ${count.critical ? "crit" : ""}`}>{count.count}</span>}
              </a>
            );
          })}
        </div>
      )}
    </div>
  );
}
