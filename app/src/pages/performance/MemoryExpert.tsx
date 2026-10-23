// SPDX-License-Identifier: Apache-2.0
// Expert sections for memory (docs/01-product/expert-coverage.md, section 2): where every byte
// goes, hard page faults and swapping per second, memory per app and service, real memory per
// process, swap and compression, the out-of-memory guard, and memory errors. Read only while
// these sections are open.

import { type AppError, type Groups, type MemoryDetail, type ProcessMemory, getInventory } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { bytes } from "../../metrics";
import { perSecond, useLive, useLiveWithPrevious } from "../../useLive";
import { NoData } from "../NoData";

const loadDetail = () => getInventory("memory_detail");
const loadGroups = () => getInventory("cgroups");
const loadProcesses = () => getInventory("process_memory");

const KIND: Record<Groups["groups"][number]["kind"], string> = { app: "App", service: "System service", "user-service": "Your service" };
const size = (b?: number) => (b === undefined ? "-" : bytes(b));
const rate = (r?: number) => (r === undefined ? "-" : r >= 100 ? Math.round(r).toLocaleString() : r.toFixed(1));

/** Where RAM goes, from /proc/meminfo, in the order people read it. */
export function breakdown(m: Record<string, number>): { part: string; means: string; bytes: number }[] {
  const v = (k: string) => m[k];
  const rows: [string, string, number | undefined][] = [
    ["Apps", "Memory programs asked for (anonymous pages)", v("AnonPages")],
    ["File cache", "Copies of files; freed at once when apps need memory", v("Cached") !== undefined && v("Shmem") !== undefined ? v("Cached") - v("Shmem") : v("Cached")],
    ["Shared memory", "Memory shared between programs, and RAM disks (tmpfs)", v("Shmem")],
    ["Kernel caches (can be freed)", "Reclaimable kernel memory", v("SReclaimable")],
    ["Kernel caches (cannot be freed)", "Unreclaimable kernel memory", v("SUnreclaim")],
    ["Page tables", "The kernel's map of each program's memory", v("PageTables")],
    ["Kernel stacks", "One per running thread", v("KernelStack")],
    ["Buffers", "Disk blocks the kernel keeps", v("Buffers")],
    ["Waiting to be written", "Changed file data not yet on disk", v("Dirty")],
    ["Being written now", "", v("Writeback")],
    ["Huge pages reserved", "Set aside for programs that ask for huge pages", v("HugePages_Total") && v("Hugepagesize") ? v("HugePages_Total") * v("Hugepagesize") : undefined],
    ["Free", "Not used at all", v("MemFree")],
  ];
  return rows.filter((r): r is [string, string, number] => r[2] !== undefined).map(([part, means, b]) => ({ part, means, bytes: b }));
}

type GroupRow = Groups["groups"][number];
const GROUP_COLUMNS: Column<GroupRow>[] = [
  { key: "name", label: "Name", text: (g) => g.name },
  { key: "kind", label: "Kind", text: (g) => KIND[g.kind] },
  { key: "own", label: "Own memory", text: (g) => size(g.memory_own ?? g.memory), numeric: true },
  { key: "cache", label: "File cache", text: (g) => size(g.memory_cache), numeric: true },
  { key: "peak", label: "Peak", text: (g) => size(g.memory_peak), numeric: true },
  { key: "swap", label: "Swap", text: (g) => size(g.swap), numeric: true },
  { key: "pressure", label: "Waiting for memory", text: (g) => (g.memory_pressure === undefined ? "-" : `${g.memory_pressure.toFixed(1)}%`), numeric: true },
  { key: "unit", label: "Unit", text: (g) => g.unit, expert: true },
];

const PROCESS_COLUMNS: Column<ProcessMemory>[] = [
  { key: "name", label: "Process", text: (p) => p.name },
  { key: "pid", label: "PID", text: (p) => String(p.pid), numeric: true },
  { key: "rss", label: "Resident", text: (p) => size(p.rss), numeric: true },
  { key: "pss", label: "Real (PSS)", text: (p) => size(p.pss), numeric: true },
  { key: "uss", label: "Only its own (USS)", text: (p) => size(p.uss), numeric: true },
  { key: "swap", label: "Swap share", text: (p) => size(p.swap_pss), numeric: true },
  { key: "faults", label: "Hard faults", text: (p) => p.major_faults.toLocaleString(), numeric: true },
  { key: "oom", label: "Out-of-memory score", text: (p) => (p.oom_score === undefined ? "-" : String(p.oom_score)), numeric: true },
];

function Loading({ title, error }: { title: string; error: AppError | null }) {
  return (
    <Panel title={title} span={12}>
      <NoData error={error} waiting="Loading…" />
    </Panel>
  );
}

export function MemoryExpertSections() {
  const { data: d, before, error } = useLiveWithPrevious<MemoryDetail>(loadDetail, 5000);
  const groups = useLive(loadGroups, 10_000);
  const processes = useLive(loadProcesses, 30_000);
  if (!d) return <Loading title="Memory details" error={error} />;
  const seconds = before ? (d.at - before.at) / 1000 : 0;
  const now = (key: string) => perSecond(d.vmstat[key], before?.vmstat[key], seconds);
  const total = d.meminfo.MemTotal;
  const rows = breakdown(d.meminfo);
  const pageSize = 4096;
  const apps = [...(groups.data?.groups ?? [])].filter((g) => (g.memory ?? 0) > 0).sort((a, b) => (b.memory_own ?? b.memory ?? 0) - (a.memory_own ?? a.memory ?? 0));
  const top = (processes.data ?? []).slice(0, 40);
  const other = top.filter((p) => p.pss === undefined).length;
  const swapIn = now("pswpin");
  const swapOut = now("pswpout");
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Hard page faults", value: rate(now("pgmajfault")), unit: before ? "/s" : undefined, subtext: before ? "each waited for the disk" : "per second after the next reading", status: (now("pgmajfault") ?? 0) > 50 ? "warn" : undefined },
          { label: "Swapped in", value: swapIn === undefined ? "-" : bytes(swapIn * pageSize), unit: before ? "/s" : undefined, subtext: `out ${swapOut === undefined ? "-" : `${bytes(swapOut * pageSize)}/s`}`, status: (swapIn ?? 0) * pageSize > 1_000_000 ? "warn" : undefined },
          { label: "Waited while memory was freed", value: rate(now("pgscan_direct")), unit: before ? "pages/s" : undefined, subtext: "apps paused for the kernel to free memory" },
          { label: "Out-of-memory kills", value: d.vmstat.oom_kill === undefined ? "-" : String(d.vmstat.oom_kill), subtext: "since start", status: d.vmstat.oom_kill ? "warn" : "ok" },
        ]}
      />
      <Panel title="Where memory goes" info="From the kernel's own count. Apps plus kernel memory that cannot be freed is what really fills RAM; file cache is given back at once." source="/proc/meminfo" span={6} flush>
        <DataTable
          rows={rows}
          columns={[
            { key: "part", label: "Part", text: (r) => r.part },
            { key: "bytes", label: "Size", text: (r) => bytes(r.bytes), numeric: true },
            { key: "share", label: "Of RAM", text: (r) => (total ? `${((r.bytes / total) * 100).toFixed(1)}%` : "-"), numeric: true },
            { key: "means", label: "What it is", text: (r) => r.means, wrap: true },
          ]}
          rowKey={(r) => r.part}
          pageSize={16}
        />
      </Panel>
      <Panel title="Swap and compression" source="/proc/swaps · /sys/module/zswap · /sys/block/zram*" span={6} flush>
        <KeyValueList
          rows={[
            ...d.swaps.map((s): [string, React.ReactNode] => [s.path, `${bytes(s.used)} used of ${bytes(s.size)} · ${s.kind} · priority ${s.priority}`]),
            ...(d.swaps.length ? [] : [["Swap", "None: this computer has no swap"] as [string, string]]),
            ["Compressed swap in RAM (zswap)", d.zswap ? (d.zswap.enabled ? `On · ${d.zswap.compressor ?? "?"} · up to ${d.zswap.max_pool_percent ?? "?"}% of RAM` : "Off") : "Not in this kernel"],
            ...d.zram.map((z): [string, string] => [z.name, `${bytes(z.original)} stored in ${bytes(z.compressed_used)} of RAM${z.algorithm ? ` · ${z.algorithm}` : ""}`]),
            ["Huge pages", d.huge_pages ? `${d.huge_pages}${d.huge_pages_defrag ? ` · made ${d.huge_pages_defrag === "madvise" ? "when a program asks" : d.huge_pages_defrag}` : ""}` : "-"],
          ]}
        />
      </Panel>
      <Panel title="Memory per app and service" subtitle="each app's processes counted once" info="Own memory is what the app asked for. File cache is files it read, which the kernel frees when needed. Waiting for memory is the share of the last 10 seconds its tasks waited." source="/sys/fs/cgroup" span={12} flush>
        {groups.data ? (
          <DataTable rows={apps} columns={GROUP_COLUMNS} rowKey={(g) => g.unit} pageSize={12} search={{ placeholder: "Filter apps and services" }} />
        ) : (
          <NoData error={groups.error} waiting="Loading…" />
        )}
      </Panel>
      <Panel
        title="Real memory per process"
        subtitle={`the ${top.length} biggest`}
        info="Resident counts shared memory in full for every process. Real (PSS) splits shared memory between the processes that use it, so the numbers add up. Hard faults each waited for the disk. The highest out-of-memory score is stopped first when memory runs out."
        source="/proc/[pid]/smaps_rollup · stat · oom_score"
        span={12}
        flush
      >
        {processes.data ? (
          <>
            <DataTable rows={top} columns={PROCESS_COLUMNS} rowKey={(p) => String(p.pid)} pageSize={12} />
            {other > 0 && <p className="muted panel-pad">{other} of them belong to other users: their real memory needs Full access.</p>}
          </>
        ) : (
          <NoData error={processes.error} waiting="Loading…" />
        )}
      </Panel>
      <Panel title="Out-of-memory guard" info="systemd-oomd stops the biggest app in a group before the whole computer freezes." source="oomctl" span={12} flush>
        {d.oomd ? (
          <KeyValueList
            rows={[
              ["State", <StatusBadge key="s" variant="ok" label="Running" />],
              ["Acts when memory pressure stays above", d.oomd.pressure_limit_percent === undefined ? "-" : `${d.oomd.pressure_limit_percent}%${d.oomd.pressure_duration ? ` for ${d.oomd.pressure_duration}` : ""}`],
              ["Acts when swap use is above", d.oomd.swap_limit_percent === undefined ? "-" : `${d.oomd.swap_limit_percent}%`],
              ["Watches for pressure", d.oomd.pressure_watched.length ? d.oomd.pressure_watched.map(([p, l]) => `${p}${l !== null ? ` (${l}%)` : ""}`).join(", ") : "Nothing"],
              ["Watches for swap", d.oomd.swap_watched.length ? d.oomd.swap_watched.join(", ") : "Nothing"],
            ]}
          />
        ) : (
          <div className="empty muted">systemd-oomd is not running, so only the kernel stops programs when memory runs out.</div>
        )}
      </Panel>
    </>
  );
}

/** Memory errors (ECC), for Hardware › Memory. */
export function MemoryErrors() {
  const { data: d, error } = useLive(loadDetail, 60_000);
  if (!d) return <Loading title="Memory errors" error={error} />;
  const dimms = d.controllers.flatMap((c) => c.dimms.map((m) => ({ ...m, controller: c.name })));
  const uncorrected = d.controllers.reduce((n, c) => n + c.uncorrected, 0);
  const corrected = d.controllers.reduce((n, c) => n + c.corrected, 0);
  return (
    <Panel
      title="Memory errors"
      subtitle={d.controllers.length ? `${corrected} corrected · ${uncorrected} uncorrected` : undefined}
      info="Counted by the memory controller since it was last reset. Corrected errors were fixed on the fly; uncorrected errors mean failing memory."
      source="/sys/devices/system/edac/mc"
      span={12}
      flush
    >
      {d.controllers.length === 0 ? (
        <div className="empty muted">This computer's memory controller does not report errors.</div>
      ) : (
        <DataTable
          rows={dimms}
          columns={[
            { key: "label", label: "Module", text: (m) => m.label ?? "-" },
            { key: "controller", label: "Controller", text: (m) => m.controller },
            { key: "size", label: "Size", text: (m) => (m.size_mb ? bytes(m.size_mb * 1024 * 1024, 0) : "-"), numeric: true },
            { key: "ce", label: "Corrected", text: (m) => (m.corrected === undefined ? "-" : String(m.corrected)), numeric: true },
            {
              key: "ue",
              label: "Uncorrected",
              text: (m) => (m.uncorrected === undefined ? "-" : String(m.uncorrected)),
              render: (m) => (m.uncorrected ? <StatusBadge variant="crit" label={String(m.uncorrected)} /> : String(m.uncorrected ?? "-")),
              numeric: true,
            },
          ]}
          rowKey={(m) => `${m.controller}|${m.label}`}
          empty="No modules listed"
        />
      )}
    </Panel>
  );
}
