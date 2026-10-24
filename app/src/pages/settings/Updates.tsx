// SPDX-License-Identifier: Apache-2.0
// Settings › About › Updates (docs/02-architecture/updates.md): how often DeeCure looks for a
// new release, which releases, and whether it installs them by itself. Installing always goes
// through the fix preview: the root helper downloads the release from GitHub, checks every
// package, and installs it with apt.

import { useState } from "react";
import { type UpdateSettings, type UpdateStatus, checkUpdates, getUpdateStatus, setUpdateSettings, toAppError } from "../../api";
import { AlertBanner } from "../../components/AlertBanner";
import { Button } from "../../components/Button";
import { FieldRow, Select } from "../../components/FormControls";
import { Panel } from "../../components/Panel";
import { when } from "../../metrics";
import { useFix } from "../../shell/FixDrawer";
import { useToast } from "../../shell/Toast";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const load = () => getUpdateStatus();

const CHECK = [
  { value: "daily", label: "Every day" },
  { value: "weekly", label: "Every week" },
  { value: "off", label: "Never" },
] as const;
const CHANNEL = [
  { value: "prerelease", label: "Pre-releases too" },
  { value: "stable", label: "Stable releases only" },
] as const;
const INSTALL = [
  { value: "ask", label: "Ask me first" },
  { value: "auto", label: "Automatically" },
] as const;

/** The status line under the version. */
export function updateLine(s: UpdateStatus): string {
  if (s.from_apt) return "DeeCure comes from its APT repository: Software Updater installs its updates.";
  if (s.available && s.latest) return `DeeCure ${s.latest.version} is available.`;
  if (s.error) return `The last check failed: ${s.error}.`;
  if (!s.checked_at) return s.settings.check === "off" ? "Update checks are off." : "Not checked yet.";
  return "DeeCure is up to date.";
}

export function Updates() {
  const openFix = useFix();
  const toast = useToast();
  const { data, error } = useLive(load, 300_000);
  const [status, setStatus] = useState<UpdateStatus | null>(null);
  const [checking, setChecking] = useState(false);
  const s = status ?? data;
  if (!s) {
    return (
      <Panel title="Updates" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const save = async (next: UpdateSettings) => {
    const before = s;
    setStatus({ ...s, settings: next });
    try {
      const saved = await setUpdateSettings(next);
      setStatus(saved);
      // Automatic updates live in a root timer: setting them up asks for the password once,
      // and so does a change of how often or which releases while they are on.
      if (next.auto !== before.settings.auto || (next.auto && (next.check !== before.settings.check || next.channel !== before.settings.channel))) {
        if (next.auto || saved.auto_ready) openFix("system.set_auto_update", next.auto ? "on" : "off");
      }
    } catch (e) {
      setStatus(before);
      toast(`Not saved: ${toAppError(e).message}`);
    }
  };
  const check = async () => {
    setChecking(true);
    try {
      setStatus(await checkUpdates());
    } catch (e) {
      toast(`Could not check: ${toAppError(e).message}`);
    }
    setChecking(false);
  };
  const st = s.settings;
  return (
    <Panel
      title="Updates"
      info="Checking asks GitHub for DeeCure's release list. That is the only time DeeCure contacts the internet, and nothing about your computer is sent."
      span={12}
    >
      <FieldRow label={`DeeCure ${s.current}`} description={`${updateLine(s)}${s.checked_at ? ` Last checked ${when(s.checked_at)}.` : ""}`}>
        <span className="btn-row">
          <Button small disabled={checking || s.from_apt} onClick={() => void check()}>
            {checking ? "Checking…" : "Check now"}
          </Button>
          {s.available && s.latest && !s.from_apt && (
            <Button small variant="primary" onClick={() => openFix("system.update_deecure", s.latest?.version)}>
              Install {s.latest.version}
            </Button>
          )}
        </span>
      </FieldRow>
      {s.available && s.latest && (
        <FieldRow label="What is new" description={s.latest.notes.split("\n").find((l) => l.trim()) ?? "See the release page."}>
          <a href={s.latest.url} target="_blank" rel="noreferrer">
            Release page
          </a>
        </FieldRow>
      )}
      <FieldRow label="Check for updates" description="Daily is the default. Never means DeeCure does not contact the internet at all.">
        <Select value={st.check} options={CHECK} onChange={(v) => void save({ ...st, check: v, auto: v === "off" ? false : st.auto })} label="Check for updates" />
      </FieldRow>
      <FieldRow label="Releases" description="All releases so far are pre-releases, so stable only finds nothing yet.">
        <Select value={st.channel} options={CHANNEL} onChange={(v) => void save({ ...st, channel: v })} label="Releases" />
      </FieldRow>
      <FieldRow label="Install updates" description="Automatic: a small timer installs new releases with the same checks, not on a metered connection, and you get a notification afterwards. Setting it up asks for your password once.">
        <Select
          value={st.auto ? "auto" : "ask"}
          options={st.check === "off" ? INSTALL.filter((o) => o.value === "ask") : INSTALL}
          onChange={(v) => void save({ ...st, auto: v === "auto" })}
          label="Install updates"
        />
      </FieldRow>
      {st.auto && !s.auto_ready && (
        <AlertBanner
          tone="warn"
          title="Automatic updates are not set up yet"
          text="They need your password once, so the timer may install them."
          actions={
            <Button small onClick={() => openFix("system.set_auto_update", "on")}>
              Set up
            </Button>
          }
        />
      )}
      {s.last_auto && <FieldRow label="Last automatic update" description={s.last_auto}>{null}</FieldRow>}
    </Panel>
  );
}
