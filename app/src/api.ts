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
  /** 0-100. Missing until the rules have run once. */
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
  kind: "agent_not_running" | "agent_outdated" | "other";
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

/** Deep processor details (Expert); counters are totals since start. */
export interface CpuDetail {
  /** When it was read, ms since 1970: two readings give rates. */
  at: number;
  driver?: string;
  driver_mode?: string;
  turbo?: boolean;
  vendor?: string;
  family?: number;
  model?: number;
  stepping?: number;
  microcode_loaded?: string;
  microcode_on_disk?: string;
  package_throttle_count?: number;
  package_throttle_max_ms?: number;
  cores: {
    cpu: number;
    now_khz?: number;
    min_khz?: number;
    max_khz?: number;
    hardware_max_khz?: number;
    governor?: string;
    energy_preference?: string;
    highest_perf?: number;
    throttle_count?: number;
    throttle_max_ms?: number;
    idle: { name: string; description?: string; latency_us?: number; entries?: number; time_us?: number; disabled: boolean }[];
  }[];
  caches: { level: number; kind: string; bytes: number; shared_cpus: string; copies: number }[];
  power_limits: { zone: string; name: string; watts: number; window_s?: number; enabled?: boolean }[];
  flaws: { name: string; state: "not_affected" | "mitigated" | "vulnerable" | "unknown"; detail: string }[];
  interrupts: { irq: string; count: number; device: string }[];
}

/** Deep memory details (Expert); counters are totals since start. */
export interface MemoryDetail {
  at: number;
  /** Every /proc/meminfo line, in bytes (page counts as they are). */
  meminfo: Record<string, number>;
  /** pgfault, pgmajfault, pswpin, pswpout, oom_kill, pgscan_direct, pgsteal_direct. */
  vmstat: Record<string, number>;
  swaps: { path: string; kind: string; size: number; used: number; priority: number }[];
  zswap?: { enabled: boolean; compressor?: string; max_pool_percent?: number };
  zram: { name: string; algorithm?: string; disk_size: number; original: number; compressed_used: number }[];
  huge_pages?: string;
  huge_pages_defrag?: string;
  controllers: { name: string; corrected: number; uncorrected: number; seconds_since_reset?: number; dimms: { label?: string; size_mb?: number; corrected?: number; uncorrected?: number }[] }[];
  oomd?: { swap_limit_percent?: number; pressure_limit_percent?: number; pressure_duration?: string; swap_watched: string[]; pressure_watched: [string, number | null][] };
}

/** Real memory of one process; PSS and USS only for the biggest ones the user may read. */
export interface ProcessMemory {
  pid: number;
  name: string;
  rss: number;
  pss?: number;
  uss?: number;
  swap_pss?: number;
  major_faults: number;
  oom_score?: number;
}

/** Use per app and per service (cgroups); totals since start. */
export interface Groups {
  at: number;
  groups: {
    kind: "service" | "user-service" | "app";
    unit: string;
    name: string;
    memory?: number;
    memory_own?: number;
    memory_cache?: number;
    memory_peak?: number;
    swap?: number;
    memory_pressure?: number;
    cpu_usec?: number;
    disk_read?: number;
    disk_write?: number;
    tasks?: number;
  }[];
}

/** Drives and filesystems in depth (Expert). */
export interface StorageDetail {
  disks: {
    name: string;
    rotational: boolean;
    scheduler?: string;
    schedulers: string[];
    queue_size?: number;
    read_ahead_kb?: number;
    logical_block?: number;
    physical_block?: number;
    trim: boolean;
    table?: string;
    partitions: { name: string; start_sector: number; size: number; aligned: boolean }[];
    sata_speed?: string;
    sata_limit?: string;
    nvme?: { firmware?: string; transport?: string; state?: string; pci_address?: string; temp_warning?: number; temp_critical?: number };
  }[];
  filesystems: { device: string; mount: string; fstype: string; options: string[]; inodes?: number; inodes_free?: number; errors?: number; first_error_at?: number; last_error_at?: number }[];
  nvme_max_latency_us?: number;
}

/** Disk use per process since it started (own processes only). */
export interface ProcessesIo {
  at: number;
  processes: { pid: number; name: string; read_bytes: number; write_bytes: number; cancelled_write_bytes: number }[];
  unreadable: number;
}

/** One PCI device's link, link errors, and power saving. */
export interface PciDevice {
  address: string;
  class?: string;
  vendor?: string;
  model?: string;
  driver?: string;
  link_speed?: string;
  max_link_speed?: string;
  link_width?: number;
  max_link_width?: number;
  slot_max_link_speed?: string;
  slot_max_link_width?: number;
  errors?: { corrected: number; nonfatal: number; fatal: number };
  sys_path?: string;
  power_control?: string;
  power_state?: string;
}

/** Graphics cards and per-app use (Expert); counters are totals since start. */
export interface GpuDetail {
  at: number;
  cards: {
    card: string;
    driver?: string;
    pci_address?: string;
    boot_vga: boolean;
    mhz_now?: number;
    mhz_min?: number;
    mhz_max?: number;
    mhz_hardware_max?: number;
    mhz_boost?: number;
    power_saving_ms?: number;
    slowed_by: string[];
    watts?: number;
    runtime_status?: string;
    connectors: [string, string][];
  }[];
  clients: { pid: number; name: string; pci_address?: string; engines: Record<string, number>; memory?: number }[];
}

/** The graphics software (Expert). */
export interface GraphicsStack {
  session_type?: string;
  mesa_version?: string;
  nvidia_version?: string;
  renderer?: string;
  software_rendering: boolean;
  video_drivers: string[];
  video_decode?: string[];
  firmware: string[];
}

/** Network in depth (Expert); counters are totals since start. */
export interface NetDeep {
  at: number;
  wifi: { interface: string; ssid: string; uuid?: string; channel?: number; frequency_mhz?: number; band?: string; rate_mbps?: number; signal_percent?: number; signal_dbm?: number; noise_dbm?: number; security?: string; power_save?: boolean }[];
  interfaces: { name: string; rx_errors: number; tx_errors: number; rx_dropped: number; tx_dropped: number; collisions: number; carrier_changes?: number }[];
  /** /proc/net/snmp counters such as "Tcp.RetransSegs". */
  counters: Record<string, number>;
  dns?: { link: string; current?: string; servers: string[]; default_route: boolean; dnssec?: string; dns_over_tls: boolean }[];
  routes: { destination: string; gateway?: string; device?: string; metric?: number; protocol?: string; down: boolean; family: number }[];
  extra_rules: number;
}

/** Open TCP connections with their bytes (TCP only). */
export interface TcpTraffic {
  at: number;
  connections: { local: string; remote: string; program?: string; pid?: number; received: number; sent: number; resent: number }[];
}

/** A setting with the choices the hardware offers. */
export interface Choice {
  current?: string;
  offered: string[];
}

/** Power in depth (Expert). */
export interface PowerDetail {
  batteries: { name: string; volts?: number; volts_min_design?: number; amps?: number; watts?: number; status?: string; temp_c?: number; health?: string; made?: string; serial?: string; charge_start?: number; charge_stop?: number; charge_mode?: Choice; charge_behaviour?: Choice }[];
  chargers: { name: string; kind: string; online: boolean; usb_type?: Choice; max_watts?: number }[];
  power_delivery: { name: string; offered: PdOffer[]; accepted: PdOffer[] }[];
  suspend?: Choice;
  wake_sources: { name: string; device?: string; pci_address?: string; product?: string; events: number; wakeups: number; active: number; may_wake?: boolean }[];
  last_wake_irq?: string;
  last_wake_device?: string;
  usb: { port: string; sys_path?: string; product?: string; maker?: string; control?: string; state?: string; keep_awake: boolean; may_wake?: boolean }[];
  radios: { name: string; kind: string; soft_blocked: boolean; hard_blocked: boolean }[];
  keyboard_backlight?: { level: number; max: number };
}

export interface PdOffer {
  kind: string;
  millivolts?: number;
  max_millivolts?: number;
  milliamps?: number;
  milliwatts?: number;
}

/** Thermal zones and cooling devices (Expert). */
export interface ThermalInfo {
  zones: { zone: string; kind: string; celsius?: number; policy?: string; trips: { kind: string; celsius: number }[] }[];
  cooling: { kind: string; count: number; level: number; max_level: number }[];
}

/** The boot's critical chain and every unit's start time (Expert). */
export interface BootDetail {
  critical_chain: { unit: string; at_ms?: number; took_ms?: number; depth: number }[];
  blame: [string, number][];
}

/** One system service's unit file facts (Expert). */
export interface UnitFile {
  unit: string;
  file?: string;
  local_overrides: string[];
  package_overrides: string[];
  wanted_by: string[];
  restart?: string;
  restarts: number;
  state?: string;
  masked_by?: "local" | "runtime" | "package";
}

/** How locked down a service is: 0 locked down, 10 every right. */
export interface Exposure {
  unit: string;
  score: number;
  level: string;
}

/** Kernel settings and system limits (Expert). */
export interface KernelDetail {
  settings: { key: string; value: string; file: string; line: number; package?: string; file_changed: boolean; now?: string; overridden: boolean }[];
  watch_limit?: number;
  watch_instances_limit?: number;
  watches_used: number;
  watch_users: { pid: number; name: string; watches: number }[];
  open_files?: number;
  open_files_limit?: number;
  modules: { name: string; size: number; taint?: string; used_by: string[]; devices: number }[];
  dkms?: { module: string; version: string; kernels: string[] }[];
  kernels: string[];
  lockdown?: string;
  time_sync: Record<string, string>;
}

/** Firmware, platform, and ports (Expert). */
export interface PlatformDetail {
  hsi?: string;
  security_checks: { id: string; name: string; result: string; passed: boolean; runtime: boolean }[];
  boot_entries: { number: string; name: string; active: boolean; file?: string; order?: number; current: boolean }[];
  secure_boot?: string;
  mok_keys?: string[];
  management_engine?: string;
  tpm_version?: string;
  tpm_driver?: string;
  disk_unlocked_by_tpm: boolean;
  disk_encrypted: boolean;
  iommu_units: number;
  thunderbolt_security?: string;
  thunderbolt_dma_protection?: boolean;
  rtc_drift_s?: number;
  virtualization?: string;
  kvm_ready: boolean;
  usb: { port: string; product?: string; maker?: string; version?: string; speed_mbps?: number; max_power_ma?: number; hub: boolean; drivers: string[] }[];
  typec: { name: string; data_role?: string; power_role?: string; power_mode?: string; revision?: string; connected: boolean; partner_type?: string; partner_power_delivery: boolean; alt_modes: { name: string; active: boolean }[] }[];
  thunderbolt: { id: string; name?: string; vendor?: string; authorized: boolean }[];
}

/** Built-in devices, screens, and sound hardware (Expert). */
export interface DevicesDetail {
  cameras: { node: string; name: string; driver?: string }[];
  fingerprint?: { device?: string; enrolled: string[] };
  network: { interface: string; driver?: string; firmware?: string; bus?: string; speed?: string; duplex?: string; fastest?: string; link?: boolean; energy_saving?: string }[];
  pointing: string[];
  batteries: { kind: string; model?: string; percent?: number }[];
  printers?: { name: string; state: string; default: boolean; jobs: number }[];
  screens: { connector: string; maker?: string; model?: string; product_code: number; year?: number; width_cm?: number; height_cm?: number; native?: [number, number]; refresh_min?: number; refresh_max?: number; bits?: number; hdr: string[] }[];
  backlights: { name: string; level: number; max: number; kind?: string }[];
  sound: { index: number; name: string; codecs: string[] }[];
}

/** Hardware errors from the kernel log, crash dumps, and drivers Ubuntu offers (Expert). */
export interface HwErrors {
  kinds?: { kind: string; count: number; last_at?: number; examples: string[] }[];
  crash_dumps: string[];
  driver_offers?: { vendor?: string; model?: string; drivers: [string, boolean][] }[];
}

/** Software health (Expert): held, broken, and local packages, snaps, Flatpak runtimes. */
export interface SoftwareHealth {
  held: string[];
  broken: string[];
  local: [string, string][];
  snaps?: { name: string; version: string; revision: string; tracking?: string; publisher?: string; notes: string[] }[];
  snap_refresh: Record<string, string>;
  runtimes?: { id: string; branch: string; installation: string; used: boolean }[];
}

/** The full package file check: runs only when asked. */
export interface VerifyAnswer {
  running: boolean;
  finished_at?: number;
  took_s?: number;
  problems?: { file: string; problem: "missing" | "changed" | "unreadable"; config: boolean; package?: string }[];
  failed?: string;
}

/** Security in depth (Expert): AppArmor per process, automatic updates. */
export interface SecurityDetail {
  confinement: { profile: string; mode: string; processes: number; names: string[] }[];
  periodic: [string, string][];
  unattended_installed: boolean;
  upgrade_runs?: { at: string; level: string; message: string }[];
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

// Reports (deecure-core report.rs).

export interface Report {
  created_at: number;
  since: number;
  until: number;
  host: string;
  os: string;
  score?: number;
  score_trend: [number, number][];
  findings: Finding[];
  changes: { packages: number; config: number; devices: number; crashes: number; failed_services: number; boots: number; unclean_shutdowns: number; fixes: number };
  highlights: HistoryEvent[];
  disks: { mount_point: string; used_percent: number; free_bytes: number; days_until_full?: number }[];
  tip: string;
}

export function getWeeklyReports(): Promise<Report[]> {
  return invoke<Report[]>("weekly_reports");
}

/** Saves a 7-day report as HTML in the downloads folder; returns the file's path. */
export function exportReport(): Promise<string> {
  return invoke<string>("export_report");
}

export type KeepHistory = "90d" | "1y" | "2y";

export function getDataSettings(): Promise<{ keep_history: KeepHistory }> {
  return invoke<{ keep_history: KeepHistory }>("data_settings");
}

export function setDataSettings(settings: { keep_history: KeepHistory }): Promise<{ keep_history: KeepHistory }> {
  return invoke<{ keep_history: KeepHistory }>("set_data_settings", { settings });
}

/** Deletes all history; restore points and settings stay. */
export function deleteHistory(): Promise<{ rows: number; bytes_freed: number }> {
  return invoke<{ rows: number; bytes_freed: number }>("delete_history");
}

export interface TrayState {
  enabled: boolean;
  active: boolean;
  /** This desktop has a tray area. */
  available: boolean;
}

export function getTrayState(): Promise<TrayState> {
  return invoke<TrayState>("tray_state");
}

/** Rejects with a plain-English reason when this desktop has no tray area. */
export function setTray(on: boolean): Promise<TrayState> {
  return invoke<TrayState>("set_tray", { on });
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
  containers: { name: string; image: string; status: string; state: string; ports: string[]; cpu?: string; memory?: string; net_io?: string; block_io?: string }[];
  images: { repository: string; tag: string; size: string; created: string; containers: number }[];
  volumes: { name: string; size: string; links: number }[];
  build_cache?: { size: string; in_use: boolean; last_used: string }[];
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
export function getInventory(kind: "cpu_detail"): Promise<CpuDetail>;
export function getInventory(kind: "memory_detail"): Promise<MemoryDetail>;
export function getInventory(kind: "process_memory"): Promise<ProcessMemory[]>;
export function getInventory(kind: "cgroups"): Promise<Groups>;
export function getInventory(kind: "storage_detail"): Promise<StorageDetail>;
export function getInventory(kind: "process_io"): Promise<ProcessesIo>;
export function getInventory(kind: "pci_detail"): Promise<PciDevice[]>;
export function getInventory(kind: "gpu_detail"): Promise<GpuDetail>;
export function getInventory(kind: "graphics_stack"): Promise<GraphicsStack>;
export function getInventory(kind: "net_deep"): Promise<NetDeep>;
export function getInventory(kind: "tcp_traffic"): Promise<TcpTraffic>;
export function getInventory(kind: "power_detail"): Promise<PowerDetail>;
export function getInventory(kind: "thermal"): Promise<ThermalInfo>;
export function getInventory(kind: "boot_detail"): Promise<BootDetail>;
export function getInventory(kind: "unit_files"): Promise<UnitFile[]>;
export function getInventory(kind: "service_security"): Promise<Exposure[]>;
export function getInventory(kind: "kernel_detail"): Promise<KernelDetail>;
export function getInventory(kind: "platform_detail"): Promise<PlatformDetail>;
export function getInventory(kind: "devices_detail"): Promise<DevicesDetail>;
export function getInventory(kind: "hw_errors"): Promise<HwErrors>;
export function getInventory(kind: "software_health"): Promise<SoftwareHealth>;
export function getInventory(kind: "security_detail"): Promise<SecurityDetail>;
export function getInventory(kind: "package_verify" | "package_verify_run"): Promise<VerifyAnswer>;
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

/** Everything about one process (Expert process drawer). Parts other users' processes hide are left out. */
export interface ProcessDetail {
  pid: number;
  name: string;
  state: string;
  user_id?: number;
  ancestors: { pid: number; name: string }[];
  children: { pid: number; name: string }[];
  command_line: string[];
  program?: string;
  folder?: string;
  package?: string;
  cgroup?: string;
  started_at?: number;
  cpu_seconds: number;
  nice: number;
  policy: string;
  realtime_priority: number;
  disk_priority?: string;
  cpus_allowed?: string;
  threads: number;
  top_threads: { tid: number; name: string; cpu_seconds: number }[];
  voluntary_switches?: number;
  forced_switches?: number;
  open_files?: number;
  sockets?: number;
  limits: { name: string; soft: string; hard: string; unit?: string }[];
  apparmor?: string;
  capabilities: string[];
  seccomp?: string;
  no_new_privileges: boolean;
  own_namespaces?: string[];
  rss: number;
  pss?: number;
  uss?: number;
  swap?: number;
  major_faults: number;
  oom_score?: number;
  oom_score_adj?: number;
  disk_read?: number;
  disk_write?: number;
  gpu_ns?: number;
}

export function getProcessDetail(pid: number): Promise<ProcessDetail> {
  return invoke<ProcessDetail>("process_detail", { pid });
}

/** One installed package in depth (Expert package drawer). */
export interface PackageDetail {
  name: string;
  fields: Record<string, string>;
  manual: boolean;
  needs: string[];
  needed_by: string[];
  installed?: string;
  candidate?: string;
  sources: [string, string[]][];
  files: number;
  changed_files: [string, string][];
}

export function getPackageDetail(name: string): Promise<PackageDetail> {
  return invoke<PackageDetail>("package_detail", { name });
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
