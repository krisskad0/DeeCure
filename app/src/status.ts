// SPDX-License-Identifier: Apache-2.0
// Page status (docs/03-ui-design/04-behavior/page-status.md), worked out from real findings.
// A page is "Healthy" only if DeeCure really checks that area; otherwise it is "Unknown".

import type { AppError, Finding, StatusReport } from "./api";
import type { StatusVariant } from "./components/StatusBadge";

/**
 * Pages that are not a health area never show a status badge: tools, Settings, and pages that
 * show records or live views (Logs, Activity, Processes, Display & Audio). No warning covers
 * them, so a badge could only ever say "Unknown".
 */
const NO_BADGE = new Set(["optimize", "diagnostics", "assistant", "settings", "logs", "activity", "processes", "display"]);
const RANK: Record<Finding["severity"], number> = { info: 1, warning: 2, critical: 3 };
const VARIANT: Record<Finding["severity"], StatusVariant> = { info: "info", warning: "warn", critical: "crit" };

export function severityVariant(severity: Finding["severity"]): StatusVariant {
  return VARIANT[severity];
}

/** The page a finding belongs to: the category before the dot in its rule ID. */
export function pageOf(finding: Finding): string {
  return finding.rule.split(".")[0];
}

export function findingsFor(pageId: string, report: StatusReport | null): Finding[] {
  if (!report) return [];
  return pageId === "overview" ? report.findings : report.findings.filter((f) => pageOf(f) === pageId);
}

/**
 * The sub-page (tab) that shows what a rule is about, so the sidebar can mark it too.
 * Every rule in rules/ has an entry; a test keeps that true.
 */
export const RULE_TAB: Record<string, string> = {
  "accounts.extra_root": "users",
  "backups.none": "jobs",
  "backups.overdue": "jobs",
  "containers.port_exposed": "containers",
  "hardware.cooling_weak": "sensors",
  "hardware.cpu_hot": "sensors",
  "hardware.fan_stopped": "sensors",
  "hardware.firmware_crash": "summary",
  "hardware.gpu_hang": "graphics",
  "hardware.machine_check": "summary",
  "hardware.memory_errors": "memory",
  "hardware.no_acceleration": "graphics",
  "hardware.part_hot": "sensors",
  "hardware.pcie_errors": "peripherals",
  "hardware.thermal_shutdown": "sensors",
  "hardware.usb_overcurrent": "peripherals",
  "jobs.job_failed": "all",
  "network.errors": "interfaces",
  "network.no_internet": "interfaces",
  "network.offline": "interfaces",
  "network.weak_wifi": "interfaces",
  "packages.broken": "installed",
  "packages.updates_available": "installed",
  "performance.disk_stall": "disk",
  "performance.memory_short": "memory",
  "performance.oom_kill": "memory",
  "performance.swap_full": "memory",
  "power.battery_empty": "battery",
  "power.battery_fault": "battery",
  "power.battery_hot": "battery",
  "power.battery_worn": "battery",
  "power.charger_weak": "battery",
  "privacy.popcon": "settings",
  "security.apparmor_off": "overview",
  "security.auto_updates_stopped": "overview",
  "security.cpu_flaws": "overview",
  "security.firewall_off": "overview",
  "security.secure_boot_off": "overview",
  "security.ssh_root_password": "overview",
  "security.updates_pending": "overview",
  "services.restart_loop": "system",
  "services.unit_failed": "system",
  "storage.boot_low": "overview",
  "storage.disk_errors": "filesystems",
  "storage.disk_low": "overview",
  "storage.drive_wear": "overview",
  "storage.filling_up": "overview",
  "storage.fs_errors": "filesystems",
  "storage.inodes_full": "overview",
  "storage.read_only": "filesystems",
  "storage.root_almost_full": "overview",
  "system.clock_unsynced": "time",
  "system.kernel_crash": "overview",
  "system.reboot_required": "updates",
  "system.time_sync_off": "time",
};

/** The tab a finding belongs to, or undefined when no rule says. */
export function tabOf(finding: Finding): string | undefined {
  return RULE_TAB[finding.rule];
}

/** A warning or critical dot for one sub-page; `null` when it has nothing open. */
export function tabStatus(pageId: string, tabId: string, report: StatusReport | null): StatusVariant | null {
  const findings = findingsFor(pageId, report).filter((f) => tabOf(f) === tabId);
  if (findings.length === 0) return null;
  return VARIANT[findings.reduce((a, b) => (RANK[b.severity] > RANK[a.severity] ? b : a)).severity];
}

/** `null` means "show no badge". */
export function pageStatus(pageId: string, report: StatusReport | null, error: AppError | null): StatusVariant | null {
  if (NO_BADGE.has(pageId)) return null;
  if (error || !report?.checked_at) return "off";
  const findings = findingsFor(pageId, report);
  if (findings.length === 0) {
    const checked = pageId === "overview" ? (report.checked ?? []).length > 0 : (report.checked ?? []).includes(pageId);
    return checked ? "ok" : "off";
  }
  const worst = findings.reduce((a, b) => (RANK[b.severity] > RANK[a.severity] ? b : a));
  return VARIANT[worst.severity];
}
