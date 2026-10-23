// SPDX-License-Identifier: Apache-2.0
// Settings › General, Access, Notifications, Assistant, Data, About
// (docs/03-ui-design/05-pages/22-settings.md). View settings work now; settings for
// features that come later are shown honestly as "later".

import { type NotifySettings, exportReport, getInventory, getNotifications, getNotifySettings, sendTestNotification, setNotifySettings, toAppError } from "../../api";
import { useState } from "react";
import { Button } from "../../components/Button";
import { FieldRow, RadioCards, Select, Switch } from "../../components/FormControls";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { bytes, duration, when } from "../../metrics";
import { usePref } from "../../prefs";
import { useToast } from "../../shell/Toast";
import type { Mode, Range } from "../../ui";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";
import { useHelperInstalled } from "../../components/AdminRead";

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
  const [, setOnboarded] = usePref<boolean>("onboarded", true);
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
      <FieldRow label="Welcome screens" description="Show the first-launch screens and the first check again">
        <Button small onClick={() => setOnboarded(false)}>
          Show again
        </Button>
      </FieldRow>
      <FieldRow label="Start at login" description="The background agent starts by itself when you sign in (systemd user service)">
        <StatusBadge variant="ok" label="On" />
      </FieldRow>
    </Panel>
  );
}

export function SettingsAccess() {
  const toast = useToast();
  const helper = useHelperInstalled();
  return (
    <>
      <Panel title="Access level" span={8}>
        <RadioCards
          name="access"
          value="basic"
          options={[
            { value: "basic", title: "Basic", description: "No password · everything you can read yourself" },
            { value: "full", title: "Full", description: "Admin helper · disk health, firewall rules, all users' jobs, admin fixes · the password is asked each time" },
          ]}
          onChange={(v) => v === "full" && toast(helper ? "Full access is on: admin read-outs and fixes ask for your password when used." : "Full access needs the admin helper: install the deecure package from DeeCure's repository.")}
        />
      </Panel>
      <Panel title="Admin helper" span={4} flush>
        <KeyValueList
          rows={[
            ["Service", <code key="s">deecure-helper</code>],
            ["Status", helper === null ? "Checking…" : <StatusBadge key="st" variant={helper ? "ok" : "off"} label={helper ? "Installed" : "Not installed"} />],
            ["Activation", "D-Bus, on demand; exits after 2 idle minutes"],
            ["Allowed actions", "12 fixed actions and 6 read-outs"],
            ["Approval", "polkit asks for your password for each action"],
            ["Audit log", <code key="a">/var/log/deecure/helper.log</code>],
          ]}
        />
      </Panel>
    </>
  );
}

const loadNotify = () => getNotifySettings();
const loadSent = () => getNotifications();
const QUIET = [
  { value: "off", label: "Off" },
  { value: "22-07", label: "22:00–07:00" },
  { value: "23-08", label: "23:00–08:00" },
] as const;

export function SettingsNotifications() {
  const toast = useToast();
  const { data, error } = useLive(loadNotify, 300_000);
  const sent = useLive(loadSent, 60_000).data;
  const [saved, setSaved] = useState<NotifySettings | null>(null);
  const [testing, setTesting] = useState(false);
  const current = saved ?? data;
  if (!current) {
    return (
      <Panel title="Notifications" span={8}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const change = async (next: NotifySettings) => {
    const before = current;
    setSaved(next);
    try {
      setSaved(await setNotifySettings(next));
    } catch (e) {
      setSaved(before);
      toast(`Not saved: ${toAppError(e).message}`);
    }
  };
  const test = async () => {
    setTesting(true);
    try {
      await sendTestNotification();
      toast("A test notification was sent. Did you see it?");
    } catch (e) {
      toast(`No notification: ${toAppError(e).message}`);
    }
    setTesting(false);
  };
  return (
    <>
      <Panel title="Notifications" info="The background agent shows these, even when this window is closed." span={8}>
        <FieldRow label="Critical issues" description="Disk almost full or filling fast, failing drive, overheating — right away">
          <Switch checked={current.critical} onChange={(on) => void change({ ...current, critical: on })} label="Critical issues" />
        </FieldRow>
        <FieldRow label="Warnings" description="Included in the weekly summary when off">
          <Switch checked={current.warnings} onChange={(on) => void change({ ...current, warnings: on })} label="Warnings" />
        </FieldRow>
        <FieldRow label="Weekly summary" description="Sunday 18:00: health trend, what changed, and one tip">
          <Switch checked={current.weekly} onChange={(on) => void change({ ...current, weekly: on })} label="Weekly summary" />
        </FieldRow>
        <FieldRow label="Quiet hours" description="Nothing is shown then; it waits until they end">
          <Select value={current.quiet_hours} options={QUIET} onChange={(v) => void change({ ...current, quiet_hours: v })} label="Quiet hours" />
        </FieldRow>
        <FieldRow label="Test" description="The same alert never repeats within 24 hours">
          <Button small disabled={testing} onClick={() => void test()}>
            {testing ? "Sending…" : "Send a test notification"}
          </Button>
        </FieldRow>
      </Panel>
      <Panel title="Recently shown" span={4} flush>
        {!sent ? (
          <div className="empty muted">Loading…</div>
        ) : sent.length ? (
          <KeyValueList rows={sent.slice(0, 8).map((n) => [when(n.at), n.title])} />
        ) : (
          <div className="empty muted">No notifications yet.</div>
        )}
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

function ExportButton() {
  const toast = useToast();
  const [busy, setBusy] = useState(false);
  const run = async () => {
    setBusy(true);
    try {
      toast(`Saved: ${await exportReport()}`);
    } catch (e) {
      toast(`Not saved: ${toAppError(e).message}`);
    }
    setBusy(false);
  };
  return (
    <Button small disabled={busy} onClick={() => void run()}>
      {busy ? "Saving…" : "Export"}
    </Button>
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
      <FieldRow label="Export report" description="The last 7 days as one HTML file in your Downloads folder, to keep or share">
        <ExportButton />
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
