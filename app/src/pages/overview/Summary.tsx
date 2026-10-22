// SPDX-License-Identifier: Apache-2.0
// Overview › Summary (docs/03-ui-design/05-pages/01-overview.md), with real data.
// Not shown yet: the disk breakdown by folder (needs a disk-use scan), recent activity, and
// the error heatmap. They join when the Storage, Activity, and Logs pages exist.

import { type Finding, type Metric, type Process, type SystemInfo, getInventory, getLatest, getMetrics, getSnapshot } from "../../api";
import { LinkButton } from "../../components/Button";
import { type Column, DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { type Kpi, KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { type StatusRow, StatusList } from "../../components/StatusList";
import { BarList } from "../../components/charts/BarList";
import { Donut } from "../../components/charts/Donut";
import { Gauge } from "../../components/charts/Gauge";
import { LineChart } from "../../components/charts/LineChart";
import { RANGE_MS, align, bytes, duration, pick, rate, scoreTone, when } from "../../metrics";
import { href } from "../../route";
import { pageStatus } from "../../status";
import { type Range, useUi } from "../../ui";
import { useLive } from "../../useLive";
import { IssuesPanel } from "../IssuesPanel";
import { NoData } from "../NoData";
import { type PageProps, areaLabel, plural, worst } from "./shared";

interface FsEntry {
  mount_point: string;
  device: string;
  fs_type: string;
  size: number;
}

const loadLatest = () => getLatest();
const loadSystem = () => getInventory("system");
const loadProcesses = () => getInventory("processes");
const loadDisks = () => getSnapshot<FsEntry[]>("filesystems");

async function loadHistory(range: Range) {
  const until = Date.now();
  const since = until - RANGE_MS[range];
  const [cpu, memory, rx, tx] = await Promise.all(
    ["cpu.usage", "memory.used_percent", "net.rx", "net.tx"].map((name) => getMetrics(name, since, until)),
  );
  return { cpu, memory, rx, tx };
}

/** "8.6 GB" → value "8.6", unit "GB", for KPI tiles. */
function split(text: string): { value: string; unit: string } {
  const [value, ...unit] = text.split(" ");
  return { value, unit: unit.join(" ") };
}

export function OverviewSummary({ report, error, now }: PageProps) {
  const { range } = useUi();
  const latest = useLive(loadLatest, 15_000).data;
  const system = useLive(loadSystem, 60_000).data;
  const processes = useLive(loadProcesses, 10_000).data;
  const disks = useLive(loadDisks, 60_000).data;
  const history = useLive(() => loadHistory(range), 60_000, range).data;
  const findings = report?.findings ?? [];

  if (error) {
    return (
      <div className="span-12 panel">
        <NoData error={error} waiting="" />
      </div>
    );
  }

  return (
    <>
      <KpiRow tiles={kpis(report?.score, findings.length, latest, system, history, findings)} />
      <Panel
        title="CPU and memory utilization"
        info="How busy the processor and memory were. Change the time range at the top to zoom out."
        source="/proc/stat · /proc/meminfo"
        span={8}
      >
        <UsageChart history={history} range={range} />
      </Panel>
      <Panel title="Subsystems" info="Status of every area DeeCure checks. Select a row to open it." span={4} flush>
        <StatusList rows={subsystems(report, latest, system)} />
      </Panel>
      <IssuesPanel
        title="Active issues"
        subtitle={`${findings.length} open`}
        findings={findings.slice(0, 6)}
        report={report}
        error={error}
        now={now}
        span={8}
        actions={<LinkButton to={href("overview", "issues")}>View all</LinkButton>}
      />
      <ScorePanel report={report} />
      <DiskPanel latest={latest} disks={disks?.data} />
      <TopProcesses processes={processes} />
      <SystemPanel system={system} now={now} />
    </>
  );
}

type History = Awaited<ReturnType<typeof loadHistory>> | undefined;

function kpis(score: number | undefined, open: number, latest: Metric[] | undefined, system: SystemInfo | undefined, history: History, findings: Finding[]): Kpi[] {
  const tiles: Kpi[] = [
    {
      label: "Health score",
      value: score === undefined ? "—" : String(score),
      unit: score === undefined ? undefined : "/100",
      status: score === undefined ? "off" : scoreTone(score),
      subtext: score === undefined ? "Waiting for the first check" : plural(open, "open issue"),
      visual: score === undefined ? undefined : { kind: "bar", percent: score },
      href: href("overview", "score"),
    },
  ];

  const cpu = pick(latest, "cpu.usage");
  const load = pick(latest, "cpu.load_1m");
  tiles.push({
    label: "CPU",
    value: cpu === undefined ? "—" : String(Math.round(cpu)),
    unit: cpu === undefined ? undefined : "%",
    subtext: [load !== undefined && `Load ${load.toFixed(2)}`, system && `${system.cpu_threads} threads`].filter(Boolean).join(" · "),
    visual: history?.cpu.length ? { kind: "spark", data: history.cpu.map((p) => p.value), slot: 1 } : undefined,
    href: href("performance", "cpu"),
  });

  const used = pick(latest, "memory.used");
  const total = pick(latest, "memory.total");
  const memPct = pick(latest, "memory.used_percent");
  const mem = used === undefined ? { value: "—", unit: "" } : split(bytes(used));
  tiles.push({
    label: "Memory",
    value: mem.value,
    unit: mem.unit || undefined,
    subtext: memPct !== undefined && total !== undefined ? `${Math.round(memPct)}% of ${bytes(total)}` : "",
    visual: history?.memory.length ? { kind: "spark", data: history.memory.map((p) => p.value), slot: 2 } : undefined,
    href: href("performance", "memory"),
  });

  const diskPct = pick(latest, "disk.used_percent", "/");
  const free = pick(latest, "disk.free", "/");
  const rootFindings = findings.filter((f) => f.rule.startsWith("storage.") && f.subject === "/");
  tiles.push({
    label: "Disk ( / )",
    value: diskPct === undefined ? "—" : String(Math.round(diskPct)),
    unit: diskPct === undefined ? undefined : "%",
    status: worst(rootFindings),
    subtext: free !== undefined ? `${bytes(free)} free` : "",
    visual: diskPct === undefined ? undefined : { kind: "bar", percent: diskPct },
    href: href("storage", "overview"),
  });

  const rx = pick(latest, "net.rx");
  const tx = pick(latest, "net.tx");
  const net = rx === undefined || tx === undefined ? null : split(rate(rx + tx));
  const both = history ? align([history.rx, history.tx]) : null;
  tiles.push({
    label: "Network",
    value: net?.value ?? "—",
    unit: net?.unit,
    subtext: rx !== undefined && tx !== undefined ? `↓ ${rate(rx)} · ↑ ${rate(tx)}` : "Measuring…",
    visual: both && both.times.length ? { kind: "spark", data: both.times.map((_, i) => both.data[0][i] + both.data[1][i]), slot: 3 } : undefined,
    href: href("performance", "network"),
  });

  const battery = latest?.find((m) => m.name === "battery.charge");
  if (battery) {
    const health = pick(latest, "battery.health", battery.subject);
    const left = pick(latest, "battery.time_left", battery.subject);
    tiles.push({
      label: "Battery",
      value: String(Math.round(battery.value)),
      unit: "%",
      status: worst(findings.filter((f) => f.rule.startsWith("power."))),
      subtext: [health !== undefined && `Health ${Math.round(health)}%`, left !== undefined && `${duration(left)} left`].filter(Boolean).join(" · "),
      visual: { kind: "bar", percent: battery.value },
      href: href("power", "battery"),
    });
  }
  return tiles;
}

function UsageChart({ history, range }: { history: History; range: Range }) {
  if (!history) return <div className="chart-empty muted">Loading…</div>;
  const { times, data } = align([history.cpu, history.memory]);
  return (
    <LineChart
      series={[
        { name: "CPU", data: data[0] ?? [], slot: 1 },
        { name: "Memory", data: data[1] ?? [], slot: 2 },
      ]}
      x={{ kind: "time", times, range }}
      height={300}
      max={100}
      unit="%"
    />
  );
}

const NOT_YET = "Not checked yet";

function subsystems(report: PageProps["report"], latest: Metric[] | undefined, system: SystemInfo | undefined): StatusRow[] {
  const status = (id: string) => pageStatus(id, report, null) ?? "off";
  const temp = pick(latest, "temp.cpu");
  const battery = latest?.find((m) => m.name === "battery.health");
  const failed = pick(latest, "services.failed");
  const running = pick(latest, "services.running");
  const disk = pick(latest, "disk.used_percent", "/");
  const rx = pick(latest, "net.rx");
  const tx = pick(latest, "net.tx");
  const row = (id: string, icon: StatusRow["icon"], metric: string | undefined, tab: string): StatusRow => ({
    icon,
    label: areaLabel(id),
    metric: metric ?? NOT_YET,
    status: status(id),
    href: href(id, tab),
  });
  return [
    row("hardware", "chip", temp === undefined ? undefined : `CPU ${Math.round(temp)} °C`, "summary"),
    row("power", "bat", battery ? `Health ${Math.round(battery.value)}%` : latest ? "No battery" : undefined, "battery"),
    row("system", "sys", system ? (system.reboot_required ? "Reboot required" : system.uptime_seconds ? `Up ${duration(system.uptime_seconds)}` : undefined) : undefined, "overview"),
    row("services", "cog", failed ? `${failed} failed` : running !== undefined ? `${running} running` : undefined, "system"),
    row("jobs", "cal", undefined, "calendar"),
    row("packages", "pkg", undefined, "installed"),
    row("containers", "cube", undefined, "containers"),
    row("storage", "db", disk === undefined ? undefined : `${Math.round(disk)}% used`, "overview"),
    row("backups", "backup", undefined, "jobs"),
    row("network", "wifi", rx !== undefined && tx !== undefined ? rate(rx + tx) : undefined, "interfaces"),
    row("accounts", "users", undefined, "users"),
    row("security", "shield", undefined, "overview"),
    row("privacy", "eye", undefined, "access"),
  ];
}

function ScorePanel({ report }: { report: PageProps["report"] }) {
  const score = report?.score;
  return (
    <Panel
      title="Health score"
      info="Starts at 100. Each open issue removes points from its area."
      actions={<LinkButton to={href("overview", "score")}>Details</LinkButton>}
      span={4}
    >
      {score === undefined ? (
        <div className="empty muted">The score appears after DeeCure's first check.</div>
      ) : (
        <>
          <Gauge value={score} subtitle="out of 100" status={scoreTone(score)} />
          {report?.deductions?.length ? (
            <BarList rows={report.deductions.map((d) => ({ label: areaLabel(d.category), value: d.points, display: `−${d.points}`, tone: "warn" }))} />
          ) : (
            <p className="muted center">Nothing is taking points off.</p>
          )}
        </>
      )}
    </Panel>
  );
}

function DiskPanel({ latest, disks }: { latest: Metric[] | undefined; disks: FsEntry[] | undefined }) {
  const size = disks?.find((d) => d.mount_point === "/")?.size;
  const free = pick(latest, "disk.free", "/");
  const pct = pick(latest, "disk.used_percent", "/");
  return (
    <Panel
      title="Disk usage ( / )"
      subtitle={size === undefined ? undefined : bytes(size, 0)}
      source="statvfs /"
      actions={<LinkButton to={href("storage", "overview")}>Open</LinkButton>}
      span={4}
    >
      {size === undefined || free === undefined || pct === undefined ? (
        <div className="empty muted">DeeCure has not measured the disk yet.</div>
      ) : (
        <Donut
          segments={[
            { name: "Used", value: size - free, slot: 1, display: bytes(size - free) },
            { name: "Free", value: free, tone: "neutral", display: bytes(free) },
          ]}
          center={{ value: `${Math.round(pct)}%`, subtitle: "used" }}
        />
      )}
    </Panel>
  );
}

const PROCESS_COLUMNS: Column<Process>[] = [
  { key: "name", label: "Process", text: (p) => p.name },
  { key: "cpu", label: "CPU", text: (p) => `${p.cpu_percent.toFixed(1)}%`, numeric: true },
  { key: "memory", label: "Memory", text: (p) => bytes(p.memory_bytes), numeric: true },
];

function TopProcesses({ processes }: { processes: Process[] | undefined }) {
  const top = [...(processes ?? [])].sort((a, b) => b.memory_bytes - a.memory_bytes).slice(0, 6);
  return (
    <Panel
      title="Top processes"
      subtitle="by memory"
      source="/proc/[pid]/stat · status"
      actions={<LinkButton to={href("processes", "all")}>View all</LinkButton>}
      span={4}
      flush
    >
      {processes ? (
        <DataTable rows={top} columns={PROCESS_COLUMNS} rowKey={(p) => String(p.pid)} pageSize={6} />
      ) : (
        <div className="empty muted">Loading…</div>
      )}
    </Panel>
  );
}

function SystemPanel({ system, now }: { system: SystemInfo | undefined; now: number }) {
  return (
    <Panel title="System" source="/etc/os-release · /proc" span={4} flush>
      {system ? (
        <KeyValueList
          rows={[
            ["Hostname", system.hostname ?? "—"],
            ["OS", system.os ?? "—"],
            ["Kernel", system.kernel ? <code>{system.kernel}</code> : "—"],
            ["Uptime", system.uptime_seconds === undefined ? "—" : duration(system.uptime_seconds)],
            ["Last boot", system.uptime_seconds === undefined ? "—" : when(now - system.uptime_seconds * 1000, now)],
            ["Reboot required", system.reboot_required ? <StatusBadge variant="warn" label="Yes" /> : "No"],
          ]}
        />
      ) : (
        <div className="empty muted">Loading…</div>
      )}
    </Panel>
  );
}
