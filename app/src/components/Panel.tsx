// SPDX-License-Identifier: Apache-2.0
// 01 Panel (docs/03-ui-design/03-components/01-panel.md): the frame around every widget.

import { ErrorBoundary } from "./ErrorBoundary";
import { type ReactNode, createContext, useContext } from "react";
import { panelMatches } from "../status";
import { useUi } from "../ui";
import { Icon } from "./Icon";
import { StatusBadge, type StatusVariant } from "./StatusBadge";

/** Panel titles on this sub-page that show an open warning, with how serious it is. */
export const FlaggedPanels = createContext<[string, StatusVariant][]>([]);

export type Span = 3 | 4 | 5 | 6 | 7 | 8 | 9 | 12;

export function Panel(props: {
  title?: string;
  subtitle?: string;
  /** Shown in a tooltip on the info icon. */
  info?: string;
  /** Where the data comes from, such as `/proc/stat`. Shown in Expert mode only. */
  source?: string;
  actions?: ReactNode;
  /** No body padding: for tables, lists, checklists, timelines, and forms. */
  flush?: boolean;
  span?: Span;
  children: ReactNode;
}) {
  const { mode } = useUi();
  const flagged = useContext(FlaggedPanels);
  const title = props.title;
  const flag = title ? flagged.find(([wanted]) => panelMatches(title, wanted))?.[1] : undefined;
  const hasHead = props.title || props.actions;
  return (
    <section className={`panel span-${props.span ?? 12}${flag ? ` panel-flag panel-flag-${flag}` : ""}`} data-panel={title}>
      {hasHead && (
        <div className="panel-head">
          {props.title && <h2>{props.title}</h2>}
          {flag && <StatusBadge variant={flag} label={flag === "crit" ? "Problem here" : flag === "warn" ? "Warning here" : "Note here"} />}
          {props.subtitle && <span className="panel-sub">{props.subtitle}</span>}
          {props.info && (
            <span className="info" tabIndex={0} aria-label={props.info}>
              <Icon name="i" size={16} />
              <span className="tip" role="tooltip">
                {props.info}
              </span>
            </span>
          )}
          <span className="panel-right">
            {mode === "expert" && props.source && (
              <code className="source" title={props.source}>
                {props.source}
              </code>
            )}
            {props.actions}
          </span>
        </div>
      )}
      <div className={props.flush ? "panel-body flush" : "panel-body"}>
        <ErrorBoundary inline what={props.title ?? "This panel"}>
          {props.children}
        </ErrorBoundary>
      </div>
    </section>
  );
}
