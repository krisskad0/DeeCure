// SPDX-License-Identifier: Apache-2.0
// Status badge: icon + word + color, never color alone.
// Spec: docs/03-ui-design/03-components/02-status-badge.md

import type { IconName } from "../spec/icons";
import { Icon } from "./Icon";

export type StatusVariant = "ok" | "warn" | "crit" | "info" | "off";

const LABELS: Record<StatusVariant, string> = {
  ok: "Healthy",
  warn: "Warning",
  crit: "Critical",
  info: "Info",
  off: "Unknown",
};

const ICON: Record<StatusVariant, IconName | null> = {
  ok: "check",
  warn: "excl",
  crit: "x",
  info: "info",
  off: null,
};

export function StatusBadge({ variant, label }: { variant: StatusVariant; label?: string }) {
  const icon = ICON[variant];
  return (
    <span className={`badge badge-${variant}`}>
      {icon && <Icon name={icon} size={12} strokeWidth={2.6} />}
      {label ?? LABELS[variant]}
    </span>
  );
}
