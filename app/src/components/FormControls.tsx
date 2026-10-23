// SPDX-License-Identifier: Apache-2.0
// 21 Form controls (docs/03-ui-design/03-components/21-form-controls.md).

import type { ReactNode } from "react";
import type { IconName } from "../spec/icons";
import { Icon } from "./Icon";

/** A switch: a real checkbox under a 36×20 track. */
export function Switch({ checked, onChange, label }: { checked: boolean; onChange: (checked: boolean) => void; label: string }) {
  return (
    <span className="switch">
      <input type="checkbox" role="switch" checked={checked} aria-label={label} onChange={(e) => onChange(e.target.checked)} />
      <span className="switch-track" aria-hidden="true" />
    </span>
  );
}

export function Select<T extends string>(props: { value: T; options: readonly { value: T; label: string }[]; onChange: (value: T) => void; label: string }) {
  return (
    <select className="select" value={props.value} aria-label={props.label} onChange={(e) => props.onChange(e.target.value as T)}>
      {props.options.map((o) => (
        <option key={o.value} value={o.value}>
          {o.label}
        </option>
      ))}
    </select>
  );
}

export function RadioCards<T extends string>(props: {
  name: string;
  value: T;
  options: readonly { value: T; title: string; description: string; disabled?: boolean }[];
  onChange: (value: T) => void;
}) {
  return (
    <div className="radio-cards" role="radiogroup">
      {props.options.map((o) => (
        <label key={o.value} className={`radio-card ${o.value === props.value ? "checked" : ""} ${o.disabled ? "disabled" : ""}`}>
          <input type="radio" name={props.name} value={o.value} checked={o.value === props.value} disabled={o.disabled} onChange={() => props.onChange(o.value)} />
          <span>
            <span className="rc-title">{o.title}</span>
            <span className="rc-desc">{o.description}</span>
          </span>
        </label>
      ))}
    </div>
  );
}

/** One setting: label and description on the left, its control on the right. */
export function FieldRow({ label, description, children }: { label: string; description?: string; children: ReactNode }) {
  return (
    <div className="field-row">
      <span>
        <span className="field-label">{label}</span>
        {description && <span className="field-desc">{description}</span>}
      </span>
      <span>{children}</span>
    </div>
  );
}

export function StateTiles({ tiles }: { tiles: { icon: IconName; title: string; state: string }[] }) {
  return (
    <div className="state-tiles">
      {tiles.map((t) => (
        <div className="state-tile" key={t.title}>
          <span className="st-icon">
            <Icon name={t.icon} size={18} />
          </span>
          <span>
            <span className="st-title">{t.title}</span>
            <span className="st-state">{t.state}</span>
          </span>
        </div>
      ))}
    </div>
  );
}
