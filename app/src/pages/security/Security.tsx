// SPDX-License-Identifier: Apache-2.0
// Security › Overview, Vulnerabilities, Hardening, File Integrity, Malware Scan
// (docs/03-ui-design/05-pages/18-security.md), with real data. All checks are read-only.

import { atLeast, useUi } from "../../ui";
import { SecurityExpertSections } from "./SecurityExpert";
import { FileCheck } from "../packages/SoftwareExpert";
import { type SecurityInfo, type UpdatesInfo, getInventory, getSnapshot } from "../../api";
import { AlertBanner } from "../../components/AlertBanner";
import { LinkButton } from "../../components/Button";
import { type Check, Checklist } from "../../components/Checklist";
import { type Column, DataTable } from "../../components/DataTable";
import { Icon } from "../../components/Icon";
import { KpiRow } from "../../components/KpiTile";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { Donut } from "../../components/charts/Donut";
import { href } from "../../route";
import { ago } from "../../time";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadSecurity = () => getInventory("security");
const loadUpdates = () => getSnapshot<UpdatesInfo>("updates");
const loadNet = () => getInventory("net_details");
const loadHardware = () => getInventory("hardware");
const loadStorage = () => getInventory("storage");
const loadAccounts = () => getInventory("accounts");
const loadOs = () => getInventory("os");

function Waiting({ title, error }: { title: string; error: Parameters<typeof NoData>[0]["error"] }) {
  return (
    <Panel title={title} span={12}>
      <NoData error={error} waiting="Running checks…" />
    </Panel>
  );
}

export function SecurityOverview() {
  const { mode } = useUi();
  const { data: sec, error } = useLive(loadSecurity, 300_000);
  const updates = useLive(loadUpdates, 300_000).data?.data;
  const net = useLive(loadNet, 300_000).data;
  const hw = useLive(loadHardware, 600_000).data;
  const storage = useLive(loadStorage, 600_000).data;
  const accounts = useLive(loadAccounts, 600_000).data;
  const os = useLive(loadOs, 600_000).data;
  if (!sec) return <Waiting title="Security posture" error={error} />;
  const security = updates?.pending.filter((p) => p.security).length;
  const failedHardening = sec.hardening.filter((h) => h.result === "fail").length;
  const ssh = accounts?.ssh;
  const checks: Check[] = [];
  const add = (ok: boolean | undefined, pass: string, fail: string, detail = "", warnOnly = false, action?: React.ReactNode) => {
    if (ok === undefined) return;
    checks.push({ state: ok ? "pass" : warnOnly ? "warn" : "fail", title: ok ? pass : fail, detail, action: ok ? undefined : action });
  };
  add(security === undefined ? undefined : security === 0, "Security updates installed", `${security} security updates waiting`, "", false, <LinkButton to={href("system", "updates")}>View</LinkButton>);
  add(updates?.automatic_security, "Automatic security updates on", "Automatic security updates off", "unattended-upgrades", true);
  add(net?.firewall.installed ? net.firewall.enabled : undefined, "Firewall enabled", "Firewall off", net?.firewall.enabled ? `ufw · incoming ${net.firewall.incoming === "DROP" ? "denied" : (net?.firewall.incoming ?? "")}` : "ufw is installed but off", true, <LinkButton to={href("network", "firewall")}>View</LinkButton>);
  add(hw?.machine.secure_boot, "Secure Boot enabled", "Secure Boot off", "", true);
  add(storage?.encrypted, "Disk encryption on", "Disk encryption off", storage?.encrypted ? "LUKS" : "Files are readable if the computer is stolen", true);
  add(sec.apparmor, "AppArmor enabled", "AppArmor off", "Limits what programs may do");
  add(sec.screen_lock, "Screen locks when idle", "Screen lock off", "", true);
  add(failedHardening === 0, "Kernel hardening settings at safe values", `${failedHardening} kernel settings could be safer`, "", true, <LinkButton to={href("security", "hardening")}>View</LinkButton>);
  add(ssh?.server_installed ? !ssh.password_login : undefined, "SSH accepts keys only", "SSH accepts passwords", "", true);
  add(sec.heuristics.every((h) => h.passed), "No signs of malware", "Some signs worth checking", "", true, <LinkButton to={href("security", "malware")}>View</LinkButton>);
  if (os?.livepatch) checks.push({ state: os.livepatch === "enabled" ? "pass" : "na", title: os.livepatch === "enabled" ? "Kernel Livepatch on" : "Kernel Livepatch off", detail: "Needs Ubuntu Pro" });
  const count = (s: Check["state"]) => checks.filter((c) => c.state === s).length;
  return (
    <>
      <KpiRow
        tiles={[
          { label: "Security updates", value: security === undefined ? "-" : String(security), status: security ? "warn" : "ok", href: href("system", "updates") },
          { label: "Firewall", value: !net ? "-" : !net.firewall.installed ? "Not installed" : net.firewall.enabled ? "Active" : "Off", status: net?.firewall.installed && !net.firewall.enabled ? "warn" : "ok" },
          { label: "Checks passing", value: `${count("pass")}/${checks.length}` },
        ]}
      />
      <Panel title="Security posture" span={8} flush>
        <Checklist checks={checks} />
      </Panel>
      <Panel title="Checks" span={4}>
        <Donut
          segments={[
            { name: "Pass", value: count("pass"), slot: 3 },
            { name: "Warning", value: count("warn"), tone: "warn" },
            { name: "Fail", value: count("fail"), tone: "crit" },
            { name: "Not available", value: count("na"), tone: "neutral" },
          ]}
          center={{ value: `${count("pass")}/${checks.length}`, subtitle: "passing" }}
        />
      </Panel>
      {atLeast(mode, "expert") && <SecurityExpertSections />}
    </>
  );
}

type Pending = UpdatesInfo["pending"][number];

export function SecurityVulns() {
  const { data: snapshot, error } = useLive(loadUpdates, 300_000);
  if (snapshot === undefined) return <Waiting title="Known vulnerabilities" error={error} />;
  const fixes = (snapshot?.data.pending ?? []).filter((p) => p.security);
  const columns: Column<Pending>[] = [
    { key: "name", label: "Package", text: (p) => p.name },
    { key: "severity", label: "Kind", text: () => "Security fix", render: () => <StatusBadge variant="warn" label="Security fix" /> },
    { key: "installed", label: "Installed", text: (p) => p.installed, render: (p) => <code>{p.installed}</code> },
    { key: "fixed", label: "Fixed in", text: (p) => p.candidate, render: (p) => <code>{p.candidate}</code> },
    { key: "origin", label: "From", text: (p) => p.origin, expert: true },
  ];
  return (
    <>
      <Panel title="Packages with security fixes waiting" subtitle="each one closes at least one known hole" source="apt list --upgradable" span={12} flush>
        <DataTable rows={fixes} columns={columns} rowKey={(p) => p.name} empty="No known vulnerabilities in installed packages" />
      </Panel>
      <AlertBanner tone="info" title="CVE numbers are not shown yet" text="Matching each fix to its CVE and severity needs the Ubuntu security database, which comes in a later version. Installing the updates fixes them either way." />
    </>
  );
}

type Hard = SecurityInfo["hardening"][number];
const HARD: Column<Hard>[] = [
  { key: "name", label: "Check", text: (h) => h.name },
  { key: "result", label: "Result", text: (h) => h.result, render: (h) => <StatusBadge variant={h.result === "pass" ? "ok" : h.result === "fail" ? "warn" : "off"} label={h.result === "pass" ? "Pass" : h.result === "fail" ? "Could be safer" : "Not available"} /> },
  { key: "value", label: "Value", text: (h) => h.value, render: (h) => <code>{h.value}</code> },
  { key: "rec", label: "Recommendation", text: (h) => h.recommendation || "-", wrap: true },
];

export function SecurityHardening() {
  const { data: sec, error } = useLive(loadSecurity, 300_000);
  if (!sec) return <Waiting title="Hardening checks" error={error} />;
  return (
    <Panel title="Hardening checks" source="/proc/sys" span={12} flush>
      <DataTable rows={sec.hardening} columns={HARD} rowKey={(h) => h.name} pageSize={20} />
    </Panel>
  );
}

type Suid = SecurityInfo["suid"][number];
const SUID: Column<Suid>[] = [
  { key: "path", label: "Path", text: (s) => s.path, render: (s) => <code>{s.path}</code> },
  { key: "owner", label: "Owner", text: (s) => s.owner },
  { key: "package", label: "Package", text: (s) => s.package ?? "None", render: (s) => s.package ?? <StatusBadge variant="warn" label="No package" /> },
  { key: "ok", label: "From a package", text: (s) => (s.package ? "Yes" : "No"), render: (s) => (s.package ? <Icon name="check" size={16} /> : <Icon name="excl" size={16} />) },
];

export function SecurityIntegrity() {
  const { data: sec, error } = useLive(loadSecurity, 300_000);
  if (!sec) return <Waiting title="Programs with SUID bit" error={error} />;
  const loose = sec.suid.filter((s) => !s.package).length;
  return (
    <>
      <KpiRow
        tiles={[
          { label: "SUID programs", value: String(sec.suid.length), subtext: "run with their owner's rights" },
          { label: "From a package", value: String(sec.suid.length - loose) },
          { label: "Not from a package", value: String(loose), status: loose ? "warn" : "ok" },
        ]}
      />
      <Panel title="Programs with SUID bit" source="file modes · dpkg -S" span={12} flush>
        <DataTable rows={sec.suid} columns={SUID} rowKey={(s) => s.path} pageSize={15} />
      </Panel>
      <FileCheck />
    </>
  );
}

export function SecurityMalware() {
  const { data: sec, error } = useLive(loadSecurity, 300_000);
  if (!sec) return <Waiting title="Heuristic checks" error={error} />;
  const checks: Check[] = sec.heuristics.map((h) => ({ state: h.passed ? "pass" : "warn", title: h.title, detail: h.detail }));
  const stale = sec.clamav_updated !== undefined && Date.now() - sec.clamav_updated > 7 * 86_400_000;
  return (
    <>
      {sec.scanners.length ? (
        <AlertBanner
          tone={stale ? "warn" : "ok"}
          title={`Scanner installed: ${sec.scanners.join(", ")}`}
          text={sec.clamav_updated ? `ClamAV signatures updated ${ago(sec.clamav_updated)}.${stale ? " Old signatures miss new malware; freshclam updates them." : ""}` : "Signature scans reduce risk but cannot prove a system is clean."}
        />
      ) : (
        <AlertBanner tone="info" title="No malware scanner installed" text="Optional on Ubuntu. Signature scans reduce risk but cannot prove a system is clean." />
      )}
      <Panel title="Heuristic checks" span={12} flush>
        <Checklist checks={checks} />
      </Panel>
    </>
  );
}
