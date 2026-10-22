// SPDX-License-Identifier: Apache-2.0
// Settings › General, Access, Notifications, Assistant, Data, About
// (docs/03-ui-design/05-pages/22-settings.md). View settings work now; settings for
// features that come later are shown honestly as "later".

import { getInventory } from "../../api";
import { AlertBanner } from "../../components/AlertBanner";
import { FieldRow, RadioCards, Select, Switch } from "../../components/FormControls";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { bytes, duration } from "../../metrics";
import { usePref } from "../../prefs";
import { useToast } from "../../shell/Toast";
import type { Mode, Range } from "../../ui";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadAgent = () => getInventory("agent");
const RANGES = [
  { value: "1h", label: "1 hour" },
  { value: "24h", label: "24 hours" },
  { value: "7d", label: "7 days" },
  { value: "30d", label: "30 days" },
] as const;

export function SettingsGeneral() {
  const [mode, setMode] = usePref<Mode>("mode", "beginner");
  const [range, setRange] = usePref<Range>("range", "24h");
  const [rail, setRail] = usePref<boolean>("rail", false);
  return (
    <Panel title="General" span={8}>
      <FieldRow label="Expert mode" description="Show data sources, commands, and raw columns">
        <Switch checked={mode === "expert"} onChange={(on) => setMode(on ? "expert" : "beginner")} label="Expert mode" />
      </FieldRow>
      <FieldRow label="Default time range" description="Used by charts; also in the top bar">
        <Select value={range} options={RANGES} onChange={setRange} label="Default time range" />
      </FieldRow>
      <FieldRow label="Narrow sidebar" description="Show only icons in the sidebar">
        <Switch checked={rail} onChange={setRail} label="Narrow sidebar" />
      </FieldRow>
      <FieldRow label="Start at login" description="The background agent starts by itself when you sign in (systemd user service)">
        <StatusBadge variant="ok" label="On" />
      </FieldRow>
    </Panel>
  );
}

export function SettingsAccess() {
  const toast = useToast();
  return (
    <>
      <Panel title="Access level" span={8}>
        <RadioCards
          name="access"
          value="basic"
          options={[
            { value: "basic", title: "Basic", description: "No password · everything you can read yourself" },
            { value: "full", title: "Full", description: "Admin helper approved once · SMART, firewall rules, per-app traffic, fixes" },
          ]}
          onChange={(v) => v === "full" && toast("Full access needs the admin helper, which comes with the fix engine.")}
        />
      </Panel>
      <Panel title="Admin helper" span={4} flush>
        <KeyValueList
          rows={[
            ["Service", <code key="s">deecure-helper</code>],
            ["Status", <StatusBadge key="st" variant="off" label="Not installed yet" />],
            ["Activation", "D-Bus, on demand (planned)"],
            ["Allowed actions", "A fixed list (planned)"],
          ]}
        />
      </Panel>
    </>
  );
}

export function SettingsNotifications() {
  const [critical, setCritical] = usePref<boolean>("notify.critical", true);
  const [warnings, setWarnings] = usePref<boolean>("notify.warnings", false);
  const [weekly, setWeekly] = usePref<boolean>("notify.weekly", true);
  return (
    <>
      <AlertBanner tone="info" title="Desktop notifications come later" text="Your choices are saved now and will be used when notifications arrive." />
      <Panel title="Notifications" span={8}>
        <FieldRow label="Critical issues" description="Disk almost full, failing drive, overheating">
          <Switch checked={critical} onChange={setCritical} label="Critical issues" />
        </FieldRow>
        <FieldRow label="Warnings" description="Included in the weekly summary when off">
          <Switch checked={warnings} onChange={setWarnings} label="Warnings" />
        </FieldRow>
        <FieldRow label="Weekly summary" description="Sunday evening">
          <Switch checked={weekly} onChange={setWeekly} label="Weekly summary" />
        </FieldRow>
      </Panel>
    </>
  );
}

export function SettingsAssistant() {
  return (
    <Panel title="Assistant" span={8}>
      <div className="empty muted">The assistant comes in a later version. It will run a local model on this computer, read-only, and suggest fixes without running them. A cloud fallback, if you turn it on, will remove names, host names, addresses, and serial numbers first.</div>
    </Panel>
  );
}

export function SettingsData() {
  const { data: agent, error } = useLive(loadAgent, 60_000);
  return (
    <Panel title="Data retention" span={8}>
      <FieldRow label="Keep history for" description="Fixed for now: detail fades with age to keep the database small">
        <span>1-minute points 7 days · 15-minute 90 days · daily 2 years</span>
      </FieldRow>
      <FieldRow label="Database size" description={agent?.database ?? "~/.local/share/deecure/deecure.db"}>
        <span>{agent ? bytes(agent.database_bytes) : error ? "—" : "…"}</span>
      </FieldRow>
      <FieldRow label="Export report" description="HTML or JSON snapshot of all pages">
        <span className="muted">Later</span>
      </FieldRow>
      <FieldRow label="Delete history" description="Restore points are kept">
        <span className="muted">Later</span>
      </FieldRow>
    </Panel>
  );
}

export function SettingsAbout() {
  const { data: agent, error } = useLive(loadAgent, 30_000);
  if (!agent) {
    return (
      <Panel title="About" span={6}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  return (
    <>
      <Panel title="About" span={6} flush>
        <KeyValueList
          rows={[
            ["Version", `DeeCure ${agent.version}`],
            ["Background agent", `deecure-agent ${agent.version} · running for ${duration(agent.uptime_seconds)}`],
            ["Agent memory", bytes(agent.memory_bytes)],
            ["Rules loaded", `${agent.rules} from ${agent.rules_dir}`],
            ["Distro adapter", "Ubuntu / Debian"],
            ["License", "Apache-2.0"],
            ["Source code", <code key="g">github.com/krisskad0/DeeCure</code>],
          ]}
        />
      </Panel>
      <Panel title="Privacy" span={6}>
        <p>DeeCure reads your computer to explain it. Everything stays on this computer: there is no account, no tracking, and nothing is sent anywhere.</p>
      </Panel>
    </>
  );
}
