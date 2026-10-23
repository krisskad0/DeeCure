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

export interface LogEntry {
  at: number;
  /** 0 emergency … 3 error, 4 warning, 5 notice, 6 info, 7 debug. */
  priority: number;
  unit: string;
  message: string;
}

export interface JournalInfo {
  volume: number[];
  entries: LogEntry[];
  total: number;
}

export interface ErrorsInfo {
  total: number;
  groups: { message: string; unit: string; count: number; last_seen: number; class: "attention" | "harmless" | "review" }[];
  times: number[];
}

export interface KernelLine {
  at: number;
  priority: number;
  subsystem: string;
  message: string;
}

export interface Crash {
  at: number;
  app: string;
  signal: string;
  source: "kernel" | "systemd" | "apport";
  count: number;
}

export interface BootRecord {
  started: number;
  ended: number;
  shutdown: "running" | "clean" | "unclean";
  kernel?: string;
}

export interface DeviceEvent {
  at: number;
  event: "connected" | "disconnected";
  device: string;
  id?: string;
}

export interface ConfigChange {
  path: string;
  modified: number;
  owner: string;
}

// What changed (docs/02-architecture/storage.md#events).

export type EventKind =
  | "package_installed"
  | "package_upgraded"
  | "package_removed"
  | "config_changed"
  | "device_added"
  | "device_removed"
  | "boot"
  | "unclean_shutdown"
  | "service_failed"
  | "crash"
  | "out_of_memory_kill"
  | "login_failed"
  | "fix_applied"
  | "fix_undone";

export interface HistoryEvent {
  id: number;
  kind: EventKind;
  at: number;
  title: string;
  detail?: string;
  subject?: string;
  source: string;
  /** Depends on the kind: package versions, a config diff, a device ID, a fix run. */
  data?: {
    action?: PackageChange["action"];
    format?: "deb" | "snap" | "flatpak";
    from?: string | null;
    to?: string | null;
    run?: number;
    failed?: boolean;
    kernel?: boolean | string | null;
    change?: "added" | "changed" | "removed";
    diff?: string;
    added?: number;
    removed?: number;
    count?: number;
    first_scan?: boolean;
    bus?: string;
    id?: string | null;
    signal?: string;
    result?: string;
    boot_id?: string;
    previous_kernel?: string | null;
    kernel_changed?: boolean;
    run_id?: number;
    status?: string;
    undoable?: boolean;
  };
}

export interface EventList {
  events: HistoryEvent[];
  /** More matched than were sent. */
  truncated: boolean;
}

export function getEvents(since: number, opts: { until?: number; kinds?: EventKind[]; limit?: number } = {}): Promise<EventList> {
  return invoke<EventList>("events", { since, until: opts.until ?? null, kinds: opts.kinds ?? null, limit: opts.limit ?? null });
}

/** One event with all details (a config diff), or `null` when it is gone. */
export function getEvent(id: number): Promise<HistoryEvent | null> {
  return invoke<HistoryEvent | null>("event", { id });
}

// Notifications (docs/02-architecture/notifications.md).

export interface NotifySettings {
  critical: boolean;
  warnings: boolean;
  weekly: boolean;
  quiet_hours: "off" | "22-07" | "23-08";
}

export interface SentNotification {
  at: number;
  title: string;
  body: string;
}

export function getNotifySettings(): Promise<NotifySettings> {
  return invoke<NotifySettings>("notify_settings");
}

export function setNotifySettings(settings: NotifySettings): Promise<NotifySettings> {
  return invoke<NotifySettings>("set_notify_settings", { settings });
}

export function sendTestNotification(): Promise<boolean> {
  return invoke<boolean>("notify_test");
}

export function getNotifications(): Promise<SentNotification[]> {
  return invoke<SentNotification[]>("notifications");
}

export interface PackageChange {
  /** Local time as apt wrote it: `YYYY-MM-DD  HH:MM:SS`. */
  start: string;
  action: "install" | "upgrade" | "remove" | "purge" | "downgrade" | "reinstall";
  name: string;
  from?: string;
  to?: string;
  source: string;
}

export interface NetDetails {
  links: { name: string; ipv4: string[]; ipv6: string[]; rx: number; tx: number }[];
  gateway?: string;
  wifi?: { interface: string; ssid?: string; strength?: number; dbm?: number; frequency_mhz?: number; bitrate_mbps?: number };
  connectivity?: "full" | "limited" | "portal" | "none" | "unknown";
  resolver: string;
  dns_servers: string[];
  vpns: { name: string; kind: string; connected: boolean }[];
  firewall: { installed: boolean; enabled: boolean; incoming?: string; outgoing?: string; rules?: string[] };
}

export interface SocketInfo {
  proto: "tcp" | "udp";
  local: string;
  local_port: number;
  remote: string;
  state: string;
  process?: string;
  exposure: "local" | "all" | "network";
}

export interface AccountsInfo {
  users: { name: string; full_name: string; uid: number; groups: string[]; admin: boolean; shell: string; home: string; last_sign_in?: number }[];
  groups: { name: string; gid: number; members: string[]; privilege?: string }[];
  sessions: { id: string; user: string; seat: string; kind: string; class: string; state: string; remote: boolean; since?: number }[];
  events: { at: number; user: string; event: "sign-in" | "sudo"; source: string; success: boolean }[];
  ssh: {
    server_installed: boolean;
    password_login?: boolean;
    root_login?: string;
    authorized_keys: number;
    client_keys: { file: string; kind: string; bits: number; fingerprint: string; created: number }[];
  };
}

export interface SecurityInfo {
  hardening: { name: string; value: string; result: "pass" | "fail" | "na"; recommendation: string }[];
  apparmor: boolean;
  suid: { path: string; owner: string; package?: string }[];
  scanners: string[];
  clamav_updated?: number;
  heuristics: { title: string; passed: boolean; detail: string }[];
  screen_lock?: boolean;
}

export interface PrivacyInfo {
  in_use: { microphone: string[]; camera: string[]; screen: string[] };
  apps: { app: string; kind: "snap" | "flatpak"; camera: boolean; microphone: boolean; location: boolean; files: boolean; network: boolean }[];
  settings: { label: string; description: string; on: boolean }[];
}

export interface PackageInfo {
  name: string;
  version: string;
  source: "apt" | "snap" | "flatpak" | "appimage";
  size: number;
  manual: boolean;
  summary?: string;
  section?: string;
}

export interface PackagesInfo {
  packages: PackageInfo[];
  sources: { line: string; kind: "Official" | "PPA" | "Third-party"; signed: boolean; own_key: boolean; enabled: boolean; file: string }[];
  removable: { action: string; title: string; count: number; size: number; names: string[]; undo: "reinstallable" | "irreversible" }[];
}

export interface ContainersInfo {
  docker: "running" | "not running" | "no access" | "not installed";
  containers: { name: string; image: string; status: string; state: string; ports: string[]; cpu?: string; memory?: string }[];
  images: { repository: string; tag: string; size: string; created: string; containers: number }[];
  volumes: { name: string; size: string; links: number }[];
  vms: { name: string; engine: string; running: boolean }[];
}

export interface BackupsInfo {
  deja_dup: { installed: boolean; scheduled: boolean; every_days?: number; last_backup?: number; backend?: string };
  timeshift: { installed: boolean; configured: boolean; mode?: string; schedule: string[]; snapshots: { created: number; kind: string; comment?: string }[] };
}

export interface DiagnosticReport {
  steps: { name: string; result: "ok" | "warn" | "crit" | "na"; value: string }[];
  verdict: string;
  tone: "ok" | "warn" | "crit";
  top?: [string, number][];
}

export interface AgentInfo {
  version: string;
  database: string;
  database_bytes: number;
  rules: number;
  rules_dir: string;
  uptime_seconds: number;
  memory_bytes: number;
}

export interface VersionInfo {
  agent: string;
  protocol: number;
}

export interface Kb {
  services: { name: string; plain: string; safe_to_disable?: string; if_failed?: string }[];
  ports: { port: number; proto: string; name: string; plain: string; risk_if_open: "low" | "medium" | "high" }[];
  errors: { match: string; plain: string; severity: "info" | "warning" | "critical"; next_step: string }[];
  packages: { name: string; plain: string }[];
  topics: { id: string; title: string; summary: string; steps: string[]; notes?: string }[];
  /** Rule ID → topic ID. */
  rules: Record<string, string>;
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
export function getInventory(kind: "journal"): Promise<JournalInfo>;
export function getInventory(kind: "errors"): Promise<ErrorsInfo>;
export function getInventory(kind: "kernel_log"): Promise<KernelLine[]>;
export function getInventory(kind: "crashes"): Promise<Crash[]>;
export function getInventory(kind: "boots"): Promise<BootRecord[]>;
export function getInventory(kind: "device_events"): Promise<DeviceEvent[]>;
export function getInventory(kind: "config_changes"): Promise<ConfigChange[]>;
export function getInventory(kind: "package_history"): Promise<PackageChange[]>;
export function getInventory(kind: "net_details"): Promise<NetDetails>;
export function getInventory(kind: "sockets"): Promise<SocketInfo[]>;
export function getInventory(kind: "accounts"): Promise<AccountsInfo>;
export function getInventory(kind: "security"): Promise<SecurityInfo>;
export function getInventory(kind: "privacy"): Promise<PrivacyInfo>;
export function getInventory(kind: "packages"): Promise<PackagesInfo>;
export function getInventory(kind: "containers"): Promise<ContainersInfo>;
export function getInventory(kind: "backups"): Promise<BackupsInfo>;
export function getInventory(kind: "diagnose_performance" | "diagnose_network"): Promise<DiagnosticReport>;
export function getInventory(kind: "agent"): Promise<AgentInfo | null>;
export function getInventory(kind: "kb"): Promise<Kb | null>;
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

// Fixes (docs/02-architecture/fix-engine.md). Planning reads only; applying runs the steps
// in this window's session, so polkit can show the password box for admin steps.

export type Undo = "full" | "regenerates" | "none";

export interface FixCall {
  op: string;
  [key: string]: unknown;
}

export interface FixStep {
  id: string;
  title: string;
  description: string;
  size?: number;
  undo: Undo;
  checked: boolean;
  command: string;
  call: FixCall;
}

export interface FixPlan {
  action: string;
  title: string;
  subtitle: string;
  rule?: string;
  subject?: string;
  steps: FixStep[];
}

export interface StepResult {
  id: string;
  title: string;
  ok: boolean;
  message: string;
  undo?: FixCall;
}

export interface FixRun {
  id: number;
  action: string;
  title: string;
  rule?: string;
  subject?: string;
  started_at: number;
  finished_at: number;
  status: "succeeded" | "failed" | "undone";
  steps: StepResult[];
}

/** Calls that need the root helper; everything else runs as the user. */
export const USER_CALLS = ["empty_trash", "clear_thumbnails", "docker_prune_images", "docker_prune_build_cache", "set_startup_app", "end_process", "set_power_profile"];

export function getFixPlan(action: string, subject?: string): Promise<FixPlan> {
  return invoke<FixPlan>("fix_plan", { action, subject: subject ?? null });
}

export function applyFix(plan: FixPlan, chosen: string[]): Promise<{ run: FixRun }> {
  return invoke<{ run: FixRun }>("fix_apply", { plan, chosen });
}

export function getFixRuns(): Promise<FixRun[]> {
  return invoke<FixRun[]>("fix_runs");
}

export function undoFix(id: number): Promise<string[]> {
  return invoke<string[]>("fix_undo", { id });
}

export function getHelperStatus(): Promise<boolean> {
  return invoke<boolean>("helper_status");
}

export function helperRead(what: "smart" | "firewall_rules" | "crontabs" | "accounts" | "memory_modules" | "serial", arg = ""): Promise<string> {
  return invoke<string>("helper_read", { what, arg });
}
