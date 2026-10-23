// SPDX-License-Identifier: Apache-2.0
// Expert sections for drives and filesystems (docs/01-product/expert-coverage.md, sections 3
// and 4): disk use per service and per process, each drive's settings, PCIe or SATA link, TRIM,
// and partitions, every filesystem's options, inodes, and recorded errors, and every PCIe link.

import { type Groups, type JobsInfo, type PciDevice, type ProcessesIo, type StorageDetail, getInventory } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { bytes, rate } from "../../metrics";
import { ago } from "../../time";
import { perSecond, useLive, useLiveWithPrevious } from "../../useLive";
import { NoData } from "../NoData";

const loadGroups = () => getInventory("cgroups");
const loadIo = () => getInventory("process_io");
const loadStorage = () => getInventory("storage_detail");
const loadPci = () => getInventory("pci_detail");
const loadJobs = () => getInventory("jobs");

const gts = (speed?: string) => (speed ? parseFloat(speed) : undefined);
const linkText = (speed?: string, width?: number) => (speed ? `${speed.replace(" PCIe", "")}${width ? ` × ${width}` : ""}` : "-");

/**
 * Whether a PCIe link runs as fast as both sides allow. The best possible link is the slower of
 * the device's maximum and its slot's maximum. Graphics cards and some others slow their link
 * down on purpose when idle, so for them a slower link is only a hint to check under load.
 */
export function linkVerdict(d: PciDevice): { variant: StatusVariant; text: string } {
  if (d.link_width === 0) return { variant: "off", text: "Nothing connected" };
  const now = gts(d.link_speed);
  const max = Math.min(gts(d.max_link_speed) ?? Infinity, gts(d.slot_max_link_speed) ?? Infinity);
  const lanes = Math.min(d.max_link_width ?? Infinity, d.slot_max_link_width ?? Infinity);
  if (now === undefined || !Number.isFinite(max)) return { variant: "off", text: "Not reported" };
  const slowSpeed = now < max;
  const fewLanes = d.link_width !== undefined && Number.isFinite(lanes) && d.link_width < lanes;
  if (!slowSpeed && !fewLanes) return { variant: "ok", text: "Full speed" };
  const what = [slowSpeed && `${now} of ${max} GT/s`, fewLanes && `${d.link_width} of ${lanes} lanes`].filter(Boolean).join(", ");
  if (d.class === "Display" || d.power_state === "suspended") return { variant: "info", text: `${what}: it may slow down when idle; check under load` };
  return { variant: "warn", text: `Slower than both sides support: ${what}` };
}

// ---------- Performance › Disk I/O ----------

export function DiskIoExpertSections() {
  const groups = useLiveWithPrevious<Groups>(loadGroups, 10_000);
  const io = useLiveWithPrevious<ProcessesIo>(loadIo, 10_000);
  const gSeconds = groups.data && groups.before ? (groups.data.at - groups.before.at) / 1000 : 0;
  const services = (groups.data?.groups ?? [])
    .filter((g) => g.kind === "service" && ((g.disk_read ?? 0) > 0 || (g.disk_write ?? 0) > 0))
    .map((g) => {
      const b = groups.before?.groups.find((x) => x.unit === g.unit);
      return { ...g, readNow: perSecond(g.disk_read, b?.disk_read, gSeconds), writeNow: perSecond(g.disk_write, b?.disk_write, gSeconds) };
    })
    .sort((a, b) => (groups.before ? (b.readNow ?? 0) + (b.writeNow ?? 0) - (a.readNow ?? 0) - (a.writeNow ?? 0) : (b.disk_write ?? 0) + (b.disk_read ?? 0) - (a.disk_write ?? 0) - (a.disk_read ?? 0)));
  const pSeconds = io.data && io.before ? (io.data.at - io.before.at) / 1000 : 0;
  const processes = (io.data?.processes ?? [])
    .map((p) => {
      const b = io.before?.processes.find((x) => x.pid === p.pid);
      return { ...p, readNow: perSecond(p.read_bytes, b?.read_bytes, pSeconds), writeNow: perSecond(p.write_bytes, b?.write_bytes, pSeconds) };
    })
    .sort((a, b) => (io.before ? (b.readNow ?? 0) + (b.writeNow ?? 0) - (a.readNow ?? 0) - (a.writeNow ?? 0) : 0));
  const perS = (v?: number) => (v === undefined ? "-" : rate(v));
  return (
    <>
      <Panel title="Disk use per service" subtitle={groups.before ? "now, busiest first" : "since start (rates after the next reading)"} source="/sys/fs/cgroup/system.slice/*/io.stat" span={6} flush>
        {groups.data ? (
          <DataTable
            rows={services}
            columns={[
              { key: "name", label: "Service", text: (g) => g.name },
              { key: "r", label: "Read/s", text: (g) => perS(g.readNow), numeric: true },
              { key: "w", label: "Write/s", text: (g) => perS(g.writeNow), numeric: true },
              { key: "rt", label: "Read since start", text: (g) => bytes(g.disk_read ?? 0), numeric: true },
              { key: "wt", label: "Written since start", text: (g) => bytes(g.disk_write ?? 0), numeric: true },
            ]}
            rowKey={(g) => g.unit}
            pageSize={10}
            empty="No service used the disk"
          />
        ) : (
          <NoData error={groups.error} waiting="Loading…" />
        )}
      </Panel>
      <Panel title="Disk use per process" subtitle={io.before ? "now, busiest first" : "since each started (rates after the next reading)"} info="Bytes really read from and written to storage, not served from the cache." source="/proc/[pid]/io" span={6} flush>
        {io.data ? (
          <>
            <DataTable
              rows={processes}
              columns={[
                { key: "name", label: "Process", text: (p) => p.name },
                { key: "pid", label: "PID", text: (p) => String(p.pid), numeric: true },
                { key: "r", label: "Read/s", text: (p) => perS(p.readNow), numeric: true },
                { key: "w", label: "Write/s", text: (p) => perS(p.writeNow), numeric: true },
                { key: "wt", label: "Written since start", text: (p) => bytes(p.write_bytes), numeric: true },
              ]}
              rowKey={(p) => String(p.pid)}
              pageSize={10}
              empty="No process of yours used the disk"
            />
            {io.data.unreadable > 0 && <p className="muted panel-pad">{io.data.unreadable} processes of other users are not counted: that needs Full access.</p>}
          </>
        ) : (
          <NoData error={io.error} waiting="Loading…" />
        )}
      </Panel>
    </>
  );
}

// ---------- Hardware › Storage Devices ----------

export function DriveExpertSections() {
  const storage = useLive(loadStorage, 60_000);
  const pci = useLive(loadPci, 60_000);
  const jobs = useLive<JobsInfo>(loadJobs, 300_000);
  const d = storage.data;
  if (!d) {
    return (
      <Panel title="Drive details" span={12}>
        <NoData error={storage.error} waiting="Loading…" />
      </Panel>
    );
  }
  const trim = jobs.data?.timers.find((t) => t.name.startsWith("fstrim"));
  const discard = d.filesystems.filter((f) => f.options.includes("discard")).map((f) => f.mount);
  const anySsd = d.disks.some((k) => !k.rotational);
  return (
    <>
      {d.disks.map((k) => {
        const link = k.nvme?.pci_address ? pci.data?.find((p) => p.address === k.nvme?.pci_address) : undefined;
        const verdict = link && linkVerdict(link);
        return (
          <Panel key={k.name} title={`Drive details · ${k.name}`} source="/sys/block · /sys/class/nvme · /sys/class/ata_link" span={6} flush>
            <KeyValueList
              rows={[
                ["Kind", k.rotational ? "Spinning disk" : "Solid state"],
                ["Scheduler", k.scheduler ? `${k.scheduler}${k.schedulers.length > 1 ? ` (offered: ${k.schedulers.join(", ")})` : ""}` : "-"],
                ["Queue size · read-ahead", `${k.queue_size ?? "-"} · ${k.read_ahead_kb === undefined ? "-" : `${k.read_ahead_kb} KB`}`],
                ["Block size (logical · physical)", `${k.logical_block ?? "-"} · ${k.physical_block ?? "-"} bytes`],
                ["Accepts TRIM", k.trim ? "Yes" : "No"],
                ["Partition table", k.table ? k.table.toUpperCase().replace("DOS", "MBR (dos)") : "-"],
                ...(k.sata_speed ? [["SATA link", `${k.sata_speed}${k.sata_limit ? ` (limit ${k.sata_limit})` : ""}`] as [string, string]] : []),
                ...(k.nvme
                  ? ([
                      ["NVMe firmware", k.nvme.firmware ?? "-"],
                      ["Drive's own temperature limits", k.nvme.temp_warning || k.nvme.temp_critical ? `warning ${k.nvme.temp_warning?.toFixed(0) ?? "-"} °C · critical ${k.nvme.temp_critical?.toFixed(0) ?? "-"} °C` : "-"],
                    ] as [string, string][])
                  : []),
                ...(link
                  ? ([
                      [
                        "PCIe link",
                        <span key="l">
                          {linkText(link.link_speed, link.link_width)} · {verdict && <StatusBadge variant={verdict.variant} label={verdict.text} />}
                        </span>,
                      ],
                      ["PCIe power saving", link.power_control === "auto" ? "On" : link.power_control === "on" ? "Off (always awake)" : "-"],
                    ] as [string, React.ReactNode][])
                  : []),
              ]}
            />
            {k.partitions.length > 0 && (
              <DataTable
                rows={k.partitions}
                columns={[
                  { key: "name", label: "Partition", text: (p) => p.name },
                  { key: "size", label: "Size", text: (p) => bytes(p.size, 0), numeric: true },
                  { key: "start", label: "Starts at sector", text: (p) => p.start_sector.toLocaleString(), numeric: true },
                  { key: "aligned", label: "Aligned", text: (p) => (p.aligned ? "Yes" : "No"), render: (p) => (p.aligned ? "Yes" : <StatusBadge variant="warn" label="No: slower on an SSD" />) },
                ]}
                rowKey={(p) => p.name}
              />
            )}
          </Panel>
        );
      })}
      {anySsd && (
        <Panel title="TRIM" info="TRIM tells an SSD which blocks are free, so it stays fast. Ubuntu does it once a week with fstrim.timer." source="fstrim.timer · /proc/mounts" span={6} flush>
          <KeyValueList
            rows={[
              ["Weekly TRIM (fstrim.timer)", !jobs.data ? "Checking…" : !trim ? <StatusBadge key="t" variant="warn" label="Not set up" /> : trim.next ? "On" : <StatusBadge key="t" variant="warn" label="Off" />],
              ["Last run", trim?.last ? `${ago(trim.last, Date.now())}${trim.result && trim.result !== "success" ? ` · ${trim.result}` : ""}` : "-"],
              ["Next run", trim?.next ? new Date(trim.next).toLocaleString() : "-"],
              ["Trimmed on every delete (discard option)", discard.length ? discard.join(", ") : "No filesystem"],
              ["NVMe power saving", d.nvme_max_latency_us === undefined ? "-" : d.nvme_max_latency_us === 0 ? "Off" : `On (wake-up up to ${d.nvme_max_latency_us / 1000} ms)`],
            ]}
          />
        </Panel>
      )}
    </>
  );
}

// ---------- Storage › Filesystems ----------

type Fs = StorageDetail["filesystems"][number];
const FS_COLUMNS: Column<Fs>[] = [
  { key: "mount", label: "Mounted at", text: (f) => f.mount },
  { key: "type", label: "Type", text: (f) => f.fstype },
  { key: "options", label: "Options", text: (f) => f.options.join(", "), wrap: true },
  {
    key: "inodes",
    label: "Inodes used",
    text: (f) => (f.inodes && f.inodes_free !== undefined ? `${(((f.inodes - f.inodes_free) / f.inodes) * 100).toFixed(1)}% of ${f.inodes.toLocaleString()}` : "-"),
    numeric: true,
  },
  {
    key: "errors",
    label: "Errors recorded",
    text: (f) => (f.errors === undefined ? "-" : String(f.errors)),
    render: (f) => (f.errors ? <StatusBadge variant="warn" label={`${f.errors}${f.last_error_at ? `, last ${new Date(f.last_error_at * 1000).toLocaleDateString()}` : ""}`} /> : f.errors === undefined ? "-" : "0"),
    numeric: true,
  },
];

export function FilesystemExpert() {
  const { data, error } = useLive(loadStorage, 60_000);
  return (
    <Panel
      title="Filesystem details"
      info="Inodes are the file slots of a filesystem: it can be full with free space left when they run out. ext4 counts the errors it found on disk."
      source="/proc/mounts · statvfs · /sys/fs/ext4"
      span={12}
      flush
    >
      {data ? <DataTable rows={data.filesystems} columns={FS_COLUMNS} rowKey={(f) => f.mount} empty="No local filesystems" /> : <NoData error={error} waiting="Loading…" />}
    </Panel>
  );
}

// ---------- Hardware › Peripherals ----------

export function PcieLinks() {
  const { data, error } = useLive(loadPci, 60_000);
  const links = (data ?? []).filter((d) => d.link_speed || d.link_width !== undefined);
  return (
    <Panel
      title="PCIe links"
      info="The link can only be as fast as the slower of the device and its slot. Link errors are counted by the hardware since start."
      source="/sys/bus/pci/devices"
      span={12}
      flush
    >
      {data ? (
        <DataTable
          rows={links}
          columns={[
            { key: "device", label: "Device", text: (d) => d.model ?? d.address, wrap: true },
            { key: "class", label: "Kind", text: (d) => d.class ?? "-" },
            { key: "now", label: "Now", text: (d) => linkText(d.link_speed, d.link_width) },
            { key: "max", label: "Device max", text: (d) => linkText(d.max_link_speed, d.max_link_width) },
            { key: "slot", label: "Slot max", text: (d) => linkText(d.slot_max_link_speed, d.slot_max_link_width) },
            { key: "verdict", label: "Speed", text: (d) => linkVerdict(d).text, render: (d) => <StatusBadge variant={linkVerdict(d).variant} label={linkVerdict(d).text} /> },
            {
              key: "errors",
              label: "Link errors",
              text: (d) => (d.errors ? `${d.errors.corrected + d.errors.nonfatal + d.errors.fatal}` : "-"),
              render: (d) =>
                !d.errors ? "-" : d.errors.fatal || d.errors.nonfatal ? <StatusBadge variant="crit" label={`${d.errors.fatal} fatal, ${d.errors.nonfatal} not fixed`} /> : d.errors.corrected ? <StatusBadge variant="warn" label={`${d.errors.corrected} corrected`} /> : "0",
            },
            { key: "address", label: "Address", text: (d) => d.address, expert: true },
          ]}
          rowKey={(d) => d.address}
          pageSize={30}
          empty="No PCIe links reported"
        />
      ) : (
        <NoData error={error} waiting="Loading…" />
      )}
    </Panel>
  );
}
