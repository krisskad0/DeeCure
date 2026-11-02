// SPDX-License-Identifier: Apache-2.0
// 21 Form controls (docs/03-ui-design/03-components/21-form-controls.md).

import { type KeyboardEvent, type ReactNode, useEffect, useId, useRef, useState } from "react";
import type { IconName } from "../spec/icons";
import { Icon } from "./Icon";

/** A switch: a real checkbox under a 36×20 track. */
export function Switch({ checked, onChange, label, disabled }: { checked: boolean; onChange: (checked: boolean) => void; label: string; disabled?: boolean }) {
  return (
    <span className={`switch${disabled ? " switch-disabled" : ""}`}>
      <input type="checkbox" role="switch" checked={checked} aria-label={label} disabled={disabled} onChange={(e) => onChange(e.target.checked)} />
      <span className="switch-track" aria-hidden="true" />
    </span>
  );
}

/**
 * A dropdown drawn by DeeCure, not by the system: the system's open list cannot be styled and
 * looks out of place (GTK in the desktop app). Same keys as a system one: click, Space, Enter,
 * or the arrows open it; the arrows, Home, and End move; a letter jumps; Enter picks; Escape
 * or a click outside closes. Screen readers hear a combobox with a listbox.
 */
export function Select<T extends string>(props: { value: T; options: readonly { value: T; label: string }[]; onChange: (value: T) => void; label: string; disabled?: boolean }) {
  const { value, options, onChange, label, disabled } = props;
  const [open, setOpen] = useState(false);
  const [up, setUp] = useState(false);
  const selected = Math.max(0, options.findIndex((o) => o.value === value));
  const [active, setActive] = useState(selected);
  const root = useRef<HTMLSpanElement>(null);
  const button = useRef<HTMLButtonElement>(null);
  const list = useId();

  useEffect(() => {
    if (!open) return;
    const outside = (e: MouseEvent) => {
      if (root.current && !root.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", outside);
    return () => document.removeEventListener("mousedown", outside);
  }, [open]);

  const show = () => {
    if (disabled) return;
    // Opens upward when there is not enough room below.
    const box = button.current?.getBoundingClientRect();
    const need = Math.min(options.length, 8) * 32 + 12;
    setUp(box !== undefined && window.innerHeight - box.bottom < need && box.top > need);
    setActive(selected);
    setOpen(true);
  };
  const pick = (i: number) => {
    const o = options[i];
    setOpen(false);
    button.current?.focus();
    if (o && o.value !== value) onChange(o.value);
  };
  const keys = (e: KeyboardEvent) => {
    const last = options.length - 1;
    const move = (to: number) => {
      e.preventDefault();
      if (!open) show();
      setActive(Math.max(0, Math.min(last, to)));
    };
    switch (e.key) {
      case "ArrowDown":
        return move(open ? active + 1 : selected);
      case "ArrowUp":
        return move(open ? active - 1 : selected);
      case "Home":
        return move(0);
      case "End":
        return move(last);
      case "Enter":
      case " ":
        e.preventDefault();
        return open ? pick(active) : show();
      case "Escape":
      case "Tab":
        if (open) setOpen(false);
        return;
      default:
        if (e.key.length === 1 && /\S/.test(e.key)) {
          const k = e.key.toLowerCase();
          const start = open ? active : selected;
          const hit = options.findIndex((o, i) => i > start && o.label.toLowerCase().startsWith(k));
          const at = hit >= 0 ? hit : options.findIndex((o) => o.label.toLowerCase().startsWith(k));
          if (at >= 0) move(at);
        }
    }
  };

  return (
    <span className="dd" ref={root}>
      <button
        ref={button}
        type="button"
        className="select"
        role="combobox"
        aria-label={label}
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-controls={list}
        aria-activedescendant={open ? `${list}-${active}` : undefined}
        disabled={disabled}
        data-value={value}
        onClick={() => (open ? setOpen(false) : show())}
        onKeyDown={keys}
      >
        <span className="dd-label">{options[selected]?.label ?? ""}</span>
      </button>
      {open && (
        <ul className={`dd-menu${up ? " dd-up" : ""}`} role="listbox" id={list} aria-label={label}>
          {options.map((o, i) => (
            <li
              key={o.value}
              id={`${list}-${i}`}
              role="option"
              aria-selected={o.value === value}
              data-value={o.value}
              className={`dd-option${i === active ? " active" : ""}`}
              onMouseEnter={() => setActive(i)}
              onMouseDown={(e) => {
                e.preventDefault();
                pick(i);
              }}
            >
              <span>{o.label}</span>
              {o.value === value && <Icon name="check" size={14} />}
            </li>
          ))}
        </ul>
      )}
    </span>
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
