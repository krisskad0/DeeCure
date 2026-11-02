// SPDX-License-Identifier: Apache-2.0
// Expert sections for memory (docs/01-product/expert-coverage.md, section 2): where every byte
// goes (a memory map), hard page faults and swapping per second, the top apps and services and
// the top processes by real memory (full tables under "Show all"), swap and compression, the
// out-of-memory guard, and memory errors. Read only while these sections are open.

import { type AppError, type Groups, type MemoryDetail, type ProcessMemory, getInventory } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { InlineMeter } from "../../components/InlineMeter";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { BarList } from "../../components/charts/BarList";
import { bytes } from "../../metrics";
import { perSecond, useLive, useLiveWithPrevious } from "../../useLive";
import { NoData } from "../NoData";
import { MemoryMap, memoryParts } from "./MemoryMap";

const loadDetail = () => getInventory("memory_detail");
const loadGroups = () => getInventory("cgroups");
const loadProcesses = () => getInventory("process_memory");

const KIND: Record<Groups["groups"][number]["kind"], string> = { app: "App", service: "System service", "user-service": "Your service" };
const size = (b?: number) => (b === undefined ? "-" : bytes(b));
const rate = (r?: number) => (r === undefined ? "-" : r >= 100 ? Math.round(r).toLocaleString() : r.toFixed(1));

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

/** The full table behind a top-10 view, folded away until asked for. */
function ShowAll({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <details className="show-all">
      <summary>{label}</summary>
      {children}
    </details>
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
  const pageSize = 4096;
  const own = (g: GroupRow) => g.memory_own ?? g.memory ?? 0;
  const apps = [...(groups.data?.groups ?? [])].filter((g) => (g.memory ?? 0) > 0).sort((a, b) => own(b) - own(a));
  const top = (processes.data ?? []).slice(0, 40);
  const other = top.filter((p) => p.pss === undefined).length;
  const real = (p: ProcessMemory) => p.pss ?? p.rss;
  const byReal = [...top].sort((a, b) => real(b) - real(a));
  const swapIn = now("pswpin");
  const swapOut = now("pswpout");
  const m = d.meminfo;
  const writing = [
    m.Dirty ? `${bytes(m.Dirty)} waiting to be written to disk` : "",
    m.Writeback ? `${bytes(m.Writeback)} being written now` : "",
    m.HugePages_Total && m.Hugepagesize ? `${bytes(m.HugePages_Total * m.Hugepagesize)} set aside as huge pages` : "",
  ].filter(Boolean);
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
      <Panel title="Where memory goes" info="From the kernel's own count. Apps plus kernel memory that cannot be freed is what really fills RAM; striped parts are given back at once when an app needs memory." source="/proc/meminfo" span={12}>
        {total ? <MemoryMap parts={memoryParts(m)} total={total} detailed /> : <div className="empty muted">The kernel did not report its memory.</div>}
        {writing.length > 0 && <p className="muted mm-note">Also: {writing.join(" · ")}.</p>}
      </Panel>
      <Panel title="Apps and services" subtitle="top 10 by own memory" info="Each app's processes counted once. Own memory is what the app asked for; file cache is files it read, which the kernel frees when needed." source="/sys/fs/cgroup" span={6}>
        {groups.data ? (
          <>
            <BarList
              rows={apps.slice(0, 10).map((g) => ({
                id: g.unit,
                label: g.name,
                hint: `${KIND[g.kind]}${g.memory_cache ? ` · +${bytes(g.memory_cache)} file cache` : ""}${g.memory_pressure ? ` · waited ${g.memory_pressure.toFixed(1)}%` : ""}`,
                value: own(g),
                display: bytes(own(g)),
              }))}
            />
            <ShowAll label={`Show all ${apps.length} apps and services`}>
              <DataTable rows={apps} columns={GROUP_COLUMNS} rowKey={(g) => g.unit} pageSize={12} search={{ placeholder: "Filter apps and services" }} />
            </ShowAll>
          </>
        ) : (
          <NoData error={groups.error} waiting="Loading…" />
        )}
      </Panel>
      <Panel
        title="Real memory per process"
        subtitle="top 10"
        info="Real memory (PSS) splits memory shared between processes, so the numbers add up. Resident counts shared memory in full for every process. The highest out-of-memory score is stopped first when memory runs out."
        source="/proc/[pid]/smaps_rollup · stat · oom_score"
        span={6}
      >
        {processes.data ? (
          <>
            <BarList
              rows={byReal.slice(0, 10).map((p) => ({
                id: String(p.pid),
                label: p.name,
                hint: [`PID ${p.pid}`, p.uss !== undefined ? `own ${bytes(p.uss)}` : `resident ${bytes(p.rss)}`].join(" · "),
                value: real(p),
                display: bytes(real(p)),
              }))}
            />
            {other > 0 && <p className="muted mm-note">{other} of the biggest belong to other users: their real memory needs Full access, so their resident size is shown.</p>}
            <ShowAll label={`Show all ${top.length} with resident size, swap, hard faults, and out-of-memory score`}>
              <DataTable rows={top} columns={PROCESS_COLUMNS} rowKey={(p) => String(p.pid)} pageSize={12} />
            </ShowAll>
          </>
        ) : (
          <NoData error={processes.error} waiting="Loading…" />
        )}
      </Panel>
      <Panel title="Swap and compression" source="/proc/swaps · /sys/module/zswap · /sys/block/zram*" span={6} flush>
        <KeyValueList
          rows={[
            ...d.swaps.map((s): [string, React.ReactNode] => [
              s.path,
              <span key={s.path} className="mm-swap">
                <InlineMeter percent={s.size ? (s.used / s.size) * 100 : 0} label={`${bytes(s.used)} of ${bytes(s.size)}`} />
                <span className="muted">{s.kind} · priority {s.priority}</span>
              </span>,
            ]),
            ...(d.swaps.length ? [] : [["Swap", "None: this computer has no swap"] as [string, string]]),
            ["Compressed swap in RAM (zswap)", d.zswap ? (d.zswap.enabled ? <StatusBadge key="z" variant="ok" label={`On · ${d.zswap.compressor ?? "?"} · up to ${d.zswap.max_pool_percent ?? "?"}% of RAM`} /> : "Off") : "Not in this kernel"],
            ...d.zram.map((z): [string, string] => [z.name, `${bytes(z.original)} stored in ${bytes(z.compressed_used)} of RAM${z.algorithm ? ` · ${z.algorithm}` : ""}`]),
            ["Huge pages", d.huge_pages ? `${d.huge_pages}${d.huge_pages_defrag ? ` · made ${d.huge_pages_defrag === "madvise" ? "when a program asks" : d.huge_pages_defrag}` : ""}` : "-"],
          ]}
        />
      </Panel>
      <Panel title="Out-of-memory guard" info="systemd-oomd stops the biggest app in a group before the whole computer freezes." source="oomctl" span={6} flush>
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
