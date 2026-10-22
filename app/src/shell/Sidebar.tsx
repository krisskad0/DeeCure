// SPDX-License-Identifier: Apache-2.0
// Sidebar (docs/03-ui-design/02-shell/sidebar.md): groups of pages, each with a sub-page tree.

import { useState } from "react";
import type { StatusVariant } from "../components/StatusBadge";
import { Icon } from "../components/Icon";
import { href, type Route } from "../route";
import { PAGES, type PageSpec } from "../spec/pages";

const GROUPS = [...new Set(PAGES.filter((p) => p.group !== "Footer").map((p) => p.group))];
const SETTINGS = PAGES.find((p) => p.group === "Footer");

export function Sidebar(props: {
  route: Route;
  statusOf: (pageId: string) => StatusVariant | null;
  issueCount: number;
  rail: boolean;
  onRail: (rail: boolean) => void;
  drawerOpen: boolean;
  onNavigate: () => void;
}) {
  const { route, rail } = props;
  // Which pages have their sub-page tree open. The current page starts open.
  const [open, setOpen] = useState<Record<string, boolean>>({ [route.page.id]: true });

  const choosePage = (page: PageSpec, event: React.MouseEvent) => {
    if (page.id === route.page.id && !rail) {
      event.preventDefault();
      setOpen({ ...open, [page.id]: !open[page.id] });
      return;
    }
    setOpen({ ...open, [page.id]: true });
    props.onNavigate();
  };

  const item = (page: PageSpec) => {
    const active = page.id === route.page.id;
    const status = props.statusOf(page.id);
    const hasTree = page.tabs.length > 1;
    const treeOpen = !rail && hasTree && (open[page.id] ?? false);
    return (
      <li key={page.id}>
        <a
          className={`nav-item ${active ? "active" : ""}`}
          href={href(page.id, page.tabs[0].id)}
          aria-current={active ? "page" : undefined}
          title={rail ? page.label : undefined}
          onClick={(event) => choosePage(page, event)}
        >
          <Icon name={page.icon} size={17} />
          <span className="nav-label">{page.label}</span>
          {page.id === "overview" && props.issueCount > 0 ? (
            <span className="pill">{props.issueCount}</span>
          ) : (
            (status === "warn" || status === "crit") && <span className={`nav-dot nav-dot-${status}`} aria-label={status === "crit" ? "Critical" : "Warning"} />
          )}
          {hasTree && !rail && <span className={`chev ${treeOpen ? "open" : ""}`}><Icon name="chev" size={14} /></span>}
        </a>
        {treeOpen && (
          <ul className="subtree">
            {page.tabs.map((tab) => (
              <li key={tab.id}>
                <a
                  className={`sub-item ${active && tab.id === route.tab.id ? "active" : ""}`}
                  href={href(page.id, tab.id)}
                  onClick={props.onNavigate}
                >
                  {tab.label}
                </a>
              </li>
            ))}
          </ul>
        )}
      </li>
    );
  };

  return (
    <nav className={`sidebar ${rail ? "rail" : ""} ${props.drawerOpen ? "drawer-open" : ""}`} aria-label="Pages">
      <div className="nav-scroll">
        {GROUPS.map((group) => (
          <div className="nav-group" key={group}>
            <div className="group-label">{group}</div>
            <ul>{PAGES.filter((p) => p.group === group).map(item)}</ul>
          </div>
        ))}
      </div>
      <div className="nav-footer">
        <ul>{SETTINGS && item(SETTINGS)}</ul>
        <button className="nav-item collapse" type="button" onClick={() => props.onRail(!rail)}>
          <Icon name="rail" size={17} />
          <span className="nav-label">{rail ? "Expand" : "Collapse"}</span>
        </button>
      </div>
    </nav>
  );
}
