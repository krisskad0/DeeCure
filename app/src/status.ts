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

/**
 * The panels on that sub-page that show what a rule is about, so they can be marked. A panel
 * whose title starts with one of these and " ·" (such as "Battery details · BAT0") counts too.
 * A rule with no entry is only explained at the top of the sub-page.
 */
export const RULE_PANELS: Record<string, string[]> = {
  "accounts.extra_root": ["Users"],
  "backups.none": ["Backups"],
  "backups.overdue": ["Backups"],
  "containers.port_exposed": ["Docker containers"],
  "hardware.cooling_weak": ["Temperatures", "Fans"],
  "hardware.cpu_hot": ["Temperatures"],
  "hardware.fan_stopped": ["Fans"],
  "hardware.firmware_crash": ["Hardware errors"],
  "hardware.gpu_hang": ["Driver checks"],
  "hardware.machine_check": ["Hardware errors", "Components"],
  "hardware.memory_errors": ["Health"],
  "hardware.no_acceleration": ["Driver checks"],
  "hardware.part_hot": ["Temperatures"],
  "hardware.pcie_errors": ["PCIe links"],
  "hardware.thermal_shutdown": ["Temperature history"],
  "hardware.usb_overcurrent": ["Devices"],
  "jobs.job_failed": ["Timers"],
  "network.errors": ["Errors and drops per interface"],
  "network.no_internet": ["Connectivity"],
  "network.offline": ["Connectivity"],
  "network.weak_wifi": ["Wi-Fi"],
  "packages.broken": ["Packages"],
  "packages.updates_available": ["Packages"],
  "performance.disk_stall": ["Devices"],
  "performance.memory_short": ["Memory usage"],
  "performance.oom_kill": ["Out-of-memory guard", "Memory usage"],
  "performance.swap_full": ["Swap and compression", "Memory usage"],
  "power.battery_empty": ["Battery"],
  "power.battery_fault": ["Battery details", "Battery"],
  "power.battery_hot": ["Battery details", "Battery"],
  "power.battery_worn": ["Battery health"],
  "power.charger_weak": ["Chargers"],
  "privacy.popcon": ["Settings"],
  "security.apparmor_off": ["Security posture"],
  "security.auto_updates_stopped": ["Security posture"],
  "security.cpu_flaws": ["Security posture"],
  "security.firewall_off": ["Security posture"],
  "security.secure_boot_off": ["Security posture"],
  "security.ssh_root_password": ["Security posture"],
  "security.updates_pending": ["Security posture"],
  "services.restart_loop": ["Units"],
  "services.unit_failed": ["Units"],
  "storage.boot_low": ["Mount points"],
  "storage.disk_errors": ["Filesystem checks"],
  "storage.disk_low": ["Mount points"],
  "storage.filling_up": ["Usage trend and forecast"],
  "storage.fs_errors": ["Filesystem checks"],
  "storage.inodes_full": ["Space allocation"],
  "storage.read_only": ["Filesystems"],
  "storage.root_almost_full": ["Space allocation"],
  "system.clock_unsynced": ["Date and time"],
  "system.reboot_required": ["Pending updates"],
  "system.time_sync_off": ["Date and time"],
};

/** Whether a panel title is one of the given titles, or one of them with " · something" after it. */
export function panelMatches(title: string, wanted: string): boolean {
  return title === wanted || title.startsWith(`${wanted} ·`);
}

/** Open findings shown on one sub-page, most serious first. */
export function findingsHere(pageId: string, tabId: string, report: StatusReport | null): Finding[] {
  if (pageId === "overview") return [];
  return findingsFor(pageId, report)
    .filter((f) => tabOf(f) === tabId)
    .sort((a, b) => RANK[b.severity] - RANK[a.severity]);
}

/** The tab a finding belongs to, or undefined when no rule says. */
export function tabOf(finding: Finding): string | undefined {
  return RULE_TAB[finding.rule];
}

/** A warning or critical dot for one sub-page; `null` when it has nothing open. */
export function tabStatus(pageId: string, tabId: string, report: StatusReport | null): StatusVariant | null {
  const worst = findingsHere(pageId, tabId, report)[0];
  return worst ? VARIANT[worst.severity] : null;
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
