// SPDX-License-Identifier: Apache-2.0
// Page header (docs/03-ui-design/02-shell/page-header.md): breadcrumb, title, status, actions, tabs.
// Beginner mode shows fewer tabs (see menuPages in visible.ts).

import { Icon } from "../components/Icon";
import { StatusBadge, type StatusVariant } from "../components/StatusBadge";
import { href, type Route } from "../route";
import { useUi } from "../ui";
import { menuPages } from "../visible";

export function PageHeader(props: {
  route: Route;
  status: StatusVariant | null;
  tabCounts: Record<string, { count: number; critical: boolean }>;
  onRefresh: () => void;
}) {
  const { page, tab } = props.route;
  const tabs = menuPages(useUi().mode, { pageId: page.id, tabId: tab.id }).find((p) => p.id === page.id)?.tabs ?? [tab];
  const single = tabs.length === 1;
  const crumbs = page.group === "Footer" ? [page.label] : [page.group, page.label];
  return (
    <div className="page-header">
      <div className="crumbs">
        {crumbs.map((crumb, i) => (
          <span key={crumb} className="crumb">
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
          <button className="btn" type="button" onClick={props.onRefresh}>
            <Icon name="ref" size={15} />
            <span className="btn-label">Refresh</span>
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
