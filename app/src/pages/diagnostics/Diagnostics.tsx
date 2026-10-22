// SPDX-License-Identifier: Apache-2.0
// Diagnostics › Performance Check, Network Test, Hardware Tests
// (docs/03-ui-design/05-pages/20-diagnostics.md), with real checks run by the agent.

import { type DiagnosticReport, getInventory, getLatest, toAppError } from "../../api";
import { AlertBanner } from "../../components/AlertBanner";
import { LinkButton } from "../../components/Button";
import { type Column, DataTable } from "../../components/DataTable";
import { DiagnosticsRunner } from "../../components/DiagnosticsRunner";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { BarList } from "../../components/charts/BarList";
import { href } from "../../route";
import { useLive } from "../../useLive";

type Report = DiagnosticReport;

const RESULT: Record<Report["steps"][number]["result"], [StatusVariant, string]> = { ok: ["ok", "Good"], warn: ["warn", "High"], crit: ["crit", "Problem"], na: ["off", "Not available"] };

const STEPS: Column<Report["steps"][number]>[] = [
  { key: "name", label: "Check", text: (s) => s.name },
  { key: "value", label: "Measured", text: (s) => s.value },
  { key: "result", label: "Result", text: (s) => RESULT[s.result][1], render: (s) => <StatusBadge variant={RESULT[s.result][0]} label={RESULT[s.result][1]} /> },
];

function Result({ report, title }: { report: Report; title: string }) {
  return (
    <>
      <AlertBanner tone={report.tone} title={title} text={report.verdict} />
      <DataTable rows={report.steps} columns={STEPS} rowKey={(s) => s.name} />
      {report.top && report.top.length > 0 && (
        <div>
          <div className="field-label">Busiest programs during the check</div>
          <BarList rows={report.top.map(([name, cpu]) => ({ label: name, value: cpu, display: `${cpu.toFixed(1)}%` }))} />
        </div>
      )}
    </>
  );
}

const failed = (e: unknown) => <AlertBanner tone="crit" title="The check could not run" text={toAppError(e).message} />;

export function DiagnosticsPerformance() {
  return (
    <Panel title="Performance check" info="Run while the computer feels slow. Finds the main cause in a few seconds." span={12}>
      <DiagnosticsRunner
        steps={["CPU load", "Memory", "Memory pressure", "Swap activity", "Disk I/O", "Temperature", "Top processes"]}
        button="Run check"
        run={() => getInventory("diagnose_performance")}
        result={(r) => <Result report={r} title={r.tone === "ok" ? "No bottleneck found" : "Found the main cause"} />}
        onError={failed}
      />
    </Panel>
  );
}

export function DiagnosticsNetwork() {
  return (
    <Panel title="Network test" info="Checks each hop: the router, name lookup, the internet, and IPv6. Sends a few test packets." span={12}>
      <DiagnosticsRunner
        steps={["Gateway", "Packet loss", "DNS resolution", "Internet latency", "IPv6"]}
        button="Run test"
        run={() => getInventory("diagnose_network")}
        result={(r) => <Result report={r} title={r.tone === "ok" ? "Network is healthy" : "Network problem found"} />}
        onError={failed}
      />
    </Panel>
  );
}

interface HardwareTest {
  test: string;
  duration: string;
  result: [StatusVariant, string];
  action: React.ReactNode;
}

const loadLatest = () => getLatest();

export function DiagnosticsHardware() {
  const latest = useLive(loadLatest, 30_000).data;
  const health = latest?.find((m) => m.name === "battery.health")?.value;
  const temp = latest?.find((m) => m.name === "temp.cpu")?.value;
  const rows: HardwareTest[] = [
    { test: "SMART self-test (disk)", duration: "2 min", result: ["off", "Needs Full access"], action: <LinkButton to={href("settings", "access")}>Access</LinkButton> },
    { test: "Memory test (memtest86+)", duration: "30–60 min", result: ["off", "Runs from the boot menu"], action: <span className="muted">Restart, hold Shift, choose Memory test</span> },
    { test: "Battery capacity report", duration: "instant", result: health === undefined ? ["off", "No battery"] : [health < 60 ? "warn" : "ok", `${Math.round(health)}% health`], action: <LinkButton to={href("power", "battery")}>Open</LinkButton> },
    { test: "Sensor check", duration: "instant", result: temp === undefined ? ["off", "No CPU sensor"] : [temp >= 90 ? "crit" : temp >= 80 ? "warn" : "ok", `CPU ${Math.round(temp)} °C`], action: <LinkButton to={href("hardware", "sensors")}>Open</LinkButton> },
  ];
  return (
    <Panel title="Available tests" span={12} flush>
      <DataTable
        rows={rows}
        columns={[
          { key: "test", label: "Test", text: (r) => r.test },
          { key: "duration", label: "Duration", text: (r) => r.duration },
          { key: "result", label: "Result", text: (r) => r.result[1], render: (r) => <StatusBadge variant={r.result[0]} label={r.result[1]} /> },
          { key: "action", label: "", text: () => "", render: (r) => r.action },
        ]}
        rowKey={(r) => r.test}
      />
    </Panel>
  );
}
