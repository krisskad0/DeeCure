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
  /** The areas (page IDs) that have rules. Other areas are "Unknown", not "Healthy". */
  checked?: string[];
  findings: Finding[];
}

export interface Machine {
  hostname: string;
  os: string;
  user: string;
}

export interface AppError {
  kind: "agent_not_running" | "other";
  message: string;
}

export function getStatus(): Promise<StatusReport> {
  return invoke<StatusReport>("status");
}

export interface Snapshot<T = unknown> {
  kind: string;
  taken_at: number;
  data: T;
}

export interface UnitState {
  name: string;
  description: string;
  load: string;
  active: string;
  sub: string;
  /** `enabled`, `disabled`, `static`, … Missing when systemd does not say. */
  enabled?: string;
  /** Restarts since boot. */
  restarts?: number;
}

export function getSnapshot<T>(kind: "services" | "filesystems"): Promise<Snapshot<T> | null> {
  return invoke<Snapshot<T> | null>("snapshot", { kind });
}

export function getMachine(): Promise<Machine> {
  return invoke<Machine>("machine");
}

/** Turns whatever `invoke` threw into an AppError. */
export function toAppError(error: unknown): AppError {
  if (typeof error === "object" && error !== null && "kind" in error && "message" in error) {
    return error as AppError;
  }
  return { kind: "other", message: String(error) };
}
