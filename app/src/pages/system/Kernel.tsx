// SPDX-License-Identifier: Apache-2.0
// System › Kernel (docs/03-ui-design/05-pages/05-system.md), with real data.
// Removing old kernels needs the helper (Phase 2).

import { type KernelInfo, getInventory } from "../../api";
import { type Column, DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
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

const KERNELS: Column<Installed>[] = [
  { key: "version", label: "Version", text: (k) => k.version, render: (k) => <code>{k.version}</code> },
  { key: "status", label: "Status", text: (k) => STATUS[k.status][1], render: (k) => <StatusBadge variant={STATUS[k.status][0]} label={STATUS[k.status][1]} /> },
  { key: "size", label: "Size", text: (k) => bytes(k.size, 0), numeric: true },
  { key: "installed", label: "Installed", text: (k) => (k.installed_at ? new Date(k.installed_at).toLocaleDateString(undefined, { day: "numeric", month: "short", year: "numeric" }) : "—") },
];

const MODULES: Column<Module>[] = [
  { key: "name", label: "Module", text: (m) => m.name, render: (m) => <code>{m.name}</code> },
  { key: "size", label: "Size", text: (m) => bytes(m.size, 0), numeric: true },
  { key: "used", label: "Used by", text: (m) => (m.used_by.length ? m.used_by.join(", ") : String(m.used)), wrap: true },
];

export function SystemKernel() {
  const { data: k, error } = useLive(loadKernel, 300_000);
  const openFix = useFix();
  if (!k) {
    return (
      <Panel title="Running kernel" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  return (
    <>
      <Panel title="Running kernel" source="/proc/sys/kernel · /proc/cmdline" span={5} flush>
        <KeyValueList
          rows={[
            ["Version", <code key="v">{k.running}</code>],
            ["Tainted", <StatusBadge key="t" variant={k.tainted ? "warn" : "ok"} label={k.tainted ? `Yes (${k.tainted})` : "No (0)"} />],
            ["Modules loaded", String(k.modules.length)],
            ["Command line", <code key="c" className="wrap-anywhere">{k.command_line}</code>],
          ]}
        />
      </Panel>
      <Panel title="Installed kernels" span={7} flush actions={k.installed.some((i) => i.status === "old") && <Button small onClick={() => openFix("system.remove_old_kernels")}>Remove old…</Button>}>
        <DataTable rows={k.installed} columns={KERNELS} rowKey={(i) => i.version} empty="No kernel images found in /boot" />
      </Panel>
      <Panel title="Kernel modules" source="/proc/modules" span={12} flush>
        <DataTable rows={k.modules} columns={MODULES} rowKey={(m) => m.name} search={{ placeholder: "Search modules" }} pageSize={8} total={k.modules.length} />
      </Panel>
    </>
  );
}
