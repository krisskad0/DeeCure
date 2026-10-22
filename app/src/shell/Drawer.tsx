// SPDX-License-Identifier: Apache-2.0
// Side drawer (docs/03-ui-design/02-shell/overlays.md): row inspect and fix actions.

import { type ReactNode, useEffect, useRef } from "react";
import { IconButton } from "../components/Button";

export function Drawer(props: { title: string; subtitle?: string; note?: string; actions?: ReactNode; onClose: () => void; children: ReactNode }) {
  const panel = useRef<HTMLDivElement>(null);
  const { onClose } = props;
  useEffect(() => {
    panel.current?.focus();
    const onKey = (event: KeyboardEvent) => event.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);
  return (
    <>
      <div className="scrim scrim-palette" onClick={onClose} />
      <aside className="drawer" role="dialog" aria-label={props.title} tabIndex={-1} ref={panel}>
        <header className="drawer-head">
          <div>
            <h2>{props.title}</h2>
            {props.subtitle && <div className="drawer-sub">{props.subtitle}</div>}
          </div>
          <IconButton icon="x" label="Close" onClick={onClose} />
        </header>
        <div className="drawer-body">{props.children}</div>
        {(props.note || props.actions) && (
          <footer className="drawer-foot">
            <span className="drawer-note">{props.note}</span>
            <span className="drawer-actions">{props.actions}</span>
          </footer>
        )}
      </aside>
    </>
  );
}
