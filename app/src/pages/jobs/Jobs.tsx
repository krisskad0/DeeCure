// SPDX-License-Identifier: Apache-2.0
// Scheduled Jobs › Calendar, Cron, Systemd Timers, Anacron, At Jobs
// (docs/03-ui-design/05-pages/11-jobs.md), with real data.

import { type JobsInfo, type Timer, getInventory } from "../../api";
import { AlertBanner } from "../../components/AlertBanner";
import { type Column, DataTable } from "../../components/DataTable";
import { Icon } from "../../components/Icon";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { Heatmap } from "../../components/charts/Heatmap";
import { type Lane, SwimLanes } from "../../components/charts/SwimLanes";
import { describeCron, nextRun, parseCron, runsBetween } from "../../cron";
import { heatRows, when } from "../../metrics";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

type CronJob = JobsInfo["cron"][number];

const loadJobs = () => getInventory("jobs");
const useJobs = () => useLive(loadJobs, 60_000);
const DAY = 86_400_000;
const HOUR = 3_600_000;

/** A short name for a cron command, for the swim lanes: the program, without paths. */
export function shortCommand(command: string): string {
  // Skip wrappers such as "cd / &&" and "test -x … || {", which may come one after another.
  const words = command.replace(/^(?:cd \S+ && |test -x \S+ \|\| \{ )+/, "").split(/\s+/);
  const program = words.find((w) => !w.includes("=") && w !== "command" && w !== "-v") ?? command;
  // For run-parts, the folder it runs says what the job is, such as cron.daily.
  const folder = words.filter((w) => w.startsWith("/")).pop()?.replace(/[;}]+$/, "");
  return program === "run-parts" && folder ? `run-parts ${folder.split("/").pop()}` : (program.split("/").pop() ?? program);
}

function Loading({ error }: { error: Parameters<typeof NoData>[0]["error"] }) {
  return (
    <Panel title="Scheduled jobs" span={12}>
      <NoData error={error} waiting="Loading…" />
    </Panel>
  );
}

export function JobsCalendar() {
  const { data: jobs, error } = useJobs();
  if (!jobs) return <Loading error={error} />;
  const now = Date.now();
  const lanes: Lane[] = [];
  for (const job of jobs.cron) {
    const c = parseCron(job.schedule);
    const runs = c ? runsBetween(c, now, now + DAY, 200) : [];
    if (runs.length) lanes.push({ name: shortCommand(job.command), runs: runs.map((t) => (t - now) / HOUR), frequent: runs.length > 12 });
  }
  for (const t of jobs.timers) {
    if (t.next && t.next - now <= DAY && t.next >= now) lanes.push({ name: t.name.replace(/\.timer$/, ""), runs: [(t.next - now) / HOUR] });
  }
  lanes.sort((a, b) => a.runs[0] - b.runs[0]);
  const nexts = [
    ...jobs.cron.map((j) => ({ name: shortCommand(j.command), at: parseCron(j.schedule) ? nextRun(parseCron(j.schedule)!, now) : null })),
    ...jobs.timers.map((t) => ({ name: t.name, at: t.next ?? null })),
  ].filter((n): n is { name: string; at: number } => n.at !== null && n.at >= now);
  const soonest = nexts.sort((a, b) => a.at - b.at)[0];
  const failed = jobs.runs.filter((r) => r.failed);
  const minutes = soonest ? Math.max(0, Math.round((soonest.at - now) / 60_000)) : 0;
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Total jobs", value: String(jobs.cron.length + jobs.timers.length + jobs.anacron.length + jobs.at.length), subtext: "cron · timer · anacron · at" },
          { label: "Next run", value: soonest ? (minutes < 120 ? String(minutes) : when(soonest.at)) : "-", unit: soonest && minutes < 120 ? "min" : undefined, subtext: soonest?.name ?? "" },
          { label: "Failed", value: String(failed.length), status: failed.length ? "warn" : "ok", subtext: failed.length ? [...new Set(failed.map((f) => f.job))].slice(0, 2).join(", ") : "last 7 days" },
          { label: "Runs", value: jobs.runs.length.toLocaleString(), subtext: "last 7 days" },
        ]}
      />
      <Panel title="Next 24 hours" source="crontab · systemd timers" span={12}>
        {lanes.length ? <SwimLanes lanes={lanes.slice(0, 14)} now={new Date(now)} /> : <div className="empty muted">Nothing runs in the next 24 hours.</div>}
      </Panel>
      <Panel title="Job runs by hour" subtitle="last 7 days" source="journal: CRON · systemd" span={12}>
        <Heatmap rows={heatRows(jobs.runs.map((r) => r.at))} unit="runs" />
      </Panel>
    </>
  );
}

const CRON: Column<CronJob>[] = [
  { key: "schedule", label: "Schedule", text: (j) => j.schedule, render: (j) => (<span className="cell-two"><b>{j.schedule}</b><span>{describeCron(j.schedule)}</span></span>) },
  { key: "command", label: "Command", text: (j) => j.command, render: (j) => <code className="wrap-anywhere">{j.command}</code>, wrap: true },
  { key: "user", label: "User", text: (j) => j.user },
  { key: "source", label: "Source", text: (j) => j.source, render: (j) => <code>{j.source}</code> },
  {
    key: "next",
    label: "Next run",
    text: (j) => {
      const c = parseCron(j.schedule);
      const n = c && nextRun(c, Date.now());
      return n ? when(n) : j.schedule === "@reboot" ? "At next start" : "-";
    },
  },
  { key: "result", label: "Last result", text: () => "Not recorded", render: () => <StatusBadge variant="off" label="Not recorded" /> },
];

export function JobsCron() {
  const { data: jobs, error } = useJobs();
  if (!jobs) return <Loading error={error} />;
  return (
    <>
      <Panel title="Cron jobs" source="crontab -l · /etc/crontab · /etc/cron.d" span={12} flush>
        <DataTable rows={jobs.cron} columns={CRON} rowKey={(j) => `${j.source}|${j.schedule}|${j.command}`} empty="No cron jobs" />
      </Panel>
      <AlertBanner tone="info" title="Cron does not record job results" text="Cron only logs that a job started. Tracking exit codes and output comes in a later version." />
    </>
  );
}

const RESULT: Record<string, ["ok" | "crit" | "warn", string]> = { success: ["ok", "Success"], "exit-code": ["crit", "Failed"], timeout: ["crit", "Timed out"], signal: ["crit", "Killed"], "core-dump": ["crit", "Crashed"] };

const TIMERS: Column<Timer>[] = [
  { key: "name", label: "Timer", text: (t) => t.name, render: (t) => <code>{t.name}</code> },
  { key: "activates", label: "Activates", text: (t) => t.activates, render: (t) => <code>{t.activates}</code>, expert: true },
  { key: "schedule", label: "Schedule", text: (t) => t.schedule.join(", "), render: (t) => <code className="wrap-anywhere">{t.schedule.join(", ") || "-"}</code>, wrap: true },
  { key: "last", label: "Last trigger", text: (t) => (t.last ? when(t.last) : "Never") },
  { key: "next", label: "Next", text: (t) => (t.next ? when(t.next) : "-") },
  {
    key: "result",
    label: "Result",
    text: (t) => t.result ?? "-",
    render: (t) => (t.result ? <StatusBadge variant={RESULT[t.result]?.[0] ?? "warn"} label={RESULT[t.result]?.[1] ?? t.result} /> : <span className="muted">{t.last ? "-" : "Not run yet"}</span>),
  },
  { key: "persistent", label: "Persistent", text: (t) => (t.persistent ? "Yes" : "No"), render: (t) => (t.persistent ? <Icon name="check" size={16} /> : <span className="muted">-</span>) },
  { key: "scope", label: "Scope", text: (t) => t.scope, expert: true },
];

export function JobsTimers() {
  const { data: jobs, error } = useJobs();
  if (!jobs) return <Loading error={error} />;
  return (
    <Panel title="Timers" subtitle={`${jobs.timers.length} timers`} source="systemctl list-timers --all" span={12} flush>
      <DataTable rows={jobs.timers} columns={TIMERS} rowKey={(t) => `${t.scope}|${t.name}`} filter={{ column: "scope", options: ["system", "user"] }} pageSize={14} />
    </Panel>
  );
}

type Anacron = JobsInfo["anacron"][number];
const period = (p: string) => (p === "@monthly" ? "1 month" : p === "@weekly" ? "1 week" : p === "@yearly" ? "1 year" : `${p} day${p === "1" ? "" : "s"}`);

const ANACRON: Column<Anacron>[] = [
  { key: "id", label: "Job", text: (a) => a.id, render: (a) => <code>{a.id}</code> },
  { key: "period", label: "Period", text: (a) => period(a.period) },
  { key: "delay", label: "Delay", text: (a) => `${a.delay} min`, numeric: true },
  { key: "command", label: "Command", text: (a) => a.command, render: (a) => <code>{a.command}</code> },
  { key: "last", label: "Last run", text: (a) => (a.last_run ? when(a.last_run) : "-") },
];

export function JobsAnacron() {
  const { data: jobs, error } = useJobs();
  if (!jobs) return <Loading error={error} />;
  return (
    <Panel title="Anacron jobs" source="/etc/anacrontab · /var/spool/anacron" span={12} flush>
      <DataTable rows={jobs.anacron} columns={ANACRON} rowKey={(a) => a.id} empty="Anacron is not installed" />
    </Panel>
  );
}

type At = JobsInfo["at"][number];
const AT: Column<At>[] = [
  { key: "id", label: "Job", text: (a) => `#${a.id}`, render: (a) => <code>#{a.id}</code> },
  { key: "when", label: "Run at", text: (a) => a.when, render: (a) => (Number.isNaN(Date.parse(a.when)) ? a.when : when(Date.parse(a.when))) },
  { key: "queue", label: "Queue", text: (a) => a.queue },
  { key: "user", label: "User", text: (a) => a.user },
];

export function JobsAt() {
  const { data: jobs, error } = useJobs();
  if (!jobs) return <Loading error={error} />;
  return (
    <Panel title="One-time jobs" source="atq" span={12} flush>
      {jobs.at_installed ? <DataTable rows={jobs.at} columns={AT} rowKey={(a) => a.id} empty="No one-time jobs" /> : <div className="empty muted">The at tools are not installed, so there are no one-time jobs.</div>}
    </Panel>
  );
}
