// SPDX-License-Identifier: Apache-2.0
// Honest empty states shared by every page: agent not running, an error, or nothing collected yet.

import type { AppError } from "../api";
import { Command } from "../components/CodeBlock";
import { StatusBadge } from "../components/StatusBadge";

export function NoData({ error, waiting }: { error: AppError | null; waiting: string }) {
  if (error?.kind === "agent_not_running") {
    return (
      <div className="empty">
        <StatusBadge variant="off" label="Agent not running" />
        <p>DeeCure's background agent is not running, so there is nothing to show yet.</p>
        <p>Start it with:</p>
        <Command>systemctl --user start deecure-agent.socket</Command>
        <p>Working from a source checkout? Run:</p>
        <Command>make agent</Command>
      </div>
    );
  }
  if (error) {
    return (
      <div className="empty">
        <StatusBadge variant="crit" label="Error" />
        <p>{error.message}</p>
      </div>
    );
  }
  return <div className="empty muted">{waiting}</div>;
}
