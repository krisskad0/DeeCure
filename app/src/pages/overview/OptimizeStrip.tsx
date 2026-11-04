// SPDX-License-Identifier: Apache-2.0
// Overview › Summary, at the bottom of the Health score card: a round scanner with "Scan" and how
// many items wait now; the row opens the Optimize page, where the user starts the scan. Expert only,
// like the Optimize page itself (docs/01-product/view-levels.md). It reads the same scan as the
// Optimize page, so the computer is never checked twice at once.

import { Icon } from "../../components/Icon";
import { when } from "../../metrics";
import { href } from "../../route";
import { offered } from "../optimize/Optimize";
import { Scanner } from "../optimize/ScanButton";
import { PARTS, useOptimizeScan } from "../optimize/scan";
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
      <a className="scan-row" href={href("optimize", "all")} aria-label={`Scan. ${status}. Opens Optimize`}>
        <Scanner busy={scanning} progress={scanning && parts ? finished / parts.length : undefined} />
        <span className="scan-row-text">
          <strong>Scan</strong>
          <span className="sc-opt-status" title={status}>
            {!scanning && todo.length > 0 && <span className="sc-opt-count">{todo.length}</span>}
            <span>{status}</span>
          </span>
        </span>
        <Icon name="chev" size={16} />
      </a>
    </section>
  );
}
