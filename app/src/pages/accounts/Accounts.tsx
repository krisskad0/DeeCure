// SPDX-License-Identifier: Apache-2.0
// Accounts › Users, Groups, Sessions, Login History, SSH Keys
// (docs/03-ui-design/05-pages/17-accounts.md), with real data. Password hashes are never read.

import { type AccountsInfo, getInventory } from "../../api";
import { AlertBanner } from "../../components/AlertBanner";
import { type Column, DataTable } from "../../components/DataTable";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { BarChart } from "../../components/charts/BarChart";
import { when } from "../../metrics";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

type User = AccountsInfo["users"][number];
type Group = AccountsInfo["groups"][number];
type Session = AccountsInfo["sessions"][number];
type Event = AccountsInfo["events"][number];
type Key = AccountsInfo["ssh"]["client_keys"][number];

const loadAccounts = () => getInventory("accounts");
const useAccounts = () => useLive(loadAccounts, 60_000);

function Waiting({ title, error }: { title: string; error: Parameters<typeof NoData>[0]["error"] }) {
  return (
    <Panel title={title} span={12}>
      <NoData error={error} waiting="Loading…" />
    </Panel>
  );
}

const tags = (list: string[]) => <span className="tags">{list.map((g) => <span key={g} className="tag">{g}</span>)}</span>;

const USERS: Column<User>[] = [
  { key: "user", label: "User", text: (u) => `${u.name} ${u.full_name}`, render: (u) => (<span className="cell-two"><b>{u.name}</b><span>{u.full_name || "-"}</span></span>) },
  { key: "uid", label: "UID", text: (u) => String(u.uid), numeric: true },
  { key: "groups", label: "Groups", text: (u) => u.groups.join(" "), render: (u) => tags(u.groups.slice(0, 6)), wrap: true },
  { key: "role", label: "Role", text: (u) => (u.admin ? "Administrator" : "Standard"), render: (u) => <StatusBadge variant={u.admin ? "info" : "off"} label={u.uid === 0 ? "System admin (root)" : u.admin ? "Administrator" : "Standard"} /> },
  { key: "shell", label: "Shell", text: (u) => u.shell, render: (u) => <code>{u.shell}</code>, expert: true },
  { key: "last", label: "Last sign-in", text: (u) => (u.last_sign_in ? String(u.last_sign_in) : "-"), render: (u) => (u.last_sign_in ? when(u.last_sign_in) : <span className="muted">not in records</span>) },
];

export function AccountsUsers() {
  const { data: a, error } = useAccounts();
  if (!a) return <Waiting title="Users" error={error} />;
  const people = a.users.filter((u) => u.uid !== 0);
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Human users", value: String(people.length) },
          { label: "Administrators", value: String(people.filter((u) => u.admin).length) },
          { label: "Signed in now", value: String(new Set(a.sessions.filter((s) => s.class === "user").map((s) => s.user)).size) },
        ]}
      />
      <Panel title="Users" source="/etc/passwd · /etc/group · last" span={12} flush>
        <DataTable rows={a.users} columns={USERS} rowKey={(u) => u.name} />
      </Panel>
      <AlertBanner tone="info" title="Password strength cannot be checked" text="Linux stores only salted hashes, and DeeCure never reads them." />
    </>
  );
}

const GROUPS: Column<Group>[] = [
  { key: "name", label: "Group", text: (g) => g.name },
  { key: "gid", label: "GID", text: (g) => String(g.gid), numeric: true },
  { key: "members", label: "Members", text: (g) => g.members.join(", ") },
  { key: "privilege", label: "Privilege", text: (g) => g.privilege ?? "-", render: (g) => (g.privilege ? <StatusBadge variant={/Admin|Root/.test(g.privilege) ? "warn" : "info"} label={g.privilege} /> : <span className="muted">-</span>) },
];

export function AccountsGroups() {
  const { data: a, error } = useAccounts();
  if (!a) return <Waiting title="Groups with members" error={error} />;
  return (
    <Panel title="Groups with members" source="/etc/group" span={12} flush>
      <DataTable rows={a.groups} columns={GROUPS} rowKey={(g) => g.name} search={{ placeholder: "Filter groups" }} />
    </Panel>
  );
}

const SESSIONS: Column<Session>[] = [
  { key: "id", label: "Session", text: (s) => s.id, render: (s) => <code>{s.id}</code> },
  { key: "user", label: "User", text: (s) => s.user },
  { key: "seat", label: "Seat", text: (s) => s.seat || "-" },
  { key: "type", label: "Type", text: (s) => `${s.kind}${s.remote ? " (remote)" : ""}` },
  { key: "since", label: "Since", text: (s) => (s.since ? when(s.since) : "-") },
  { key: "state", label: "State", text: (s) => s.state, render: (s) => <StatusBadge variant={s.state === "active" ? "ok" : "off"} label={s.state} /> },
];

export function AccountsSessions() {
  const { data: a, error } = useAccounts();
  if (!a) return <Waiting title="Active sessions" error={error} />;
  return (
    <Panel title="Active sessions" source="logind" span={12} flush>
      <DataTable rows={a.sessions} columns={SESSIONS} rowKey={(s) => s.id} empty="No sessions (logind not reachable)" />
    </Panel>
  );
}

const EVENTS: Column<Event>[] = [
  { key: "time", label: "Time", text: (e) => String(e.at), render: (e) => when(e.at) },
  { key: "user", label: "User", text: (e) => e.user },
  { key: "event", label: "Event", text: (e) => (e.event === "sudo" ? "sudo" : "Sign-in") },
  { key: "source", label: "Source", text: (e) => e.source, render: (e) => <code className="wrap-anywhere">{e.source}</code>, wrap: true },
  { key: "result", label: "Result", text: (e) => (e.success ? "Success" : "Failed"), render: (e) => <StatusBadge variant={e.success ? "ok" : "crit"} label={e.success ? "Success" : "Failed"} /> },
];

export function AccountsLogins() {
  const { data: a, error } = useAccounts();
  if (!a) return <Waiting title="Events" error={error} />;
  const day = new Date();
  day.setHours(0, 0, 0, 0);
  const bars = Array.from({ length: 7 }, (_, i) => {
    const start = day.getTime() - (6 - i) * 86_400_000;
    return { label: new Date(start + 43_200_000).toLocaleDateString(undefined, { weekday: "short" }), value: a.events.filter((e) => e.event === "sign-in" && e.at >= start && e.at < start + 86_400_000).length };
  });
  const failed = a.events.filter((e) => !e.success).length;
  return (
    <>
      <Panel title="Sign-ins per day" subtitle="last 7 days" span={5}>
        <BarChart bars={bars} height={180} />
        {failed > 0 && <p className="muted">{failed} failed sudo attempts in 30 days.</p>}
      </Panel>
      <Panel title="Events" subtitle="last 30 days" source="last · journal (sudo)" span={7} flush>
        <DataTable rows={a.events} columns={EVENTS} rowKey={(e) => `${e.at}|${e.event}|${e.source}`} pageSize={10} empty="No sign-ins or sudo use recorded" />
      </Panel>
    </>
  );
}

const KEYS: Column<Key>[] = [
  { key: "file", label: "File", text: (k) => k.file, render: (k) => <code>{k.file}</code> },
  { key: "type", label: "Type", text: (k) => `${k.kind} ${k.bits}`, render: (k) => `${k.kind}${k.kind === "RSA" ? ` ${k.bits}` : ""}` },
  { key: "fp", label: "Fingerprint", text: (k) => k.fingerprint, render: (k) => <code>{k.fingerprint.length > 24 ? `${k.fingerprint.slice(0, 14)}…${k.fingerprint.slice(-6)}` : k.fingerprint}</code> },
  { key: "created", label: "Created", text: (k) => String(k.created), render: (k) => new Date(k.created).toLocaleDateString(undefined, { day: "numeric", month: "short", year: "numeric" }) },
];

export function AccountsSsh() {
  const { data: a, error } = useAccounts();
  if (!a) return <Waiting title="SSH" error={error} />;
  const s = a.ssh;
  const weakRsa = s.client_keys.filter((k) => k.kind === "RSA" && k.bits < 3072).length;
  return (
    <>
      <KpiRow
        tiles={[
          { label: "OpenSSH server", value: s.server_installed ? "Installed" : "Not installed", subtext: s.server_installed ? `password sign-in ${s.password_login ? "on" : "off"}` : "" },
          { label: "Authorized keys", value: String(s.authorized_keys), subtext: "can sign in to this account" },
          { label: "Client keys", value: String(s.client_keys.length), status: weakRsa ? "warn" : "ok", subtext: [...new Set(s.client_keys.map((k) => k.kind.toLowerCase()))].join(" · ") },
        ]}
      />
      {s.server_installed && s.password_login && <AlertBanner tone="warn" title="SSH allows password sign-in" text="Keys are much harder to guess than passwords. Turn off PasswordAuthentication once your keys work." />}
      <Panel title="Client keys" subtitle="~/.ssh" source="ssh-keygen -lf" span={12} flush>
        <DataTable rows={s.client_keys} columns={KEYS} rowKey={(k) => k.file} empty="No SSH keys in ~/.ssh" />
      </Panel>
    </>
  );
}

