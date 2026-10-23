// SPDX-License-Identifier: Apache-2.0
// The Expert part of the process drawer (docs/01-product/expert-coverage.md, section 9): where the
// process comes from, how it is scheduled, its open files and limits, its busiest threads, its
// sandbox, and everything it costs. Other users' processes show what the kernel lets everyone
// read, and say what is hidden.

import { type ProcessDetail, getProcessDetail } from "../../api";
import { KeyValueList } from "../../components/KeyValueList";
import { StatusBadge } from "../../components/StatusBadge";
import { bytes } from "../../metrics";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

/** "1h 5m", "3m 20s", or "42 s". */
export function duration(seconds: number): string {
  if (seconds >= 3600) return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
  if (seconds >= 60) return `${Math.floor(seconds / 60)}m ${Math.round(seconds % 60)}s`;
  return `${seconds.toFixed(seconds < 10 ? 1 : 0)} s`;
}

/** The app or service from a control group path, such as `snap.firefox.firefox`. */
export function unitOf(cgroup?: string): string | undefined {
  const last = cgroup?.split("/").filter(Boolean).at(-1);
  return last && (last.endsWith(".scope") || last.endsWith(".service")) ? last.replace(/\\x2d/g, "-") : undefined;
}

/** A plain word for how confined a process is. */
export function sandbox(d: ProcessDetail): { variant: "ok" | "info" | "off"; text: string } {
  const confined = d.apparmor && d.apparmor !== "unconfined";
  const ns = d.own_namespaces?.length ?? 0;
  if (confined && d.apparmor?.includes("(enforce)")) return { variant: "ok", text: `Sandboxed (AppArmor: ${d.apparmor.replace(" (enforce)", "")})` };
  if (confined) return { variant: "info", text: `AppArmor watches it: ${d.apparmor}` };
  if (ns > 0) return { variant: "info", text: `In its own ${d.own_namespaces?.join(", ")} space (container or sandbox)` };
  return { variant: "off", text: "Not sandboxed" };
}

const HIDDEN = "Hidden: another user's process";

export function ProcessExpertDetail({ pid }: { pid: number }) {
  const { data: d, error } = useLive(() => getProcessDetail(pid), 5000, String(pid));
  if (!d) return <NoData error={error} waiting="Loading details…" />;
  const own = d.program !== undefined;
  const files = d.limits.find((l) => l.name === "Max open files");
  const box = sandbox(d);
  return (
    <>
      <div className="field-label">Where it comes from</div>
      <KeyValueList
        rows={[
          ["Program", d.program ?? HIDDEN],
          ["Package", d.package ?? (own ? "Not from a package" : "-")],
          ["Started by", d.ancestors.length ? d.ancestors.map((a) => `${a.name} (${a.pid})`).join(" ← ") : "-"],
          ["Its own processes", d.children.length ? `${d.children.length}: ${[...new Set(d.children.map((c) => c.name))].slice(0, 5).join(", ")}` : "None"],
          ["Runs as part of", unitOf(d.cgroup) ?? d.cgroup ?? "-"],
          ["Working folder", d.folder ?? (own ? "-" : HIDDEN)],
          ["Started", d.started_at ? new Date(d.started_at).toLocaleString() : "-"],
        ]}
      />
      <div className="field-label">How it is scheduled</div>
      <KeyValueList
        rows={[
          ["CPU time used", duration(d.cpu_seconds)],
          ["Priority (nice)", `${d.nice}${d.nice > 0 ? " (gives way to others)" : d.nice < 0 ? " (goes first)" : ""}`],
          ["Scheduling", d.policy === "normal" ? "Normal" : `${d.policy}${d.realtime_priority ? `, real-time priority ${d.realtime_priority}` : ""}`],
          ["Disk priority", d.disk_priority ?? "-"],
          ["May run on CPUs", d.cpus_allowed ?? "-"],
          ["Switches (gave way · was pushed off)", d.voluntary_switches !== undefined ? `${d.voluntary_switches.toLocaleString()} · ${(d.forced_switches ?? 0).toLocaleString()}` : "-"],
        ]}
      />
      {d.top_threads.length > 1 && (
        <>
          <div className="field-label">Busiest threads ({d.threads})</div>
          <KeyValueList rows={d.top_threads.map((t): [string, string] => [`${t.name} (${t.tid})`, duration(t.cpu_seconds)])} />
        </>
      )}
      <div className="field-label">Files and limits</div>
      <KeyValueList
        rows={[
          [
            "Open files",
            d.open_files === undefined ? (
              HIDDEN
            ) : (
              <span key="f">
                {d.open_files.toLocaleString()}
                {files && files.soft !== "unlimited" ? ` of ${Number(files.soft).toLocaleString()}` : ""}
                {files && files.soft !== "unlimited" && d.open_files > Number(files.soft) * 0.8 ? <> <StatusBadge variant="warn" label="Near its limit" /></> : null}
                {d.sockets ? ` · ${d.sockets} connections` : ""}
              </span>
            ),
          ],
          ...d.limits
            .filter((l) => l.soft !== "unlimited" && l.name !== "Max open files")
            .slice(0, 6)
            .map((l): [string, string] => [l.name, `${l.soft}${l.hard !== l.soft ? ` (up to ${l.hard})` : ""}${l.unit ? ` ${l.unit}` : ""}`]),
        ]}
      />
      <div className="field-label">Sandbox and rights</div>
      <KeyValueList
        rows={[
          ["Confinement", <StatusBadge key="b" variant={box.variant} label={box.text} />],
          ["Special rights (capabilities)", d.capabilities.length ? (d.capabilities.length > 12 ? `All ${d.capabilities.length} (full admin rights)` : d.capabilities.join(", ")) : "None"],
          ["System call filter (seccomp)", d.seccomp === "filter" ? "On" : d.seccomp === "strict" ? "Strict" : d.seccomp === "none" ? "Off" : "-"],
          ["Can never gain more rights", d.no_new_privileges ? "Yes" : "No"],
          ["Own spaces (namespaces)", d.own_namespaces === undefined ? HIDDEN : d.own_namespaces.length ? d.own_namespaces.join(", ") : "None: shares the system's"],
        ]}
      />
      <div className="field-label">What it costs</div>
      <KeyValueList
        rows={[
          ["Real memory (PSS)", d.pss !== undefined ? `${bytes(d.pss)} (only its own: ${d.uss !== undefined ? bytes(d.uss) : "-"})` : `${bytes(d.rss)} resident · real memory: ${HIDDEN.toLowerCase()}`],
          ["Swap", d.swap !== undefined ? bytes(d.swap) : "-"],
          ["Hard page faults", d.major_faults.toLocaleString()],
          ["Disk read · written", d.disk_read !== undefined ? `${bytes(d.disk_read)} · ${bytes(d.disk_write ?? 0)}` : own ? "-" : HIDDEN],
          ["Graphics time", d.gpu_ns ? duration(d.gpu_ns / 1e9) : own ? "None" : HIDDEN],
          ["Out-of-memory score", d.oom_score !== undefined ? `${d.oom_score}${d.oom_score_adj ? ` (adjusted ${d.oom_score_adj > 0 ? "+" : ""}${d.oom_score_adj})` : ""}` : "-"],
        ]}
      />
    </>
  );
}
