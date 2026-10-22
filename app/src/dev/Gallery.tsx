// SPDX-License-Identifier: Apache-2.0
// Component gallery: every component with the prototype's sample data, so they can be
// checked against the spec one by one. Development builds only (#/gallery).

import { ActivityTimeline } from "../components/ActivityTimeline";
import { AlertBanner } from "../components/AlertBanner";
import { Button, IconButton } from "../components/Button";
import { Checklist } from "../components/Checklist";
import { Command, LogLine } from "../components/CodeBlock";
import { InlineMeter } from "../components/InlineMeter";
import { KeyValueList } from "../components/KeyValueList";
import { KpiRow } from "../components/KpiTile";
import { Panel } from "../components/Panel";
import { StatusBadge } from "../components/StatusBadge";
import { StatusList } from "../components/StatusList";
import { useUi } from "../ui";
import { gen } from "./sample";

export function Gallery() {
  const { range } = useUi();
  return (
    <>
      <AlertBanner
        tone="info"
        title="Component gallery (development only)"
        text="Each panel shows one component from docs/03-ui-design/03-components with sample data from the prototype. Switch Light/Dark in your system settings, and Beginner/Expert in the top bar, to check every variant."
      />

      <KpiRow
        tiles={[
          { label: "Health score", value: "82", unit: "/100", status: "warn", subtext: "9 open issues", visual: { kind: "bar", percent: 82 } },
          { label: "CPU", value: "12", unit: "%", subtext: "Load 0.84 · 12 threads", visual: { kind: "spark", data: gen(range, [3, 14, 8, 6, 2, 60, 12]), slot: 1 } },
          { label: "Memory", value: "5.9", unit: "GB", subtext: "78% of 7.6 GB", visual: { kind: "spark", data: gen(range, [5, 70, 8, 3, 40, 95, 78]), slot: 2 } },
          { label: "Disk ( / )", value: "94", unit: "%", status: "crit", subtext: "13.8 GB free of 238 GB", visual: { kind: "bar", percent: 94.2 }, href: "#/storage/overview" },
          { label: "Network", value: "1.2", unit: "MB/s", subtext: "↑ 0.3 MB/s · Wi-Fi", visual: { kind: "spark", data: gen(range, [9, 0.8, 0.6, 0.8, 0, 6, 1.2]), slot: 3 } },
          { label: "Battery", value: "64", unit: "%", status: "warn", subtext: "Health 71% · 3 h 10 m left", visual: { kind: "bar", percent: 64 } },
        ]}
      />

      <Panel title="01 Panel" subtitle="last 7 days" info="Hover or focus the i to see this tooltip." source="/proc/stat · /proc/meminfo" span={6}
        actions={<Button small>Action</Button>}>
        <p className="muted" style={{ margin: 0 }}>
          The source chip on the right appears only in Expert mode.
        </p>
      </Panel>

      <Panel title="02 Status badge and 03 Buttons" span={6}>
        <div className="gallery-row">
          <StatusBadge variant="ok" />
          <StatusBadge variant="warn" />
          <StatusBadge variant="crit" />
          <StatusBadge variant="info" />
          <StatusBadge variant="off" />
        </div>
        <div className="gallery-row">
          <Button>Default</Button>
          <Button variant="primary" icon="play">Primary</Button>
          <Button small>Small</Button>
          <Button variant="link">Link</Button>
          <Button variant="danger">Danger</Button>
          <Button disabled>Disabled</Button>
          <IconButton icon="ref" label="Refresh" />
        </div>
      </Panel>

      <Panel title="08 Status list" subtitle="Subsystems" span={4} flush>
        <StatusList
          rows={[
            { icon: "chip", label: "Hardware", metric: "CPU 54 °C", status: "ok", href: "#/hardware/summary" },
            { icon: "bat", label: "Power", metric: "Health 71%", status: "warn", href: "#/power/battery" },
            { icon: "sys", label: "System", metric: "Reboot required", status: "info", href: "#/system/overview" },
            { icon: "cog", label: "Services", metric: "1 failed", status: "crit", href: "#/services/system" },
            { icon: "cal", label: "Scheduled Jobs", metric: "10 jobs", status: "ok", href: "#/jobs/calendar" },
            { icon: "db", label: "Storage", metric: "94% used", status: "crit", href: "#/storage/overview" },
          ]}
        />
      </Panel>

      <Panel title="07 Key-value list" subtitle="System facts" span={4}>
        <KeyValueList
          rows={[
            ["OS", "Ubuntu 24.04.1 LTS"],
            ["Kernel", <code key="k">6.8.0-47-generic</code>],
            ["Uptime", "12 h 36 m"],
            ["Desktop", "GNOME 46 · Wayland"],
            ["Model", "Lenovo IdeaPad 5 14ARE05"],
          ]}
        />
      </Panel>

      <Panel title="06 Inline meter" span={4}>
        <div className="gallery-col">
          <InlineMeter percent={42} />
          <InlineMeter percent={84} />
          <InlineMeter percent={94.2} label="94%" />
          <InlineMeter percent={30} tone="ok" label="30% (forced ok)" />
        </div>
      </Panel>

      <Panel title="10 Activity timeline" subtitle="Recent activity" span={6} flush actions={<Button variant="link" small>View all</Button>}>
        <ActivityTimeline
          rows={[
            { time: "10:40", tone: "info", title: "Unattended upgrade: libc6, openssl, sudo", detail: "Reboot required" },
            { time: "09:12", tone: "crit", title: "bluetooth.service entered crash loop", detail: "SIGSEGV · 14 restarts" },
            { time: "08:58", tone: "ok", title: "Boot completed in 41.2 s", detail: "Kernel 6.8.0-47-generic" },
            { time: "02:14", tone: "warn", title: "Unclean shutdown", detail: "Battery depleted" },
            { time: "Tue 18:41", title: "linux-firmware upgraded", detail: "…0ubuntu2.5" },
          ]}
        />
      </Panel>

      <Panel title="09 Checklist" subtitle="Hardening" span={6} flush>
        <Checklist
          checks={[
            { state: "pass", title: "Firewall is on", detail: "UFW active · deny incoming by default" },
            { state: "fail", title: "Port 5432 bypasses the firewall", detail: "Docker publishes PostgreSQL to all interfaces", action: <Button small>Inspect</Button> },
            { state: "warn", title: "4 security updates waiting", detail: "openssl, libssl3t64, sudo, linux-firmware" },
            { state: "na", title: "Disk encryption", detail: "Your hardware does not report this" },
          ]}
        />
      </Panel>

      <AlertBanner
        tone="crit"
        title="Port 5432 bypasses the UFW firewall"
        text="Docker opened PostgreSQL to every network. Other devices on your Wi-Fi can reach it."
        actions={
          <>
            <Button small>Inspect</Button>
            <Button small variant="primary">Fix</Button>
          </>
        }
      />

      <Panel title="22 Code and log blocks" span={12}>
        <div className="gallery-col">
          <Command>snap remove firefox --revision=4793</Command>
          <LogLine>Oct 21 09:12:04 riya-laptop bluetoothd[1123]: Segmentation fault (core dumped)</LogLine>
          <p style={{ margin: 0 }}>
            Inline: <code>/etc/UPower/UPower.conf</code>
          </p>
        </div>
      </Panel>
    </>
  );
}
