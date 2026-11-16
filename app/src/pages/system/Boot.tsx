// SPDX-License-Identifier: Apache-2.0
// Operating System › Boot (docs/03-ui-design/05-pages/05-system.md), with real data.
// The history holds the boots since DeeCure was installed.

import { Guard } from "../../components/ErrorBoundary";
import { BootExpertSections } from "../services/ServicesExpert";
import { atLeast, useUi } from "../../ui";
import { type BootInfo, type BootRecord, getInventory, getMetrics, getSnapshot } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { duration, when } from "../../metrics";
import { KpiRow, type Kpi } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { BarChart } from "../../components/charts/BarChart";
import { BarList } from "../../components/charts/BarList";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadBoot = () => getSnapshot<BootInfo>("boot");
const loadHistory = () => {
  const until = Date.now();
  return getMetrics("boot.time", until - 365 * 86_400_000, until, { maxPoints: 2000 });
};

const loadBoots = () => getInventory("boots");

const SHUTDOWN: Record<BootRecord["shutdown"], [StatusVariant, string]> = { running: ["info", "Running"], clean: ["ok", "Clean"], unclean: ["crit", "Unclean"] };

const BOOTS: Column<BootRecord>[] = [
  { key: "started", label: "Started", text: (b) => String(b.started), render: (b) => when(b.started) },
  { key: "ended", label: "Ended", text: (b) => (b.shutdown === "running" ? "-" : when(b.ended)) },
  { key: "duration", label: "Duration", text: (b) => duration((b.ended - b.started) / 1000), numeric: true },
  { key: "shutdown", label: "Shutdown", text: (b) => SHUTDOWN[b.shutdown][1], render: (b) => <StatusBadge variant={SHUTDOWN[b.shutdown][0]} label={SHUTDOWN[b.shutdown][1]} /> },
  { key: "kernel", label: "Kernel", text: (b) => b.kernel ?? "-", render: (b) => (b.kernel ? <code>{b.kernel.replace(/-generic$/, "")}</code> : "-") },
];

/** Every boot the journal keeps. It was Logs › Boots until 2026-11-16. */
function BootHistory() {
  const { data: boots, error } = useLive(loadBoots, 300_000);
  return (
    <Panel title="Boot history" source="journalctl --list-boots" span={12} flush>
      {boots ? <DataTable rows={boots} columns={BOOTS} rowKey={(b) => String(b.started)} empty="The journal keeps no earlier boots" /> : <NoData error={error} waiting="Loading…" />}
    </Panel>
  );
}

/** systemd escapes "-" in unit names as "\x2d". */
const unescape = (name: string) => name.replace(/\\x([0-9a-f]{2})/gi, (_, hex: string) => String.fromCharCode(parseInt(hex, 16)));

export function SystemBoot() {
  const { mode } = useUi();
  const { data: snapshot, error } = useLive(loadBoot, 300_000);
  const history = useLive(loadHistory, 300_000).data ?? [];
  const boot = snapshot?.data;
  if (!boot?.times) {
    // The boot history does not wait for the boot times.
    return (
      <>
        <Panel title="Boot" span={12}>
          <NoData error={error} waiting={snapshot === undefined ? "Loading…" : "DeeCure reads the boot times once the boot has finished. Check back in a few minutes."} />
        </Panel>
        <BootHistory />
      </>
    );
  }
  const t = boot.times;
  const recent = history.slice(-10);
  const earlier = history.slice(0, -1);
  const average = earlier.length ? earlier.reduce((s, p) => s + p.value, 0) / earlier.length : undefined;
  const slow = average !== undefined && t.total > average * 1.3;
  const phase = (label: string, value?: number): Kpi => ({ label, value: value === undefined ? "N/A" : value.toFixed(1), unit: value === undefined ? undefined : "s", subtext: value === undefined ? "Not reported" : "" });
  const today = new Date().toDateString();
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Last boot", value: t.total.toFixed(1), unit: "s", status: slow ? "warn" : "ok", subtext: average === undefined ? "First boot DeeCure saw" : `Average ${average.toFixed(1)} s` },
          phase("Firmware", t.firmware),
          phase("Loader", t.loader),
          phase("Kernel", t.kernel),
          ...(t.initrd !== undefined ? [phase("Initrd", t.initrd)] : []),
          phase("Userspace", t.userspace),
        ]}
      />
      <Panel title="Slowest units" source="systemd over D-Bus (like systemd-analyze blame)" span={6}>
        <BarList rows={boot.slowest.slice(0, 8).map((u) => ({ label: unescape(u.name), value: u.seconds, display: `${u.seconds.toFixed(1)} s`, mono: true }))} />
      </Panel>
      <Panel title="Boot time history" subtitle={`last ${recent.length} ${recent.length === 1 ? "boot" : "boots"}`} span={6}>
        <BarChart
          bars={recent.map((p) => {
            const d = new Date(p.at);
            return { label: d.toDateString() === today ? "Today" : d.toLocaleDateString(undefined, { day: "numeric", month: "short" }), long: d.toLocaleString(), value: p.value };
          })}
          unit="s"
          highlight={slow ? [recent.length - 1] : []}
        />
      </Panel>
      <BootHistory />
      {atLeast(mode, "expert") && <Guard><BootExpertSections /></Guard>}
    </>
  );
}
