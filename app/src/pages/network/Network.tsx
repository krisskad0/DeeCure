// SPDX-License-Identifier: Apache-2.0
// Network › Interfaces, Connections, Open Ports, Firewall, DNS & VPN, Traffic
// (docs/03-ui-design/05-pages/16-network.md), with real data.

import { Guard } from "../../components/ErrorBoundary";
import { atLeast, useUi } from "../../ui";
import { DnsRoutesExpert, InterfacesExpert, TrafficExpert } from "./NetworkExpert";
import { type SocketInfo, getInventory } from "../../api";
import { AlertBanner } from "../../components/AlertBanner";
import { type Column, DataTable } from "../../components/DataTable";
import { InlineMeter } from "../../components/InlineMeter";
import { KeyValueList } from "../../components/KeyValueList";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { bytes } from "../../metrics";
import { useLive } from "../../useLive";
import { kbPort, useKb } from "../../kb";
import { Button } from "../../components/Button";
import { useFix } from "../../shell/FixDrawer";
import { AdminRead } from "../../components/AdminRead";
import { LogLine } from "../../components/CodeBlock";
import { MB, TimeChart, useHistory } from "../performance/shared";
import { NoData } from "../NoData";

const loadDetails = () => getInventory("net_details");
const loadInterfaces = () => getInventory("network");
const loadSockets = () => getInventory("sockets");

function Waiting({ title, error }: { title: string; error: Parameters<typeof NoData>[0]["error"] }) {
  return (
    <Panel title={title} span={12}>
      <NoData error={error} waiting="Loading…" />
    </Panel>
  );
}

interface Row {
  name: string;
  kind: string;
  state: string;
  ipv4: string;
  mac?: string;
  speed?: number;
  rx: number;
  tx: number;
}

const INTERFACES: Column<Row>[] = [
  { key: "name", label: "Interface", text: (r) => r.name, render: (r) => <code>{r.name}</code> },
  { key: "kind", label: "Type", text: (r) => r.kind },
  { key: "state", label: "State", text: (r) => r.state, render: (r) => <StatusBadge variant={r.state === "up" ? "ok" : "off"} label={r.state === "up" ? "connected" : r.state} /> },
  { key: "ipv4", label: "IPv4", text: (r) => r.ipv4 || "-", render: (r) => (r.ipv4 ? <code>{r.ipv4}</code> : "-") },
  { key: "mac", label: "MAC", text: (r) => r.mac ?? "-", expert: true },
  { key: "speed", label: "Speed", text: (r) => (r.speed ? `${r.speed} Mb/s` : "-"), numeric: true },
  { key: "rx", label: "RX", text: (r) => bytes(r.rx), numeric: true },
  { key: "tx", label: "TX", text: (r) => bytes(r.tx), numeric: true },
];

const CONNECTIVITY: Record<string, [StatusVariant, string]> = { full: ["ok", "Online"], limited: ["warn", "Limited"], portal: ["warn", "Sign-in page (captive portal)"], none: ["crit", "Offline"], unknown: ["off", "Unknown"] };

export function NetworkInterfaces() {
  const { mode } = useUi();
  const { data: d, error } = useLive(loadDetails, 15_000);
  const ifaces = useLive(loadInterfaces, 15_000).data;
  if (!d || !ifaces) return <Waiting title="Interfaces" error={error} />;
  const rows: Row[] = ifaces.interfaces.map((i) => {
    const link = d.links.find((l) => l.name === i.name);
    return { name: i.name, kind: i.kind, state: i.state, ipv4: link?.ipv4.join(", ") ?? "", mac: i.mac, speed: i.speed_mbps ?? (d.wifi?.interface === i.name ? d.wifi.bitrate_mbps : undefined), rx: link?.rx ?? 0, tx: link?.tx ?? 0 };
  });
  rows.sort((a, b) => Number(b.state === "up") - Number(a.state === "up") || b.rx - a.rx);
  const w = d.wifi;
  const conn = d.connectivity ? CONNECTIVITY[d.connectivity] : null;
  return (
    <>
      <Panel title="Interfaces" source="ip addr · /sys/class/net" span={12} flush>
        <DataTable rows={rows} columns={INTERFACES} rowKey={(r) => r.name} />
      </Panel>
      <Panel title="Wi-Fi" source="NetworkManager · /proc/net/wireless" span={6} flush>
        {w ? (
          <KeyValueList
            rows={[
              ["Network", w.ssid ?? "-"],
              ["Signal", w.strength === undefined ? "-" : <InlineMeter key="s" percent={w.strength} label={w.dbm === undefined ? `${w.strength}%` : `${w.dbm} dBm`} tone={w.strength >= 50 ? "ok" : w.strength >= 30 ? "warn" : "crit"} />],
              ["Band", w.frequency_mhz ? `${(w.frequency_mhz / 1000).toFixed(1)} GHz (channel frequency ${w.frequency_mhz} MHz)` : "-"],
              ["Link speed", w.bitrate_mbps ? `${w.bitrate_mbps} Mb/s` : "-"],
              ["Interface", <code key="i">{w.interface}</code>],
            ]}
          />
        ) : (
          <div className="empty muted">Not connected to Wi-Fi.</div>
        )}
      </Panel>
      <Panel title="Connectivity" span={6} flush>
        <KeyValueList
          rows={[
            ["Internet", conn ? <StatusBadge key="c" variant={conn[0]} label={conn[1]} /> : "Unknown (NetworkManager not running)"],
            ["Gateway", d.gateway ? <code key="g">{d.gateway}</code> : "None"],
            ["DNS", d.dns_servers.length ? <code key="d">{d.dns_servers.join(", ")}</code> : "-"],
            ["Firewall", d.firewall.installed ? (d.firewall.enabled ? "On (ufw)" : <StatusBadge key="f" variant="warn" label="Off" />) : "Not installed"],
          ]}
        />
      </Panel>
      {atLeast(mode, "expert") && <Guard><InterfacesExpert /></Guard>}
    </>
  );
}

const STATE: Record<string, StatusVariant> = { ESTAB: "ok", LISTEN: "info", UNCONN: "off" };

const SOCKETS: Column<SocketInfo>[] = [
  { key: "proto", label: "Proto", text: (s) => s.proto },
  { key: "local", label: "Local", text: (s) => s.local, render: (s) => <code>{s.local}</code> },
  { key: "remote", label: "Remote", text: (s) => s.remote, render: (s) => <code>{s.remote}</code> },
  { key: "state", label: "State", text: (s) => s.state, render: (s) => <StatusBadge variant={STATE[s.state] ?? "off"} label={s.state} /> },
  { key: "process", label: "Process", text: (s) => s.process ?? "-", render: (s) => s.process ?? <span className="muted">other user</span> },
];

export function NetworkConnections() {
  const { data: sockets, error } = useLive(loadSockets, 10_000);
  if (!sockets) return <Waiting title="Active connections" error={error} />;
  return (
    <Panel title="Active connections" source="/proc/net/{tcp,udp}" span={12} flush>
      <DataTable rows={sockets} columns={SOCKETS} rowKey={(s) => `${s.proto}|${s.local}|${s.remote}|${s.state}`} search={{ placeholder: "Filter connections" }} filter={{ column: "state", options: ["ESTAB", "LISTEN", "UNCONN"] }} pageSize={12} total={sockets.length} />
    </Panel>
  );
}

const EXPOSURE: Record<SocketInfo["exposure"], [StatusVariant, string]> = { local: ["ok", "This computer only"], all: ["warn", "Every network"], network: ["info", "One network"] };

export function NetworkPorts() {
  const { data: sockets, error } = useLive(loadSockets, 15_000);
  const details = useLive(loadDetails, 60_000).data;
  const kb = useKb();
  if (!sockets) return <Waiting title="Listening ports" error={error} />;
  // A port shows up once per address family; one row per port, protocol, and exposure.
  const seen = new Set<string>();
  const ports = sockets.filter((s) => s.state === "LISTEN" || (s.proto === "udp" && s.state === "UNCONN" && s.remote === "*")).filter((s) => {
    const key = `${s.proto}|${s.local_port}|${s.exposure}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
  const fw = details?.firewall;
  const exposed = ports.filter((p) => p.exposure !== "local");
  const columns: Column<SocketInfo>[] = [
    { key: "port", label: "Port", text: (s) => String(s.local_port), render: (s) => <code>{s.local_port}</code>, numeric: true },
    { key: "proto", label: "Protocol", text: (s) => s.proto.toUpperCase() },
    {
      key: "what",
      label: "What it is",
      text: (s) => kbPort(kb, s.local_port, s.proto)?.name ?? "",
      render: (s) => {
        const k = kbPort(kb, s.local_port, s.proto);
        return k ? <span className="cell-two"><b>{k.name}</b><span>{k.plain}</span></span> : <span className="muted">-</span>;
      },
      wrap: true,
    },
    { key: "bind", label: "Bind address", text: (s) => s.local.slice(0, s.local.lastIndexOf(":")), render: (s) => <code>{s.local.slice(0, s.local.lastIndexOf(":"))}</code> },
    { key: "exposure", label: "Exposure", text: (s) => EXPOSURE[s.exposure][1], render: (s) => <StatusBadge variant={EXPOSURE[s.exposure][0]} label={EXPOSURE[s.exposure][1]} /> },
    { key: "process", label: "Process", text: (s) => s.process ?? "-", render: (s) => s.process ?? <span className="muted">system (needs Full access)</span> },
    {
      key: "firewall",
      label: "Firewall",
      text: (s) => (s.exposure === "local" ? "Not needed" : fw?.enabled ? "Filtered" : "Not filtered"),
      render: (s) => (s.exposure === "local" ? <span className="muted">Not needed</span> : fw?.enabled ? <StatusBadge variant="ok" label="Filtered" /> : <StatusBadge variant="warn" label="Not filtered" />),
    },
  ];
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Listening", value: String(ports.length) },
          { label: "This computer only", value: String(ports.length - exposed.length) },
          { label: "Reachable from networks", value: String(exposed.length), status: exposed.length && !fw?.enabled ? "warn" : "ok" },
        ]}
      />
      <Panel title="Listening ports" source="/proc/net/{tcp,udp}" span={12} flush>
        <DataTable rows={ports} columns={columns} rowKey={(s) => `${s.proto}|${s.local_port}|${s.exposure}`} empty="Nothing is listening" />
      </Panel>
      <AlertBanner tone="info" title="Reachability from the internet is not tested" text="Whether your router forwards a port can only be checked from outside your network." />
    </>
  );
}

export function NetworkFirewall() {
  const { data: d, error } = useLive(loadDetails, 30_000);
  const openFix = useFix();
  if (!d) return <Waiting title="Firewall" error={error} />;
  const fw = d.firewall;
  const policy = (p?: string) => (p === "DROP" || p === "REJECT" ? "Deny" : p === "ACCEPT" ? "Allow" : (p ?? "-"));
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Status", value: !fw.installed ? "Not installed" : fw.enabled ? "Active" : "Off", status: fw.installed && !fw.enabled ? "warn" : "ok", subtext: "ufw" },
          { label: "Incoming", value: fw.enabled ? policy(fw.incoming) : "Allow", subtext: fw.enabled ? "default" : "firewall off" },
          { label: "Outgoing", value: fw.enabled ? policy(fw.outgoing) : "Allow", subtext: "default" },
          { label: "Rules", value: fw.rules ? String(fw.rules.length) : "-", subtext: fw.rules ? "" : "need Full access to read" },
        ]}
      />
      <Panel title="Rules" source="/etc/ufw/user.rules" span={12}>
        {fw.rules ? (
          fw.rules.length ? <ul className="plain-list">{fw.rules.map((r) => <li key={r}><code>{r}</code></li>)}</ul> : <div className="empty muted">No rules.</div>
        ) : (
          <AdminRead what="firewall_rules" label="Show the rules (asks for your password)">
            {(text) => <LogLine>{text}</LogLine>}
          </AdminRead>
        )}
      </Panel>
      {fw.installed && !fw.enabled && (
        <AlertBanner
          tone="warn"
          title="The firewall is off"
          text="Any program that listens on every network can be reached by other computers on the same network, such as public Wi-Fi."
          actions={<Button small variant="primary" onClick={() => openFix("security.enable_firewall")}>Turn on…</Button>}
        />
      )}
    </>
  );
}

export function NetworkDns() {
  const { mode } = useUi();
  const { data: d, error } = useLive(loadDetails, 30_000);
  if (!d) return <Waiting title="DNS" error={error} />;
  return (
    <>
      <Panel title="DNS" source="/etc/resolv.conf · systemd-resolved" span={6} flush>
        <KeyValueList
          rows={[
            ["Resolver", d.resolver],
            ["Servers", d.dns_servers.length ? <code key="s">{d.dns_servers.join(", ")}</code> : "-"],
          ]}
        />
      </Panel>
      <Panel title="VPN connections" source="NetworkManager" span={6} flush>
        <DataTable
          rows={d.vpns}
          columns={[
            { key: "name", label: "Name", text: (v) => v.name },
            { key: "kind", label: "Type", text: (v) => (v.kind === "wireguard" ? "WireGuard" : v.kind === "vpn" ? "VPN" : v.kind) },
            { key: "state", label: "State", text: (v) => (v.connected ? "Connected" : "Disconnected"), render: (v) => <StatusBadge variant={v.connected ? "ok" : "off"} label={v.connected ? "Connected" : "Disconnected"} /> },
          ]}
          rowKey={(v) => v.name}
          empty="No VPN connections are set up"
        />
      </Panel>
      {atLeast(mode, "expert") && <Guard><DnsRoutesExpert /></Guard>}
    </>
  );
}

export function NetworkTraffic() {
  const { mode } = useUi();
  const history = useHistory(["net.rx", "net.tx"]);
  return (
    <>
      <Panel title="Throughput" source="/proc/net/dev" span={12}>
        <TimeChart history={history} names={["Download", "Upload"]} unit="MB/s" scale={(v) => v / MB} />
      </Panel>
      {atLeast(mode, "expert") && <Guard><TrafficExpert /></Guard>}
    </>
  );
}

