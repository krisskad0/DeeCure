// SPDX-License-Identifier: Apache-2.0
// Expert sections for graphics (docs/01-product/expert-coverage.md, section 5): each card's clock,
// power saving, slowdown reasons, and sleep; graphics use per app; and the graphics software
// (session type, renderer, Mesa, video decoding, firmware).

import { type GpuDetail, type GraphicsStack, getInventory } from "../../api";
import { DataTable } from "../../components/DataTable";
import { KeyValueList } from "../../components/KeyValueList";
import { Panel } from "../../components/Panel";
import { StatusBadge } from "../../components/StatusBadge";
import { bytes } from "../../metrics";
import { perSecond, useLive, useLiveWithPrevious } from "../../useLive";
import { NoData } from "../NoData";

const loadDetail = () => getInventory("gpu_detail");
const loadStack = () => getInventory("graphics_stack");

/** Intel's reasons for slowing the graphics down, in plain words. */
const SLOWED: Record<string, string> = {
  pl1: "long-term power limit",
  pl2: "short-term power limit",
  pl4: "peak power limit",
  thermal: "heat",
  prochot: "the processor is too hot",
  ratl: "average temperature limit",
  vr_tdc: "power supply current limit",
  vr_thermalert: "power supply heat",
};

/** The video-decoding drivers that fit a kernel graphics driver. */
const VIDEO_FOR: Record<string, string[]> = {
  i915: ["iHD", "i965"],
  xe: ["iHD"],
  amdgpu: ["radeonsi"],
  radeon: ["r600", "radeonsi"],
  nouveau: ["nouveau"],
  nvidia: ["nvidia"],
};

type Card = GpuDetail["cards"][number];

export function GpuExpertSections() {
  const { data: d, before, error } = useLiveWithPrevious<GpuDetail>(loadDetail, 5000);
  if (!d) {
    return (
      <Panel title="Graphics details" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const wallNs = before ? (d.at - before.at) * 1e6 : 0;
  const seconds = wallNs / 1e9;
  const share = (now?: number, then?: number) => {
    const r = perSecond(now, then, seconds);
    return r === undefined ? undefined : (r / 1e9) * 100;
  };
  const pct = (v?: number) => (v === undefined ? "-" : `${v.toFixed(1)}%`);
  const clients = d.clients.map((c) => {
    const b = before?.clients.find((x) => x.pid === c.pid && x.pci_address === c.pci_address);
    return {
      ...c,
      render: share(c.engines.render, b?.engines.render),
      video: share(c.engines.video, b?.engines.video),
      other: share(
        Object.entries(c.engines).filter(([e]) => e !== "render" && e !== "video").reduce((n, [, v]) => n + v, 0),
        b ? Object.entries(b.engines).filter(([e]) => e !== "render" && e !== "video").reduce((n, [, v]) => n + v, 0) : undefined,
      ),
    };
  });
  clients.sort((a, b) => (before ? (b.render ?? 0) + (b.video ?? 0) - (a.render ?? 0) - (a.video ?? 0) : (b.engines.render ?? 0) - (a.engines.render ?? 0)));
  const several = d.cards.length > 1;
  const cardPanel = (c: Card) => {
    const b = before?.cards.find((x) => x.card === c.card);
    const resting = share(c.power_saving_ms === undefined ? undefined : c.power_saving_ms * 1e6, b?.power_saving_ms === undefined ? undefined : b.power_saving_ms * 1e6);
    const users = clients.filter((x) => x.pci_address === c.pci_address);
    const second = several && !c.boot_vga;
    return (
      <Panel key={c.card} title={`Card · ${c.card}${c.driver ? ` (${c.driver})` : ""}`} subtitle={second ? "second card" : several ? "main card" : undefined} source="/sys/class/drm" span={6} flush>
        <KeyValueList
          rows={[
            ["Clock now", c.mhz_now === undefined ? "Not reported by this driver" : c.mhz_now === 0 ? "Resting (0 MHz)" : `${c.mhz_now} MHz`],
            ["Allowed range", c.mhz_min !== undefined && c.mhz_max !== undefined ? `${c.mhz_min} to ${c.mhz_max} MHz` : "-"],
            ["Hardware maximum · boost", `${c.mhz_hardware_max ?? "-"} · ${c.mhz_boost ?? "-"} MHz`],
            ["Time in power saving", c.power_saving_ms === undefined ? "Not reported by this driver" : resting !== undefined ? `${pct(Math.min(100, resting))} now` : "Measuring…"],
            ["Slowed down by", c.slowed_by.length ? c.slowed_by.map((r) => SLOWED[r] ?? r).join(", ") : c.driver === "i915" || c.driver === "xe" ? "Nothing right now" : "Not reported by this driver"],
            ...(c.watts !== undefined ? [["Power now", `${c.watts.toFixed(1)} W`] as [string, string]] : []),
            [
              "Asleep or awake",
              c.runtime_status === "suspended" ? (
                "Asleep (uses almost no power)"
              ) : c.runtime_status === "active" ? (
                second ? (
                  <span key="s">
                    <StatusBadge variant={users.length ? "info" : "warn"} label="Awake" /> {users.length ? `used by ${[...new Set(users.map((u) => u.name))].join(", ")}` : "and no app of yours uses it"}
                  </span>
                ) : (
                  "Awake"
                )
              ) : (
                "-"
              ),
            ],
            ["Screens", c.connectors.length ? c.connectors.map(([n, s]) => `${n}${s === "connected" ? " (in use)" : ""}`).join(", ") : "None"],
          ]}
        />
      </Panel>
    );
  };
  return (
    <>
      {d.cards.map(cardPanel)}
      <Panel
        title="Graphics use per app"
        subtitle={before ? "now, busiest first" : "since each app connected (shares after the next reading)"}
        info="Share of the time each app kept a graphics engine busy. Render draws the screen; video decodes and encodes video. Only your own apps can be read."
        source="/proc/[pid]/fdinfo (DRM)"
        span={12}
        flush
      >
        <DataTable
          rows={clients}
          columns={[
            { key: "name", label: "App", text: (c) => c.name },
            { key: "pid", label: "PID", text: (c) => String(c.pid), numeric: true },
            { key: "render", label: "Render", text: (c) => (before ? pct(c.render) : `${((c.engines.render ?? 0) / 1e9).toFixed(0)} s`), numeric: true },
            { key: "video", label: "Video", text: (c) => (before ? pct(c.video) : `${((c.engines.video ?? 0) / 1e9).toFixed(0)} s`), numeric: true },
            { key: "other", label: "Other engines", text: (c) => (before ? pct(c.other) : "-"), numeric: true },
            { key: "memory", label: "Graphics memory", text: (c) => (c.memory === undefined ? "-" : bytes(c.memory)), numeric: true },
            { key: "card", label: "Card", text: (c) => c.pci_address ?? "-", expert: true },
          ]}
          rowKey={(c) => `${c.pid}|${c.pci_address}`}
          pageSize={12}
          empty="No app of yours uses the graphics card, or this driver does not report it"
        />
      </Panel>
    </>
  );
}

export function GraphicsStackSection({ drivers }: { drivers: string[] }) {
  const { data: s, error } = useLive<GraphicsStack>(loadStack, 300_000);
  if (!s) {
    return (
      <Panel title="Graphics software" span={12}>
        <NoData error={error} waiting="Loading…" />
      </Panel>
    );
  }
  const fitting = [...new Set(drivers.flatMap((d) => VIDEO_FOR[d] ?? []))];
  const installed = fitting.filter((v) => s.video_drivers.includes(v));
  const failed = s.firmware.filter((l) => /fail|error/i.test(l));
  return (
    <>
      <Panel title="Graphics software" source="logind · glxinfo · dpkg · /usr/lib/x86_64-linux-gnu/dri · vainfo" span={6} flush>
        <KeyValueList
          rows={[
            ["Desktop session", s.session_type ? (s.session_type === "wayland" ? "Wayland" : s.session_type === "x11" ? "X11" : s.session_type) : "-"],
            ["Renderer", s.renderer ? s.software_rendering ? <StatusBadge key="r" variant="warn" label={`${s.renderer}: software, no graphics card`} /> : s.renderer : "Install mesa-utils to see it"],
            ["Mesa", s.mesa_version ?? "Not installed"],
            ...(s.nvidia_version ? [["NVIDIA driver", s.nvidia_version] as [string, string]] : []),
            [
              "Video decoding driver",
              fitting.length === 0 ? (
                "-"
              ) : installed.length ? (
                `${installed.join(", ")} installed`
              ) : (
                <StatusBadge key="v" variant="warn" label={`Missing: ${fitting.join(" or ")}. Video then runs on the CPU`} />
              ),
            ],
            ["Formats decoded in hardware", s.video_decode ? (s.video_decode.length ? s.video_decode.join(", ") : "None") : "Install the vainfo package to list them"],
          ]}
        />
      </Panel>
      <Panel title="Graphics firmware" subtitle="this boot" info="Modern graphics cards need firmware files for power saving and video. Missing firmware makes a card slower or less efficient." source="journalctl -k" span={6} flush>
        {s.firmware.length ? (
          <>
            {failed.length > 0 && <div className="panel-pad"><StatusBadge variant="warn" label={`${failed.length} firmware problem${failed.length === 1 ? "" : "s"}`} /></div>}
            <ul className="plain-list panel-pad">
              {s.firmware.map((l) => (
                <li key={l}>
                  <code>{l}</code>
                </li>
              ))}
            </ul>
          </>
        ) : (
          <div className="empty muted">No graphics firmware messages in this boot's kernel log (or the log cannot be read).</div>
        )}
      </Panel>
    </>
  );
}
