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
  /** 0–100. Missing until the rules have run once. */
  score?: number;
  /** Points each area takes off the score, largest first. */
  deductions?: { category: string; points: number }[];
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

export interface Metric {
  name: string;
  subject?: string;
  value: number;
  unit: "percent" | "bytes" | "bytes_per_second" | "celsius" | "watts" | "seconds" | "count";
  /** Milliseconds since 1970. */
  at: number;
}

/** Points for one metric over a time range, oldest first, averaged down to about `maxPoints`. */
export function getMetrics(name: string, since: number, until: number, opts: { subject?: string; maxPoints?: number } = {}): Promise<Metric[]> {
  return invoke<Metric[]>("metrics", { name, subject: opts.subject ?? null, since, until, maxPoints: opts.maxPoints ?? 160 });
}

/** The newest value of every metric from the last 10 minutes. */
export function getLatest(): Promise<Metric[]> {
  return invoke<Metric[]>("latest");
}

export interface SystemInfo {
  hostname?: string;
  os?: string;
  os_version?: string;
  kernel?: string;
  arch: string;
  model?: string;
  cpu?: string;
  cpu_threads: number;
  memory_total?: number;
  uptime_seconds?: number;
  reboot_required: boolean;
  reboot_packages: string[];
}

export interface Process {
  pid: number;
  name: string;
  user: string;
  /** R running, S sleeping, D waiting for disk, Z zombie, T stopped, I idle. */
  state: string;
  /** Percent of one CPU since the last reading. */
  cpu_percent: number;
  memory_bytes: number;
  threads: number;
  command: string;
}

/** Live data the agent reads now. */
export function getInventory(kind: "system"): Promise<SystemInfo>;
export function getInventory(kind: "processes"): Promise<Process[]>;
export function getInventory(kind: "system" | "processes"): Promise<unknown> {
  return invoke("inventory", { kind });
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
