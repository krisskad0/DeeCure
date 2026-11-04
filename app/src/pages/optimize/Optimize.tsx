// SPDX-License-Identifier: Apache-2.0
// Optimize (docs/03-ui-design/05-pages/23-optimize.md): every improvement a measurement on this
// computer shows is needed, each with its numbers. Fixes open the fix preview; advice says what to
// do by hand. Everything checked without a result is listed too, with what was measured.
//
// The page walks the user through the items one by one: a step bar shows how many they have
// looked at, and "Next" moves to the next card. What was looked at is kept until the app closes.

import { type CSSProperties, useEffect, useState, useSyncExternalStore } from "react";
import type { OptimizeCheck } from "../../api";
import { AlertBanner } from "../../components/AlertBanner";
import { Button } from "../../components/Button";
import { type Column, DataTable } from "../../components/DataTable";
import { Icon } from "../../components/Icon";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { when } from "../../metrics";
import { useFix } from "../../shell/FixDrawer";
import type { IconName } from "../../spec/icons";
import { NoData } from "../NoData";
import { plural } from "../overview/shared";
import { SCAN_USUAL_MS, scanOptimize, useOptimizeScan } from "./scan";

/** The areas checks belong to, by the first part of their ID, in the order shown. */
export const AREAS: { id: string; label: string; icon: IconName; what: string }[] = [
  { id: "boot", label: "Start-up", icon: "play", what: "What delays boot" },
  { id: "storage", label: "Storage", icon: "db", what: "TRIM, disk scheduler, caches, and logs" },
  { id: "packages", label: "Software", icon: "pkg", what: "Old kernels, unused packages, and Flatpak runtimes" },
  { id: "system", label: "System", icon: "sys", what: "Updates, restart, and file-watch limit" },
  { id: "power", label: "Power", icon: "bat", what: "Power mode, charge limit, and device power saving" },
  { id: "hardware", label: "Hardware", icon: "chip", what: "Heat, link speed, battery wear, and memory errors" },
  { id: "memory", label: "Memory", icon: "gauge", what: "Swapping under memory pressure" },
  { id: "graphics", label: "Graphics", icon: "mon", what: "Video driver and the second graphics card" },
  { id: "drivers", label: "Drivers", icon: "layers", what: "Recommended drivers, DKMS modules, and firmware files" },
  { id: "firmware", label: "Firmware", icon: "usb", what: "Firmware updates for devices" },
  { id: "security", label: "Security", icon: "shield", what: "Firewall" },
];
const OTHER = { id: "other", label: "Other", icon: "sliders" as IconName, what: "" };

/** The area of a check, such as Storage for `storage.trim`; Other for an ID this app does not know. */
export function areaOf(c: OptimizeCheck) {
  const prefix = c.id.split(/[.:]/)[0];
  return AREAS.find((a) => a.id === prefix) ?? OTHER;
}

const CHECKED: Column<OptimizeCheck>[] = [
  { key: "area", label: "Area", text: (c) => areaOf(c).label },
  { key: "title", label: "Check", text: (c) => c.title },
  { key: "measured", label: "What DeeCure measured", text: (c) => c.measured },
];

/** Offered items, fixes first, in the agent's order within each kind. */
export function offered(checks: OptimizeCheck[]): OptimizeCheck[] {
  return [...checks.filter((c) => c.offered && c.kind === "fix"), ...checks.filter((c) => c.offered && c.kind === "advice")];
}

// Items looked at in this run of the app, by check ID.
let seen = new Set<string>();
const seenListeners = new Set<() => void>();
function markSeen(id: string) {
  if (seen.has(id)) return;
  seen = new Set(seen).add(id);
  for (const listener of seenListeners) listener();
}
/** For tests: forget what was looked at. */
export function resetSeen() {
  seen = new Set();
}
function useSeen(): Set<string> {
  return useSyncExternalStore(
    (l) => (seenListeners.add(l), () => seenListeners.delete(l)),
    () => seen,
  );
}

/** Now, every second while `active`. */
function useSecond(active: boolean): number {
  const [now, setNow] = useState(Date.now);
  useEffect(() => {
    if (!active) return;
    setNow(Date.now());
    const timer = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(timer);
  }, [active]);
  return now;
}

/** "4 s so far, usually about 10 s". */
function elapsed(startedAt: number | undefined, now: number): string {
  const s = startedAt === undefined ? 0 : Math.max(0, Math.round((now - startedAt) / 1000));
  return `${s} s so far, usually about ${SCAN_USUAL_MS / 1000} s`;
}

export function OptimizeAll() {
  const { checks, at, error, startedAt } = useOptimizeScan();
  const scanning = startedAt !== undefined;
  const now = useSecond(scanning);
  const [area, setArea] = useState("all");

  if (!checks) {
    return (
      <Panel title="Improvements" span={12}>
        {error && !scanning ? <NoData error={error} waiting="" /> : <ScanStage startedAt={startedAt} now={now} />}
      </Panel>
    );
  }

  const todo = offered(checks);
  const fine = checks.filter((c) => !c.offered);
  const areas = AREAS.concat(OTHER).filter((a) => todo.some((c) => areaOf(c).id === a.id));
  // A filter whose items were all fixed falls back to every item.
  const shownArea = areas.some((a) => a.id === area) ? area : "all";
  const shown = shownArea === "all" ? todo : todo.filter((c) => areaOf(c).id === shownArea);

  return (
    <>
      <Panel
        title="Improvements"
        info="Offered only when a measurement on this computer shows it is needed. Each fix shows a preview first, and can be undone where undo is possible."
        actions={
          <>
            {at !== undefined && <span className="panel-sub opt-last">Last checked {when(at, now)}</span>}
            <Button small icon="ref" onClick={() => void scanOptimize()} disabled={scanning}>
              {scanning ? "Checking…" : "Check again"}
            </Button>
          </>
        }
        span={12}
      >
        {scanning && (
          <div className="opt-rescan" role="status">
            <span className="opt-rescan-bar" aria-hidden="true" />
            <span>Checking again: {elapsed(startedAt, now)}. The list below is from the last check.</span>
          </div>
        )}
        {error && !scanning && <AlertBanner tone="warn" title="Could not check again" text={`${error.message} The list below is from the last check.`} />}
        <Summary checks={checks} todo={todo} />
      </Panel>
      {todo.length > 0 && (
        <Panel title={`Your path (${todo.length})`} info="Go through the items one by one. Each one says what DeeCure measured, and why it is offered." span={12}>
          <Path todo={todo} />
          {areas.length > 1 && (
            <div className="opt-filter" role="group" aria-label="Show items from">
              {[{ id: "all", label: "All", icon: "grid" as IconName }, ...areas].map((a) => {
                const n = a.id === "all" ? todo.length : todo.filter((c) => areaOf(c).id === a.id).length;
                return (
                  <button key={a.id} type="button" className={`opt-chip${shownArea === a.id ? " on" : ""}`} aria-pressed={shownArea === a.id} onClick={() => setArea(a.id)}>
                    <Icon name={a.icon} size={14} />
                    {a.label}
                    <span className="opt-chip-n">{n}</span>
                  </button>
                );
              })}
            </div>
          )}
          <ol className="opt-cards">
            {shown.map((c, i) => (
              <Card key={c.id} c={c} step={todo.indexOf(c) + 1} total={todo.length} next={shown[i + 1]} index={i} />
            ))}
          </ol>
        </Panel>
      )}
      <Panel title={`Checked, nothing needed (${fine.length})`} span={12} flush>
        <details>
          <summary className="optimize-summary">Show what was checked and measured</summary>
          <DataTable rows={fine} columns={CHECKED} rowKey={(c) => c.id} empty="No other checks" />
        </details>
      </Panel>
    </>
  );
}

/** The first scan: what DeeCure is checking, while it checks. */
function ScanStage({ startedAt, now }: { startedAt: number | undefined; now: number }) {
  return (
    <div className="opt-stage" role="status" aria-live="polite">
      <Radar />
      <div className="opt-stage-text">
        <h3>Checking this computer</h3>
        <p className="muted">DeeCure runs real system tools and reads this computer's history. Nothing is changed while it checks.</p>
        <div className="opt-progress" aria-hidden="true">
          <span />
        </div>
        <p className="opt-elapsed">{elapsed(startedAt, now)}</p>
        <ul className="opt-areas" aria-label="Areas being checked">
          {AREAS.map((a, i) => (
            <li key={a.id} style={{ "--i": i } as CSSProperties}>
              <span className="opt-area-icon">
                <Icon name={a.icon} size={15} />
              </span>
              <span>
                <strong>{a.label}</strong>
                <span className="muted">{a.what}</span>
              </span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

/** A sweeping radar: rings, a turning beam, and blips. Still when motion is reduced. */
export function Radar({ small = false }: { small?: boolean }) {
  return (
    <div className={`opt-radar${small ? " small" : ""}`} aria-hidden="true">
      <span className="opt-radar-ring r1" />
      <span className="opt-radar-ring r2" />
      <span className="opt-radar-ring r3" />
      <span className="opt-radar-beam" />
      <span className="opt-radar-blip b1" />
      <span className="opt-radar-blip b2" />
      <span className="opt-radar-blip b3" />
      <span className="opt-radar-core">
        <Icon name="bolt" size={small ? 14 : 22} strokeWidth={2.2} />
      </span>
    </div>
  );
}

/** The result in one look: a ring of checks that passed and checks that need something. */
function Summary({ checks, todo }: { checks: OptimizeCheck[]; todo: OptimizeCheck[] }) {
  if (checks.length === 0) {
    return <AlertBanner tone="info" title="No checks ran" text="The agent answered without any check. Restart the agent, then check again." />;
  }
  const fixes = todo.filter((c) => c.kind === "fix");
  const admin = fixes.filter((c) => c.access === "admin").length;
  const passed = checks.length - todo.length;
  const r = 34;
  const length = 2 * Math.PI * r;
  const share = todo.length / checks.length;
  return (
    <div className="opt-summary">
      <svg className="opt-ring" viewBox="0 0 84 84" width="96" height="96" role="img" aria-label={`${passed} of ${checks.length} checks passed`}>
        <circle cx="42" cy="42" r={r} className="opt-ring-ok" />
        {todo.length > 0 && <circle cx="42" cy="42" r={r} className="opt-ring-todo" strokeDasharray={`${share * length} ${length}`} transform="rotate(-90 42 42)" />}
        <text x="42" y="40" textAnchor="middle" className="opt-ring-n">
          {todo.length || "✓"}
        </text>
        <text x="42" y="56" textAnchor="middle" className="opt-ring-l">
          {todo.length ? "to improve" : "all good"}
        </text>
      </svg>
      <div className="opt-summary-text">
        {todo.length === 0 ? (
          <>
            <h3>Nothing to improve now</h3>
            <p className="muted">Every check passed. DeeCure checks again every 2 minutes while this page is open.</p>
          </>
        ) : (
          <>
            <h3>{plural(todo.length, "improvement")} for this computer</h3>
            <p className="muted">
              {plural(passed, "check")} of {checks.length} passed. Each item below was measured on this computer.
            </p>
          </>
        )}
        <ul className="opt-stats">
          {fixes.length > 0 && <Stat n={fixes.length} label={fixes.length === 1 ? "fix with a preview" : "fixes with a preview"} tone="accent" />}
          {todo.length > fixes.length && <Stat n={todo.length - fixes.length} label="advice to do by hand" tone="warn" />}
          {admin > 0 && <Stat n={admin} label={admin === 1 ? "needs your password" : "need your password"} tone="info" />}
          <Stat n={passed} label="passed" tone="ok" />
        </ul>
      </div>
    </div>
  );
}

function Stat({ n, label, tone }: { n: number; label: string; tone: "accent" | "warn" | "info" | "ok" }) {
  return (
    <li className={`opt-stat opt-stat-${tone}`}>
      <span className="opt-stat-n">{n}</span>
      <span>{label}</span>
    </li>
  );
}

/** One segment per item; filled once the user looked at it. Each segment jumps to its card. */
function Path({ todo }: { todo: OptimizeCheck[] }) {
  const looked = useSeen();
  const done = todo.filter((c) => looked.has(c.id)).length;
  return (
    <div className="opt-path">
      <div className="opt-path-head">
        <span>
          <strong>
            {done} of {todo.length}
          </strong>{" "}
          looked at
        </span>
        {done === todo.length && (
          <span className="opt-path-done">
            <Icon name="check" size={14} strokeWidth={2.6} />
            You went through every item
          </span>
        )}
      </div>
      <ol className="opt-path-steps">
        {todo.map((c, i) => (
          <li key={c.id}>
            <button type="button" className={`opt-seg${looked.has(c.id) ? " done" : ""}`} title={`${i + 1}. ${c.title}`} aria-label={`Go to item ${i + 1}: ${c.title}`} onClick={() => goTo(c.id)} />
          </li>
        ))}
      </ol>
    </div>
  );
}

const cardId = (id: string) => `opt-${id.replace(/[^a-zA-Z0-9_-]/g, "-")}`;

/** Scrolls to a card, focuses it, and flashes it once. */
function goTo(id: string) {
  const el = document.getElementById(cardId(id));
  if (!el) return;
  el.scrollIntoView?.({ behavior: "smooth", block: "center" });
  el.focus({ preventScroll: true });
  el.classList.remove("flash");
  void el.offsetWidth;
  el.classList.add("flash");
}

function Card({ c, step, total, next, index }: { c: OptimizeCheck; step: number; total: number; next?: OptimizeCheck; index: number }) {
  const openFix = useFix();
  const looked = useSeen().has(c.id);
  const area = areaOf(c);
  const advice = c.kind === "advice";
  const onNext = () => {
    markSeen(c.id);
    if (next) goTo(next.id);
  };
  return (
    <li id={cardId(c.id)} tabIndex={-1} className={`opt-card${advice ? " advice" : ""}${looked ? " seen" : ""}`} style={{ "--i": Math.min(index, 8) } as CSSProperties}>
      <div className="opt-card-rail" aria-hidden="true">
        <span className="opt-step">{looked ? <Icon name="check" size={14} strokeWidth={2.6} /> : step}</span>
      </div>
      <div className="opt-card-body">
        <div className="opt-card-top">
          <span className="opt-card-area">
            <Icon name={area.icon} size={13} />
            {area.label}
          </span>
          <span className="opt-card-count muted">
            Item {step} of {total}
          </span>
        </div>
        <strong className="opt-card-title">{c.title}</strong>
        <div className="opt-measured">
          <span className="opt-label">Measured</span>
          <span>{c.measured}</span>
        </div>
        {c.advice && (
          <div className="opt-advice">
            <span className="opt-label">What to do</span>
            <span>{c.advice}</span>
          </div>
        )}
        <div className="opt-card-foot">
          <span className="opt-badges">
            {advice ? <StatusBadge variant="warn" label="Advice" /> : <StatusBadge variant="info" label="Fix with preview" />}
            {!advice && (c.access === "admin" ? <StatusBadge variant="info" label="Needs your password" /> : <StatusBadge variant="off" label="No password" />)}
            {looked && <StatusBadge variant="ok" label="Looked at" />}
          </span>
          <span className="btn-row">
            {!advice && c.action && (
              <Button
                small
                variant="primary"
                onClick={() => {
                  markSeen(c.id);
                  openFix(c.action as string, c.subject);
                }}
              >
                Preview
              </Button>
            )}
            {(next || !looked) && (
              <Button small onClick={onNext}>
                {next ? (advice ? "Got it, next" : "Next") : advice ? "Got it" : "Done"}
              </Button>
            )}
          </span>
        </div>
      </div>
    </li>
  );
}
