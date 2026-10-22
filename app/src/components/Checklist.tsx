// SPDX-License-Identifier: Apache-2.0
// 09 Checklist (docs/03-ui-design/03-components/09-checklist.md).

import type { ReactNode } from "react";
import type { IconName } from "../spec/icons";
import { Icon } from "./Icon";

export type CheckState = "pass" | "fail" | "warn" | "na";

const ICON: Record<CheckState, IconName> = { pass: "check", fail: "x", warn: "excl", na: "minus" };
const WORD: Record<CheckState, string> = { pass: "Passed", fail: "Failed", warn: "Warning", na: "Not applicable" };

export interface Check {
  state: CheckState;
  title: string;
  detail?: string;
  action?: ReactNode;
}

export function Checklist({ checks }: { checks: Check[] }) {
  return (
    <ul className="checklist">
      {checks.map((check) => (
        <li className="check" key={check.title}>
          <span className={`check-icon check-${check.state}`} role="img" aria-label={WORD[check.state]}>
            <Icon name={ICON[check.state]} size={12} strokeWidth={2.8} />
          </span>
          <span>
            <span className="check-title">{check.title}</span>
            {check.detail && <span className="check-detail">{check.detail}</span>}
          </span>
          {check.action ?? <span />}
        </li>
      ))}
    </ul>
  );
}
