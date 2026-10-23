// SPDX-License-Identifier: Apache-2.0
// Settings › General, Access, Notifications, Assistant, Data, About
// (docs/03-ui-design/05-pages/22-settings.md). View settings work now; settings for
// features that come later are shown honestly as "later".

import { type KeepHistory, type NotifySettings, type TrayState, deleteHistory, exportReport, getDataSettings, setDataSettings, getInventory, getTrayState, setTray, getNotifications, getNotifySettings, sendTestNotification, setNotifySettings, toAppError } from "../../api";
import { useState } from "react";
import { Button } from "../../components/Button";
import { FieldRow, RadioCards, Select, Switch } from "../../components/FormControls";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { bytes, duration, when } from "../../metrics";
import { usePref } from "../../prefs";
import type { RefreshChoice, TempUnit } from "../../units";
import { useToast } from "../../shell/Toast";
import type { Mode, Range } from "../../ui";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";
import { accessOptions, useAccess } from "../../access";

const loadAgent = () => getInventory("agent");
const RANGES = [
  { value: "1h", label: "1 hour" },
  { value: "24h", label: "24 hours" },
  { value: "7d", label: "7 days" },
  { value: "30d", label: "30 days" },
] as const;

const loadTray = () => getTrayState();

function TrayRow() {
  const toast = useToast();
  const { data } = useLive(loadTray, 600_000);
  const [state, setState] = useState<TrayState | null>(null);
  const current = state ?? data;
  const change = async (on: boolean) => {
    try {
      setState(await setTray(on));
      toast(on ? "DeeCure now stays in the tray when you close the window, and starts there at login." : "Tray icon turned off.");
    } catch (e) {
      toast(`Tray icon not turned on: ${typeof e === "string" ? e : toAppError(e).message}`);
    }
  };
  return (
    <FieldRow label="Tray icon" description={current && !current.available ? "This desktop has no tray area" : "Keep DeeCure in the tray when the window closes, and start it there at login"}>
      <Switch checked={current?.enabled ?? false} onChange={(on) => void change(on)} label="Tray icon" />
    </FieldRow>
  );
}

const REFRESH = [
  { value: "2s", label: "2 seconds" },
  { value: "5s", label: "5 seconds" },
  { value: "15s", label: "15 seconds" },
  { value: "manual", label: "Manual" },
] as const;
const UNITS = [
  { value: "celsius", label: "Celsius (°C)" },
  { value: "fahrenheit", label: "Fahrenheit (°F)" },
] as const;

export function SettingsGeneral() {
  const [mode, setMode] = usePref<Mode>("mode", "beginner");
  const [range, setRange] = usePref<Range>("range", "24h");
  const [rail, setRail] = usePref<boolean>("rail", false);
  const [refresh, setRefresh] = usePref<RefreshChoice>("refresh", "5s");
  const [unit, setUnit] = usePref<TempUnit>("temp", "celsius");
  const [, setOnboarded] = usePref<boolean>("onboarded", true);
  return (
    <Panel title="General" span={8}>
      <FieldRow label="Expert mode" description="Show every page and tab, data sources, commands, and raw columns">
        <Switch checked={mode === "expert"} onChange={(on) => setMode(on ? "expert" : "beginner")} label="Expert mode" />
      </FieldRow>
      <FieldRow label="Default time range" description="Used by charts; also in the top bar">
        <Select value={range} options={RANGES} onChange={setRange} label="Default time range" />
      </FieldRow>
      <FieldRow label="Refresh interval" description="How often live numbers update while a page is open. Manual: only when you press Refresh">
        <Select value={refresh} options={REFRESH} onChange={setRefresh} label="Refresh interval" />
      </FieldRow>
      <FieldRow label="Temperature unit" description="How temperatures are shown">
        <Select value={unit} options={UNITS} onChange={setUnit} label="Temperature unit" />
      </FieldRow>
      <FieldRow label="Narrow sidebar" description="Show only icons in the sidebar">
        <Switch checked={rail} onChange={setRail} label="Narrow sidebar" />
      </FieldRow>
      <FieldRow label="Welcome screens" description="Show the first-launch screens and the first check again">
        <Button small onClick={() => setOnboarded(false)}>
          Show again
        </Button>
      </FieldRow>
      <TrayRow />
      <FieldRow label="Start at login" description="The background agent starts by itself when you sign in (systemd user service)">
        <StatusBadge variant="ok" label="On" />
      </FieldRow>
    </Panel>
  );
}

export function SettingsAccess() {
  const toast = useToast();
  const access = useAccess();
  const helper = access.helper;
  return (
    <>
      <Panel title="Access level" span={8}>
        <RadioCards
          name="access"
          value={access.full ? "full" : "basic"}
          options={accessOptions(helper)}
          onChange={(v) => {
            access.set(v);
            toast(v === "full" ? "Full access is on: admin read-outs and admin fixes ask for your password each time." : "Basic access: DeeCure will not ask for your password. Admin read-outs and admin fix steps are off.");
          }}
        />
        {helper === false && <p className="muted panel-pad">Full access needs the admin helper. It comes with the deecure package installed from DeeCure's repository; a development build does not install it.</p>}
      </Panel>
      <Panel title="Admin helper" span={4} flush>
        <KeyValueList
          rows={[
            ["Service", <code key="s">deecure-helper</code>],
            ["Status", helper === null ? "Checking…" : <StatusBadge key="st" variant={helper ? "ok" : "off"} label={helper ? "Installed" : "Not installed"} />],
            ["Access level", access.full ? "Full" : "Basic"],
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
  { value: "22-07", label: "22:00-07:00" },
  { value: "23-08", label: "23:00-08:00" },
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
        <FieldRow label="Critical issues" description="Disk almost full or filling fast, failing drive, overheating - right away">
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

const loadData = () => getDataSettings();
const KEEP = [
  { value: "90d", label: "90 days" },
  { value: "1y", label: "1 year" },
  { value: "2y", label: "2 years" },
] as const;

function DeleteHistory({ onDone }: { onDone: () => void }) {
  const toast = useToast();
  const [stage, setStage] = useState<"idle" | "confirm" | "busy">("idle");
  if (stage === "idle")
    return (
      <Button small onClick={() => setStage("confirm")}>
        Delete…
      </Button>
    );
  const run = async () => {
    setStage("busy");
    try {
      const r = await deleteHistory();
      toast(`History deleted · ${bytes(r.bytes_freed)} freed. New history starts now.`);
      onDone();
    } catch (e) {
      toast(`Not deleted: ${toAppError(e).message}`);
    }
    setStage("idle");
  };
  return (
    <span className="confirm-row">
      <span className="muted">Delete all history? This cannot be undone.</span>
      <Button small variant="danger" disabled={stage === "busy"} onClick={() => void run()}>
        {stage === "busy" ? "Deleting…" : "Delete history"}
      </Button>
      <Button small disabled={stage === "busy"} onClick={() => setStage("idle")}>
        Cancel
      </Button>
    </span>
  );
}

export function SettingsData() {
  const toast = useToast();
  const { data: agent, error } = useLive(loadAgent, 60_000);
  const data = useLive(loadData, 600_000).data;
  const [keep, setKeep] = useState<KeepHistory | null>(null);
  const [version, setVersion] = useState(0);
  const current = keep ?? data?.keep_history;
  const change = async (next: KeepHistory) => {
    const before = current ?? null;
    setKeep(next);
    try {
      await setDataSettings({ keep_history: next });
      toast(next === "2y" ? "History is kept 2 years." : `History older than ${next === "90d" ? "90 days" : "1 year"} is deleted now and from now on.`);
      setVersion((v) => v + 1);
    } catch (e) {
      setKeep(before);
      toast(`Not saved: ${toAppError(e).message}`);
    }
  };
  return (
    <Panel title="Data retention" span={8} key={version}>
      <FieldRow label="Keep history for" description="Metrics and change history. Detail fades with age: 1-minute points for 7 days, 15-minute for 90 days, then daily">
        {current ? <Select value={current} options={KEEP} onChange={(v) => void change(v)} label="Keep history for" /> : <span className="muted">…</span>}
      </FieldRow>
      <FieldRow label="Database size" description={agent?.database ?? "~/.local/share/deecure/deecure.db"}>
        <span>{agent ? bytes(agent.database_bytes) : error ? "-" : "…"}</span>
      </FieldRow>
      <FieldRow label="Export report" description="The last 7 days as one HTML file in your Downloads folder, to keep or share">
        <ExportButton />
      </FieldRow>
      <FieldRow label="Delete history" description="Metrics, change history, reports, and notifications. Restore points (DeeCure's fixes) and settings are kept">
        <DeleteHistory onDone={() => setVersion((v) => v + 1)} />
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
