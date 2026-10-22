// SPDX-License-Identifier: Apache-2.0
// The app shell: top bar, sidebar, page header with tabs, and the 12-column content grid.
// Spec: docs/03-ui-design/02-shell/

import { useCallback, useEffect, useState } from "react";
import { type AppError, type Machine, type StatusReport, getMachine, getStatus, toAppError } from "./api";
import { IssuesPanel } from "./pages/IssuesPanel";
import { NotBuilt } from "./pages/NotBuilt";
import { usePref } from "./prefs";
import { useRoute } from "./route";
import { CommandPalette } from "./shell/CommandPalette";
import { PageHeader } from "./shell/PageHeader";
import { MachinesPopover, NotificationsPopover } from "./shell/Popovers";
import { Sidebar } from "./shell/Sidebar";
import { useToast } from "./shell/Toast";
import { type Mode, type Range, TopBar } from "./shell/TopBar";
import { findingsFor, pageStatus } from "./status";
import { UiContext } from "./ui";
import { Gallery } from "./dev/Gallery";

const REFRESH_MS = 10_000;

export default function App() {
  const route = useRoute();
  const [report, setReport] = useState<StatusReport | null>(null);
  const [error, setError] = useState<AppError | null>(null);
  const [machine, setMachine] = useState<Machine | null>(null);
  const [now, setNow] = useState(Date.now());
  const [rail, setRail] = usePref<boolean>("rail", false);
  const [mode, setMode] = usePref<Mode>("mode", "beginner");
  const [range, setRange] = usePref<Range>("range", "24h");
  const [drawer, setDrawer] = useState(false);
  const [palette, setPalette] = useState(false);
  const [popover, setPopover] = useState<{ kind: "bell" | "machine"; anchor: HTMLElement } | null>(null);
  const toast = useToast();

  const refresh = useCallback(async () => {
    try {
      setReport(await getStatus());
      setError(null);
    } catch (e) {
      setError(toAppError(e));
    }
    setNow(Date.now());
  }, []);

  useEffect(() => {
    void refresh();
    const timer = setInterval(() => void refresh(), REFRESH_MS);
    return () => clearInterval(timer);
  }, [refresh]);

  useEffect(() => {
    getMachine().then(setMachine, () => setMachine(null));
  }, []);

  useEffect(() => {
    document.title = `${route.gallery ? "Component gallery" : route.page.label} · DeeCure`;
  }, [route.gallery, route.page.label]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setDrawer(false);
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setPopover(null);
        setPalette((open) => !open);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const all = report?.findings ?? [];
  const pageFindings = findingsFor(route.page.id, report);
  const alerts = all.filter((f) => f.severity !== "info").length;
  const tabCounts: Record<string, { count: number; critical: boolean }> =
    route.page.id === "overview" ? { issues: { count: all.length, critical: all.some((f) => f.severity === "critical") } } : {};
  const shared = { report, error, now };

  let content;
  if (route.gallery) {
    content = <Gallery />;
  } else if (route.page.id === "overview" && route.tab.id === "summary") {
    content = <IssuesPanel title="Active issues" subtitle="Most serious first" findings={all} {...shared} />;
  } else if (route.page.id === "overview" && route.tab.id === "issues") {
    content = <IssuesPanel title="All open issues" subtitle="Everything DeeCure found, most serious first" findings={all} {...shared} />;
  } else {
    content = <NotBuilt route={route} findings={pageFindings} {...shared} />;
  }

  return (
    <UiContext.Provider value={{ mode, range }}>
    <div className={`app ${rail ? "is-rail" : ""} mode-${mode}`}>
      <TopBar
        machine={machine}
        report={report}
        error={error}
        now={now}
        range={range}
        onRange={setRange}
        mode={mode}
        onMode={setMode}
        alerts={alerts}
        onMenu={() => setDrawer(true)}
        onSearch={() => setPalette(true)}
        onBell={(anchor) => setPopover({ kind: "bell", anchor })}
        onMachine={(anchor) => setPopover({ kind: "machine", anchor })}
      />
      {palette && <CommandPalette onClose={() => setPalette(false)} />}
      {popover?.kind === "bell" && (
        <NotificationsPopover anchor={popover.anchor} findings={all} now={now} onClose={() => setPopover(null)} />
      )}
      {popover?.kind === "machine" && (
        <MachinesPopover
          anchor={popover.anchor}
          machine={machine}
          onClose={() => setPopover(null)}
          onPlanned={() => {
            setPopover(null);
            toast("Multi-machine view is planned for a later phase.");
          }}
        />
      )}
      {drawer && <div className="scrim" onClick={() => setDrawer(false)} />}
      <Sidebar
        route={route}
        statusOf={(id) => pageStatus(id, report, error)}
        issueCount={all.length}
        rail={rail}
        onRail={setRail}
        drawerOpen={drawer}
        onNavigate={() => setDrawer(false)}
      />
      <main className="main">
        {route.gallery ? (
          <div className="page-header">
            <div className="crumbs">Development</div>
            <div className="title-row">
              <h1>Component gallery</h1>
            </div>
            <div className="tabs-spacer" />
          </div>
        ) : (
          <PageHeader
            route={route}
            status={pageStatus(route.page.id, report, error)}
            tabCounts={tabCounts}
            onRefresh={() => void refresh()}
          />
        )}
        <div className="grid">{content}</div>
      </main>
    </div>
    </UiContext.Provider>
  );
}
