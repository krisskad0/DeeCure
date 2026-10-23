// SPDX-License-Identifier: Apache-2.0
// Data only root can read. With the admin helper installed, a button asks for the
// password (polkit) and shows the result; without it, the panel says why it is empty.

import { type ReactNode, useEffect, useState } from "react";
import { getHelperStatus, helperRead, toAppError } from "../api";
import { Button } from "./Button";
import { usePref } from "../prefs";
import { href } from "../route";

let installed: Promise<boolean> | null = null;

export function useHelperInstalled(): boolean | null {
  const [value, setValue] = useState<boolean | null>(null);
  useEffect(() => {
    // Any failure (no Tauri, an older app) means "not installed", never a crash.
    installed ??= Promise.resolve()
      .then(() => getHelperStatus())
      .catch(() => false);
    let alive = true;
    void installed.then((v) => alive && setValue(v));
    return () => {
      alive = false;
    };
  }, []);
  return value;
}

export function AdminRead({ what, arg = "", label, children }: { what: Parameters<typeof helperRead>[0]; arg?: string; label: string; children: (text: string) => ReactNode }) {
  const helper = useHelperInstalled();
  const [chosen] = usePref<"basic" | "full">("access", "basic");
  const [text, setText] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  if (text !== null) return <>{children(text)}</>;
  if (helper === false) return <div className="empty muted">Only the admin can read this. It needs Full access, which needs the DeeCure admin helper (Settings › Access).</div>;
  if (helper && chosen !== "full")
    return (
      <div className="empty muted">
        Only the admin can read this. Turn on Full access in <a href={href("settings", "access")}>Settings › Access</a> to read it with your password.
      </div>
    );
  return (
    <div className="empty">
      <p className="muted">{error ?? "Only the admin can read this."}</p>
      <Button
        small
        disabled={busy || helper === null}
        onClick={() => {
          setBusy(true);
          setError(null);
          helperRead(what, arg).then(
            (t) => setText(t),
            (e) => {
              setError(toAppError(e).message);
              setBusy(false);
            },
          );
        }}
      >
        {busy ? "Waiting for approval…" : label}
      </Button>
    </div>
  );
}
