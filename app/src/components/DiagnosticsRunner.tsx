// SPDX-License-Identifier: Apache-2.0
// 23 Diagnostics runner (docs/03-ui-design/03-components/23-diagnostics-runner.md).
// Idle: a grid of steps and a Run button. Running: a progress bar, and steps that
// advance every 450 ms while the real check runs. Done: the result as an inline alert.

import { type ReactNode, useEffect, useRef, useState } from "react";
import { Button } from "./Button";
import { Icon } from "./Icon";

export type RunnerState = "idle" | "running" | "done";

export function DiagnosticsRunner<R>(props: {
  steps: string[];
  button: string;
  run: () => Promise<R>;
  /** What to show when the check is done (an alert and any extra widgets). */
  result: (r: R) => ReactNode;
  /** Shown when the check failed to run. */
  onError?: (e: unknown) => ReactNode;
}) {
  const [state, setState] = useState<RunnerState>("idle");
  const [step, setStep] = useState(0);
  const [result, setResult] = useState<R | null>(null);
  const [error, setError] = useState<unknown>(null);
  const timer = useRef<ReturnType<typeof setInterval> | null>(null);
  const n = props.steps.length;

  useEffect(() => () => {
    if (timer.current) clearInterval(timer.current);
  }, []);

  const start = async () => {
    setState("running");
    setStep(0);
    setResult(null);
    setError(null);
    // The animation stops one step short until the real answer arrives.
    timer.current = setInterval(() => setStep((s) => Math.min(s + 1, n - 1)), 450);
    try {
      const r = await props.run();
      setResult(r);
    } catch (e) {
      setError(e);
    }
    if (timer.current) clearInterval(timer.current);
    setStep(n);
    setState("done");
  };

  return (
    <div className="runner">
      {state === "running" && (
        <div className="runner-progress" aria-hidden="true">
          <span style={{ width: `${(step / n) * 100}%` }} />
        </div>
      )}
      <ol className="runner-steps" aria-label="Steps">
        {props.steps.map((name, i) => {
          const done = state === "done" || i < step;
          const current = state === "running" && i === step;
          return (
            <li key={name} className={`runner-step ${done ? "done" : ""} ${current ? "current" : ""}`}>
              {done ? (
                <span className="runner-check">
                  <Icon name="check" size={12} strokeWidth={3} />
                </span>
              ) : current ? (
                <span className="runner-spin" aria-label="Running" />
              ) : (
                <span className="runner-dot" />
              )}
              <span>{name}</span>
            </li>
          );
        })}
      </ol>
      {state === "done" && error !== null && (props.onError ? props.onError(error) : <p className="muted">The check could not run: {String(error)}</p>)}
      {state === "done" && result !== null && props.result(result)}
      <div className="runner-actions">
        <Button variant="primary" icon="play" disabled={state === "running"} onClick={() => void start()}>
          {state === "done" ? "Run again" : state === "running" ? "Running…" : props.button}
        </Button>
      </div>
    </div>
  );
}
