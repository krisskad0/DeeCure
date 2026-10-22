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
  /** Such as `ubuntu:GNOME`. */
  desktop?: string | null;
  /** `wayland` or `x11`. */
  session_type?: string | null;
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

export function getSnapshot<T>(kind: "services" | "filesystems" | "boot" | "updates"): Promise<Snapshot<T> | null> {
  return invoke<Snapshot<T> | null>("snapshot", { kind });
}

export interface Metric {
  name: string;
  subject?: string;
  value: number;
  unit: "percent" | "bytes" | "bytes_per_second" | "celsius" | "watts" | "seconds" | "hertz" | "count";
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
  /** Highest CPU clock speed, in hertz. */
  cpu_max_hz?: number;
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
  swap_bytes: number;
  threads: number;
  /** Milliseconds since 1970. */
  started_at?: number;
  command: string;
}

export interface NetInterface {
  name: string;
  /** Wi-Fi, Ethernet, Ethernet (USB), Bridge, Loopback, Mobile, VPN or tunnel, Virtual. */
  kind: string;
  /** `up`, `down`, `dormant`, or `unknown`. */
  state: string;
  mac?: string;
  speed_mbps?: number;
  mtu?: number;
}

export interface NetworkInfo {
  interfaces: NetInterface[];
  connections: { tcp_established: number; tcp_listening: number; udp: number };
}

export interface Gpu {
  card: string;
  vendor?: string;
  model?: string;
  pci_id?: string;
  driver?: string;
  boot_vga: boolean;
  busy_percent?: number;
  vram_total?: number;
  vram_used?: number;
  temperature?: number;
}

export interface OsDetails {
  distribution?: string;
  lifecycle?: { release: string; eol: string; eol_esm?: string };
  virtualization: string;
  machine_id?: string;
  users_signed_in: number;
  pro_attached?: boolean;
  livepatch?: string;
  timezone?: string;
  ntp_synchronized: boolean;
  time_server: string;
  rtc_local: boolean;
  locale?: string;
  keyboard?: string;
}

export interface KernelInfo {
  running: string;
  tainted: number;
  command_line: string;
  installed: { version: string; status: "running" | "fallback" | "old"; size: number; installed_at?: number }[];
  modules: { name: string; size: number; used: number; used_by: string[] }[];
}

export interface AptRun {
  /** Local time as apt wrote it: `YYYY-MM-DD  HH:MM:SS`. */
  start: string;
  source: string;
  packages: number;
  changes: string[];
  failed: boolean;
}

export interface BootInfo {
  boot_id: string;
  booted_at: number;
  systemd?: string | null;
  times?: { firmware?: number; loader?: number; kernel: number; initrd?: number; userspace: number; total: number } | null;
  slowest: { name: string; seconds: number }[];
}

export interface UpdatesInfo {
  pending: { name: string; installed: string; candidate: string; origin: string; security: boolean }[];
  lists_updated_at?: number | null;
  automatic_security: boolean;
}

export interface HardwareInfo {
  machine: {
    vendor?: string;
    model?: string;
    chassis?: string;
    bios_vendor?: string;
    bios_version?: string;
    /** As the firmware writes it: MM/DD/YYYY. */
    bios_date?: string;
    boot_mode: string;
    secure_boot?: boolean;
    tpm?: string;
  };
  processor: {
    model?: string;
    cores: number;
    /** [performance, efficient] cores on hybrid CPUs. */
    hybrid?: [number, number];
    threads: number;
    base_hz?: number;
    max_hz?: number;
    l3_bytes?: number;
    microcode?: string;
    features: string[];
    cpus: { cpu: number; core_id: number; kind?: string; mhz?: number }[];
  };
  memory_total?: number;
  ecc_reporting: boolean;
  disks: { name: string; model?: string; kind: string; size: number; firmware?: string; removable: boolean }[];
  devices: { name: string; bus: string; vendor?: string; drivers: string[]; class?: string; id?: string }[];
  bluetooth_adapters: number;
}

export interface PowerInfo {
  batteries: {
    name: string;
    status: string;
    charge?: number;
    full_wh?: number;
    design_wh?: number;
    cycles?: number;
    technology?: string;
    manufacturer?: string;
    model?: string;
    charge_limit?: number;
  }[];
  on_ac?: boolean;
  profile?: string;
  profiles: string[];
  policy?: { low_percent: number; critical_percent: number; action_percent: number; critical_action: string };
  screen_blank_seconds?: number;
  suspend_on_battery_seconds?: number;
  suspend_on_ac_seconds?: number;
  lid_action: string;
  sleep_mode?: string;
}

export interface DisplayInfo {
  compositor?: string;
  shell_version?: string;
  fractional_scaling: boolean;
  outputs: { connector: string; connected: boolean; name?: string; builtin: boolean; width?: number; height?: number; refresh_hz?: number; scale?: number; primary: boolean }[];
}

export interface AudioDevice {
  name: string;
  description: string;
  volume?: number;
  muted: boolean;
  default: boolean;
}

export interface AudioInfo {
  server?: string;
  outputs: AudioDevice[];
  inputs: AudioDevice[];
  streams: string[];
}

export interface BluetoothInfo {
  available: boolean;
  adapters: number;
  powered: boolean;
  devices: { name: string; kind?: string; paired: boolean; connected: boolean; battery?: number }[];
}

export interface FirmwareInfo {
  available: boolean;
  devices: { name: string; vendor?: string; version?: string; available?: string; updatable: boolean }[];
}

export interface SleepEvent {
  at: number;
  kind: string;
  duration_ms?: number;
  failed: boolean;
}

export interface StorageInfo {
  encrypted: boolean;
  root_chain: string[];
  root_reserved: number;
  fstab_entries: number;
  fstab_problems: { line: number; problem: string }[];
  candidates: { action: string; title: string; details: string; size: number; undo: "irreversible" | "regenerates"; needs_password: boolean }[];
}

export interface SizedPath {
  path: string;
  size: number;
  modified?: number;
}

export interface UsageAnswer {
  scanning: boolean;
  scan?: { scanned_at: number; seconds: number; home: SizedPath[]; system: SizedPath[]; files: SizedPath[]; unreadable: number; partial: boolean };
}

export interface UserService {
  name: string;
  description: string;
  active: string;
  sub: string;
  enabled?: string;
  memory?: number;
  restarts?: number;
}

export interface StartupApp {
  id: string;
  name: string;
  command: string;
  scope: "User" | "System";
  enabled: boolean;
}

export interface Timer {
  name: string;
  scope: "system" | "user";
  activates: string;
  schedule: string[];
  next?: number;
  last?: number;
  result?: string;
  persistent: boolean;
}

export interface JobsInfo {
  cron: { schedule: string; command: string; user: string; source: string }[];
  timers: Timer[];
  anacron: { id: string; period: string; delay: number; command: string; last_run?: number }[];
  at: { id: string; when: string; queue: string; user: string }[];
  at_installed: boolean;
  runs: { at: number; job: string; failed: boolean }[];
}

/** Live data the agent reads now. */
export function getInventory(kind: "system"): Promise<SystemInfo>;
export function getInventory(kind: "processes"): Promise<Process[]>;
export function getInventory(kind: "network"): Promise<NetworkInfo>;
export function getInventory(kind: "gpu"): Promise<Gpu[]>;
export function getInventory(kind: "os"): Promise<OsDetails>;
export function getInventory(kind: "kernel"): Promise<KernelInfo>;
export function getInventory(kind: "update_history"): Promise<AptRun[]>;
export function getInventory(kind: "hardware"): Promise<HardwareInfo>;
export function getInventory(kind: "power"): Promise<PowerInfo>;
export function getInventory(kind: "display"): Promise<DisplayInfo>;
export function getInventory(kind: "audio"): Promise<AudioInfo>;
export function getInventory(kind: "bluetooth"): Promise<BluetoothInfo>;
export function getInventory(kind: "firmware"): Promise<FirmwareInfo>;
export function getInventory(kind: "sleep"): Promise<SleepEvent[]>;
export function getInventory(kind: "storage"): Promise<StorageInfo>;
export function getInventory(kind: "disk_usage"): Promise<UsageAnswer>;
export function getInventory(kind: "user_services"): Promise<UserService[]>;
export function getInventory(kind: "service_memory"): Promise<Record<string, number>>;
export function getInventory(kind: "startup"): Promise<StartupApp[]>;
export function getInventory(kind: "jobs"): Promise<JobsInfo>;
export function getInventory(kind: string): Promise<unknown> {
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
