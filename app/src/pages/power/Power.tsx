// SPDX-License-Identifier: Apache-2.0
// Power › Battery, Power Profile, and Sleep (docs/03-ui-design/05-pages/07-power.md), with real data.
// Changing the power mode and the low-battery power-off go through the fix engine.

import { Guard } from "../../components/ErrorBoundary";
import { atLeast, useUi } from "../../ui";
import { CpuSleepStates } from "../performance/CpuExpert";
import { BatteryExpertSections, WakeExpertSections } from "./PowerExpert";
import { PowerControls } from "./PowerControls";
import { type SleepEvent, getInventory } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { RadioCards } from "../../components/FormControls";
import { StatusBadge } from "../../components/StatusBadge";
import { Gauge } from "../../components/charts/Gauge";
import { LineChart } from "../../components/charts/LineChart";
import { duration, pick, when } from "../../metrics";
import { useFix } from "../../shell/FixDrawer";
import { Button } from "../../components/Button";
import { useLive } from "../../useLive";
import { TimeChart, spark, useHistory, useLatest } from "../performance/shared";
import { NoData } from "../NoData";

const loadPower = () => getInventory("power");
const loadSleep = () => getInventory("sleep");
const usePower = () => useLive(loadPower, 15_000);

function NoBattery() {
  return (
    <Panel title="Battery" span={12}>
      <div className="empty muted">This computer has no battery.</div>
    </Panel>
  );
}

export function PowerBattery() {
  const { mode } = useUi();
  const { data: power, error } = usePower();
  const latest = useLatest();
  const battery = power?.batteries[0];
  const subject = battery?.name;
  const charge = useHistory([{ name: "battery.charge", subject }, { name: "battery.power", subject }]);
  const health = useHistory([{ name: "battery.health", subject }], undefined, "30d");
  if (!power) {
    return (
      <Panel title="Battery" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  if (!battery) return <NoBattery />;
  const healthNow = pick(latest, "battery.health", subject) ?? (battery.full_wh && battery.design_wh ? (battery.full_wh / battery.design_wh) * 100 : undefined);
  const left = pick(latest, "battery.time_left", subject);
  const watts = pick(latest, "battery.power", subject);
  const tone = healthNow === undefined ? undefined : healthNow < 60 ? "warn" : healthNow < 80 ? "info" : "ok";
  const plugged = power.on_ac || battery.status === "Charging" || battery.status === "Full" || battery.status === "Not charging";
  const trend = health?.series[0] ?? [];
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Charge", value: battery.charge === undefined ? "-" : String(Math.round(battery.charge)), unit: "%", subtext: battery.status, visual: battery.charge === undefined ? undefined : { kind: "bar", percent: battery.charge } },
          { label: "Health", value: healthNow === undefined ? "-" : String(Math.round(healthNow)), unit: "%", status: tone === "ok" ? "ok" : tone, subtext: "of design capacity" },
          { label: "Time remaining", value: plugged ? "Plugged in" : left === undefined ? "-" : duration(left), subtext: plugged ? "" : "Estimate" },
          { label: "Power draw", value: watts === undefined ? "-" : watts.toFixed(1), unit: "W", visual: spark(charge, 1) && { kind: "spark", data: spark(charge, 1) ?? [], slot: 2 } },
          { label: "Capacity", value: battery.full_wh === undefined ? "-" : battery.full_wh.toFixed(1), unit: "Wh", subtext: battery.design_wh ? `Design ${battery.design_wh.toFixed(0)} Wh` : "" },
          { label: "Cycle count", value: battery.cycles === undefined ? "N/A" : String(battery.cycles), subtext: battery.cycles === undefined ? "Not reported" : "" },
        ]}
      />
      <Panel title="Charge level" source="/sys/class/power_supply" span={8}>
        <TimeChart history={charge && { ...charge, series: charge.series.slice(0, 1) }} names={["Charge"]} unit="%" max={100} threshold={power.policy ? { value: power.policy.low_percent, label: `Low ${power.policy.low_percent}%` } : undefined} />
      </Panel>
      <Panel title="Battery health" info="How much charge the battery holds compared with when it was new." span={4}>
        {healthNow === undefined ? (
          <div className="empty muted">Not reported by this battery.</div>
        ) : (
          <>
            <Gauge value={healthNow} display={`${Math.round(healthNow)}%`} subtitle={battery.full_wh && battery.design_wh ? `${battery.full_wh.toFixed(1)} of ${battery.design_wh.toFixed(0)} Wh` : "of design"} status={tone === "info" ? "warn" : tone} />
            <p className="muted center">Think about replacing it below 60%</p>
          </>
        )}
      </Panel>
      <Panel title="Health trend" subtitle="30 days" span={12}>
        {trend.length >= 2 ? (
          <LineChart series={[{ name: "Health", data: trend.map((p) => p.value), slot: 1 }]} x={{ kind: "time", times: trend.map((p) => p.at), range: "30d" }} height={180} min={Math.max(0, Math.floor(Math.min(...trend.map((p) => p.value)) / 10) * 10 - 10)} max={100} unit="%" />
        ) : (
          <div className="chart-empty muted">The trend fills in as DeeCure keeps measuring.</div>
        )}
      </Panel>
      {atLeast(mode, "expert") && <Guard><BatteryExpertSections /></Guard>}
    </>
  );
}

const PROFILES = [
  { value: "power-saver", title: "Power Saver", description: "Longer battery life" },
  { value: "balanced", title: "Balanced", description: "Default" },
  { value: "performance", title: "Performance", description: "Plugged in only" },
] as const;

const minutes = (s?: number) => (s === undefined ? "-" : s === 0 ? "Never" : duration(s));
const SLEEP_MODE: Record<string, string> = { s2idle: "Modern standby (s2idle)", deep: "Deep sleep (S3)", shallow: "Light sleep (S1)" };

export function PowerProfile() {
  const { mode } = useUi();
  const { data: power, error } = usePower();
  const openFix = useFix();
  if (!power) {
    return (
      <Panel title="Power mode" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const battery = power.batteries[0];
  const options = PROFILES.filter((p) => power.profiles.length === 0 || power.profiles.includes(p.value));
  return (
    <>
      <Panel title="Power mode" source="power-profiles-daemon" span={6}>
        {power.profile ? (
          <RadioCards name="pp" value={power.profile} options={options} onChange={(p) => openFix("power.set_profile", p)} />
        ) : (
          <div className="empty muted">The power profiles service is not running, so the power mode is unknown.</div>
        )}
      </Panel>
      <Panel title="Battery policies" source="/etc/UPower/UPower.conf · logind · GNOME settings" span={6} flush>
        <KeyValueList
          rows={[
            ["Low battery warning", power.policy ? `${power.policy.low_percent}%` : "-"],
            [
              "Critical action",
              power.policy ? (
                <span key="c" className="btn-row">
                  {`${power.policy.critical_action} at ${power.policy.action_percent}%`}
                  {battery && power.policy.critical_action !== "PowerOff" && <Button small onClick={() => openFix("power.set_battery_action")}>Power off cleanly…</Button>}
                </span>
              ) : (
                "-"
              ),
            ],
            ["Charge limit", !battery ? "No battery" : battery.charge_limit === undefined ? "Not supported" : battery.charge_limit >= 100 ? "Supported · Off" : `Stops at ${battery.charge_limit}%`],
            ["Screen blank", minutes(power.screen_blank_seconds)],
            ["Automatic suspend", battery ? `${minutes(power.suspend_on_battery_seconds)} on battery · ${minutes(power.suspend_on_ac_seconds)} plugged in` : minutes(power.suspend_on_ac_seconds)],
            ["Closing the lid", power.lid_action],
            ["Sleep mode", power.sleep_mode ? SLEEP_MODE[power.sleep_mode] ?? power.sleep_mode : "-"],
          ]}
        />
      </Panel>
      {atLeast(mode, "expert") && <Guard><PowerControls /></Guard>}
    </>
  );
}

const EVENTS: Column<SleepEvent>[] = [
  { key: "time", label: "Time", text: (e) => String(e.at), render: (e) => when(e.at) },
  { key: "event", label: "Event", text: (e) => e.kind[0].toUpperCase() + e.kind.slice(1) },
  { key: "duration", label: "Duration", text: (e) => (e.duration_ms === undefined ? "-" : duration(e.duration_ms / 1000)), numeric: true },
  { key: "result", label: "Result", text: (e) => (e.failed ? "Failed" : e.duration_ms === undefined ? "No resume logged" : "Success"), render: (e) => (e.failed ? <StatusBadge variant="crit" label="Failed" /> : e.duration_ms === undefined ? <StatusBadge variant="off" label="No resume logged" /> : <StatusBadge variant="ok" label="Success" />) },
];

export function PowerSleep() {
  const { mode } = useUi();
  const { data: events, error } = useLive(loadSleep, 300_000);
  if (!events) {
    return (
      <Panel title="Sleep events" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const week = events.filter((e) => e.at >= Date.now() - 7 * 86_400_000);
  const asleep = week.reduce((s, e) => s + (e.duration_ms ?? 0), 0) / 1000;
  const failures = week.filter((e) => e.failed).length;
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Suspends", value: String(week.length), subtext: "last 7 days" },
          { label: "Failures", value: String(failures), status: failures ? "warn" : "ok" },
          { label: "Time asleep", value: String(Math.round(asleep / 3600)), unit: "h", subtext: "last 7 days" },
        ]}
      />
      <Panel title="Sleep events" subtitle="last 30 days" source="journal: systemd-sleep" span={12} flush>
        <DataTable rows={events} columns={EVENTS} rowKey={(e) => String(e.at)} pageSize={12} empty="No sleep in the last 30 days, or the journal cannot be read" />
      </Panel>
      {atLeast(mode, "expert") && (
        <Guard>
        <>
          <WakeExpertSections />
          <CpuSleepStates />
        </>
        </Guard>
      )}
    </>
  );
}

