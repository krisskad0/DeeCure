// SPDX-License-Identifier: Apache-2.0
// Overview › Summary: how many Optimize items wait now, with a way straight there. Expert only,
// like the Optimize page itself (docs/01-product/view-levels.md). It reads the same scan as the
// Optimize page, so opening both never checks the computer twice.

import { LinkButton } from "../../components/Button";
import { Icon } from "../../components/Icon";
import { Panel } from "../../components/Panel";
import { href } from "../../route";
import { Radar, areaOf, offered } from "../optimize/Optimize";
import { useOptimizeScan } from "../optimize/scan";
import { plural } from "./shared";

export function OptimizeStrip() {
  const { checks, error, startedAt } = useOptimizeScan();
  const todo = checks ? offered(checks) : [];
  const fixes = todo.filter((c) => c.kind === "fix").length;
  const open = href("optimize", "all");
  return (
    <Panel title="Optimize" subtitle={checks ? `${plural(checks.length, "check")} on this computer` : undefined} actions={<LinkButton to={open}>Open Optimize</LinkButton>} span={12}>
      {!checks ? (
        error && startedAt === undefined ? (
          <div className="opt-strip">
            <span className="opt-strip-icon">
              <Icon name="bolt" size={18} />
            </span>
            <span className="muted">Could not check for improvements: {error.message}</span>
          </div>
        ) : (
          <div className="opt-strip" role="status">
            <Radar small />
            <span>
              <strong>Checking this computer for improvements…</strong>
              <span className="muted"> This takes about 10 seconds.</span>
            </span>
          </div>
        )
      ) : checks.length === 0 ? (
        <div className="opt-strip">
          <span className="opt-strip-icon">
            <Icon name="bolt" size={18} />
          </span>
          <span className="muted">No checks ran. Restart the agent, then check again on the Optimize page.</span>
        </div>
      ) : todo.length === 0 ? (
        <a className="opt-strip opt-strip-link" href={open}>
          <span className="opt-strip-icon ok">
            <Icon name="check" size={18} strokeWidth={2.4} />
          </span>
          <span>
            <strong>Nothing to improve now.</strong>
            <span className="muted"> All {plural(checks.length, "check")} passed.</span>
          </span>
        </a>
      ) : (
        <div className="opt-strip">
          <a className="opt-strip-count" href={open} aria-label={`${plural(todo.length, "improvement")} ready. Open Optimize`}>
            <span className="opt-strip-n">{todo.length}</span>
            <span>
              <strong>{todo.length === 1 ? "improvement ready" : "improvements ready"}</strong>
              <span className="muted">
                {[fixes && plural(fixes, "fix", "fixes"), todo.length - fixes && `${todo.length - fixes} advice`].filter(Boolean).join(" · ")}
                {startedAt !== undefined && " · checking again"}
              </span>
            </span>
          </a>
          <ul className="opt-strip-items">
            {todo.slice(0, 3).map((c) => (
              <li key={c.id}>
                <a href={open} title={c.measured}>
                  <Icon name={areaOf(c).icon} size={14} />
                  <span>{c.title}</span>
                </a>
              </li>
            ))}
            {todo.length > 3 && (
              <li>
                <a href={open} className="opt-strip-more">
                  +{todo.length - 3} more
                </a>
              </li>
            )}
          </ul>
        </div>
      )}
    </Panel>
  );
}
