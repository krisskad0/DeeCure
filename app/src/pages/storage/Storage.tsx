// SPDX-License-Identifier: Apache-2.0
// Storage › Overview, Filesystems, Disk Usage, and Cleanup (docs/03-ui-design/05-pages/14-storage.md),
// with real data. Cleaning goes through the fix engine, with a preview and undo.

import { Guard } from "../../components/ErrorBoundary";
import { FilesystemExpert } from "../performance/DiskExpert";
import { atLeast, useUi } from "../../ui";
import { type Finding, type SizedPath, type StorageInfo, getInventory, getMetrics, getSnapshot } from "../../api";
import { useFinding } from "../../report";
import { AlertBanner } from "../../components/AlertBanner";
import { Button, LinkButton } from "../../components/Button";
import { type Check, Checklist } from "../../components/Checklist";
import { type Column, DataTable } from "../../components/DataTable";
import { Icon } from "../../components/Icon";
import { InlineMeter } from "../../components/InlineMeter";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { type Allocation, AllocationBar } from "../../components/charts/AllocationBar";
import { BarList } from "../../components/charts/BarList";
import { LineChart } from "../../components/charts/LineChart";
import { RANGE_MS, bytes, pick, when } from "../../metrics";
import { href } from "../../route";
import { useFix } from "../../shell/FixDrawer";
import { useEventCount } from "../../useEventCount";
import { useLive } from "../../useLive";
import { useLatest } from "../performance/shared";
import { NoData } from "../NoData";

interface FsEntry {
  mount_point: string;
  device: string;
  fs_type: string;
  size: number;
}

const loadStorage = () => getInventory("storage");
const loadFilesystems = () => getSnapshot<FsEntry[]>("filesystems");
const loadUsage = () => getInventory("disk_usage");
const loadTrend = () => {
  const until = Date.now();
  return getMetrics("disk.used_percent", until - RANGE_MS["30d"], until, { subject: "/", maxPoints: 30 });
};
const GB = 1e9;

/**
 * The "Full in" tile from the agent's forecast, which is the same trend as the "filling up"
 * finding: both the last day and the last 3 days must show filling, so a one-time jump (an
 * install, a build) does not count. `size` turns percent into GB.
 */
export function fullIn(s: StorageInfo | undefined, size?: number): { value: string; unit?: string; subtext: string; days: number | null } {
  if (!s) return { value: "-", subtext: "Checking…", days: null };
  const f = s.root_forecast;
  if (!f) {
    const have = s.root_history_days ?? 0;
    return { value: "Not known yet", subtext: `needs 3 days of readings (${have.toFixed(1)} so far)`, days: null };
  }
  const perDay = size ? `${((f.percent_per_day / 100) * (size / GB)).toFixed(1)} GB` : `${f.percent_per_day.toFixed(2)}%`;
  if (f.days_until_full === undefined) {
    return { value: "Not filling", subtext: f.percent_per_day > 0 ? `${perDay} a day now, too slow to matter` : "steady or emptying", days: null };
  }
  return { value: `~${Math.max(0, Math.round(f.days_until_full))}`, unit: "days", subtext: `at ${perDay} a day, over the last 3 days`, days: f.days_until_full };
}

const reclaimable = (s?: StorageInfo) => s?.candidates.reduce((sum, c) => sum + c.size, 0) ?? 0;
/** The tile's colour follows the agent's "main disk low on space" warning, with the user's limits. */
const tone = (f?: Finding) => (!f ? "ok" : f.severity === "critical" ? "crit" : "warn");

export function StorageOverview() {
  const latest = useLatest();
  const storage = useLive(loadStorage, 300_000).data;
  const filesystems = useLive(loadFilesystems, 60_000).data?.data;
  const usage = useLive(loadUsage, 30_000).data;
  const trend = useLive(loadTrend, 300_000).data ?? [];
  const pct = pick(latest, "disk.used_percent", "/");
  const used = pick(latest, "disk.used", "/");
  const free = pick(latest, "disk.free", "/");
  const size = filesystems?.find((f) => f.mount_point === "/")?.size;
  const inodes = pick(latest, "disk.inodes_percent", "/");
  const forecast = fullIn(storage, size);
  // Where "Main disk low on space" starts with its default limits: under 10% and under 20 GB free.
  const warnAt = size ? 100 - Math.min(10, (20 * GB * 100) / size) : 90;
  const days = forecast.days;
  const reclaim = reclaimable(storage);
  const lowSpace = useFinding("storage.root_almost_full");
  const t = tone(lowSpace);

  const scan = usage?.scan;
  const segments: Allocation[] = [];
  if (used !== undefined && free !== undefined) {
    if (scan) {
      const sum = (list: SizedPath[], names?: string[]) => list.filter((s) => !names || names.some((n) => s.path === n)).reduce((a, s) => a + s.size, 0);
      const home = sum(scan.home);
      const apps = sum(scan.system, ["/usr", "/opt", "/snap"]);
      const system = sum(scan.system) - apps;
      const other = Math.max(0, used - home - apps - system);
      segments.push({ name: "Home", gb: home / GB, kind: "s1" }, { name: "Apps", gb: apps / GB, kind: "s2" }, { name: "System", gb: system / GB, kind: "s3" }, { name: "Other", gb: other / GB, kind: "s4" });
    } else {
      segments.push({ name: "Used", gb: used / GB, kind: "s1" });
    }
    segments.push({ name: "Free", gb: free / GB, kind: "free" });
  }
  const step = trend.length >= 2 ? (trend[trend.length - 1].at - trend[0].at) / (trend.length - 1) / 86_400_000 : 1;
  const ahead = days !== null && days <= 30 ? Math.max(1, Math.ceil(days / (step || 1))) : 0;
  const fullDate = days !== null ? new Date(Date.now() + days * 86_400_000).toLocaleDateString(undefined, { day: "numeric", month: "short" }) : "";
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Used", value: used === undefined ? "-" : bytes(used).split(" ")[0], unit: used === undefined ? undefined : bytes(used).split(" ")[1], status: t, subtext: pct !== undefined && size ? `${Math.round(pct)}% of ${bytes(size, 0)}` : "", visual: pct === undefined ? undefined : { kind: "bar", percent: pct } },
          { label: "Free", value: free === undefined ? "-" : bytes(free).split(" ")[0], unit: free === undefined ? undefined : bytes(free).split(" ")[1] },
          { label: "Reclaimable", value: storage ? bytes(reclaim).split(" ")[0] : "-", unit: storage ? bytes(reclaim).split(" ")[1] : undefined, subtext: storage ? `${storage.candidates.length} items` : "Checking…", href: href("storage", "cleanup") },
          { label: "Full in", value: forecast.value, unit: forecast.unit, status: days !== null && days < 14 ? "crit" : days !== null && days < 60 ? "warn" : "ok", subtext: forecast.subtext },
          { label: "Inodes used", value: inodes === undefined ? "N/A" : String(Math.round(inodes)), unit: inodes === undefined ? undefined : "%", subtext: inodes === undefined ? "No fixed limit on this filesystem" : "" },
          { label: "Encryption", value: storage ? (storage.encrypted ? "On" : "Off") : "-", subtext: storage ? (storage.encrypted ? "LUKS" : "LUKS not used") : "" },
        ]}
      />
      {lowSpace && (
        <AlertBanner
          tone={t === "crit" ? "crit" : "warn"}
          title={lowSpace.title}
          text={reclaim ? `${bytes(reclaim)} can be cleaned. Personal files are not touched except Trash.` : "Free up space soon: apps and updates can fail on a full disk."}
          actions={<LinkButton to={href("storage", "cleanup")}>See what to clean</LinkButton>}
        />
      )}
      <Panel title="Space allocation" subtitle={filesystems?.find((f) => f.mount_point === "/") ? `${filesystems.find((f) => f.mount_point === "/")?.device} · ${filesystems.find((f) => f.mount_point === "/")?.fs_type}` : undefined} source="statvfs · folder scan" span={12}>
        {segments.length ? <AllocationBar segments={segments} totalGb={(size ?? 0) / GB} /> : <div className="empty muted">Measuring…</div>}
        {!scan && usage?.scanning && <p className="muted">Scanning folders to split the used space… this can take a minute.</p>}
      </Panel>
      <Panel title="Usage trend and forecast" subtitle="30 days" info="The dashed line shows when the disk fills at the current rate." span={8}>
        {trend.length >= 2 ? (
          <LineChart
            series={[{ name: "Used", data: trend.map((p) => p.value), slot: 1 }]}
            x={{ kind: "time", times: trend.map((p) => p.at), range: "30d" }}
            height={220}
            min={Math.max(0, Math.floor(Math.min(...trend.map((p) => p.value)) / 10) * 10 - 10)}
            max={100}
            unit="%"
            threshold={{ value: warnAt, label: `Low on space from ${Math.round(warnAt)}%` }}
            forecast={ahead ? { ahead, value: 100, label: `Full ~${fullDate}` } : undefined}
          />
        ) : (
          <div className="chart-empty muted">The trend fills in as DeeCure keeps measuring, one point a day.</div>
        )}
      </Panel>
      <Panel title="Mount points" span={4} flush>
        <DataTable
          rows={filesystems ?? []}
          columns={[
            { key: "mount", label: "Mount", text: (f) => f.mount_point, render: (f) => <code>{f.mount_point}</code> },
            { key: "size", label: "Size", text: (f) => bytes(f.size, 0), numeric: true },
            { key: "used", label: "Used", text: (f) => `${Math.round(pick(latest, "disk.used_percent", f.mount_point) ?? 0)}%`, render: (f) => <InlineMeter percent={pick(latest, "disk.used_percent", f.mount_point) ?? 0} /> },
          ]}
          rowKey={(f) => f.mount_point}
          empty="Loading…"
        />
      </Panel>
    </>
  );
}

export function StorageFilesystems() {
  const { mode } = useUi();
  const latest = useLatest();
  const filesystems = useLive(loadFilesystems, 60_000).data?.data;
  const storage = useLive(loadStorage, 300_000).data;
  const diskErrors = useEventCount(["hardware_error"], "disk_io");
  const columns: Column<FsEntry>[] = [
    { key: "device", label: "Device", text: (f) => f.device, render: (f) => <code>{f.device}</code> },
    { key: "mount", label: "Mount", text: (f) => f.mount_point, render: (f) => <code>{f.mount_point}</code> },
    { key: "type", label: "Type", text: (f) => f.fs_type },
    { key: "size", label: "Size", text: (f) => bytes(f.size), numeric: true },
    { key: "used", label: "Used", text: (f) => { const u = pick(latest, "disk.used", f.mount_point); return u === undefined ? "-" : bytes(u); }, numeric: true },
    { key: "avail", label: "Available", text: (f) => { const a = pick(latest, "disk.free", f.mount_point); return a === undefined ? "-" : bytes(a); }, numeric: true },
    { key: "use", label: "Use", text: (f) => `${Math.round(pick(latest, "disk.used_percent", f.mount_point) ?? 0)}%`, render: (f) => <InlineMeter percent={pick(latest, "disk.used_percent", f.mount_point) ?? 0} /> },
    { key: "inodes", label: "Inodes", text: (f) => { const i = pick(latest, "disk.inodes_percent", f.mount_point); return i === undefined ? "-" : `${Math.round(i)}%`; }, numeric: true, expert: true },
  ];
  const checks: Check[] = storage
    ? [
        storage.fstab_problems.length ? { state: "fail", title: `${storage.fstab_problems.length} problem${storage.fstab_problems.length === 1 ? "" : "s"} in /etc/fstab`, detail: storage.fstab_problems.map((p) => `line ${p.line}: ${p.problem}`).join(" · ") } : { state: "pass", title: "fstab entries valid", detail: `${storage.fstab_entries} entries, every device found` },
        diskErrors === undefined
          ? { state: "na", title: "Filesystem errors in the kernel log", detail: "See the kernel log", action: <LinkButton to={href("logs", "kernel")}>View</LinkButton> }
          : diskErrors === 0
            ? { state: "pass", title: "No disk read or write errors in the kernel log", detail: "Last 7 days" }
            : { state: "fail", title: `${diskErrors} disk read or write error${diskErrors === 1 ? "" : "s"} in the kernel log`, detail: "Last 7 days · back up your files", action: <LinkButton to={href("logs", "kernel")}>View</LinkButton> },
        storage.root_reserved ? { state: "pass", title: `Reserved for root: ${bytes(storage.root_reserved)}`, detail: "Lets the system keep working when the disk is full" } : { state: "na", title: "No space reserved for root", detail: "Normal for btrfs and some other filesystems" },
        storage.encrypted ? { state: "pass", title: "Disk encryption on", detail: "LUKS" } : { state: "warn", title: "Disk encryption off", detail: "Files are readable if the computer is stolen · needs a reinstall to turn on" },
        { state: "na", title: "Full filesystem check (fsck)", detail: "Runs at start when needed" },
      ]
    : [];
  return (
    <>
      <Panel title="Filesystems" source="/proc/mounts · statvfs" span={12} flush>
        {filesystems ? <DataTable rows={filesystems} columns={columns} rowKey={(f) => f.mount_point} /> : <div className="empty muted">Loading…</div>}
      </Panel>
      <Panel title="Filesystem checks" span={12} flush>
        {storage ? <Checklist checks={checks} /> : <div className="empty muted">Loading…</div>}
      </Panel>
      {atLeast(mode, "expert") && <Guard><FilesystemExpert /></Guard>}
    </>
  );
}

const home = (path: string) => path.replace(/^\/home\/[^/]+/, "~");

export function StorageUsage() {
  const { data: usage, error } = useLive(loadUsage, 5_000);
  const scan = usage?.scan;
  if (!scan) {
    return (
      <Panel title="Disk usage" span={12}>
        <NoData error={error} waiting={usage?.scanning ? "Scanning folders… this can take a minute on a full disk." : "Loading…"} />
      </Panel>
    );
  }
  const files: Column<SizedPath>[] = [
    { key: "path", label: "File", text: (f) => home(f.path), render: (f) => <code>{home(f.path)}</code>, wrap: true },
    { key: "size", label: "Size", text: (f) => bytes(f.size), numeric: true },
    { key: "modified", label: "Changed", text: (f) => (f.modified ? when(f.modified) : "-") },
  ];
  return (
    <>
      <Panel title="Largest folders" subtitle="home" source="like du -h --max-depth=1 ~" span={6}>
        <BarList rows={scan.home.slice(0, 10).map((s) => ({ label: home(s.path), value: s.size, display: bytes(s.size), mono: true }))} />
      </Panel>
      <Panel title="Largest system folders" source="like du -xh --max-depth=1 /" span={6}>
        <BarList rows={scan.system.slice(0, 10).map((s) => ({ label: s.path, value: s.size, display: bytes(s.size), mono: true }))} />
        {scan.unreadable > 0 && <p className="muted">{scan.unreadable} folders could not be read (they belong to other users or root), so system sizes are lower than the real ones.</p>}
      </Panel>
      <Panel title="Largest files" subtitle={`in home · scanned ${when(scan.scanned_at)}${usage?.scanning ? " · rescanning" : ""}`} span={12} flush>
        <DataTable rows={scan.files} columns={files} rowKey={(f) => f.path} search={{ placeholder: "Filter files" }} empty="No files over 50 MB" />
      </Panel>
    </>
  );
}

type Candidate = StorageInfo["candidates"][number];

const CANDIDATES: Column<Candidate>[] = [
  { key: "item", label: "Item", text: (c) => `${c.title} ${c.details}`, render: (c) => (<span className="cell-two"><b>{c.title}</b><span>{c.details}</span></span>) },
  { key: "size", label: "Size", text: (c) => bytes(c.size), numeric: true },
  { key: "undo", label: "Undo", text: (c) => c.undo, render: (c) => <StatusBadge variant={c.undo === "irreversible" ? "warn" : "info"} label={c.undo === "irreversible" ? "Irreversible" : "Regenerates"} /> },
  { key: "pw", label: "Password", text: (c) => (c.needs_password ? "Yes" : "No"), render: (c) => (c.needs_password ? <Icon name="lock" size={15} /> : <span className="muted">-</span>) },
];

export function StorageCleanup() {
  const { data: storage, error } = useLive(loadStorage, 300_000);
  const openFix = useFix();
  if (!storage) {
    return (
      <Panel title="Cleanup candidates" span={12}>
        <NoData error={error} waiting="Looking for things to clean… (Trash, caches, old snaps, Docker)" />
      </Panel>
    );
  }
  const total = reclaimable(storage);
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Reclaimable", value: bytes(total).split(" ")[0], unit: bytes(total).split(" ")[1], status: total > 5e9 ? "warn" : "ok" },
          { label: "Items", value: String(storage.candidates.length) },
        ]}
      />
      <Panel
        title="Cleanup candidates"
        source="Trash · caches · snap · journal · docker"
        actions={storage.candidates.length > 0 && <Button small variant="primary" onClick={() => openFix("storage.free_space")}>Clean up…</Button>}
        span={12}
        flush
      >
        <DataTable rows={storage.candidates} columns={CANDIDATES} rowKey={(c) => c.action} empty="Nothing to clean right now" />
      </Panel>
    </>
  );
}
