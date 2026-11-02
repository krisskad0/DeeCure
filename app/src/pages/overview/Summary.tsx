// SPDX-License-Identifier: Apache-2.0
// Overview › Summary (docs/03-ui-design/05-pages/01-overview.md), with real data.

import { temp as tempText } from "../../units";
import { type Finding, type Gpu, type Metric, type Process, type SystemInfo, getInventory, getLatest, getMetrics, getSnapshot } from "../../api";
import { ActivityTimeline } from "../../components/ActivityTimeline";
import { LinkButton } from "../../components/Button";
import { type Column, DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { type Kpi, KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { type StatusRow, StatusList } from "../../components/StatusList";
import { Donut } from "../../components/charts/Donut";
import { Heatmap } from "../../components/charts/Heatmap";
import { LineChart } from "../../components/charts/LineChart";
import { RANGE_MS, align, bytes, duration, heatRows, pick, rate, when } from "../../metrics";
import { useChanges } from "../activity/Activity";
import { href } from "../../route";
import { pageStatus } from "../../status";
import { type Range, useUi } from "../../ui";
import { useLive } from "../../useLive";
import { IssuesPanel } from "../IssuesPanel";
import { NoData } from "../NoData";
import { ScoreCard } from "./ScoreCard";
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
const loadErrors = () => getInventory("errors");
const loadGpus = () => getInventory("gpu");

/** Graphics cards whose driver can report busy time (AMD, Intel). */
const BUSY_DRIVERS = ["amdgpu", "i915", "xe"];

async function loadHistory(range: Range, cards: string[]) {
  const until = Date.now();
  const since = until - RANGE_MS[range];
  const [cpu, memory, rx, tx] = await Promise.all(
    ["cpu.usage", "memory.used_percent", "net.rx", "net.tx"].map((name) => getMetrics(name, since, until)),
  );
  // One series per graphics card; a card without readings has an empty one.
  const gpu = Object.fromEntries(
    await Promise.all(cards.map(async (card) => [card, await getMetrics("gpu.busy", since, until, { subject: card }).catch(() => [])] as const)),
  );
  return { cpu, memory, rx, tx, gpu };
}

/** "Raptor Lake-P [Iris Xe Graphics]" → "Iris Xe Graphics". */
export function gpuName(g: Gpu): string {
  return g.model?.match(/\[(.+)\]/)?.[1] ?? g.model ?? g.vendor ?? g.card;
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
  const gpus = useLive(loadGpus, 300_000).data;
  const cards = (gpus ?? []).map((g) => g.card);
  const history = useLive(() => loadHistory(range, cards), 60_000, `${range}|${cards.join(",")}`).data;
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
      <KpiRow tiles={kpis(latest, system, history, findings, gpus)} />
      <Panel
        title={gpuSeries(history, gpus).length ? "CPU, GPU, and memory utilization" : "CPU and memory utilization"}
        info="How busy the processor, the graphics, and memory were. Change the time range at the top to zoom out."
        source="/proc/stat · /proc/meminfo · /sys/class/drm"
        span={8}
      >
        <UsageChart history={history} range={range} gpus={gpus} />
      </Panel>
      <ScoreCard report={report} />
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
      <Panel title="Subsystems" info="Status of every area DeeCure checks. Select a row to open it." span={4} flush>
        <StatusList rows={subsystems(report, latest, system)} />
      </Panel>
      <DiskPanel latest={latest} disks={disks?.data} />
      <TopProcesses processes={processes} />
      <SystemPanel system={system} now={now} />
      <RecentActivity />
      <ErrorHeatmap />
    </>
  );
}

type History = Awaited<ReturnType<typeof loadHistory>> | undefined;

/** The GPU tile: the busiest card, or why there is no reading. No graphics card, no tile. */
export function gpuTile(gpus: Gpu[] | undefined, latest: Metric[] | undefined, history: History): Kpi | null {
  if (!gpus?.length) return null;
  const readings = gpus.map((g) => ({ g, busy: pick(latest, "gpu.busy", g.card) }));
  const shown = readings.filter((r) => r.busy !== undefined).sort((a, b) => (b.busy ?? 0) - (a.busy ?? 0))[0] ?? readings[0];
  const { g, busy } = shown;
  const why = BUSY_DRIVERS.includes(g.driver ?? "") ? "Measuring…" : g.driver === "nvidia" ? "The NVIDIA driver does not report busy time" : "Its driver does not report busy time";
  const spark = history?.gpu[g.card];
  return {
    label: "GPU",
    value: busy === undefined ? "-" : String(Math.round(busy)),
    unit: busy === undefined ? undefined : "%",
    subtext: busy === undefined ? why : gpus.length > 1 ? `${gpuName(g)} · busiest of ${gpus.length}` : gpuName(g),
    visual: spark && spark.length ? { kind: "spark", data: spark.map((p) => p.value), slot: 4 } : undefined,
    href: href("performance", "gpu"),
  };
}

function kpis(latest: Metric[] | undefined, system: SystemInfo | undefined, history: History, findings: Finding[], gpus: Gpu[] | undefined): Kpi[] {
  const tiles: Kpi[] = [];

  const cpu = pick(latest, "cpu.usage");
  const load = pick(latest, "cpu.load_1m");
  tiles.push({
    label: "CPU",
    value: cpu === undefined ? "-" : String(Math.round(cpu)),
    unit: cpu === undefined ? undefined : "%",
    subtext: [load !== undefined && `Load ${load.toFixed(2)}`, system && `${system.cpu_threads} threads`].filter(Boolean).join(" · "),
    visual: history?.cpu.length ? { kind: "spark", data: history.cpu.map((p) => p.value), slot: 1 } : undefined,
    href: href("performance", "cpu"),
  });

  const gpu = gpuTile(gpus, latest, history);
  if (gpu) tiles.push(gpu);

  const used = pick(latest, "memory.used");
  const total = pick(latest, "memory.total");
  const memPct = pick(latest, "memory.used_percent");
  const mem = used === undefined ? { value: "-", unit: "" } : split(bytes(used));
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
    value: diskPct === undefined ? "-" : String(Math.round(diskPct)),
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
    value: net?.value ?? "-",
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

/** The graphics cards with readings in this range, at most two (slots 4 and 5). */
export function gpuSeries(history: History, gpus: Gpu[] | undefined): Gpu[] {
  return (gpus ?? []).filter((g) => history?.gpu[g.card]?.length).slice(0, 2);
}

function UsageChart({ history, range, gpus }: { history: History; range: Range; gpus: Gpu[] | undefined }) {
  if (!history) return <div className="chart-empty muted">Loading…</div>;
  const { times, data } = align([history.cpu, history.memory]);
  // GPU readings may start later (or have gaps): a missing point is a gap, never a guess,
  // and CPU and memory keep their whole history.
  const cards = gpuSeries(history, gpus);
  const gpu = cards.map((g, k) => {
    const at = new Map(history.gpu[g.card].map((p) => [p.at, p.value]));
    return { name: cards.length > 1 ? `GPU ${gpuName(g)}` : "GPU", data: times.map((t) => at.get(t) ?? Number.NaN), slot: (4 + k) as 4 | 5 };
  });
  return (
    <LineChart
      series={[{ name: "CPU", data: data[0] ?? [], slot: 1 }, ...gpu, { name: "Memory", data: data[1] ?? [], slot: 2 }]}
      x={{ kind: "time", times, range }}
      height={420}
      max={100}
      unit="%"
    />
  );
}

const NOT_YET = "Not checked yet";

function subsystems(report: PageProps["report"], latest: Metric[] | undefined, system: SystemInfo | undefined): StatusRow[] {
  const checked = new Set(report?.checked ?? []);
  const temp = pick(latest, "temp.cpu");
  const battery = latest?.find((m) => m.name === "battery.health");
  const failed = pick(latest, "services.failed");
  const running = pick(latest, "services.running");
  const disk = pick(latest, "disk.used_percent", "/");
  const rx = pick(latest, "net.rx");
  const tx = pick(latest, "net.tx");
  const online = pick(latest, "net.default_route");
  const pending = pick(latest, "updates.pending");
  const security = pick(latest, "updates.security");
  const backupAge = pick(latest, "backup.age_days");
  const backupOn = pick(latest, "backup.configured");
  const people = pick(latest, "accounts.people");
  const firewall = pick(latest, "firewall.enabled");
  const exposed = pick(latest, "containers.exposed");
  const popcon = pick(latest, "privacy.popcon");
  const failedJobs = report?.checked_at === undefined ? undefined : (report?.findings.filter((f) => f.rule === "jobs.job_failed").length ?? 0);
  const row = (id: string, icon: StatusRow["icon"], metric: string | undefined, tab: string): StatusRow => {
    const status = pageStatus(id, report, null) ?? "off";
    // An area with no checks says so; "Unknown" stays for "the agent has not answered".
    const notChecked = status === "off" && report?.checked_at !== undefined && !checked.has(id);
    return { icon, label: areaLabel(id), metric: metric ?? (notChecked ? "Open to see details" : NOT_YET), status, statusLabel: notChecked ? "Not checked" : undefined, href: href(id, tab) };
  };
  return [
    row("hardware", "chip", temp === undefined ? undefined : `CPU ${tempText(temp)}`, "summary"),
    row("power", "bat", battery ? `Health ${Math.round(battery.value)}%` : latest ? "No battery" : undefined, "battery"),
    row("system", "sys", system ? (system.reboot_required ? "Reboot required" : system.uptime_seconds ? `Up ${duration(system.uptime_seconds)}` : undefined) : undefined, "overview"),
    row("services", "cog", failed ? `${failed} failed` : running !== undefined ? `${running} running` : undefined, "system"),
    row("jobs", "cal", failedJobs === undefined ? undefined : failedJobs ? plural(failedJobs, "failed job") : "No failures", "calendar"),
    row("packages", "pkg", pending === undefined ? undefined : pending ? plural(pending, "update") : "Up to date", "installed"),
    row("containers", "cube", exposed === undefined ? undefined : exposed ? `${plural(exposed, "container")} exposed` : "Nothing exposed", "containers"),
    row("storage", "db", disk === undefined ? undefined : `${Math.round(disk)}% used`, "overview"),
    row("backups", "backup", backupOn === 0 ? "Not set up" : backupAge === undefined ? undefined : backupAge < 1 ? "Today" : `${plural(Math.round(backupAge), "day")} ago`, "jobs"),
    row("network", "wifi", online === 0 ? "Offline" : rx !== undefined && tx !== undefined ? rate(rx + tx) : online === 1 ? "Connected" : undefined, "interfaces"),
    row("accounts", "users", people === undefined ? undefined : plural(people, "person", "people"), "users"),
    row("security", "shield", security ? plural(security, "security update") : firewall === 0 ? "Firewall off" : security === 0 ? "Up to date" : undefined, "overview"),
    row("privacy", "eye", popcon === undefined ? undefined : popcon ? "Sends package list" : "Nothing sent", "access"),
  ];
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
            ["Hostname", system.hostname ?? "-"],
            ["OS", system.os ?? "-"],
            ["Kernel", system.kernel ? <code>{system.kernel}</code> : "-"],
            ["Uptime", system.uptime_seconds === undefined ? "-" : duration(system.uptime_seconds)],
            ["Last boot", system.uptime_seconds === undefined ? "-" : when(now - system.uptime_seconds * 1000, now)],
            ["Reboot required", system.reboot_required ? <StatusBadge variant="warn" label="Yes" /> : "No"],
          ]}
        />
      ) : (
        <div className="empty muted">Loading…</div>
      )}
    </Panel>
  );
}

function RecentActivity() {
  const changes = useChanges();
  return (
    <Panel title="Recent activity" actions={<LinkButton to={href("activity", "timeline")}>View all</LinkButton>} span={6} flush>
      {!changes ? (
        <div className="empty muted">Loading…</div>
      ) : changes.length ? (
        <ActivityTimeline rows={changes.slice(0, 6).map((c) => ({ time: when(c.at), title: c.title, detail: c.detail, tone: c.tone }))} />
      ) : (
        <div className="empty muted">No changes in the last 7 days.</div>
      )}
    </Panel>
  );
}

function ErrorHeatmap() {
  const errors = useLive(loadErrors, 300_000).data;
  return (
    <Panel title="Errors by hour" subtitle="last 7 days" source="journalctl -p err" actions={<LinkButton to={href("logs", "errors")}>View all</LinkButton>} span={6}>
      {errors ? <Heatmap rows={heatRows(errors.times)} unit="errors" /> : <div className="empty muted">Loading…</div>}
    </Panel>
  );
}

