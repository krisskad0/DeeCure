// SPDX-License-Identifier: Apache-2.0
// Account settings only the admin can read, through the helper (docs/01-product/expert-coverage.md):
// every sudo rule, and the SSH server's real settings (`sshd -T`) with the weak ones.

import type { ReactNode } from "react";
import { AdminRead } from "../../components/AdminRead";
import { DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";

export interface SudoRule {
  file: string;
  line: string;
  who: string;
  no_password: boolean;
  all_commands: boolean;
}
export interface SshServer {
  installed: boolean;
  settings?: Record<string, string>;
  weak?: { level: "critical" | "warning" | "info"; text: string }[];
}

function parsed<T>(text: string, view: (v: T) => ReactNode): ReactNode {
  try {
    return view(JSON.parse(text) as T);
  } catch {
    return <div className="empty muted">The helper gave no readable answer.</div>;
  }
}

/** What a rule means in a few words. */
export function sudoMeaning(r: SudoRule): { text: string; variant?: StatusVariant } {
  if (r.who.startsWith("Defaults")) return { text: "Setting" };
  if (/^[@#]include/.test(r.who)) return { text: "Reads more files" };
  if (/_Alias$/.test(r.who)) return { text: "Name for a list" };
  if (r.no_password && r.all_commands) return { text: "Everything, no password", variant: "warn" };
  if (r.all_commands) return { text: "Everything, with password" };
  if (r.no_password) return { text: "Some commands, no password", variant: "info" };
  return { text: "Some commands, with password" };
}

export function SudoRulesView({ data }: { data: SudoRule[] }) {
  const rules = data.filter((r) => !r.who.startsWith("Defaults") && !/^[@#]include/.test(r.who) && !/_Alias$/.test(r.who));
  const risky = rules.filter((r) => r.no_password && r.all_commands);
  return (
    <>
      <KeyValueList
        rows={[
          ["Rules that give admin rights", String(rules.length)],
          ["Everything without a password", risky.length ? <StatusBadge key="r" variant="warn" label={`${risky.length}: ${risky.map((r) => r.who).join(", ")}`} /> : "None"],
        ]}
      />
      <DataTable
        rows={data}
        columns={[
          { key: "who", label: "Who", text: (r) => r.who, render: (r) => <code>{r.who}</code> },
          {
            key: "means",
            label: "Allows",
            text: (r) => sudoMeaning(r).text,
            render: (r) => {
              const m = sudoMeaning(r);
              return m.variant ? <StatusBadge variant={m.variant} label={m.text} /> : m.text;
            },
          },
          { key: "line", label: "Rule", text: (r) => r.line, render: (r) => <code>{r.line}</code>, wrap: true },
          { key: "file", label: "File", text: (r) => r.file, expert: true },
        ]}
        rowKey={(r) => `${r.file}|${r.line}`}
        empty="No sudo rules"
      />
    </>
  );
}

const LEVEL: Record<string, StatusVariant> = { critical: "crit", warning: "warn", info: "info" };
const SHOWN = ["port", "listenaddress", "permitrootlogin", "passwordauthentication", "kbdinteractiveauthentication", "pubkeyauthentication", "permitemptypasswords", "maxauthtries", "allowusers", "allowgroups", "x11forwarding", "allowtcpforwarding"];

export function SshServerView({ data }: { data: SshServer }) {
  if (!data.installed) return <div className="empty muted">The SSH server is not installed, so nobody can sign in over the network.</div>;
  const s = data.settings ?? {};
  const weak = data.weak ?? [];
  return (
    <>
      {weak.length === 0 ? (
        <div className="empty">
          <StatusBadge variant="ok" label="No weak settings found" />
        </div>
      ) : (
        <DataTable
          rows={weak}
          columns={[
            { key: "level", label: "Level", text: (w) => w.level, render: (w) => <StatusBadge variant={LEVEL[w.level] ?? "info"} label={w.level} /> },
            { key: "text", label: "Weak setting", text: (w) => w.text, wrap: true },
          ]}
          rowKey={(w) => w.text}
        />
      )}
      <KeyValueList rows={SHOWN.filter((k) => s[k] !== undefined).map((k): [string, ReactNode] => [k, <code key={k}>{s[k]}</code>])} />
    </>
  );
}

export function SudoRulesPanel() {
  return (
    <Panel title="Who may use sudo" info="Every rule in /etc/sudoers and /etc/sudoers.d. A rule that allows everything without a password lets any app running as that person become admin." source="/etc/sudoers · /etc/sudoers.d" span={12} flush>
      <AdminRead what="sudo_rules" label="Read the sudo rules (asks for your password)">
        {(t) => parsed<SudoRule[]>(t, (v) => <SudoRulesView data={v} />)}
      </AdminRead>
    </Panel>
  );
}

export function SshServerPanel() {
  return (
    <Panel title="SSH server settings" info="The settings the SSH server really uses, after every included file and default, not a guess from the config file." source="sshd -T" span={12} flush>
      <AdminRead what="ssh_server" label="Check the real settings (asks for your password)">
        {(t) => parsed<SshServer>(t, (v) => <SshServerView data={v} />)}
      </AdminRead>
    </Panel>
  );
}
