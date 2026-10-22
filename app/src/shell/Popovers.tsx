// SPDX-License-Identifier: Apache-2.0
// The notifications and machines popovers (docs/03-ui-design/02-shell/overlays.md).

import type { Finding, Machine } from "../api";
import { Icon } from "../components/Icon";
import { href } from "../route";
import { PAGES } from "../spec/pages";
import { pageOf } from "../status";
import { ago } from "../time";
import { Popover } from "./Popover";

const DOT: Record<Finding["severity"], string> = { critical: "crit", warning: "warn", info: "info" };

export function NotificationsPopover(props: { anchor: HTMLElement; findings: Finding[]; now: number; onClose: () => void }) {
  const { findings, onClose } = props;
  return (
    <Popover anchor={props.anchor} align="right" label="Notifications" onClose={onClose}>
      <div className="pop-head">
        <strong>Notifications</strong>
        <span className="muted">{findings.length} open</span>
      </div>
      {findings.length === 0 ? (
        <div className="pop-empty muted">Nothing needs your attention.</div>
      ) : (
        <ul>
          {findings.slice(0, 6).map((f) => {
            const page = PAGES.find((p) => p.id === pageOf(f));
            return (
              <li key={`${f.rule}|${f.subject ?? ""}`}>
                <a className="pop-item" href={page ? href(page.id, page.tabs[0].id) : href("overview", "issues")} onClick={onClose}>
                  <span className={`sev-dot sev-${DOT[f.severity]}`} aria-label={f.severity} />
                  <span>
                    <span className="pop-title">{f.title}</span>
                    <span className="pop-sub">
                      {page?.label ?? "Overview"} · {ago(f.first_seen, props.now)}
                    </span>
                  </span>
                </a>
              </li>
            );
          })}
        </ul>
      )}
      <a className="pop-foot" href={href("overview", "issues")} onClick={onClose}>
        View all issues
      </a>
    </Popover>
  );
}

export function MachinesPopover(props: { anchor: HTMLElement; machine: Machine | null; onClose: () => void; onPlanned: () => void }) {
  return (
    <Popover anchor={props.anchor} align="left" label="Machines" onClose={props.onClose}>
      <div className="pop-head">
        <strong>Machines</strong>
      </div>
      <ul>
        <li>
          <button className="pop-item" type="button" onClick={props.onClose}>
            <span className="sev-dot sev-ok" aria-hidden="true" />
            <span>
              <span className="pop-title">{props.machine?.hostname ?? "This computer"}</span>
              <span className="pop-sub">This computer · {props.machine?.os ?? "unknown OS"}</span>
            </span>
          </button>
        </li>
        <li>
          <button className="pop-item" type="button" onClick={props.onPlanned}>
            <span className="pop-icon">
              <Icon name="ext" size={14} />
            </span>
            <span>
              <span className="pop-title">Add a machine</span>
              <span className="pop-sub">Planned · small office view</span>
            </span>
          </button>
        </li>
      </ul>
    </Popover>
  );
}
