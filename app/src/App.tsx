// SPDX-License-Identifier: Apache-2.0
// The app shell: top bar, sidebar, page header with tabs, and the 12-column content grid.
// Spec: docs/03-ui-design/02-shell/

import { REFRESH_EVENT, type TempUnit } from "./units";
import { useCallback, useEffect, useState } from "react";
import { type AppError, type Machine, type StatusReport, getMachine, getStatus, toAppError } from "./api";
import { NotBuilt } from "./pages/NotBuilt";
import { OverviewScore } from "./pages/overview/HealthScore";
import { PerformanceCpu } from "./pages/performance/Cpu";
import { PerformanceDisk } from "./pages/performance/Disk";
import { PerformanceGpu } from "./pages/performance/Gpu";
import { PerformanceMemory } from "./pages/performance/Memory";
import { PerformanceNetwork } from "./pages/performance/Network";
import { PerformancePressure } from "./pages/performance/Pressure";
import { AccountsGroups, AccountsLogins, AccountsSessions, AccountsSsh, AccountsUsers } from "./pages/accounts/Accounts";
import { NetworkConnections, NetworkDns, NetworkFirewall, NetworkInterfaces, NetworkPorts, NetworkTraffic } from "./pages/network/Network";
import { PrivacyAccess, PrivacyPermissions, PrivacySettings } from "./pages/privacy/Privacy";
import { SecurityHardening, SecurityIntegrity, SecurityMalware, SecurityOverview, SecurityVulns } from "./pages/security/Security";
import { BackupsJobs, BackupsRestore, BackupsSnapshots } from "./pages/backups/Backups";
import { ContainersImages, ContainersList, ContainersVms, ContainersVolumes } from "./pages/containers/Containers";
import { PackagesCleanup, PackagesInstalled, PackagesPermissions, PackagesSources } from "./pages/packages/Packages";
import { AssistantChat } from "./pages/assistant/Assistant";
import { DiagnosticsHardware, DiagnosticsNetwork, DiagnosticsPerformance } from "./pages/diagnostics/Diagnostics";
import { OptimizeAll } from "./pages/optimize/Optimize";
import { SettingsAbout, SettingsAccess, SettingsAssistant, SettingsData, SettingsGeneral, SettingsNotifications } from "./pages/settings/Settings";
import { ActivityConfig, ActivityDeeCure, ActivityDevices, ActivityPackages, ActivityTimelinePage } from "./pages/activity/Activity";
import { DisplayAudio, DisplayDisplays } from "./pages/display/Display";
import { LogsBoots, LogsCrashes, LogsErrors, LogsJournal, LogsKernel } from "./pages/logs/Logs";
import { JobsAnacron, JobsAt, JobsCalendar, JobsCron, JobsTimers } from "./pages/jobs/Jobs";
import { ServicesStartup, ServicesUser } from "./pages/services/OtherServices";
import { StorageCleanup, StorageFilesystems, StorageOverview, StorageUsage } from "./pages/storage/Storage";
import { HardwareFirmware } from "./pages/hardware/Firmware";
import { HardwareGraphics } from "./pages/hardware/Graphics";
import { HardwareMemory } from "./pages/hardware/Memory";
import { HardwarePeripherals } from "./pages/hardware/Peripherals";
import { HardwareProcessor } from "./pages/hardware/Processor";
import { HardwareSensors } from "./pages/hardware/Sensors";
import { HardwareStorage } from "./pages/hardware/Storage";
import { HardwareSummary } from "./pages/hardware/Summary";
import { PowerBattery, PowerProfile, PowerSleep } from "./pages/power/Power";
import { ProcessesAll, ProcessesTop } from "./pages/processes/Processes";
import { SystemBoot } from "./pages/system/Boot";
import { SystemKernel } from "./pages/system/Kernel";
import { SystemOverview } from "./pages/system/Overview";
import { SystemTime } from "./pages/system/Time";
import { SystemUpdates } from "./pages/system/Updates";
import { OverviewIssues } from "./pages/overview/Issues";
import { OverviewSummary } from "./pages/overview/Summary";
import { SystemServices } from "./pages/services/SystemServices";
import { usePref } from "./prefs";
import { useRoute } from "./route";
import { CommandPalette } from "./shell/CommandPalette";
import { ReportContext } from "./report";
import { FixContext, FixDrawer } from "./shell/FixDrawer";
import { Onboarding } from "./shell/Onboarding";
import { PageHeader } from "./shell/PageHeader";
import { MachinesPopover, NotificationsPopover } from "./shell/Popovers";
import { Sidebar } from "./shell/Sidebar";
import { type Range, TopBar } from "./shell/TopBar";
import { findingsFor, pageStatus } from "./status";
import { type Mode, UiContext, atLeast } from "./ui";
import { stack } from "./shell/Stack";
import { inBeginner } from "./visible";
import { AlertBanner } from "./components/AlertBanner";
import { Button } from "./components/Button";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { Gallery } from "./dev/Gallery";

const REFRESH_MS = 10_000;

type Tab = () => React.JSX.Element;


/** Pages whose tabs need no shared props, by page and tab ID. */
const TABS: Record<string, Record<string, Tab>> = {
  performance: {
    cpu: PerformanceCpu,
    memory: PerformanceMemory,
    disk: PerformanceDisk,
    network: PerformanceNetwork,
    gpu: PerformanceGpu,
    pressure: PerformancePressure,
  },
  processes: { all: ProcessesAll, top: ProcessesTop },
  hardware: {
    summary: HardwareSummary,
    processor: HardwareProcessor,
    memory: HardwareMemory,
    storage: HardwareStorage,
    graphics: HardwareGraphics,
    sensors: HardwareSensors,
    peripherals: HardwarePeripherals,
    firmware: HardwareFirmware,
  },
  power: { battery: PowerBattery, profile: PowerProfile, sleep: PowerSleep },
  display: { displays: DisplayDisplays, audio: DisplayAudio },
  logs: {
    journal: stack(["journal", LogsJournal], ["kernel", LogsKernel]),
    errors: stack(["errors", LogsErrors], ["crashes", LogsCrashes]),
    boots: LogsBoots,
  },
  activity: {
    timeline: stack(["timeline", ActivityTimelinePage], ["packages", ActivityPackages, "advanced"], ["devices", ActivityDevices, "advanced"], ["deecure", ActivityDeeCure, "advanced"]),
    config: ActivityConfig,
  },
  network: { interfaces: NetworkInterfaces, connections: NetworkConnections, ports: NetworkPorts, firewall: NetworkFirewall, dns: NetworkDns, traffic: NetworkTraffic },
  accounts: {
    users: stack(["users", AccountsUsers], ["groups", AccountsGroups]),
    sessions: stack(["sessions", AccountsSessions], ["logins", AccountsLogins]),
    ssh: AccountsSsh,
  },
  security: { overview: SecurityOverview, vulns: SecurityVulns, hardening: SecurityHardening, integrity: SecurityIntegrity, malware: SecurityMalware },
  privacy: { access: PrivacyAccess, perms: PrivacyPermissions, settings: PrivacySettings },
  packages: { installed: PackagesInstalled, sources: PackagesSources, cleanup: PackagesCleanup, permissions: PackagesPermissions },
  containers: { containers: ContainersList, images: stack(["images", ContainersImages], ["volumes", ContainersVolumes]), vms: ContainersVms },
  backups: { jobs: stack(["jobs", BackupsJobs], ["snapshots", BackupsSnapshots]), restore: BackupsRestore },
  optimize: { all: OptimizeAll },
  diagnostics: { performance: DiagnosticsPerformance, network: DiagnosticsNetwork, hardware: DiagnosticsHardware },
  assistant: { chat: AssistantChat },
  settings: { general: SettingsGeneral, access: SettingsAccess, notifications: SettingsNotifications, assistant: SettingsAssistant, data: SettingsData, about: SettingsAbout },
  storage: { overview: StorageOverview, filesystems: StorageFilesystems, usage: StorageUsage, cleanup: StorageCleanup },
  services: { user: ServicesUser, startup: ServicesStartup },
  jobs: { calendar: JobsCalendar, all: stack(["cron", JobsCron], ["timers", JobsTimers], ["anacron", JobsAnacron], ["at", JobsAt]) },
  system: { overview: SystemOverview, boot: SystemBoot, updates: SystemUpdates, kernel: SystemKernel, time: SystemTime },
};

export default function App() {
  const route = useRoute();
  const [report, setReport] = useState<StatusReport | null>(null);
  const [error, setError] = useState<AppError | null>(null);
  const [machine, setMachine] = useState<Machine | null>(null);
  const [now, setNow] = useState(Date.now());
  const [rail, setRail] = usePref<boolean>("rail", false);
  const [mode, setMode] = usePref<Mode>("mode", "beginner");
  const [range, setRange] = usePref<Range>("range", "24h");
  const [onboarded, setOnboarded] = usePref<boolean>("onboarded", false);
  const [drawer, setDrawer] = useState(false);
  const [palette, setPalette] = useState(false);
  const [popover, setPopover] = useState<{ kind: "bell" | "machine"; anchor: HTMLElement } | null>(null);
  const [fix, setFix] = useState<{ action: string; subject?: string } | null>(null);
  const openFix = useCallback((action: string, subject?: string) => setFix({ action, subject }), []);

  // Re-render everything when the temperature unit changes.
  usePref<TempUnit>("temp", "celsius");
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

  // An old link to a merged sub-page scrolls to its part once the page is drawn.
  useEffect(() => {
    if (!route.section) return;
    const frame = requestAnimationFrame(() => document.getElementById(`sec-${route.section}`)?.scrollIntoView({ block: "start" }));
    return () => cancelAnimationFrame(frame);
  }, [route.page.id, route.tab.id, route.section]);

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
  const failedServices = all.filter((f) => f.rule === "services.unit_failed").length;
  const tabCounts: Record<string, { count: number; critical: boolean }> =
    route.page.id === "overview"
      ? { issues: { count: all.length, critical: all.some((f) => f.severity === "critical") } }
      : route.page.id === "services"
        ? { system: { count: failedServices, critical: true } }
        : {};
  const shared = { report, error, now };

  let content;
  if (route.gallery) {
    content = <Gallery />;
  } else if (route.page.id === "overview" && route.tab.id === "summary") {
    content = <OverviewSummary {...shared} />;
  } else if (route.page.id === "overview" && route.tab.id === "issues") {
    content = <OverviewIssues {...shared} />;
  } else if (route.page.id === "overview" && route.tab.id === "score") {
    content = <OverviewScore {...shared} />;
  } else if (TABS[route.page.id]?.[route.tab.id]) {
    const Tab = TABS[route.page.id][route.tab.id];
    content = <Tab />;
  } else if (route.page.id === "services" && route.tab.id === "system") {
    content = <SystemServices />;
  } else {
    content = <NotBuilt route={route} findings={pageFindings} {...shared} />;
  }

  // A link can open a page that the Beginner menus leave out: say so, with a way out.
  const levelNote = !atLeast(mode, "advanced") && !route.gallery && !inBeginner(route.page.id, route.tab.id) && (
    <AlertBanner
      tone="info"
      title="This page is part of the Advanced view"
      text="Beginner shows the pages most people need. You can still use this one."
      actions={<Button small onClick={() => setMode("advanced")}>Switch to Advanced</Button>}
    />
  );

  if (!onboarded && !route.gallery) {
    return <Onboarding onDone={() => setOnboarded(true)} />;
  }

  return (
    <ReportContext.Provider value={report}>
    <FixContext.Provider value={openFix}>
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
      {fix && <FixDrawer action={fix.action} subject={fix.subject} onClose={() => setFix(null)} onDone={() => void refresh()} />}
      {popover?.kind === "bell" && (
        <NotificationsPopover anchor={popover.anchor} findings={all} now={now} onClose={() => setPopover(null)} />
      )}
      {popover?.kind === "machine" && (
        <MachinesPopover
          anchor={popover.anchor}
          machine={machine}
          onClose={() => setPopover(null)}
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
            onRefresh={() => {
              void refresh();
              window.dispatchEvent(new Event(REFRESH_EVENT));
            }}
          />
        )}
        <div className="grid">
          {levelNote}
          <ErrorBoundary what={route.page.label} resetKey={`${route.page.id}/${route.tab.id}/${mode}`}>
            {content}
          </ErrorBoundary>
        </div>
      </main>
    </div>
    </UiContext.Provider>
    </FixContext.Provider>
    </ReportContext.Provider>
  );
}
