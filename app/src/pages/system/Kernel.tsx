// SPDX-License-Identifier: Apache-2.0
// System › Kernel (docs/03-ui-design/05-pages/05-system.md), with real data.
// Removing old kernels goes through the fix engine and the admin helper. Expert replaces the
// module list with the richer one in KernelExpert.tsx, so modules show once.

import { Guard } from "../../components/ErrorBoundary";
import { atLeast, useUi } from "../../ui";
import { KernelExpertSections } from "./KernelExpert";
import { type KernelInfo, getInventory } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { Panel } from "../../components/Panel";
import { ShowAll } from "../../components/ShowAll";
import { BarList } from "../../components/charts/BarList";
import { StatusBadge, type StatusVariant } from "../../components/StatusBadge";
import { bytes } from "../../metrics";
import { useLive } from "../../useLive";
import { NoData } from "../NoData";
import { Button } from "../../components/Button";
import { useFix } from "../../shell/FixDrawer";

type Installed = KernelInfo["installed"][number];
type Module = KernelInfo["modules"][number];

const loadKernel = () => getInventory("kernel");
const STATUS: Record<Installed["status"], [StatusVariant, string]> = { running: ["ok", "Running"], fallback: ["info", "Fallback"], old: ["off", "Old"] };
const date = (ms?: number) => (ms ? new Date(ms).toLocaleDateString(undefined, { day: "numeric", month: "short", year: "numeric" }) : undefined);

const MODULES: Column<Module>[] = [
  { key: "name", label: "Module", text: (m) => m.name, render: (m) => <code>{m.name}</code> },
  { key: "size", label: "Size", text: (m) => bytes(m.size, 0), numeric: true },
  { key: "used", label: "Used by", text: (m) => (m.used_by.length ? m.used_by.join(", ") : String(m.used)), wrap: true },
];

/** "BOOT_IMAGE=/vmlinuz root=… ro quiet" → its parts, so each can be read on its own. */
export function commandLineParts(line: string): string[] {
  return line.split(/\s+/).filter(Boolean);
}

/** Who uses a module, in a few words. */
export function moduleUse(m: Module): string {
  if (m.used_by.length) return `used by ${m.used_by.slice(0, 3).join(", ")}${m.used_by.length > 3 ? ` +${m.used_by.length - 3}` : ""}`;
  return m.used ? `in use ${m.used} times` : "not in use";
}

export function SystemKernel() {
  const { mode } = useUi();
  const expert = atLeast(mode, "expert");
  const { data: k, error } = useLive(loadKernel, 300_000);
  const openFix = useFix();
  if (!k) {
    return (
      <Panel title="Running kernel" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const running = k.installed.find((i) => i.status === "running");
  const inBoot = k.installed.reduce((n, i) => n + i.size, 0);
  const biggest = [...k.modules].sort((a, b) => b.size - a.size).slice(0, 10);
  return (
    <>
      <Panel title="Running kernel" source="/proc/sys/kernel · /proc/cmdline" span={5}>
        <div className="kn-now">
          <code className="kn-version">{k.running}</code>
          <div className="kn-chips">
            <StatusBadge variant={k.tainted ? "warn" : "ok"} label={k.tainted ? `Tainted (${k.tainted})` : "Not tainted"} />
            <span className="kn-chip">{k.modules.length} modules loaded</span>
            {date(running?.installed_at) && <span className="kn-chip">installed {date(running?.installed_at)}</span>}
          </div>
          <div className="kn-label">Started with these options</div>
          <div className="kn-cmdline">
            {commandLineParts(k.command_line).map((part, i) => (
              <code key={`${i}-${part}`}>{part}</code>
            ))}
          </div>
        </div>
      </Panel>
      <Panel
        title="Installed kernels"
        subtitle={`${k.installed.length} · ${bytes(inBoot, 0)} in /boot`}
        info="The running kernel, the one before it (to start with if a new kernel has a problem), and older ones that can be removed."
        span={7}
        actions={k.installed.some((i) => i.status === "old") && <Button small onClick={() => openFix("system.remove_old_kernels")}>Remove old…</Button>}
      >
        {k.installed.length ? (
          <ul className="kn-list">
            {k.installed.map((i) => (
              <li key={i.version} className={`kn-item kn-${i.status}`}>
                <code className="kn-item-version">{i.version}</code>
                <StatusBadge variant={STATUS[i.status][0]} label={STATUS[i.status][1]} />
                <span className="kn-item-meta muted">
                  {bytes(i.size, 0)}
                  {date(i.installed_at) ? ` · installed ${date(i.installed_at)}` : ""}
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <div className="empty muted">No kernel images found in /boot</div>
        )}
      </Panel>
      {!expert && (
        <Panel title="Kernel modules" subtitle={`${k.modules.length} loaded · the 10 biggest`} info="Modules are drivers and features the kernel loaded for this computer's hardware." source="/proc/modules" span={12}>
          <BarList rows={biggest.map((m) => ({ id: m.name, label: m.name, hint: moduleUse(m), value: m.size, display: bytes(m.size, 0), mono: true }))} />
          <ShowAll label={`Show all ${k.modules.length} modules`}>
            <DataTable rows={k.modules} columns={MODULES} rowKey={(m) => m.name} search={{ placeholder: "Search modules" }} pageSize={12} total={k.modules.length} />
          </ShowAll>
        </Panel>
      )}
      {expert && <Guard><KernelExpertSections /></Guard>}
    </>
  );
}
