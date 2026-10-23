// SPDX-License-Identifier: Apache-2.0
// The fix drawer (docs/03-ui-design/04-behavior/fixes.md): plan → preview (untick steps) →
// apply with live progress → verify. Admin steps make polkit ask for the password.

import { listen } from "@tauri-apps/api/event";
import { createContext, useContext, useEffect, useState } from "react";
import { type FixPlan, type StepResult, USER_CALLS, applyFix, getFixPlan, getStatus, toAppError } from "../api";
import { AlertBanner } from "../components/AlertBanner";
import { Button } from "../components/Button";
import { Icon } from "../components/Icon";
import { StatusBadge } from "../components/StatusBadge";
import { bytes } from "../metrics";
import { useUi } from "../ui";
import { Drawer } from "./Drawer";

/** Opens the fix drawer from anywhere: `openFix("storage.free_space")`. */
export const FixContext = createContext<(action: string, subject?: string) => void>(() => undefined);
export const useFix = () => useContext(FixContext);

const UNDO: Record<string, [Parameters<typeof StatusBadge>[0]["variant"], string]> = { full: ["ok", "Reversible"], regenerates: ["info", "Regenerates"], none: ["warn", "Irreversible"] };

type Phase = { kind: "loading" } | { kind: "error"; message: string } | { kind: "preview" } | { kind: "running" } | { kind: "done"; fixed: boolean | null };

export function FixDrawer({ action, subject, onClose, onDone }: { action: string; subject?: string; onClose: () => void; onDone?: () => void }) {
  const { mode } = useUi();
  const [plan, setPlan] = useState<FixPlan | null>(null);
  const [chosen, setChosen] = useState<Set<string>>(new Set());
  const [phase, setPhase] = useState<Phase>({ kind: "loading" });
  const [results, setResults] = useState<StepResult[]>([]);

  useEffect(() => {
    getFixPlan(action, subject).then(
      (p) => {
        setPlan(p);
        setChosen(new Set(p.steps.filter((s) => s.checked).map((s) => s.id)));
        setPhase({ kind: "preview" });
      },
      (e) => setPhase({ kind: "error", message: toAppError(e).message }),
    );
  }, [action, subject]);

  const apply = async () => {
    if (!plan) return;
    setPhase({ kind: "running" });
    setResults([]);
    let unlisten: (() => void) | undefined;
    try {
      unlisten = await listen<StepResult>("fix-progress", (e) => setResults((r) => [...r, e.payload]));
    } catch {
      // Outside Tauri (tests) there are no events; results arrive at the end.
    }
    try {
      const { run } = await applyFix(plan, [...chosen]);
      setResults(run.steps);
      // The agent checks again right away; give it a moment, then see if the finding is gone.
      let fixed: boolean | null = null;
      if (plan.rule && run.status === "succeeded") {
        await new Promise((r) => setTimeout(r, 2500));
        const status = await getStatus().catch(() => null);
        fixed = status ? !status.findings.some((f) => f.rule === plan.rule && (plan.subject === undefined || f.subject === plan.subject || f.subject === undefined)) : null;
      }
      setPhase({ kind: "done", fixed });
      onDone?.();
    } catch (e) {
      setPhase({ kind: "error", message: toAppError(e).message });
    } finally {
      unlisten?.();
    }
  };

  const picked = plan?.steps.filter((s) => chosen.has(s.id)) ?? [];
  const total = picked.reduce((sum, s) => sum + (s.size ?? 0), 0);
  const admin = picked.some((s) => !USER_CALLS.includes(s.call.op));
  const running = phase.kind === "running";
  const result = (id: string) => results.find((r) => r.id === id);

  return (
    <Drawer
      title={plan?.title ?? "Fix"}
      subtitle={plan?.subtitle}
      note={phase.kind === "preview" ? (admin ? "Admin steps ask for your password once each" : "No password needed") : undefined}
      onClose={() => !running && onClose()}
      actions={
        phase.kind === "preview" && plan && plan.steps.length > 0 ? (
          <Button variant="primary" disabled={chosen.size === 0} onClick={() => void apply()}>
            {total ? `Apply · frees ${bytes(total)}` : "Apply"}
          </Button>
        ) : phase.kind === "done" || phase.kind === "error" ? (
          <Button onClick={onClose}>Close</Button>
        ) : undefined
      }
    >
      {phase.kind === "loading" && <p className="muted">Working out what this fix would do…</p>}
      {phase.kind === "error" && <AlertBanner tone="crit" title="The fix could not run" text={phase.message} />}
      {plan && plan.steps.length === 0 && phase.kind === "preview" && <p className="muted">Nothing to do right now: there is nothing this fix could change.</p>}
      {plan && plan.steps.length > 0 && phase.kind !== "loading" && (
        <ul className="fix-steps">
          {plan.steps.map((s) => {
            const r = result(s.id);
            const on = chosen.has(s.id);
            return (
              <li key={s.id} className={on ? "" : "off"}>
                {phase.kind === "preview" ? (
                  <input
                    type="checkbox"
                    aria-label={s.title}
                    checked={on}
                    onChange={() => {
                      const next = new Set(chosen);
                      if (on) next.delete(s.id);
                      else next.add(s.id);
                      setChosen(next);
                    }}
                  />
                ) : r ? (
                  <span className={r.ok ? "runner-check" : "fix-fail"}>
                    <Icon name={r.ok ? "check" : "x"} size={12} strokeWidth={3} />
                  </span>
                ) : on && running ? (
                  <span className="runner-spin" aria-label="Running" />
                ) : (
                  <span className="runner-dot" />
                )}
                <span className="fix-step-body">
                  <span className="fix-step-title">{s.title}</span>
                  <span className="fix-step-desc">{r ? r.message : s.description}</span>
                  <span className="fix-step-tags">
                    <StatusBadge variant={UNDO[s.undo][0]} label={UNDO[s.undo][1]} />
                    {!USER_CALLS.includes(s.call.op) && <StatusBadge variant="off" label="Admin" />}
                    {s.size ? <span className="muted">{bytes(s.size)}</span> : null}
                  </span>
                  {mode === "expert" && <code className="fix-cmd">{s.command}</code>}
                </span>
              </li>
            );
          })}
        </ul>
      )}
      {phase.kind === "done" && (
        <AlertBanner
          tone={results.every((r) => r.ok) ? (phase.fixed === false ? "warn" : "ok") : "crit"}
          title={results.every((r) => r.ok) ? (phase.fixed === false ? "Done, but the problem is still there" : "Done") : "A step failed; later steps did not run"}
          text={phase.fixed === false ? "DeeCure checked again and the issue remains. See the details on its page." : phase.fixed ? "DeeCure checked again: the issue is gone." : "This change is listed in Activity › DeeCure Actions."}
        />
      )}
    </Drawer>
  );
}
