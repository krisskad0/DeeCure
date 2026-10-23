// SPDX-License-Identifier: Apache-2.0
// Performance › Network (docs/03-ui-design/05-pages/02-performance.md), with real data.
// Not shown yet: packet loss and latency (they need a ping), and the 14-day usage chart
// (it needs daily totals; minute averages over two weeks are not exact enough).

import { type NetInterface, getInventory, getMetrics } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { bytes, pick, rate } from "../../metrics";
import { useLive } from "../../useLive";
import { MB, TimeChart, spark, useHistory, useLatest } from "./shared";

const loadNetwork = () => getInventory("network");

/** Bytes moved since local midnight, from the stored minute averages. */
async function loadToday(): Promise<number> {
  const midnight = new Date();
  midnight.setHours(0, 0, 0, 0);
  const until = Date.now();
  const lists = await Promise.all(["net.rx", "net.tx"].map((n) => getMetrics(n, midnight.getTime(), until, { maxPoints: 2000 })));
  return lists.flat().reduce((sum, p) => sum + p.value * 60, 0);
}

interface Row extends NetInterface {
  rx?: number;
  tx?: number;
}

const COLUMNS: Column<Row>[] = [
  { key: "name", label: "Interface", text: (i) => i.name, render: (i) => <code>{i.name}</code> },
  { key: "kind", label: "Type", text: (i) => i.kind },
  { key: "state", label: "State", text: (i) => i.state, render: (i) => <StatusBadge variant={i.state === "up" ? "ok" : "off"} label={i.state} /> },
  { key: "rx", label: "RX/s", text: (i) => (i.rx === undefined ? "-" : rate(i.rx)), numeric: true },
  { key: "tx", label: "TX/s", text: (i) => (i.tx === undefined ? "-" : rate(i.tx)), numeric: true },
  { key: "speed", label: "Link speed", text: (i) => (i.speed_mbps ? `${i.speed_mbps} Mb/s` : "-"), numeric: true, expert: true },
  { key: "mac", label: "MAC", text: (i) => i.mac ?? "-", expert: true },
];

const split = (bps: number | undefined) => {
  if (bps === undefined) return { value: "-" };
  const [value, unit] = rate(bps).split(" ");
  return { value, unit };
};

export function PerformanceNetwork() {
  const latest = useLatest();
  const network = useLive(loadNetwork, 30_000).data;
  const today = useLive(loadToday, 60_000).data;
  const history = useHistory(["net.rx", "net.tx"]);
  const rx = pick(latest, "net.rx");
  const tx = pick(latest, "net.tx");
  const down = spark(history, 0);
  const up = spark(history, 1);
  const rows: Row[] = (network?.interfaces ?? [])
    .map((i) => ({ ...i, rx: pick(latest, "net.rx", i.name), tx: pick(latest, "net.tx", i.name) }))
    .sort((a, b) => Number(b.state === "up") - Number(a.state === "up") || a.name.localeCompare(b.name));
  const c = network?.connections;
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Download", ...split(rx), visual: down && { kind: "spark", data: down, slot: 1 } },
          { label: "Upload", ...split(tx), visual: up && { kind: "spark", data: up, slot: 2 } },
          { label: "Data today", value: today === undefined ? "-" : bytes(today).split(" ")[0], unit: today === undefined ? undefined : bytes(today).split(" ")[1], subtext: "while DeeCure was running" },
          { label: "Connections", value: c ? String(c.tcp_established) : "-", subtext: c ? `${c.tcp_listening} listening · ${c.udp} UDP` : "" },
        ]}
      />
      <Panel title="Throughput" source="/proc/net/dev" span={12}>
        <TimeChart history={history} names={["Download", "Upload"]} unit="MB/s" scale={(v) => v / MB} />
      </Panel>
      <Panel title="Interfaces" source="/sys/class/net" span={12} flush>
        {network ? <DataTable rows={rows} columns={COLUMNS} rowKey={(i) => i.name} pageSize={10} /> : <div className="empty muted">Loading…</div>}
      </Panel>
    </>
  );
}
