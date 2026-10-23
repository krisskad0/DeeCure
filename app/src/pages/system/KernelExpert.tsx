// SPDX-License-Identifier: Apache-2.0
// Expert sections for the kernel and system limits (docs/01-product/expert-coverage.md, section
// 11): kernel settings from files with who set each, file watches and open files against their
// limits, loaded modules, extra drivers (DKMS), lockdown, and time sync.

import { type KernelDetail, getInventory } from "../../api";
import { DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { bytes } from "../../metrics";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadDetail = () => getInventory("kernel_detail");

/** Limits this big mean "no real limit". */
const NO_LIMIT = 1e15;

/** Kernels that a DKMS module is not installed for. */
export function missingKernels(d: KernelDetail, module: { kernels: string[] }): string[] {
  return d.kernels.filter((k) => !module.kernels.includes(k));
}

type Setting = KernelDetail["settings"][number];

export function KernelExpertSections() {
  const { data: d, error } = useLive(loadDetail, 60_000);
  if (!d) {
    return (
      <Panel title="Kernel details" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const watchShare = d.watch_limit ? (d.watches_used / d.watch_limit) * 100 : undefined;
  const tainted = d.modules.filter((m) => m.taint);
  const byYou = d.settings.filter((s) => !s.package || s.file_changed);
  const offset = d.time_sync.Offset;
  return (
    <>
      <KpiRow
        tiles={[
          {
            label: "File watches",
            value: watchShare === undefined ? "-" : watchShare.toFixed(0),
            unit: watchShare === undefined ? undefined : "%",
            subtext: d.watch_limit ? `${d.watches_used.toLocaleString()} of ${d.watch_limit.toLocaleString()}${d.watch_users[0] ? ` · most: ${d.watch_users[0].name}` : ""}` : "",
            status: (watchShare ?? 0) > 80 ? "warn" : undefined,
          },
          { label: "Open files", value: d.open_files === undefined ? "-" : d.open_files.toLocaleString(), subtext: d.open_files_limit === undefined ? "" : d.open_files_limit > NO_LIMIT ? "no system-wide limit" : `of ${d.open_files_limit.toLocaleString()}` },
          { label: "Kernel lockdown", value: d.lockdown ? d.lockdown[0].toUpperCase() + d.lockdown.slice(1) : "-", subtext: d.lockdown === "none" ? "unsigned changes allowed" : d.lockdown ? "blocks unsigned kernel changes" : "" },
          { label: "Clock offset", value: offset ?? "-", subtext: d.time_sync.Server ? `from ${d.time_sync.Server}` : "time sync not running" },
        ]}
      />
      <Panel
        title="Kernel settings from files"
        subtitle={`${d.settings.length} settings · ${byYou.length} set or changed on this computer`}
        info="Settings applied at every start from the sysctl.d files. Most come from Ubuntu's packages. A later file replaces an earlier one for the same setting. Now shows the value the kernel uses at this moment."
        source="/etc/sysctl.d · /usr/lib/sysctl.d · /proc/sys · dpkg"
        span={12}
        flush
      >
        <DataTable
          rows={d.settings}
          columns={[
            { key: "key", label: "Setting", text: (s) => s.key, wrap: true },
            { key: "value", label: "File sets", text: (s) => s.value },
            { key: "now", label: "Now", text: (s) => s.now ?? "-", render: (s: Setting) => (s.now !== undefined && s.now !== s.value && !s.overridden ? <StatusBadge variant="info" label={`${s.now} (changed while running)`} /> : (s.now ?? "-")) },
            { key: "from", label: "From", text: (s) => s.package ?? "This computer", render: (s: Setting) => (s.package ? (s.file_changed ? <StatusBadge variant="info" label={`${s.package}, file changed here`} /> : s.package) : <StatusBadge variant="info" label="This computer" />) },
            { key: "state", label: "In effect", text: (s) => (s.overridden ? "No: a later file sets it" : "Yes") },
            { key: "file", label: "File", text: (s) => `${s.file}:${s.line}`, wrap: true, expert: true },
          ]}
          rowKey={(s) => `${s.file}|${s.line}`}
          pageSize={15}
          search={{ placeholder: "Filter settings" }}
        />
      </Panel>
      <Panel title="File watches by program" info="Editors, sync apps, and file indexers watch folders for changes. When all watches are used, they stop seeing changes." source="/proc/[pid]/fdinfo (inotify)" span={6} flush>
        <DataTable
          rows={d.watch_users}
          columns={[
            { key: "name", label: "Program", text: (u) => u.name },
            { key: "pid", label: "PID", text: (u) => String(u.pid), numeric: true },
            { key: "w", label: "Watches", text: (u) => u.watches.toLocaleString(), numeric: true },
          ]}
          rowKey={(u) => String(u.pid)}
          empty="None of your programs watches files"
        />
      </Panel>
      <Panel title="Time sync" source="timedatectl timesync-status" span={6} flush>
        {Object.keys(d.time_sync).length ? (
          <KeyValueList rows={["Server", "Offset", "Delay", "Jitter", "Poll interval", "Stratum", "Packet count"].filter((k) => d.time_sync[k]).map((k): [string, string] => [k, d.time_sync[k]])} />
        ) : (
          <div className="empty muted">systemd-timesyncd is not running (another time service may be in use).</div>
        )}
      </Panel>
      <Panel title="Loaded modules" subtitle={`${d.modules.length} loaded · ${tainted.length} from outside the kernel or unsigned`} info="Taint letters: O built outside the kernel, E not signed, P proprietary. Devices counts the hardware each module drives." source="/proc/modules · /sys/module" span={6} flush>
        <DataTable
          rows={[...d.modules].sort((a, b) => (b.taint ? 1 : 0) - (a.taint ? 1 : 0) || b.devices - a.devices)}
          columns={[
            { key: "name", label: "Module", text: (m) => m.name },
            { key: "devices", label: "Devices", text: (m) => String(m.devices), numeric: true },
            { key: "used", label: "Used by", text: (m) => m.used_by.join(", ") || "-", wrap: true },
            { key: "taint", label: "Taint", text: (m) => m.taint ?? "-", render: (m) => (m.taint ? <StatusBadge variant="warn" label={m.taint} /> : "-") },
            { key: "size", label: "Size", text: (m) => bytes(m.size, 0), numeric: true, expert: true },
          ]}
          rowKey={(m) => m.name}
          pageSize={12}
          search={{ placeholder: "Filter modules" }}
        />
      </Panel>
      <Panel title="Extra drivers (DKMS)" info="Drivers built on this computer, such as NVIDIA or VirtualBox. Each must be built for every installed kernel, or the device stops working after starting that kernel." source="dkms status · /lib/modules" span={6} flush>
        {!d.dkms ? (
          <div className="empty muted">DKMS is not installed, so there are no extra drivers to build.</div>
        ) : (
          <DataTable
            rows={d.dkms}
            columns={[
              { key: "module", label: "Driver", text: (m) => `${m.module} ${m.version}` },
              { key: "built", label: "Installed for", text: (m) => `${m.kernels.length} of ${d.kernels.length} kernels` },
              {
                key: "missing",
                label: "Missing for",
                text: (m) => missingKernels(d, m).join(", ") || "-",
                render: (m) => (missingKernels(d, m).length ? <StatusBadge variant="warn" label={missingKernels(d, m).join(", ")} /> : "-"),
              },
            ]}
            rowKey={(m) => `${m.module}|${m.version}`}
            empty="No DKMS drivers"
          />
        )}
      </Panel>
    </>
  );
}
