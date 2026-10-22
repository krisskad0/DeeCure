// SPDX-License-Identifier: Apache-2.0
// Calls into the Tauri side, which talks to the agent.
// Types mirror crates/deecure-core (see docs/02-architecture/ipc-protocol.md).

import { invoke } from "@tauri-apps/api/core";

export type Severity = "info" | "warning" | "critical";

export interface Finding {
  rule: string;
  subject?: string;
  severity: Severity;
  state: "open" | "resolved" | "dismissed";
  title: string;
  explain: string;
  first_seen: number;
  last_seen: number;
  fixes?: string[];
}

export interface StatusReport {
  generated_at: number;
  /** When the rules last ran. Missing until they have run once. */
  checked_at?: number;
  findings: Finding[];
}

export interface AppError {
  kind: "agent_not_running" | "other";
  message: string;
}

export function getStatus(): Promise<StatusReport> {
  return invoke<StatusReport>("status");
}

/** Turns whatever `invoke` threw into an AppError. */
export function toAppError(error: unknown): AppError {
  if (typeof error === "object" && error !== null && "kind" in error && "message" in error) {
    return error as AppError;
  }
  return { kind: "other", message: String(error) };
}
