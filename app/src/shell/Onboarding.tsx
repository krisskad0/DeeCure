// SPDX-License-Identifier: Apache-2.0
// Onboarding and first scan (docs/03-ui-design/05-pages/00-onboarding.md): four short
// screens, then a first check where each area is a real request to the agent.

import { type ReactNode, useEffect, useState } from "react";
import { type AppError, type UnitState, getInventory, getLatest, getSnapshot, getStatus, toAppError } from "../api";
import { Button } from "../components/Button";
import { accessOptions, useAccess } from "../access";
import { RadioCards } from "../components/FormControls";
import { Icon } from "../components/Icon";
import { NoData } from "../pages/NoData";
import { usePref } from "../prefs";
import type { IconName } from "../spec/icons";
import { type Mode, atLeast } from "../ui";

interface Area {
  name: string;
  /** Resolves to a one-line result, or throws. */
  check: () => Promise<string>;
}

const AREAS: Area[] = [
  {
    name: "Hardware",
    check: async () => {
      const hw = await getInventory("hardware");
      return [hw.machine.model, `${hw.processor.threads} threads`].filter(Boolean).join(" · ");
    },
  },
  {
    name: "System",
    check: async () => {
      const s = await getInventory("system");
      return [s.os, s.kernel && `kernel ${s.kernel}`].filter(Boolean).join(" · ");
    },
  },
  {
    name: "Storage",
    check: async () => {
      const used = (await getLatest()).find((m) => m.name === "disk.used_percent" && m.subject === "/")?.value;
      if (used === undefined) throw new Error("measured within a minute of starting");
      // Start the folder-size scan now, so Disk Usage is ready later.
      void getInventory("disk_usage").catch(() => undefined);
      return `${Math.round(used)}% of the main disk used`;
    },
  },
  {
    name: "Services",
    check: async () => {
      const snap = await getSnapshot<UnitState[]>("services");
      if (!snap) throw new Error("read within a few minutes of starting");
      const failed = snap.data.filter((u) => u.active === "failed").length;
      return `${snap.data.filter((u) => u.active === "active").length} running${failed ? ` · ${failed} failed` : ""}`;
    },
  },
  {
    name: "Apps",
    check: async () => {
      const p = await getInventory("packages");
      return `${p.packages.length.toLocaleString()} packages installed`;
    },
  },
  {
    name: "Network",
    check: async () => {
      const n = await getInventory("net_details");
      const online = n.connectivity === "full" ? "online" : (n.connectivity ?? "connected");
      return [n.wifi?.ssid ? `Wi-Fi ${n.wifi.ssid}` : undefined, online].filter(Boolean).join(" · ");
    },
  },
  {
    name: "Security",
    check: async () => {
      const s = await getInventory("security");
      const pass = s.hardening.filter((h) => h.result === "pass").length;
      return `${pass} of ${s.hardening.length} kernel safety settings in place`;
    },
  },
];

type Row = { state: "running" } | { state: "done"; text: string } | { state: "na"; text: string };

function FirstCheck({ onDone }: { onDone: () => void }) {
  const [rows, setRows] = useState<Row[]>(AREAS.map(() => ({ state: "running" })));
  const [score, setScore] = useState<number | undefined>(undefined);
  const [error, setError] = useState<AppError | null>(null);
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    let alive = true;
    setError(null);
    setRows(AREAS.map(() => ({ state: "running" })));
    getStatus().then(
      (r) => alive && setScore(r.score),
      (e) => alive && setError(toAppError(e)),
    );
    AREAS.forEach((area, i) => {
      area.check().then(
        (text) => alive && setRows((old) => old.map((r, k) => (k === i ? { state: "done", text } : r))),
        (e) => alive && setRows((old) => old.map((r, k) => (k === i ? { state: "na", text: e instanceof Error ? e.message : toAppError(e).message } : r))),
      );
    });
    return () => {
      alive = false;
    };
  }, [attempt]);

  if (error?.kind === "agent_not_running") {
    return (
      <>
        <NoData error={error} waiting="" />
        <div className="onb-actions">
          <span />
          <Button variant="primary" onClick={() => setAttempt((a) => a + 1)}>
            Try again
          </Button>
        </div>
      </>
    );
  }
  const finished = rows.every((r) => r.state !== "running");
  return (
    <>
      <ul className="onb-areas">
        {AREAS.map((area, i) => {
          const r = rows[i];
          return (
            <li key={area.name}>
              {r.state === "running" ? <span className="runner-spin" aria-label="Checking" /> : r.state === "done" ? <span className="runner-check"><Icon name="check" size={12} strokeWidth={3} /></span> : <span className="runner-dot" />}
              <span className="onb-area-name">{area.name}</span>
              <span className={r.state === "na" ? "muted" : ""}>{r.state === "running" ? "Checking…" : r.state === "na" ? `Not available yet: ${r.text}` : r.text}</span>
            </li>
          );
        })}
      </ul>
      <p className="onb-score">{score === undefined ? <span className="muted">The health score appears after the first rules check.</span> : <>Health score: <b>{score}</b>/100</>}</p>
      <div className="onb-actions">
        <span />
        <Button variant="primary" disabled={!finished} onClick={onDone}>
          Open DeeCure
        </Button>
      </div>
    </>
  );
}

const PROMISES: [IconName, string][] = [
  ["lock", "No account and no sign-in."],
  ["shield", "Nothing about your computer is sent anywhere. The only connection is a daily check for DeeCure updates on GitHub; turn it off in Settings › About."],
  ["eye", "It only reads. Changes always need your OK, with a preview and undo."],
];

const MODES = [
  { value: "beginner" as const, title: "Yes, keep it simple", description: "Beginner · the pages most people need, in plain words" },
  { value: "advanced" as const, title: "I know my way around", description: "Advanced · every page and tab" },
  { value: "expert" as const, title: "Show me everything", description: "Expert · every page, plus data sources, commands, raw columns, and deep details" },
];

/** The first check is the last step. */
const CHECK = 4;

export function Onboarding({ onDone }: { onDone: () => void }) {
  const [step, setStep] = useState(0);
  const access = useAccess();
  const [mode, setMode] = usePref<Mode>("mode", "beginner");
  const next = () => setStep((s) => Math.min(CHECK, s + 1));
  let body: ReactNode;
  let title: string;
  if (step === 0) {
    title = "Welcome to DeeCure";
    body = <p>DeeCure looks at your whole computer - hardware, system, apps, storage, network, and security - and explains what it finds in plain words. Problems come first, with what to do about them.</p>;
  } else if (step === 1) {
    title = "Everything stays here";
    body = (
      <ul className="onb-promises">
        {PROMISES.map(([icon, text]) => (
          <li key={text}>
            <Icon name={icon} size={18} />
            <span>{text}</span>
          </li>
        ))}
      </ul>
    );
  } else if (step === 2) {
    title = "Are you new to Linux?";
    body = (
      <>
        <RadioCards name="onb-mode" value={atLeast(mode, "expert") ? "expert" : atLeast(mode, "advanced") ? "advanced" : "beginner"} options={MODES} onChange={setMode} />
        <p className="muted">You can switch any time with Beginner / Advanced / Expert at the top.</p>
      </>
    );
  } else if (step === 3) {
    title = "How much should DeeCure see?";
    body = (
      <>
        <RadioCards name="onb-access" value={access.chosen === "full" && access.helper ? "full" : "basic"} options={accessOptions(access.helper)} onChange={access.set} />
        <p className="muted">You can change this later in Settings › Access.</p>
      </>
    );
  } else {
    title = "Checking your computer…";
    body = <FirstCheck onDone={onDone} />;
  }
  return (
    <div className="onb" role="dialog" aria-label={title}>
      <div className="onb-card">
        <div className="onb-top">
          <img src="/deecure.svg" alt="" width={48} height={48} onError={(e) => (e.currentTarget.style.display = "none")} />
          <span className="onb-dots" aria-label={`Step ${step + 1} of ${CHECK + 1}`}>
            {Array.from({ length: CHECK + 1 }, (_, i) => (
              <i key={i} className={i < step ? "done" : i === step ? "current" : ""} />
            ))}
          </span>
        </div>
        <h1>{title}</h1>
        {body}
        {step < CHECK && (
          <div className="onb-actions">
            <span>
              {step > 0 && (
                <Button variant="link" onClick={() => setStep(step - 1)}>
                  Back
                </Button>
              )}
              <Button variant="link" onClick={onDone}>
                Skip
              </Button>
            </span>
            <Button variant="primary" onClick={next} autoFocus>
              {step === CHECK - 1 ? "Start the first check" : "Next"}
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}

