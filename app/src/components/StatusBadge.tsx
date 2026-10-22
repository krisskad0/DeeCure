// SPDX-License-Identifier: Apache-2.0
// Status badge: icon + word + color, never color alone.
// Spec: docs/03-ui-design/03-components/02-status-badge.md

export type StatusVariant = "ok" | "warn" | "crit" | "info" | "off";

const LABELS: Record<StatusVariant, string> = {
  ok: "Healthy",
  warn: "Warning",
  crit: "Critical",
  info: "Info",
  off: "Unknown",
};

const ICONS: Record<StatusVariant, string | null> = {
  ok: "M5 12.5l4.5 4.5L19 7.5",
  warn: "M12 6v7.5M12 17.5v.5",
  crit: "M7 7l10 10M17 7L7 17",
  info: "M12 10.5V17M12 6.5v.5",
  off: null,
};

export function StatusBadge({ variant, label }: { variant: StatusVariant; label?: string }) {
  const icon = ICONS[variant];
  return (
    <span className={`badge badge-${variant}`}>
      {icon && (
        <svg viewBox="0 0 24 24" width="12" height="12" aria-hidden="true">
          <path d={icon} />
        </svg>
      )}
      {label ?? LABELS[variant]}
    </span>
  );
}
