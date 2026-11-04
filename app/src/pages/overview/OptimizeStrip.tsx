// SPDX-License-Identifier: Apache-2.0
// Overview › Summary, at the bottom of the Health score card: how many items wait now, and one big
// "Optimize now" button that opens Optimize and starts a scan there. Expert only,
// like the Optimize page itself (docs/01-product/view-levels.md). It reads the same scan as the
// Optimize page, so the computer is never checked twice at once.

import { Icon } from "../../components/Icon";
import { when } from "../../metrics";
import { href } from "../../route";
import { offered } from "../optimize/Optimize";
import { PARTS, scanOptimize, useOptimizeScan } from "../optimize/scan";
import { plural } from "./shared";

export function OptimizeStrip() {
  const { checks, at, error, startedAt, parts } = useOptimizeScan();
  const scanning = startedAt !== undefined;
  const todo = checks ? offered(checks) : [];
  const fixes = todo.filter((c) => c.kind === "fix").length;
  const finished = parts?.filter((p) => p.status === "done" || p.status === "failed").length ?? 0;
  const current = parts?.find((p) => p.status === "running");

  const status = scanning
    ? current
      ? `Part ${finished + 1} of ${parts?.length}: ${PARTS.find((p) => p.id === current.id)?.label}`
      : "Checking this computer…"
    : !checks
      ? error
        ? `Could not check: ${error.message}`
        : "Not checked yet"
      : checks.length === 0
        ? "No checks ran"
        : todo.length === 0
          ? `Nothing to improve · all ${plural(checks.length, "check")} passed`
          : `${todo.length === 1 ? "improvement" : "improvements"} ready · ${[fixes && plural(fixes, "fix", "fixes"), todo.length - fixes && `${todo.length - fixes} advice`].filter(Boolean).join(" · ")}`;

  return (
    <section className="sc-opt" aria-label="Optimize">
      <div className="sc-opt-head">
        <span className="sc-head">Optimize</span>
        {at !== undefined && <span className="sc-opt-when muted">Checked {when(at)}</span>}
      </div>
      <p className="sc-opt-status" title={status}>
        {!scanning && todo.length > 0 && <span className="sc-opt-count">{todo.length}</span>}
        <span>{status}</span>
      </p>
      <a className="btn btn-primary sc-opt-go" href={href("optimize", "all")} onClick={() => void scanOptimize()}>
        {scanning ? <span className="sc-opt-spin" aria-hidden="true" /> : <Icon name="bolt" size={16} strokeWidth={2.2} />}
        {scanning ? "Checking… see progress" : "Optimize now"}
        {scanning && parts && (
          <span className="sc-opt-bar" aria-hidden="true">
            <span style={{ width: `${(finished / parts.length) * 100}%` }} />
          </span>
        )}
      </a>
    </section>
  );
}
