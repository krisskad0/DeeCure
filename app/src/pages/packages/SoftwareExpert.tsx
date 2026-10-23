// SPDX-License-Identifier: Apache-2.0
// Expert sections for software (docs/01-product/expert-coverage.md, section 13): one package in
// depth, software health (held, broken, and local packages, snap revisions and refreshes, Flatpak
// runtimes), and the full check of every installed file against its package.

import { useState } from "react";
import { type SoftwareHealth, type VerifyAnswer, getInventory, getPackageDetail } from "../../api";
import { Button } from "../../components/Button";
import { DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { bytes } from "../../metrics";
import { ago } from "../../time";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const loadHealth = () => getInventory("software_health");

/** The sources line in plain words: the archive and pocket, or "installed only". */
export function sourceText(src: string): string {
  if (src === "/var/lib/dpkg/status") return "this computer (no repository)";
  const m = src.match(/^(\S+)\s+(\S+)/);
  return m ? `${m[1].replace(/^https?:\/\//, "")} ${m[2]}` : src;
}

// ---------- Packages › Installed (drawer) ----------

export function PackageExpertDetail({ name }: { name: string }) {
  const { data: d, error } = useLive(() => getPackageDetail(name), 300_000, name);
  if (!d) return <NoData error={error} waiting="Loading details…" />;
  const newer = d.candidate && d.installed && d.candidate !== d.installed;
  return (
    <>
      <div className="field-label">Package</div>
      <KeyValueList
        rows={[
          ["Installed", d.installed ?? d.fields.Version ?? "-"],
          ["Newest offered", d.candidate ? (newer ? <StatusBadge key="c" variant="info" label={`${d.candidate} (update waiting)`} /> : d.candidate) : "-"],
          ["Installed by", d.manual ? "You" : "The system (as a dependency)"],
          ["Size on disk", d.fields["Installed-Size"] ? bytes(Number(d.fields["Installed-Size"]) * 1024, 0) : "-"],
          ["Files", d.files.toLocaleString()],
          ["Maintainer", d.fields.Maintainer ?? "-"],
          ["Home page", d.fields.Homepage ?? "-"],
        ]}
      />
      <div className="field-label">Where it can come from</div>
      <KeyValueList rows={d.sources.map(([v, srcs]): [string, string] => [v, srcs.map(sourceText).join("; ")])} />
      <div className="field-label">Needs ({d.needs.length}) · needed by ({d.needed_by.length})</div>
      <KeyValueList
        rows={[
          ["Needs", d.needs.join(", ") || "Nothing"],
          ["Needed by", d.needed_by.join(", ") || "No installed package"],
        ]}
      />
      <div className="field-label">Files changed from the package</div>
      {d.changed_files.length ? <KeyValueList rows={d.changed_files.map(([f, why]): [string, string] => [f, why])} /> : <p className="muted">None: every file matches the package.</p>}
    </>
  );
}

// ---------- Packages › Sources ----------

export function SoftwareHealthExpert() {
  const { data: h, error } = useLive<SoftwareHealth>(loadHealth, 300_000);
  if (!h) {
    return (
      <Panel title="Software health" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const old = (h.snaps ?? []).filter((s) => s.notes.includes("disabled"));
  const unused = (h.runtimes ?? []).filter((r) => !r.used);
  return (
    <>
      <Panel title="Package health" info="Held packages are never updated. Half-installed packages block updates until finished. Packages from no repository get no security updates: their repository was removed, or they were installed from a file." source="apt-mark showhold · dpkg --audit · apt list --installed" span={6} flush>
        <KeyValueList
          rows={[
            ["Held back from updates", h.held.length ? h.held.join(", ") : "None"],
            ["Half installed or removed", h.broken.length ? <StatusBadge key="b" variant="warn" label={`${h.broken.length} problem${h.broken.length === 1 ? "" : "s"}`} /> : "None"],
            ["From no repository", h.local.length ? <StatusBadge key="l" variant="info" label={`${h.local.length} package${h.local.length === 1 ? "" : "s"}: no updates`} /> : "None"],
          ]}
        />
        {h.local.length > 0 && (
          <DataTable
            rows={h.local.map(([name, version]) => ({ name, version }))}
            columns={[
              { key: "name", label: "Package from no repository", text: (p) => p.name },
              { key: "version", label: "Version", text: (p) => p.version },
            ]}
            rowKey={(p) => p.name}
            pageSize={8}
          />
        )}
      </Panel>
      <Panel title="Snap refreshes" source="snap refresh --time" span={6} flush>
        {h.snaps === undefined ? (
          <div className="empty muted">snapd is not running.</div>
        ) : (
          <KeyValueList
            rows={[
              ["Schedule", h.snap_refresh.timer ?? "-"],
              ["Last refresh", h.snap_refresh.last ?? "-"],
              ["Next refresh", h.snap_refresh.next ?? "-"],
              ...(h.snap_refresh.hold ? [["Held until", h.snap_refresh.hold] as [string, string]] : []),
              ["Installed snaps", String(new Set((h.snaps ?? []).map((s) => s.name)).size)],
              ["Old versions kept", old.length ? `${old.length} (removed by Storage › Cleanup)` : "None"],
            ]}
          />
        )}
      </Panel>
      {h.snaps && (
        <Panel title="Snap versions" info="Snap keeps old versions to go back to; they are marked old. Classic snaps can read and change everything your account can." source="snap list --all" span={12} flush>
          <DataTable
            rows={h.snaps}
            columns={[
              { key: "name", label: "Snap", text: (s) => s.name },
              { key: "version", label: "Version", text: (s) => s.version },
              { key: "rev", label: "Revision", text: (s) => s.revision, numeric: true },
              { key: "channel", label: "Channel", text: (s) => s.tracking ?? "-" },
              { key: "publisher", label: "Publisher", text: (s) => s.publisher ?? "-" },
              {
                key: "notes",
                label: "Notes",
                text: (s) => s.notes.join(", ") || "-",
                render: (s) =>
                  s.notes.includes("disabled") ? "Old version" : s.notes.includes("classic") ? <StatusBadge variant="info" label="Classic: no sandbox" /> : s.notes.join(", ") || "-",
              },
            ]}
            rowKey={(s) => `${s.name}|${s.revision}`}
            pageSize={15}
            search={{ placeholder: "Filter snaps" }}
          />
        </Panel>
      )}
      {h.runtimes && (
        <Panel title="Flatpak runtimes" subtitle={`${unused.length} not used`} info="A runtime counts as used when an installed app runs on it, or when it fills an extension point an app or its runtime declares (such as a graphics driver)." source="flatpak list · flatpak info --show-metadata" span={12} flush>
          <DataTable
            rows={h.runtimes}
            columns={[
              { key: "id", label: "Runtime", text: (r) => r.id, wrap: true },
              { key: "branch", label: "Branch", text: (r) => r.branch },
              { key: "inst", label: "Installed for", text: (r) => (r.installation === "user" ? "You" : "Everyone") },
              { key: "used", label: "Used", text: (r) => (r.used ? "Yes" : "No"), render: (r) => (r.used ? "Yes" : <StatusBadge variant="info" label="Not used by any app" />) },
            ]}
            rowKey={(r) => `${r.id}|${r.branch}|${r.installation}`}
            empty="No Flatpak runtimes"
          />
        </Panel>
      )}
    </>
  );
}

// ---------- Security › File Integrity ----------

export function FileCheck() {
  const [asked, setAsked] = useState(0);
  // The status is read every few seconds; the check itself starts only on the button.
  const { data: v, error } = useLive<VerifyAnswer>(() => getInventory("package_verify"), 5000, String(asked));
  const problems = v?.problems ?? [];
  const unreadable = problems.filter((p) => p.problem === "unreadable");
  const config = problems.filter((p) => p.config && p.problem !== "unreadable");
  const real = problems.filter((p) => !p.config && p.problem !== "unreadable");
  const start = () => {
    void getInventory("package_verify_run").finally(() => setAsked((n) => n + 1));
  };
  return (
    <Panel
      title="Every installed file against its package"
      subtitle={v?.finished_at ? `last checked ${ago(v.finished_at, Date.now())}${v.took_s !== undefined ? ` in ${v.took_s} s` : ""}` : undefined}
      info="Compares every file that packages installed with the checksum the package recorded. A changed program file can mean damage or tampering. Changed configuration files are normal: people edit them. Files only the admin may read are skipped without Full access."
      source="dpkg --verify"
      span={12}
      actions={
        <Button small disabled={!v || v.running} onClick={start}>
          {v?.running ? "Checking…" : v?.finished_at ? "Check again" : "Check all files"}
        </Button>
      }
      flush
    >
      {!v ? (
        <NoData error={error} waiting="Loading…" />
      ) : v.running ? (
        <div className="empty muted">Checking every file at low priority. This can take several minutes; you can leave this page.</div>
      ) : v.failed ? (
        <div className="empty muted">{v.failed}.</div>
      ) : v.finished_at === undefined ? (
        <div className="empty muted">Not checked yet. The check reads every installed file, so it runs only when you ask.</div>
      ) : (
        <>
          <p className="panel-pad">
            {real.length ? <StatusBadge variant="warn" label={`${real.length} program or data file${real.length === 1 ? "" : "s"} differ`} /> : <StatusBadge variant="ok" label="Every program and data file matches its package" />}{" "}
            <span className="muted">
              {config.length} configuration file{config.length === 1 ? "" : "s"} changed (normal)
              {unreadable.length ? ` · ${unreadable.length} file${unreadable.length === 1 ? "" : "s"} only the admin may read, not checked` : ""}
            </span>
          </p>
          <DataTable
            rows={[...real, ...config]}
            columns={[
              { key: "file", label: "File", text: (p) => p.file, wrap: true },
              { key: "problem", label: "Problem", text: (p) => (p.problem === "missing" ? "Missing" : "Changed") },
              { key: "kind", label: "Kind", text: (p) => (p.config ? "Configuration" : "Program or data") },
              { key: "package", label: "Package", text: (p) => p.package ?? "-" },
            ]}
            rowKey={(p) => p.file}
            pageSize={15}
            search={{ placeholder: "Filter files" }}
            empty="No differences"
          />
        </>
      )}
    </Panel>
  );
}
