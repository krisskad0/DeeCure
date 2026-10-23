// SPDX-License-Identifier: Apache-2.0
// System › Overview (docs/03-ui-design/05-pages/05-system.md), with real data.

import { type BootInfo, getInventory, getMachine, getSnapshot } from "../../api";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { LifecycleBar } from "../../components/charts/LifecycleBar";
import { duration, pick, when } from "../../metrics";
import { useLive } from "../../useLive";
import { useLatest } from "../performance/shared";

const loadSystem = () => getInventory("system");
const loadOs = () => getInventory("os");
const loadMachine = () => getMachine();
const loadBoot = () => getSnapshot<BootInfo>("boot");

const monthYear = (iso: string) => new Date(`${iso}T00:00:00`).toLocaleDateString(undefined, { month: "short", year: "numeric" });

/** "ubuntu:GNOME" + "wayland" → "GNOME · Wayland". */
function desktop(name?: string | null, session?: string | null): string {
  const de = name?.split(":").pop();
  const type = session ? session[0].toUpperCase() + session.slice(1) : undefined;
  return [de, type].filter(Boolean).join(" · ") || "-";
}

export function SystemOverview() {
  const latest = useLatest();
  const system = useLive(loadSystem, 60_000).data;
  const os = useLive(loadOs, 300_000).data;
  const machine = useLive(loadMachine, 3_600_000).data;
  const boot = useLive(loadBoot, 300_000).data?.data;
  const now = Date.now();
  const load = ["cpu.load_1m", "cpu.load_5m", "cpu.load_15m"].map((n) => pick(latest, n));
  const running = pick(latest, "services.running");
  const life = os?.lifecycle;
  const lifePercent = life ? ((now - Date.parse(life.release)) / (Date.parse(life.eol) - Date.parse(life.release))) * 100 : 0;

  return (
    <>
      <Panel title="Operating system" source="/etc/os-release · /proc · /sys/class/dmi" span={6} flush>
        <KeyValueList
          rows={[
            ["Distribution", os?.distribution ?? system?.os ?? "-"],
            ["Kernel", system?.kernel ? <code>{system.kernel}</code> : "-"],
            ["Architecture", system?.arch ?? "-"],
            ["Desktop", desktop(machine?.desktop, machine?.session_type)],
            ["Init system", boot?.systemd ? `systemd ${boot.systemd.split(/[\s(]/)[0]}` : "systemd"],
            ["Hostname", system?.hostname ?? "-"],
            ["Virtualization", os?.virtualization ?? "-"],
            ["Machine ID", os?.machine_id ? <code>{os.machine_id}</code> : "-"],
          ]}
        />
      </Panel>
      <Panel title="Status" span={6} flush>
        <KeyValueList
          rows={[
            ["Uptime", system?.uptime_seconds === undefined ? "-" : duration(system.uptime_seconds)],
            [
              "Last boot",
              boot ? `${when(boot.booted_at, now)}${boot.times ? ` · ${boot.times.total.toFixed(1)} s` : ""}` : system?.uptime_seconds === undefined ? "-" : when(now - system.uptime_seconds * 1000, now),
            ],
            [
              "Reboot required",
              system?.reboot_required ? <StatusBadge variant="warn" label={["Yes", system.reboot_packages.slice(0, 3).join(", ")].filter(Boolean).join(" · ")} /> : "No",
            ],
            ["Load average", load.every((v) => v !== undefined) ? <code>{load.map((v) => v?.toFixed(2)).join(" ")}</code> : "-"],
            ["Users signed in", os ? String(os.users_signed_in) : "-"],
            ["Running services", running === undefined ? "-" : String(running)],
            ["Ubuntu Pro", os?.pro_attached === undefined ? "Not installed" : os.pro_attached ? <StatusBadge variant="ok" label="Attached" /> : <StatusBadge variant="off" label="Not attached" />],
            ["Livepatch", os?.livepatch ? <StatusBadge variant={os.livepatch === "enabled" ? "ok" : "off"} label={os.livepatch[0].toUpperCase() + os.livepatch.slice(1)} /> : "-"],
          ]}
        />
      </Panel>
      <Panel title="Release lifecycle" subtitle={os?.distribution?.replace(/\s*\(.*\)/, "")} info="How long this Ubuntu version gets security updates." source="/usr/share/distro-info/ubuntu.csv" span={12}>
        {life ? (
          <>
            <LifecycleBar percent={lifePercent} start={`Released · ${monthYear(life.release)}`} end={`Standard support ends · ${monthYear(life.eol)}`} />
            {life.eol_esm && <p className="muted">With Ubuntu Pro, security updates continue until {monthYear(life.eol_esm)}.</p>}
          </>
        ) : (
          <div className="empty muted">Release dates are not known for this system (the distro-info-data package is missing).</div>
        )}
      </Panel>
    </>
  );
}
