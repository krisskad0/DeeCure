// SPDX-License-Identifier: Apache-2.0
// Expert sections for the network (docs/01-product/expert-coverage.md, section 6): the Wi-Fi link
// in detail, errors and drops per interface, TCP and UDP health, DNS per link, routes, and TCP
// traffic per app (TCP only, said plainly).

import { type NetDeep, type TcpTraffic, getInventory } from "../../api";
import { DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { bytes, rate } from "../../metrics";
import { perSecond, useLiveWithPrevious } from "../../useLive";
import { NoData } from "../NoData";

const loadDeep = () => getInventory("net_deep");
const loadTcp = () => getInventory("tcp_traffic");

/**
 * A plain verdict for a Wi-Fi signal. −67 dBm is the usual minimum for smooth video calls;
 * below −75 dBm the link is weak and slows down.
 */
export function signalVerdict(dbm?: number): { variant: StatusVariant; text: string } | undefined {
  if (dbm === undefined) return undefined;
  if (dbm >= -67) return { variant: "ok", text: "Good" };
  if (dbm >= -75) return { variant: "warn", text: "Fair: calls may stutter" };
  return { variant: "crit", text: "Weak" };
}

const count = (n: number) => n.toLocaleString();

function useDeep() {
  return useLiveWithPrevious<NetDeep>(loadDeep, 10_000);
}

// ---------- Network › Interfaces ----------

export function InterfacesExpert() {
  const { data: d, before, error } = useDeep();
  if (!d) {
    return (
      <Panel title="Network details" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const seconds = before ? (d.at - before.at) / 1000 : 0;
  const c = (k: string) => d.counters[k];
  const delta = (k: string) => (before && d.counters[k] !== undefined && before.counters[k] !== undefined ? d.counters[k] - before.counters[k] : undefined);
  const resentNow = delta("Tcp.RetransSegs");
  const outNow = delta("Tcp.OutSegs");
  const resentShare = resentNow !== undefined && outNow ? (resentNow / outNow) * 100 : undefined;
  const allShare = c("Tcp.OutSegs") ? ((c("Tcp.RetransSegs") ?? 0) / c("Tcp.OutSegs")) * 100 : undefined;
  const ifaces = d.interfaces.map((i) => {
    const b = before?.interfaces.find((x) => x.name === i.name);
    const dropsNow = perSecond(i.rx_dropped + i.tx_dropped, b ? b.rx_dropped + b.tx_dropped : undefined, seconds);
    return { ...i, dropsNow };
  });
  return (
    <>
      {d.wifi.map((w) => {
        const verdict = signalVerdict(w.signal_dbm);
        return (
          <Panel key={w.interface} title={`Wi-Fi link · ${w.ssid}`} info="−67 dBm or better is enough for smooth video calls. Below −75 dBm the link is weak and slows down." source="NetworkManager · /proc/net/wireless" span={6} flush>
            <KeyValueList
              rows={[
                ["Signal", w.signal_dbm !== undefined ? <span key="s">{w.signal_dbm} dBm {verdict && <StatusBadge variant={verdict.variant} label={verdict.text} />}</span> : w.signal_percent !== undefined ? `${w.signal_percent}%` : "-"],
                ["Noise", w.noise_dbm !== undefined ? `${w.noise_dbm} dBm` : "Not reported by this driver"],
                ["Band · channel", `${w.band ?? "-"} · ${w.channel ?? "-"}${w.frequency_mhz ? ` (${w.frequency_mhz} MHz)` : ""}`],
                ["Network's top rate", w.rate_mbps ? `${w.rate_mbps} Mbit/s (from the last scan; the link speed now is above)` : "-"],
                ["Security", w.security ?? "Open (no password)"],
                ["Power saving", w.power_save === undefined ? "Set by the driver" : w.power_save ? "On (saves battery, can add delay)" : "Off"],
                ["Interface", w.interface],
              ]}
            />
          </Panel>
        );
      })}
      <KpiRow
        tiles={[
          {
            label: "TCP data sent again",
            value: resentShare !== undefined ? `${resentShare.toFixed(2)}` : allShare !== undefined ? allShare.toFixed(2) : "-",
            unit: "%",
            subtext: resentShare !== undefined ? "now · lost and sent again" : "since start · lost and sent again",
            status: (resentShare ?? allShare ?? 0) > 2 ? "warn" : undefined,
          },
          { label: "Connections that failed to open", value: c("Tcp.AttemptFails") === undefined ? "-" : count(c("Tcp.AttemptFails")), subtext: "since start" },
          { label: "Connections reset", value: c("Tcp.EstabResets") === undefined ? "-" : count(c("Tcp.EstabResets")), subtext: "since start" },
          { label: "UDP packets dropped", value: c("Udp.InErrors") === undefined ? "-" : count(c("Udp.InErrors")), subtext: c("Udp.RcvbufErrors") ? `${count(c("Udp.RcvbufErrors"))} because a program read too slowly` : "since start" },
        ]}
      />
      <Panel title="Errors and drops per interface" subtitle="since start" info="Errors are damaged packets (often a bad cable or weak Wi-Fi). Drops are packets thrown away, often because nobody wanted them. Link changes count how often the connection went down and up." source="/sys/class/net/*/statistics" span={12} flush>
        <DataTable
          rows={ifaces}
          columns={[
            { key: "name", label: "Interface", text: (i) => i.name },
            { key: "err", label: "Errors (in · out)", text: (i) => `${count(i.rx_errors)} · ${count(i.tx_errors)}`, numeric: true },
            { key: "drop", label: "Dropped (in · out)", text: (i) => `${count(i.rx_dropped)} · ${count(i.tx_dropped)}`, numeric: true },
            { key: "dropsNow", label: "Drops per second now", text: (i) => (i.dropsNow === undefined ? "-" : i.dropsNow.toFixed(1)), numeric: true },
            { key: "coll", label: "Collisions", text: (i) => count(i.collisions), numeric: true },
            { key: "carrier", label: "Link changes", text: (i) => (i.carrier_changes === undefined ? "-" : count(i.carrier_changes)), numeric: true },
          ]}
          rowKey={(i) => i.name}
          pageSize={12}
        />
      </Panel>
    </>
  );
}

// ---------- Network › DNS & VPN ----------

export function DnsRoutesExpert() {
  const { data: d, error } = useDeep();
  if (!d) {
    return (
      <Panel title="DNS and routes" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const used = d.dns?.filter((l) => l.servers.length || l.current) ?? [];
  const def = d.routes.filter((r) => r.destination === "default" && !r.down);
  return (
    <>
      <Panel title="DNS per connection" info="DNSSEC checks that answers are genuine. DNS over TLS encrypts the questions, so the network cannot read them." source="resolvectl status" span={6} flush>
        {d.dns === undefined ? (
          <div className="empty muted">systemd-resolved is not running, so DNS details are not available.</div>
        ) : (
          <DataTable
            rows={used}
            columns={[
              { key: "link", label: "Connection", text: (l) => l.link },
              { key: "server", label: "Server in use", text: (l) => l.current ?? l.servers[0] ?? "-" },
              { key: "dnssec", label: "DNSSEC", text: (l) => (l.dnssec?.startsWith("yes") ? "On" : "Off") },
              { key: "dot", label: "DNS over TLS", text: (l) => (l.dns_over_tls ? "On" : "Off") },
              { key: "default", label: "Used for all names", text: (l) => (l.default_route ? "Yes" : "No") },
            ]}
            rowKey={(l) => l.link}
            empty="No connection has DNS servers"
          />
        )}
      </Panel>
      <Panel title="Routes" subtitle={def.length ? `internet through ${def.map((r) => r.device).join(", ")}` : "no internet route"} info="Where traffic goes. The default route carries everything that has no route of its own. VPNs add their own routes or routing rules." source="ip -j route · ip -j rule" span={6} flush>
        <DataTable
          rows={d.routes}
          columns={[
            { key: "dst", label: "To", text: (r) => (r.destination === "default" ? `everything (IPv${r.family})` : r.destination) },
            { key: "via", label: "Through", text: (r) => [r.device, r.gateway && `via ${r.gateway}`].filter(Boolean).join(" ") || "-" },
            { key: "metric", label: "Priority", text: (r) => (r.metric === undefined ? "-" : String(r.metric)), numeric: true },
            { key: "state", label: "State", text: (r) => (r.down ? "Link down" : "In use") },
          ]}
          rowKey={(r) => `${r.family}|${r.destination}|${r.device}|${r.metric}`}
          pageSize={10}
        />
        {d.extra_rules > 0 && <p className="muted panel-pad">{d.extra_rules} extra routing rules: a VPN (such as WireGuard or Tailscale) decides some traffic itself.</p>}
      </Panel>
    </>
  );
}

// ---------- Network › Traffic ----------

/** Traffic per program between two readings, from per-connection byte counts. */
export function perApp(now: TcpTraffic, before: TcpTraffic | undefined) {
  const seconds = before ? (now.at - before.at) / 1000 : 0;
  const key = (c: TcpTraffic["connections"][number]) => `${c.local}|${c.remote}`;
  const old = new Map(before?.connections.map((c) => [key(c), c]));
  const apps = new Map<string, { program: string; connections: number; downNow: number; upNow: number; down: number; up: number; resent: number }>();
  for (const c of now.connections) {
    const name = c.program ?? "Other users' programs";
    const a = apps.get(name) ?? { program: name, connections: 0, downNow: 0, upNow: 0, down: 0, up: 0, resent: 0 };
    a.connections += 1;
    a.down += c.received;
    a.up += c.sent;
    a.resent += c.resent;
    const b = old.get(key(c));
    // A new connection counts from zero; one that closed is simply gone.
    a.downNow += perSecond(c.received, b?.received ?? (before ? 0 : undefined), seconds) ?? 0;
    a.upNow += perSecond(c.sent, b?.sent ?? (before ? 0 : undefined), seconds) ?? 0;
    apps.set(name, a);
  }
  return [...apps.values()].sort((x, y) => (before ? y.downNow + y.upNow - x.downNow - x.upNow : y.down + y.up - x.down - x.up));
}

export function TrafficExpert() {
  const { data: t, before, error } = useLiveWithPrevious<TcpTraffic>(loadTcp, 5000);
  return (
    <Panel
      title="Traffic per app"
      subtitle="TCP only"
      info="Counted from the open TCP connections. QUIC and other UDP traffic, which many video sites and Google services use, is not counted, and closed connections drop out. Exact traffic for every app comes later with Full access."
      source="ss -tinp"
      span={12}
      flush
    >
      {t ? (
        <DataTable
          rows={perApp(t, before)}
          columns={[
            { key: "program", label: "App", text: (a) => a.program },
            { key: "conn", label: "Open connections", text: (a) => String(a.connections), numeric: true },
            { key: "downNow", label: "Download now", text: (a) => (before ? rate(a.downNow) : "-"), numeric: true },
            { key: "upNow", label: "Upload now", text: (a) => (before ? rate(a.upNow) : "-"), numeric: true },
            { key: "down", label: "Received on open connections", text: (a) => bytes(a.down), numeric: true },
            { key: "up", label: "Sent on open connections", text: (a) => bytes(a.up), numeric: true },
            { key: "resent", label: "Sent again", text: (a) => bytes(a.resent), numeric: true },
          ]}
          rowKey={(a) => a.program}
          pageSize={12}
          empty="No open TCP connections"
        />
      ) : (
        <NoData error={error} waiting="Loading…" />
      )}
    </Panel>
  );
}
