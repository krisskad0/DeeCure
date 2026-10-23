// SPDX-License-Identifier: Apache-2.0
// Activity › Timeline, Package History, Config Changes, Device Events, DeeCure Actions
// (docs/03-ui-design/05-pages/04-activity.md), from the history the agent saves
// (docs/02-architecture/collectors.md#history-events).

import { useState } from "react";
import { type EventKind, type HistoryEvent, getEvent, getEvents, toAppError, undoFix } from "../../api";
import { ActivityTimeline, type TimelineRow, type TimelineTone } from "../../components/ActivityTimeline";
import { AlertBanner } from "../../components/AlertBanner";
import { Button, LinkButton } from "../../components/Button";
import { type Column, DataTable } from "../../components/DataTable";
import { HowTo } from "../../components/HowTo";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { Donut } from "../../components/charts/Donut";
import { useKb } from "../../kb";
import { when } from "../../metrics";
import { href } from "../../route";
import { Drawer } from "../../shell/Drawer";
import { FixHistory } from "../../shell/FixHistory";
import { useToast } from "../../shell/Toast";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const DAY = 86_400_000;
const PACKAGE_KINDS: EventKind[] = ["package_installed", "package_upgraded", "package_removed"];
const loadWeek = () => getEvents(Date.now() - 7 * DAY);
const loadPackages = () => getEvents(Date.now() - 365 * DAY, { kinds: PACKAGE_KINDS });
const loadConfig = () => getEvents(Date.now() - 30 * DAY, { kinds: ["config_changed"] });
const loadDevices = () => getEvents(Date.now() - 30 * DAY, { kinds: ["device_added", "device_removed"] });

export type ChangeType = "Packages" | "Configuration" | "Devices" | "System" | "Services" | "Crashes" | "DeeCure";

export interface Change {
  at: number;
  type: ChangeType;
  title: string;
  detail: string;
  tone: TimelineTone;
  /** The events behind this row, newest first. */
  events: HistoryEvent[];
}

const TYPE: Record<EventKind, ChangeType> = {
  package_installed: "Packages",
  package_upgraded: "Packages",
  package_removed: "Packages",
  config_changed: "Configuration",
  device_added: "Devices",
  device_removed: "Devices",
  boot: "System",
  unclean_shutdown: "System",
  service_failed: "Services",
  crash: "Crashes",
  out_of_memory_kill: "Crashes",
  login_failed: "System",
  fix_applied: "DeeCure",
  fix_undone: "DeeCure",
};

const TONE: Partial<Record<EventKind, TimelineTone>> = {
  package_installed: "info",
  package_upgraded: "info",
  package_removed: "info",
  device_added: "ok",
  boot: "ok",
  unclean_shutdown: "warn",
  service_failed: "warn",
  crash: "crit",
  out_of_memory_kill: "warn",
  fix_applied: "info",
};

const list = (names: string[]) => `${names.slice(0, 3).join(", ")}${names.length > 3 ? ` +${names.length - 3} more` : ""}`;

/**
 * Turns events (newest first) into timeline rows: the packages of one apt run become one
 * row, and config files changed within a minute of each other become one row.
 */
export function groupChanges(events: HistoryEvent[]): Change[] {
  const out: Change[] = [];
  const runs = new Map<string, Change>();
  let lastConfig: Change | null = null;
  for (const e of events) {
    const type = TYPE[e.kind] ?? "System";
    if (type === "Packages" && e.data?.run !== undefined) {
      const key = `${e.data.run}|${e.source}`;
      const row = runs.get(key);
      if (row) {
        row.events.push(e);
        continue;
      }
      const change: Change = { at: e.at, type, title: e.title, detail: `Packages · ${e.source}`, tone: "info", events: [e] };
      runs.set(key, change);
      out.push(change);
      continue;
    }
    if (type === "Configuration") {
      if (lastConfig && Math.abs(lastConfig.events[lastConfig.events.length - 1].at - e.at) < 60_000) {
        lastConfig.events.push(e);
        continue;
      }
      lastConfig = { at: e.at, type, title: e.title, detail: `Configuration · ${e.source}`, tone: "default", events: [e] };
      out.push(lastConfig);
      continue;
    }
    let detail = `${type} · ${e.detail ?? e.source}`;
    if (e.kind === "boot" && e.data?.kernel_changed) detail = "System · new kernel";
    out.push({ at: e.at, type, title: e.title, detail, tone: e.kind === "boot" && e.data?.kernel_changed ? "info" : (TONE[e.kind] ?? "default"), events: [e] });
  }
  // Name each group by what it holds.
  for (const c of out) {
    if (c.events.length < 2) continue;
    if (c.type === "Packages") {
      const actions = new Set(c.events.map((e) => e.data?.action));
      const verb = actions.size === 1 ? ({ install: "Installed", upgrade: "Upgraded", remove: "Removed", purge: "Removed" } as Record<string, string>)[[...actions][0] ?? ""] ?? "Changed" : "Changed";
      c.title = `${verb} ${list(c.events.map((e) => e.subject ?? "?"))}`;
      if (c.events.some((e) => e.data?.failed)) {
        c.detail += " · did not finish";
        c.tone = "warn";
      }
    } else if (c.type === "Configuration") {
      c.title = `Changed ${list(c.events.map((e) => e.subject ?? "?"))}`;
      c.detail = `Configuration · ${c.events.length} files · ${c.events[0].source}`;
    }
  }
  return out;
}

/** The newest new kernel of the last 7 days: an installed kernel package or a boot that changed it. */
export function newKernel(events: HistoryEvent[]): HistoryEvent | undefined {
  return events.find((e) => (e.kind === "package_installed" && e.data?.kernel === true) || (e.kind === "boot" && e.data?.kernel_changed));
}

/** All changes of the last 7 days, newest first, or `undefined` while loading. */
export function useChanges(): Change[] | undefined {
  const week = useLive(loadWeek, 60_000).data;
  return week ? groupChanges(week.events) : undefined;
}

function UndoButton({ event }: { event: HistoryEvent }) {
  const toast = useToast();
  const [state, setState] = useState<"idle" | "busy" | "done">("idle");
  const id = event.data?.run_id;
  if (event.kind !== "fix_applied" || !event.data?.undoable || id === undefined) return null;
  if (state === "done") return <span className="muted">Undone</span>;
  const undo = async () => {
    setState("busy");
    try {
      await undoFix(id);
      setState("done");
      toast(`Undone · ${event.title}`);
    } catch (e) {
      setState("idle");
      toast(`Could not undo: ${toAppError(e).message}`);
    }
  };
  return (
    <Button small disabled={state === "busy"} onClick={() => void undo()}>
      {state === "busy" ? "Undoing…" : "Undo"}
    </Button>
  );
}

/** A config file's diff, loaded when the drawer opens. */
function DiffDrawer({ event, onClose }: { event: HistoryEvent; onClose: () => void }) {
  const loadOne = () => getEvent(event.id);
  const { data, error } = useLive(loadOne, 600_000, String(event.id));
  const d = data?.data;
  return (
    <Drawer title={event.subject ?? event.title} subtitle={`${when(event.at)} · ${event.source}`} onClose={onClose}>
      {!data ? (
        <NoData error={error} waiting="Loading…" />
      ) : d?.diff ? (
        <>
          <p className="muted">
            {d.added ?? 0} lines added, {d.removed ?? 0} removed
          </p>
          <pre className="diff">
            {d.diff.split("\n").map((line, i) => (
              <span key={i} className={line.startsWith("+ ") ? "diff-add" : line.startsWith("- ") ? "diff-del" : undefined}>
                {line}
                {"\n"}
              </span>
            ))}
          </pre>
        </>
      ) : (
        <p className="muted">
          {d?.first_scan
            ? "Changed before DeeCure started watching this file, so there is no earlier copy to compare with."
            : d?.change === "removed"
              ? "The file was removed."
              : "No line-by-line view: the file is too big, not text, or you cannot read it."}
        </p>
      )}
    </Drawer>
  );
}

function KernelTip({ event }: { event: HistoryEvent }) {
  const kb = useKb();
  const topic = kb?.topics?.find((t) => t.id === "topics/system/previous-kernel");
  return (
    <>
      <AlertBanner
        tone="info"
        title={`A new kernel arrived ${when(event.at)}`}
        text="If something stopped working since then (Wi-Fi, sound, graphics, sleep), start once with the previous kernel. If that fixes it, the new kernel is the cause."
        actions={<LinkButton to={href("system", "kernel")}>See kernels</LinkButton>}
      />
      {topic && (
        <Panel title="Start with the previous kernel" span={12}>
          <HowTo topic={topic} label="How to" />
        </Panel>
      )}
    </>
  );
}

export function ActivityTimelinePage() {
  const { data: week, error } = useLive(loadWeek, 60_000);
  const [open, setOpen] = useState<HistoryEvent | null>(null);
  if (!week) {
    return (
      <Panel title="Change timeline" span={12}>
        <NoData error={error} waiting="Collecting changes…" />
      </Panel>
    );
  }
  const changes = groupChanges(week.events);
  const by = (t: ChangeType) => changes.filter((c) => c.type === t).length;
  const kernel = newKernel(week.events);
  const action = (c: Change) => {
    const e = c.events[0];
    if (e.kind === "fix_applied") return <UndoButton event={e} />;
    if (c.type === "Configuration" && c.events.length === 1)
      return (
        <Button small onClick={() => setOpen(e)}>
          View
        </Button>
      );
    if (c.type === "Configuration") return <LinkButton to={href("activity", "config")}>Files</LinkButton>;
    return undefined;
  };
  const rows: TimelineRow[] = changes.slice(0, 60).map((c) => ({ time: when(c.at), title: c.title, detail: c.detail, tone: c.tone, action: action(c) }));
  const types: ChangeType[] = ["Packages", "Configuration", "Devices", "System", "Services", "Crashes", "DeeCure"];
  const segments = types
    .map((t) => ({ name: t, value: by(t) }))
    .filter((s) => s.value > 0)
    .map((s, i) => ({ ...s, slot: ((i % 5) + 1) as 1 | 2 | 3 | 4 | 5 }));
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Changes", value: String(changes.length), subtext: "last 7 days" },
          { label: "Package changes", value: String(by("Packages")) },
          { label: "Configuration changes", value: String(by("Configuration")) },
          { label: "Crashes", value: String(by("Crashes")), status: by("Crashes") ? "warn" : "ok" },
          { label: "DeeCure actions", value: String(by("DeeCure")) },
        ]}
      />
      {kernel && <KernelTip event={kernel} />}
      <Panel title="Change timeline" subtitle={week.truncated ? "newest changes only" : undefined} info="What changed on this computer, newest first. Something broke? Look at what changed just before." span={8} flush>
        {rows.length ? <ActivityTimeline rows={rows} /> : <div className="empty muted">No changes in the last 7 days.</div>}
      </Panel>
      <Panel title="Changes by type" subtitle="7 days" span={4}>
        <Donut segments={segments.length ? segments : [{ name: "None", value: 0, tone: "neutral" }]} center={{ value: String(changes.length), subtitle: "changes" }} />
      </Panel>
      {open && <DiffDrawer event={open} onClose={() => setOpen(null)} />}
    </>
  );
}

type Action = "install" | "upgrade" | "downgrade" | "remove" | "purge" | "reinstall";
const ACTION: Record<Action, StatusVariant> = { install: "ok", upgrade: "info", remove: "warn", purge: "warn", downgrade: "warn", reinstall: "off" };
const word = (a: string) => a[0].toUpperCase() + a.slice(1);
const actionOf = (e: HistoryEvent): Action => e.data?.action ?? (e.kind === "package_installed" ? "install" : e.kind === "package_removed" ? "remove" : "upgrade");
const FORMAT: Record<string, string> = { deb: "deb", snap: "snap", flatpak: "Flatpak" };

const PACKAGES: Column<HistoryEvent>[] = [
  { key: "date", label: "Date", text: (e) => String(e.at), render: (e) => when(e.at) },
  { key: "action", label: "Action", text: (e) => word(actionOf(e)), render: (e) => <StatusBadge variant={ACTION[actionOf(e)] ?? "info"} label={word(actionOf(e))} /> },
  { key: "name", label: "Package", text: (e) => e.subject ?? "" },
  { key: "format", label: "Type", text: (e) => FORMAT[e.data?.format ?? "deb"] ?? "deb" },
  { key: "from", label: "From", text: (e) => e.data?.from ?? "-", render: (e) => (e.data?.from ? <code>{e.data.from}</code> : "-"), expert: true },
  { key: "to", label: "To", text: (e) => e.data?.to ?? "-", render: (e) => (e.data?.to ? <code>{e.data.to}</code> : "-") },
  { key: "source", label: "Source", text: (e) => e.source },
];

export function ActivityPackages() {
  const { data, error } = useLive(loadPackages, 300_000);
  return (
    <Panel title="Package history" subtitle="last 12 months" source="/var/log/apt/history.log · /var/log/dpkg.log · snap · flatpak" span={12} flush>
      {data ? (
        <DataTable rows={data.events} columns={PACKAGES} rowKey={(e) => String(e.id)} search={{ placeholder: "Search packages" }} filter={{ column: "action", options: ["Install", "Upgrade", "Remove"] }} pageSize={14} empty="No package changes yet. DeeCure reads the package logs every few minutes." />
      ) : (
        <NoData error={error} waiting="Loading…" />
      )}
    </Panel>
  );
}

const CHANGE: Record<string, [StatusVariant, string]> = { added: ["ok", "Added"], changed: ["info", "Changed"], removed: ["warn", "Removed"] };

export function ActivityConfig() {
  const { data, error } = useLive(loadConfig, 120_000);
  const [open, setOpen] = useState<HistoryEvent | null>(null);
  const columns: Column<HistoryEvent>[] = [
    { key: "time", label: "Time", text: (e) => String(e.at), render: (e) => when(e.at) },
    { key: "file", label: "File", text: (e) => e.subject ?? e.title, render: (e) => <code className="wrap-anywhere">{e.subject ?? e.title}</code>, wrap: true },
    { key: "change", label: "Change", text: (e) => CHANGE[e.data?.change ?? "changed"]?.[1] ?? "Changed", render: (e) => <StatusBadge variant={CHANGE[e.data?.change ?? "changed"]?.[0] ?? "info"} label={`${CHANGE[e.data?.change ?? "changed"]?.[1] ?? "Changed"}${(e.data?.count ?? 1) > 1 ? ` ×${e.data?.count}` : ""}`} /> },
    { key: "lines", label: "Lines", text: (e) => (e.data?.added !== undefined ? `+${e.data.added} −${e.data.removed ?? 0}` : "-"), expert: true },
    { key: "who", label: "Changed by", text: (e) => e.source },
    {
      key: "view",
      label: "",
      text: () => "",
      render: (e) => (
        <Button small onClick={() => setOpen(e)}>
          View
        </Button>
      ),
    },
  ];
  return (
    <Panel title="Configuration changes" subtitle="last 30 days" source="/etc · ~/.config/autostart · ~/.config/systemd/user · crontab" span={12} flush>
      {data ? <DataTable rows={data.events} columns={columns} rowKey={(e) => String(e.id)} search={{ placeholder: "Search files" }} filter={{ column: "change", options: ["Added", "Changed", "Removed"] }} pageSize={14} empty="No configuration files changed in the last 30 days" /> : <NoData error={error} waiting="Loading…" />}
      {open && <DiffDrawer event={open} onClose={() => setOpen(null)} />}
    </Panel>
  );
}

const DEVICES: Column<HistoryEvent>[] = [
  { key: "time", label: "Time", text: (e) => String(e.at), render: (e) => when(e.at) },
  { key: "event", label: "Event", text: (e) => (e.kind === "device_added" ? "Connected" : "Disconnected"), render: (e) => <StatusBadge variant={e.kind === "device_added" ? "ok" : "off"} label={`${e.kind === "device_added" ? "Connected" : "Disconnected"}${(e.data?.count ?? 1) > 1 ? ` ×${e.data?.count}` : ""}`} /> },
  { key: "device", label: "Device", text: (e) => e.subject ?? e.title },
  { key: "bus", label: "Bus", text: (e) => e.data?.bus ?? "USB" },
  { key: "id", label: "ID", text: (e) => e.data?.id ?? "-", render: (e) => (e.data?.id ? <code>{e.data.id}</code> : "-") },
];

export function ActivityDevices() {
  const { data, error } = useLive(loadDevices, 120_000);
  return (
    <Panel title="Device events" subtitle="last 30 days" source="kernel log" span={12} flush>
      {data ? <DataTable rows={data.events} columns={DEVICES} rowKey={(e) => String(e.id)} empty="No USB devices were plugged in or out" /> : <NoData error={error} waiting="Loading…" />}
    </Panel>
  );
}

export function ActivityDeeCure() {
  return (
    <Panel title="Remediation log" info="Every change DeeCure made, with undo where possible." span={12} flush>
      <FixHistory />
    </Panel>
  );
}
