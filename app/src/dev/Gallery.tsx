// SPDX-License-Identifier: Apache-2.0
// Component gallery: every component with the prototype's sample data, so they can be
// checked against the spec one by one. Development builds only (#/gallery).

import { ActivityTimeline } from "../components/ActivityTimeline";
import { AlertBanner } from "../components/AlertBanner";
import { Button, IconButton } from "../components/Button";
import { Checklist } from "../components/Checklist";
import { Command, LogLine } from "../components/CodeBlock";
import { DataTable } from "../components/DataTable";
import { FieldRow, RadioCards, Select, StateTiles, Switch } from "../components/FormControls";
import { InlineMeter } from "../components/InlineMeter";
import { KeyValueList } from "../components/KeyValueList";
import { KpiRow } from "../components/KpiTile";
import { Panel } from "../components/Panel";
import { StatusBadge } from "../components/StatusBadge";
import { StatusList } from "../components/StatusList";
import { AllocationBar } from "../components/charts/AllocationBar";
import { BarChart } from "../components/charts/BarChart";
import { BarList } from "../components/charts/BarList";
import { Donut } from "../components/charts/Donut";
import { Gauge, GaugeGroup } from "../components/charts/Gauge";
import { Heatmap } from "../components/charts/Heatmap";
import { LifecycleBar } from "../components/charts/LifecycleBar";
import { LineChart } from "../components/charts/LineChart";
import { SwimLanes } from "../components/charts/SwimLanes";
import { useUi } from "../ui";
import { useState } from "react";
import { NOW, RANGES, gen, timeAt } from "./sample";

// Top processes, in the shape of 09-processes.md.
const PROCESSES = [
  ["chrome", "riya", "S", "18.2", "2.1 GB", "42"],
  ["code", "riya", "S", "6.4", "820 MB", "38"],
  ["gnome-shell", "riya", "S", "4.1", "410 MB", "21"],
  ["dockerd", "root", "S", "1.2", "96 MB", "34"],
  ["Xwayland", "riya", "S", "0.9", "88 MB", "6"],
  ["spotify", "riya", "S", "2.6", "310 MB", "29"],
  ["slack", "riya", "S", "1.8", "540 MB", "33"],
  ["snapd", "root", "S", "0.1", "44 MB", "19"],
  ["NetworkManager", "root", "S", "0.0", "22 MB", "4"],
  ["pipewire", "riya", "S", "0.7", "18 MB", "3"],
  ["tracker-miner-fs-3", "riya", "D", "3.5", "64 MB", "6"],
  ["systemd-journald", "root", "S", "0.2", "38 MB", "1"],
  ["bluetoothd", "root", "Z", "0.0", "0 KB", "1"],
  ["postgres", "999", "S", "0.3", "1.1 GB", "8"],
].map(([name, user, state, cpu, mem, threads]) => ({ name, user, state, cpu, mem, threads }));

// Errors and warnings by hour, from 01-overview.md.
const HEAT = `Thu 1 0 0 1 1 0 0 0 1 1 0 0 0 0 0 0 0 1 1 1 2 0 0 1
Fri 0 1 0 0 0 1 0 1 0 1 1 0 1 0 1 0 0 0 1 0 0 0 0 0
Sat 1 0 0 0 1 1 1 0 0 0 0 1 0 1 1 0 0 0 1 0 0 0 0 1
Sun 1 0 1 2 1 0 1 0 0 0 0 0 0 0 0 1 0 0 0 0 0 1 1 0
Mon 0 0 0 0 0 0 0 0 2 1 1 0 0 0 0 0 7 1 0 0 0 0 0 0
Tue 1 0 0 1 1 0 0 0 1 0 0 1 0 0 0 0 0 1 0 0 2 1 0 0
Wed 1 1 3 0 0 2 0 1 1 9 0 0 1 0 2 0 0 1 0 0 0 6 0 2`
  .split("\n")
  .map((line) => {
    const [label, ...values] = line.split(" ");
    return { label, values: values.map(Number) };
  });

// Usage trend, 30 days, from 14-storage.md.
const DISK = [73.7, 74.3, 75.7, 75.6, 77.1, 78, 78, 78.6, 79.8, 80.7, 81.4, 81.8, 82.8, 82.9, 83.3, 84.7, 85.2, 85.3, 86.8, 87.5, 87.5, 88.2, 89.3, 90, 90.7, 91.4, 92.1, 92.5, 93.7, 94.2];
const DISK_DAYS = DISK.map((_, i) => new Date(2026, 8, 22 + i).toLocaleDateString("en-GB", { day: "numeric", month: "short" }));

export function Gallery() {
  const { range } = useUi();
  const [expertSwitch, setExpertSwitch] = useState(false);
  const [refresh, setRefresh] = useState<"2" | "5" | "15" | "manual">("5");
  const [access, setAccess] = useState<"basic" | "full">("full");
  const { n, step } = RANGES[range];
  const times = Array.from({ length: n }, (_, i) => timeAt(i, n, step).getTime());
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


      <AlertBanner tone="info" title="Charts (12-20)" text="Hover the line and bar charts for the crosshair and tooltip. Change the time range in the top bar to redraw the time series." />

      <Panel title="12 Line chart" subtitle="CPU and memory utilization" source="/proc/stat · /proc/meminfo" span={8}>
        <LineChart
          height={300}
          max={100}
          unit="%"
          x={{ kind: "time", times, range }}
          series={[
            { name: "CPU", data: gen(range, [3, 14, 8, 6, 2, 60, 12]), slot: 1 },
            { name: "Memory", data: gen(range, [5, 70, 8, 3, 40, 95, 78]), slot: 2 },
          ]}
        />
      </Panel>

      <Panel title="15 Gauge + 17 bar list" subtitle="Health score" span={4}>
        <Gauge value={82} subtitle="out of 100" status="warn" />
        <BarList
          rows={[
            { label: "Storage", value: 8, display: "−8", tone: "warn" },
            { label: "Services", value: 5, display: "−5", tone: "warn" },
            { label: "Security", value: 3, display: "−3", tone: "warn" },
            { label: "Power", value: 1, display: "−1", tone: "warn" },
            { label: "Backups", value: 1, display: "−1", tone: "warn" },
          ]}
        />
      </Panel>

      <Panel title="12 Stacked area chart" subtitle="CPU by type" span={12}>
        <LineChart
          stacked
          height={220}
          max="auto"
          unit="%"
          x={{ kind: "time", times, range }}
          series={[
            { name: "User", data: gen(range, [3, 9, 5, 4, 1, 40]), slot: 1 },
            { name: "System", data: gen(range, [4, 3, 2, 2, 0.5, 20]), slot: 2 },
            { name: "I/O wait", data: gen(range, [6, 1.2, 0.8, 1.5, 0, 15]), slot: 4 },
          ]}
        />
      </Panel>

      <Panel title="12 Line chart with threshold and forecast" subtitle="Usage trend and forecast · 30 days" span={8}
        info="The dashed line shows when the disk fills at the current rate.">
        <LineChart
          height={220}
          min={60}
          max={100}
          unit="%"
          x={{ kind: "labels", labels: DISK_DAYS }}
          series={[{ name: "Used", data: DISK, slot: 1 }]}
          threshold={{ value: 85, label: "Warning 85%" }}
          forecast={{ ahead: 8, value: 100, label: "Full ~29 Oct" }}
        />
      </Panel>

      <Panel title="14 Donut chart" subtitle="Disk usage ( / ) · 238 GB" span={4}>
        <Donut
          unit="GB"
          center={{ value: "94%", subtitle: "used" }}
          segments={[
            { name: "Home", value: 128, slot: 1 },
            { name: "Apps", value: 46, slot: 2 },
            { name: "System", value: 22, slot: 3 },
            { name: "Containers", value: 13, slot: 4 },
            { name: "Reclaimable", value: 15.2, tone: "crit" },
            { name: "Free", value: 13.8, tone: "neutral" },
          ]}
        />
      </Panel>

      <Panel title="13 Bar chart" subtitle="Boot time history · last 10 boots" span={6}>
        <BarChart
          height={220}
          unit="s"
          highlight={[9]}
          bars={[
            ["8 Oct", 29], ["10 Oct", 28], ["12 Oct", 31], ["13 Oct", 27], ["14 Oct", 28],
            ["15 Oct", 30], ["16 Oct", 29], ["17 Oct", 28], ["19 Oct", 28], ["Today", 41.2],
          ].map(([label, value]) => ({ label: String(label), value: Number(value) }))}
        />
      </Panel>

      <Panel title="17 Horizontal bar list" subtitle="Largest folders · home" source="du -h --max-depth=1 ~" span={6}>
        <BarList
          rows={[
            { label: "~/Videos", value: 52, display: "52 GB", mono: true },
            { label: "~/Documents", value: 41, display: "41 GB", mono: true },
            { label: "~/Downloads", value: 22, display: "22 GB", mono: true },
            { label: "~/.local/share", value: 5.2, display: "5.2 GB", mono: true },
            { label: "~/.cache", value: 3.8, display: "3.8 GB", mono: true },
            { label: "~/snap", value: 2.4, display: "2.4 GB", mono: true },
            { label: "~/Pictures", value: 1.6, display: "1.6 GB", mono: true },
          ]}
        />
      </Panel>

      <Panel title="16 Heatmap" subtitle="Errors and warnings by hour · last 7 days" source="journalctl -p warning" span={6}>
        <Heatmap rows={HEAT} unit="events" />
      </Panel>

      <Panel title="15 Gauge group" subtitle="Temperatures" span={6}>
        <GaugeGroup
          gauges={[
            { value: 54, max: 105, display: "54 °C", subtitle: "max 105 °C", status: "ok", caption: "CPU package" },
            { value: 61, max: 90, display: "61 °C", subtitle: "max 90 °C", status: "ok", caption: "GPU" },
            { value: 48, max: 70, display: "48 °C", subtitle: "max 70 °C", status: "ok", caption: "NVMe SSD" },
          ]}
        />
      </Panel>

      <Panel title="18 Allocation bar" subtitle="Space allocation · /dev/nvme0n1p2 · ext4" span={12}>
        <AllocationBar
          totalGb={238}
          segments={[
            { name: "Home", gb: 128, kind: "s1" },
            { name: "Apps", gb: 46, kind: "s2" },
            { name: "System", gb: 22, kind: "s3" },
            { name: "Containers", gb: 13, kind: "s4" },
            { name: "Reclaimable", gb: 15.2, kind: "reclaimable" },
            { name: "Free", gb: 13.8, kind: "free" },
          ]}
        />
      </Panel>

      <Panel title="19 Swim-lane schedule" subtitle="Next 24 hours" span={12}>
        <SwimLanes
          now={new Date(NOW)}
          lanes={[
            { name: "sync-notes.sh", runs: Array.from({ length: 48 }, (_, i) => i * 0.5 + 0.1), frequent: true },
            { name: "backup-photos.sh", runs: [4.43] },
            { name: "apt-daily.timer", runs: [8.9, 20.4] },
            { name: "logrotate.timer", runs: [2.43] },
            { name: "e2scrub_all.timer", runs: [5.9] },
            { name: "fstrim.timer", runs: [14.43] },
          ]}
        />
      </Panel>

      <Panel title="20 Lifecycle bar" subtitle="Release lifecycle · Ubuntu 24.04 LTS" info="How long this Ubuntu version gets security updates." span={12}>
        <LifecycleBar percent={50.4} start="Released · Apr 2024" end="Standard support ends · Apr 2029" />
      </Panel>


      <Panel title="05 Data table" subtitle="Top processes (sample)" source="ps -eo pid,user,stat,%cpu,rss,nlwp" span={12} flush>
        <DataTable
          rows={PROCESSES}
          rowKey={(r) => r.name}
          search={{ placeholder: "Filter processes" }}
          filter={{ column: "state", options: ["S", "D", "Z"] }}
          selection={{ action: "End selected", onAction: () => {} }}
          pageSize={8}
          columns={[
            { key: "name", label: "Process", text: (r) => r.name, render: (r) => <code>{r.name}</code> },
            { key: "user", label: "User", text: (r) => r.user },
            { key: "state", label: "State", text: (r) => r.state },
            { key: "cpu", label: "CPU", text: (r) => `${r.cpu}%`, numeric: true },
            { key: "mem", label: "Memory", text: (r) => r.mem, numeric: true },
            { key: "threads", label: "Threads", text: (r) => r.threads, numeric: true, expert: true },
          ]}
        />
      </Panel>

      <Panel title="21 Form controls" subtitle="Settings › General and Access" span={8} flush>
        <FieldRow label="Expert mode" description="Show every page and tab, data sources, commands, and raw columns">
          <Switch checked={expertSwitch} onChange={setExpertSwitch} label="Expert mode" />
        </FieldRow>
        <FieldRow label="Refresh interval" description="Live metrics">
          <Select
            label="Refresh interval"
            value={refresh}
            onChange={setRefresh}
            options={[
              { value: "2", label: "2 s" },
              { value: "5", label: "5 s" },
              { value: "15", label: "15 s" },
              { value: "manual", label: "Manual" },
            ]}
          />
        </FieldRow>
        <RadioCards
          name="access"
          value={access}
          onChange={setAccess}
          options={[
            { value: "basic", title: "Basic", description: "No password · about 80% of checks" },
            { value: "full", title: "Full", description: "Admin helper approved once · all checks" },
          ]}
        />
      </Panel>

      <Panel title="21 State tiles" subtitle="Device access" span={4}>
        <StateTiles
          tiles={[
            { icon: "cam", title: "Camera", state: "Not in use" },
            { icon: "mic", title: "Microphone", state: "Used by Zoom" },
            { icon: "screen", title: "Screen sharing", state: "Off" },
            { icon: "pin", title: "Location", state: "Off" },
          ]}
        />
      </Panel>

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
