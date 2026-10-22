// SPDX-License-Identifier: Apache-2.0
// Containers › Containers, Images, Volumes, Virtual Machines
// (docs/03-ui-design/05-pages/12-containers.md), with real data. Read-only.

import { type ContainersInfo, getInventory } from "../../api";
import { AlertBanner } from "../../components/AlertBanner";
import { type Column, DataTable } from "../../components/DataTable";
import { Icon } from "../../components/Icon";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { Donut } from "../../components/charts/Donut";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

type Container = ContainersInfo["containers"][number];
type Image = ContainersInfo["images"][number];
type Volume = ContainersInfo["volumes"][number];
type Vm = ContainersInfo["vms"][number];

const loadContainers = () => getInventory("containers");
const useContainers = () => useLive(loadContainers, 30_000);

const DOCKER_STATE: Record<ContainersInfo["docker"], string> = {
  running: "",
  "not running": "Docker is installed but its service is not running. DeeCure does not start it.",
  "no access": "Docker runs, but this user may not ask it (not in the docker group).",
  "not installed": "Docker is not installed.",
};

/** "3.77GB" → bytes (Docker's decimal units). */
export function dockerBytes(text: string): number {
  const m = /^([\d.]+)\s*([kMGT]?B)$/.exec(text.trim());
  if (!m) return 0;
  const unit = { B: 1, kB: 1e3, MB: 1e6, GB: 1e9, TB: 1e12 }[m[2]] ?? 1;
  return Number(m[1]) * unit;
}

/** Ports published on every network bypass ufw: Docker writes its own firewall rules. */
const public_ = (c: Container) => c.ports.filter((p) => p.startsWith("0.0.0.0:") || p.startsWith(":::"));

const CONTAINERS: Column<Container>[] = [
  { key: "name", label: "Name", text: (c) => c.name },
  { key: "image", label: "Image", text: (c) => c.image, render: (c) => <code>{c.image}</code> },
  { key: "status", label: "Status", text: (c) => c.status, render: (c) => <StatusBadge variant={c.state === "running" ? (c.status.includes("unhealthy") ? "warn" : "ok") : c.state === "restarting" ? "crit" : "off"} label={c.status} /> },
  { key: "ports", label: "Ports", text: (c) => c.ports.join(" "), render: (c) => (c.ports.length ? <span className="cell-two">{c.ports.filter((p) => !p.startsWith(":::")).map((p) => <code key={p}>{p.replace("->", "→")}</code>)}{public_(c).length > 0 && <StatusBadge variant="warn" label="Every network" />}</span> : "—") },
  { key: "cpu", label: "CPU", text: (c) => c.cpu ?? "—", numeric: true },
  { key: "memory", label: "Memory", text: (c) => c.memory ?? "—", numeric: true },
];

export function ContainersList() {
  const { data: info, error } = useContainers();
  if (!info) return <Panel title="Docker containers" span={12}><NoData error={error} waiting="Asking Docker…" /></Panel>;
  if (info.docker !== "running") return <Panel title="Docker containers" span={12}><div className="empty muted">{DOCKER_STATE[info.docker]}</div></Panel>;
  const running = info.containers.filter((c) => c.state === "running");
  const exposed = running.filter((c) => public_(c).length);
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Running", value: String(running.length) },
          { label: "Stopped", value: String(info.containers.length - running.length) },
          { label: "Unhealthy", value: String(running.filter((c) => c.status.includes("unhealthy")).length), status: running.some((c) => c.status.includes("unhealthy")) ? "warn" : "ok" },
          { label: "Images", value: String(info.images.length) },
        ]}
      />
      <Panel title="Docker containers" source="docker ps -a · docker stats" span={12} flush>
        <DataTable rows={info.containers} columns={CONTAINERS} rowKey={(c) => c.name} filter={{ column: "status", options: ["Up", "Exited"] }} pageSize={12} />
      </Panel>
      {exposed.map((c) => (
        <AlertBanner key={c.name} tone="warn" title={`${c.name} is reachable from every network`} text="Docker publishes ports with its own firewall rules, so ufw does not block them. Bind to 127.0.0.1 to keep it on this computer." />
      ))}
    </>
  );
}

const IMAGES: Column<Image>[] = [
  { key: "repo", label: "Repository", text: (i) => i.repository, render: (i) => <code>{i.repository}</code> },
  { key: "tag", label: "Tag", text: (i) => i.tag },
  { key: "size", label: "Size", text: (i) => i.size, numeric: true },
  { key: "created", label: "Created", text: (i) => i.created },
  { key: "used", label: "In use", text: (i) => (i.containers ? "Yes" : "No"), render: (i) => (i.containers ? <Icon name="check" size={16} /> : <span className="muted">—</span>) },
];

export function ContainersImages() {
  const { data: info, error } = useContainers();
  if (!info) return <Panel title="Images" span={12}><NoData error={error} waiting="Asking Docker…" /></Panel>;
  if (info.docker !== "running") return <Panel title="Images" span={12}><div className="empty muted">{DOCKER_STATE[info.docker]}</div></Panel>;
  const used = info.images.filter((i) => i.containers).reduce((s, i) => s + dockerBytes(i.size), 0);
  const unused = info.images.filter((i) => !i.containers).reduce((s, i) => s + dockerBytes(i.size), 0);
  return (
    <>
      <Panel title="Image storage" span={4}>
        <Donut segments={[{ name: "In use", value: used, slot: 1, display: `${(used / 1e9).toFixed(1)} GB` }, { name: "Unused", value: unused, tone: "warn", display: `${(unused / 1e9).toFixed(1)} GB` }]} center={{ value: ((used + unused) / 1e9).toFixed(1), subtitle: "GB" }} />
        <p className="muted">Sizes count shared layers in each image, so the total can be more than the space used.</p>
      </Panel>
      <Panel title="Images" source="docker system df -v" span={8} flush>
        <DataTable rows={info.images} columns={IMAGES} rowKey={(i) => `${i.repository}:${i.tag}`} empty="No images" />
      </Panel>
    </>
  );
}

const VOLUMES: Column<Volume>[] = [
  { key: "name", label: "Name", text: (v) => v.name, render: (v) => <code>{v.name.length > 30 ? `${v.name.slice(0, 12)}…` : v.name}</code> },
  { key: "size", label: "Size", text: (v) => v.size, numeric: true },
  { key: "links", label: "Used by", text: (v) => String(v.links), render: (v) => (v.links ? `${v.links} container${v.links === 1 ? "" : "s"}` : <StatusBadge variant="off" label="Unused" />) },
];

export function ContainersVolumes() {
  const { data: info, error } = useContainers();
  if (!info) return <Panel title="Volumes" span={12}><NoData error={error} waiting="Asking Docker…" /></Panel>;
  if (info.docker !== "running") return <Panel title="Volumes" span={12}><div className="empty muted">{DOCKER_STATE[info.docker]}</div></Panel>;
  return (
    <>
      <Panel title="Volumes" source="docker system df -v" span={12} flush>
        <DataTable rows={info.volumes} columns={VOLUMES} rowKey={(v) => v.name} pageSize={14} empty="No volumes" />
      </Panel>
      <AlertBanner tone="info" title="Volumes hold data" text="Unused volumes may still hold databases or files you need. DeeCure never offers to delete them." />
    </>
  );
}

const VMS: Column<Vm>[] = [
  { key: "name", label: "Name", text: (v) => v.name },
  { key: "engine", label: "Engine", text: (v) => v.engine },
  { key: "state", label: "State", text: (v) => (v.running ? "Running" : "Powered off"), render: (v) => <StatusBadge variant={v.running ? "ok" : "off"} label={v.running ? "Running" : "Powered off"} /> },
];

export function ContainersVms() {
  const { data: info, error } = useContainers();
  if (!info) return <Panel title="Virtual machines" span={12}><NoData error={error} waiting="Loading…" /></Panel>;
  return (
    <Panel title="Virtual machines" source="virsh · VBoxManage" span={12} flush>
      <DataTable rows={info.vms} columns={VMS} rowKey={(v) => `${v.engine}|${v.name}`} empty="No virtual machines (libvirt and VirtualBox were checked)" />
    </Panel>
  );
}
