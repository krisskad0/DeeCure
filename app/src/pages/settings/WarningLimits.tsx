// SPDX-License-Identifier: Apache-2.0
// Settings › Notifications › Warnings and limits (docs/01-product/health-warnings.md): turn
// any warning off, or change its limits. DeeCure's default is shown next to each value, and
// the agent keeps every value inside the limit's range.

import { useState } from "react";
import { type WarningRule, type WarningSettings, getWarningSettings, setWarningSettings, toAppError } from "../../api";
import { Button } from "../../components/Button";
import { Switch } from "../../components/FormControls";
import { Panel } from "../../components/Panel";
import { useToast } from "../../shell/Toast";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";

const load = () => getWarningSettings();

const AREAS: Record<string, string> = {
  hardware: "Hardware and temperatures",
  performance: "Memory and disk speed",
  storage: "Disks",
  power: "Battery",
  system: "System",
  services: "Services",
  packages: "Packages",
  network: "Network",
  security: "Security",
  accounts: "Accounts",
  backups: "Backups",
  containers: "Containers",
  jobs: "Scheduled jobs",
  privacy: "Privacy",
};

/** The choices to save: warnings turned off, and limits that differ from their default. */
export function toSettings(rules: WarningRule[]): WarningSettings {
  const limits: WarningSettings["limits"] = {};
  for (const r of rules) {
    for (const l of r.limits) {
      if (l.value !== l.default) (limits[r.id] ??= {})[l.name] = l.value;
    }
  }
  return { off: rules.filter((r) => r.off).map((r) => r.id), limits };
}

export function WarningLimits() {
  const toast = useToast();
  const { data, error } = useLive(load, 300_000);
  const [rules, setRules] = useState<WarningRule[] | null>(null);
  const [draft, setDraft] = useState<Record<string, string>>({});
  const current = rules ?? data;
  if (!current) {
    return (
      <Panel title="Warnings and limits" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const save = async (next: WarningRule[]) => {
    const before = current;
    setRules(next);
    try {
      setRules(await setWarningSettings(toSettings(next)));
      setDraft({});
    } catch (e) {
      setRules(before);
      toast(`Not saved: ${toAppError(e).message}`);
    }
  };
  const update = (id: string, change: (r: WarningRule) => WarningRule) => void save(current.map((r) => (r.id === id ? change(r) : r)));
  const setLimit = (id: string, name: string, value: number) =>
    update(id, (r) => ({ ...r, limits: r.limits.map((l) => (l.name === name ? { ...l, value: Math.min(l.max, Math.max(l.min, value)) } : l)) }));
  const areas = [...new Set(current.map((r) => r.category))].sort((a, b) => Object.keys(AREAS).indexOf(a) - Object.keys(AREAS).indexOf(b));
  return (
    <Panel
      title="Warnings and limits"
      info="DeeCure warns about harm, not about use: a busy CPU or full RAM cache never warns. Turn off any warning, or change when it fires. Each limit stays inside a safe range."
      span={12}
    >
      {areas.map((area) => (
        <section key={area} className="warning-area">
          <h3 className="warning-area-title">{AREAS[area] ?? area}</h3>
          {current
            .filter((r) => r.category === area)
            .map((r) => (
              <div key={r.id} className="warning-row">
                <span className="warning-name">
                  <Switch checked={!r.off} onChange={(on) => update(r.id, (x) => ({ ...x, off: !on }))} label={r.name} />
                  <span>{r.name}</span>
                </span>
                <span className="warning-limits">
                  {r.limits.map((l) => {
                    const key = `${r.id}/${l.name}`;
                    const shown = draft[key] ?? String(l.value);
                    const commit = () => {
                      const n = Number(draft[key]);
                      if (draft[key] !== undefined && Number.isFinite(n) && n !== l.value) setLimit(r.id, l.name, n);
                      else setDraft(({ [key]: _, ...rest }) => rest);
                    };
                    return (
                      <label key={l.name} className="warning-limit">
                        <span className="muted">{l.label}</span>
                        <input
                          className="input-num"
                          type="number"
                          min={l.min}
                          max={l.max}
                          disabled={r.off}
                          value={shown}
                          aria-label={`${r.name}: ${l.label}`}
                          onChange={(e) => setDraft({ ...draft, [key]: e.target.value })}
                          onBlur={commit}
                          onKeyDown={(e) => e.key === "Enter" && commit()}
                        />
                        <span>{l.unit}</span>
                        {l.value !== l.default ? (
                          <Button small variant="link" onClick={() => setLimit(r.id, l.name, l.default)}>
                            Use default ({l.default})
                          </Button>
                        ) : (
                          <span className="muted">default</span>
                        )}
                      </label>
                    );
                  })}
                </span>
              </div>
            ))}
        </section>
      ))}
    </Panel>
  );
}
