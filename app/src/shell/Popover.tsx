// SPDX-License-Identifier: Apache-2.0
// Popover (docs/03-ui-design/02-shell/overlays.md): fixed under its trigger; a click
// outside or Esc closes it.

import { useEffect, type ReactNode } from "react";

export function Popover(props: {
  anchor: HTMLElement;
  align: "left" | "right";
  label: string;
  onClose: () => void;
  children: ReactNode;
}) {
  const { onClose } = props;
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => event.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const box = props.anchor.getBoundingClientRect();
  const place =
    props.align === "right"
      ? { top: box.bottom + 6, right: Math.max(12, window.innerWidth - box.right) }
      : { top: box.bottom + 6, left: Math.max(12, box.left) };
  return (
    <>
      <div className="popover-layer" onClick={onClose} />
      <div className="popover" role="dialog" aria-label={props.label} style={place}>
        {props.children}
      </div>
    </>
  );
}
