// SPDX-License-Identifier: Apache-2.0
// 03 Buttons (docs/03-ui-design/03-components/03-buttons.md).

import type { ButtonHTMLAttributes } from "react";
import type { IconName } from "../spec/icons";
import { Icon } from "./Icon";

export function Button({
  variant = "default",
  small = false,
  icon,
  children,
  className = "",
  ...rest
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: "default" | "primary" | "link" | "danger";
  small?: boolean;
  icon?: IconName;
}) {
  return (
    <button type="button" className={`btn btn-${variant} ${small ? "btn-sm" : ""} ${className}`} {...rest}>
      {icon && <Icon name={icon} size={small ? 14 : 15} />}
      {children}
    </button>
  );
}

export function IconButton({ icon, label, ...rest }: ButtonHTMLAttributes<HTMLButtonElement> & { icon: IconName; label: string }) {
  return (
    <button type="button" className="ibtn" aria-label={label} title={label} {...rest}>
      <Icon name={icon} size={18} />
    </button>
  );
}
