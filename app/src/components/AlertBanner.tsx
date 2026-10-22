// SPDX-License-Identifier: Apache-2.0
// 11 Alert banner (docs/03-ui-design/03-components/11-alert-banner.md).

import type { ReactNode } from "react";
import type { IconName } from "../spec/icons";
import { Icon } from "./Icon";

const ICON: Record<"crit" | "warn" | "info" | "ok", IconName> = { crit: "x", warn: "excl", info: "info", ok: "check" };

export function AlertBanner(props: { tone: "crit" | "warn" | "info" | "ok"; title: string; text?: string; actions?: ReactNode }) {
  return (
    <div className="alert span-12" role={props.tone === "crit" ? "alert" : "status"}>
      <span className={`alert-icon alert-${props.tone}`}>
        <Icon name={ICON[props.tone]} size={16} strokeWidth={2.4} />
      </span>
      <span className="alert-body">
        <span className="alert-title">{props.title}</span>
        {props.text && <span className="alert-text">{props.text}</span>}
      </span>
      {props.actions && <span className="alert-actions">{props.actions}</span>}
    </div>
  );
}
