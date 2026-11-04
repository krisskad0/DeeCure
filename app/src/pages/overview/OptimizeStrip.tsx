// SPDX-License-Identifier: Apache-2.0
// Overview › Summary, at the bottom of the Health score card: how many Optimize items wait now,
// in one row that opens Optimize; kept short so the card stays as tall as the chart beside it. Expert only, like the Optimize page itself
// (docs/01-product/view-levels.md). It reads the same scan as the Optimize page, so opening both
// never checks the computer twice.

import { Icon } from "../../components/Icon";
import { href } from "../../route";
import { Radar, offered } from "../optimize/Optimize";
import { SCAN_USUAL_MS, useOptimizeScan } from "../optimize/scan";
import { plural } from "./shared";

export function OptimizeStrip() {
  const { checks, error, startedAt } = useOptimizeScan();
  const todo = checks ? offered(checks) : [];
  const fixes = todo.filter((c) => c.kind === "fix").length;
  const open = href("optimize", "all");
  return (
    <section className="sc-opt" aria-label="Optimize">
      <div className="sc-opt-head">
        <span className="sc-head">Optimize</span>
        <a className="btn btn-link btn-sm" href={open} aria-label="Open Optimize">
          Open
        </a>
      </div>
      {!checks ? (
        error && startedAt === undefined ? (
          <p className="sc-hint muted">Could not check for improvements: {error.message}</p>
        ) : (
          <div className="sc-opt-row" role="status">
            <Radar small />
            <span>
              Checking for improvements…
              <span className="muted"> about {SCAN_USUAL_MS / 1000} s</span>
            </span>
          </div>
        )
      ) : checks.length === 0 ? (
        <p className="sc-hint muted">No checks ran. Restart the agent, then check again on the Optimize page.</p>
      ) : todo.length === 0 ? (
        <a className="sc-opt-row sc-opt-link" href={open}>
          <span className="sc-opt-n ok">
            <Icon name="check" size={16} strokeWidth={2.6} />
          </span>
          <span>
            <strong>Nothing to improve</strong>
            <span className="muted"> · all {plural(checks.length, "check")} passed</span>
          </span>
        </a>
      ) : (
        <a className="sc-opt-row sc-opt-link" href={open} aria-label={`${plural(todo.length, "improvement")} ready. Open Optimize`}>
          <span className="sc-opt-n">{todo.length}</span>
          <span className="sc-opt-sum">
            <strong>{todo.length === 1 ? "improvement ready" : "improvements ready"}</strong>
            <span className="muted">
              {[fixes && plural(fixes, "fix", "fixes"), todo.length - fixes && `${todo.length - fixes} advice`].filter(Boolean).join(" · ")}
              {startedAt !== undefined && " · checking again"}
            </span>
          </span>
          <Icon name="chev" size={14} />
        </a>
      )}
    </section>
  );
}
