// SPDX-License-Identifier: Apache-2.0
// Expert sections for security (docs/01-product/expert-coverage.md, section 14): whether automatic
// security updates are on and really install, and the AppArmor state of every running process.

import { type SecurityDetail, getInventory } from "../../api";
import { DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { ShowAll } from "../../components/ShowAll";
import { BarList } from "../../components/charts/BarList";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadDetail = () => getInventory("security_detail");

type Run = NonNullable<SecurityDetail["upgrade_runs"]>[number];

/** What the newest unattended-upgrades run means, in plain words. */
export function lastRunVerdict(runs: Run[] | undefined): { variant: StatusVariant; text: string } {
  const last = runs?.[0];
  if (!runs) return { variant: "off", text: "The log cannot be read" };
  if (!last) return { variant: "off", text: "No run in the log yet" };
  if (last.message.includes("metered")) return { variant: "warn", text: "Stopped: the connection is marked as metered, so nothing installs" };
  if (last.level === "ERROR") return { variant: "crit", text: `Failed: ${last.message}` };
  if (last.message.includes("All upgrades installed")) return { variant: "ok", text: "Installed the waiting security updates" };
  if (last.message.includes("No packages found")) return { variant: "ok", text: "Nothing to install" };
  return { variant: last.level === "WARNING" ? "warn" : "info", text: last.message };
}

const MODE: Record<string, [StatusVariant, string]> = {
  enforce: ["ok", "Enforced"],
  complain: ["info", "Warn only (logs, does not block)"],
  unconfined: ["info", "Profile that allows everything"],
  none: ["off", "No profile"],
};

export function SecurityExpertSections() {
  const { data: d, error } = useLive(loadDetail, 300_000);
  if (!d) {
    return (
      <Panel title="Automatic security updates" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const setting = (k: string) => d.periodic.find(([key]) => key === k)?.[1];
  const on = setting("Unattended-Upgrade") !== undefined && setting("Unattended-Upgrade") !== "0";
  const verdict = lastRunVerdict(d.upgrade_runs);
  const confined = d.confinement.filter((c) => c.mode === "enforce").reduce((n, c) => n + c.processes, 0);
  const total = d.confinement.reduce((n, c) => n + c.processes, 0);
  // Processes by how much AppArmor limits them, for one bar.
  const modes = [
    { label: "Enforced", cls: "ok", n: confined },
    { label: "Warn only", cls: "warn", n: d.confinement.filter((c) => c.mode === "complain").reduce((n, c) => n + c.processes, 0) },
    { label: "Allows everything", cls: "info", n: d.confinement.filter((c) => c.mode === "unconfined").reduce((n, c) => n + c.processes, 0) },
    { label: "No profile", cls: "off", n: d.confinement.filter((c) => c.mode === "none").reduce((n, c) => n + c.processes, 0) },
  ].filter((m) => m.n > 0);
  return (
    <>
      <Panel
        title="Automatic security updates"
        info="Ubuntu installs security updates by itself with unattended-upgrades. On a connection marked as metered (such as a phone hotspot) it stops, so fixes wait until you install them."
        source="/etc/apt/apt.conf.d · /var/log/unattended-upgrades"
        span={6}
        flush
      >
        <KeyValueList
          rows={[
            ["Turned on", !d.unattended_installed ? "unattended-upgrades is not installed" : on ? "Yes" : <StatusBadge key="o" variant="warn" label="No" />],
            ["Checks for updates", setting("Update-Package-Lists") && setting("Update-Package-Lists") !== "0" ? `Every ${setting("Update-Package-Lists") === "1" ? "day" : `${setting("Update-Package-Lists")} days`}` : "Off"],
            ["Last run", <StatusBadge key="r" variant={verdict.variant} label={verdict.text} />],
            ["When", d.upgrade_runs?.[0]?.at ?? "-"],
          ]}
        />
        {d.upgrade_runs && d.upgrade_runs.length > 1 && (
          <DataTable
            rows={d.upgrade_runs}
            columns={[
              { key: "at", label: "Run", text: (r) => r.at },
              { key: "msg", label: "What it said", text: (r) => r.message, wrap: true },
            ]}
            rowKey={(r) => `${r.at}|${r.message}`}
            pageSize={6}
          />
        )}
      </Panel>
      <Panel
        title="AppArmor by process"
        subtitle={`${confined} of ${total} processes enforced`}
        info="AppArmor limits what a program may do. Enforced profiles block what is not allowed; some apps (such as Chrome and VS Code on Ubuntu 24.04) have a profile that allows everything, which only lets them use their own sandbox."
        source="/proc/[pid]/attr/apparmor/current"
        span={6}
        flush
      >
        <div className="panel-pad">
          <div className="xp">
            <div className="xp-bar" role="img" aria-label={modes.map((m) => `${m.label} ${m.n}`).join(", ")}>
              {modes.map((m) => (
                <span key={m.label} className={`xp-${m.cls}`} style={{ flex: m.n }} title={`${m.label}: ${m.n} processes`} />
              ))}
            </div>
            <ul className="xp-legend">
              {modes.map((m) => (
                <li key={m.label}>
                  <i className={`xp-${m.cls}`} aria-hidden="true" />
                  {m.label} <b>{m.n}</b>
                </li>
              ))}
            </ul>
          </div>
          <div className="kn-head">Profiles with the most processes</div>
          <BarList
            rows={d.confinement
              .filter((c) => c.mode !== "none")
              .sort((a, b) => b.processes - a.processes)
              .slice(0, 6)
              .map((c) => ({ id: `${c.profile}|${c.mode}`, label: c.profile, hint: `${(MODE[c.mode] ?? ["off", c.mode])[1]} · such as ${c.names.slice(0, 3).join(", ")}`, value: c.processes, display: String(c.processes) }))}
          />
          <ShowAll label={`Show all ${d.confinement.length} profiles as a table`}>
            <DataTable
              rows={d.confinement}
              columns={[
                { key: "profile", label: "Profile", text: (c) => (c.profile === "unconfined" ? "None" : c.profile), wrap: true },
                { key: "mode", label: "Mode", text: (c) => (MODE[c.mode] ?? ["off", c.mode])[1], render: (c) => <StatusBadge variant={(MODE[c.mode] ?? ["off", c.mode])[0]} label={(MODE[c.mode] ?? ["off", c.mode])[1]} /> },
                { key: "n", label: "Processes", text: (c) => String(c.processes), numeric: true },
                { key: "names", label: "Such as", text: (c) => c.names.join(", "), wrap: true },
              ]}
              rowKey={(c) => `${c.profile}|${c.mode}`}
              pageSize={10}
            />
          </ShowAll>
        </div>
      </Panel>
    </>
  );
}
