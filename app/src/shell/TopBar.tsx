// SPDX-License-Identifier: Apache-2.0
// Top bar (docs/03-ui-design/02-shell/top-bar.md).

import type { AppError, Machine, StatusReport } from "../api";
import { Icon } from "../components/Icon";
import { ago } from "../time";
import type { Mode, Range } from "../ui";
import { Segmented } from "./Segmented";

export type { Mode, Range };

const RANGES = [
  { value: "1h", label: "1h" },
  { value: "24h", label: "24h" },
  { value: "7d", label: "7d" },
  { value: "30d", label: "30d" },
] as const;

const MODES = [
  { value: "beginner", label: "Beginner" },
  { value: "expert", label: "Expert" },
] as const;

export function TopBar(props: {
  machine: Machine | null;
  report: StatusReport | null;
  error: AppError | null;
  now: number;
  range: Range;
  onRange: (range: Range) => void;
  mode: Mode;
  onMode: (mode: Mode) => void;
  alerts: number;
  onMenu: () => void;
  onSearch: () => void;
  onBell: (anchor: HTMLElement) => void;
  onMachine: (anchor: HTMLElement) => void;
}) {
  const { machine, report, error, now } = props;
  return (
    <header className="topbar">
      <button className="ibtn menu-btn" type="button" aria-label="Open navigation" onClick={props.onMenu}>
        <Icon name="menu" size={18} />
      </button>
      <div className="brand">
        <span className="logo" aria-hidden="true">
          <svg viewBox="0 0 24 24" width="18" height="18">
            <path d="M2.5 13h4l2.2-6 3.6 11 3-8 1.5 3H21.5" />
          </svg>
        </span>
        <span className="wordmark">DeeCure</span>
      </div>
      <button className="machine" type="button" aria-label="Machines" onClick={(e) => props.onMachine(e.currentTarget)}>
        <span className="dot" />
        <span className="machine-name">{machine?.hostname ?? "…"}</span>
        <span className="machine-os">{machine?.os}</span>
        <Icon name="chevdown" size={14} />
      </button>

      <button className="search" type="button" aria-label="Search (Ctrl K)" onClick={props.onSearch}>
        <Icon name="search" size={15} />
        <span className="search-text">Search pages, services, packages…</span>
        <kbd>Ctrl K</kbd>
      </button>

      <Segmented label="Time range" className="range" options={RANGES} value={props.range} onChange={props.onRange} />
      <span className={`live ${error ? "live-off" : ""}`}>
        <span className="dot" />
        {error ? "Agent not reachable" : report?.checked_at ? `Live · checked ${ago(report.checked_at, now)}` : "Connecting…"}
      </span>
      <Segmented label="Mode" options={MODES} value={props.mode} onChange={props.onMode} />
      <button
        className="ibtn bell"
        type="button"
        aria-label={`Notifications: ${props.alerts} open`}
        onClick={(e) => props.onBell(e.currentTarget)}
      >
        <Icon name="bell" size={18} />
        {props.alerts > 0 && <span className="bell-count">{props.alerts}</span>}
      </button>
      <span className="avatar" title={machine?.user}>
        {machine ? machine.user.charAt(0).toUpperCase() : ""}
      </span>
    </header>
  );
}
