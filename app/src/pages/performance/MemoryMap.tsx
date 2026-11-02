// SPDX-License-Identifier: Apache-2.0
// What fills RAM, at a glance: three numbers (really in use, can be freed, free), one bar with
// a segment per part (parts the kernel can free are striped), a legend with sizes, and one
// sentence on what it means. The detailed map (Expert) shows only the bar and the parts, so the
// totals appear once on the page.

import { bytes } from "../../metrics";

export interface MemoryPart {
  name: string;
  bytes: number;
  /** Series colour 1-5, or `free` for unused memory and `other` for the rest. */
  slot: 1 | 2 | 3 | 4 | 5 | "free" | "other";
  /** The kernel gives it back at once when apps need memory. */
  freeable?: boolean;
  /** One line on what it is (the detailed map only). */
  means?: string;
}

/** The parts of /proc/meminfo in a few groups people can act on; they add up to the total. */
export function memoryParts(m: Record<string, number>): MemoryPart[] {
  const v = (k: string) => m[k] ?? 0;
  const total = v("MemTotal");
  const shared = v("Shmem");
  const parts: MemoryPart[] = [
    { name: "Apps", bytes: v("AnonPages"), slot: 1, means: "Memory programs asked for" },
    { name: "Shared memory", bytes: shared, slot: 5, means: "Shared between programs, and RAM disks (tmpfs)" },
    { name: "Kernel, cannot be freed", bytes: v("SUnreclaim") + v("PageTables") + v("KernelStack"), slot: 4, means: "The kernel's own memory, page tables, and thread stacks" },
    { name: "Kernel caches", bytes: v("SReclaimable"), slot: 3, freeable: true, means: "Kernel caches it frees when needed" },
    { name: "File cache", bytes: Math.max(0, v("Cached") - shared) + v("Buffers"), slot: 2, freeable: true, means: "Copies of files, freed at once when apps need memory" },
  ];
  const free = v("MemFree");
  const counted = parts.reduce((n, p) => n + p.bytes, 0) + free;
  // Drivers, graphics memory, and other kernel memory /proc/meminfo has no line for.
  if (total > counted) parts.push({ name: "Other", bytes: total - counted, slot: "other", means: "Drivers, graphics, and other kernel memory" });
  parts.push({ name: "Free", bytes: free, slot: "free", means: "Not used at all" });
  return parts.filter((p) => p.bytes > 0);
}

const pct = (part: number, total: number) => (total ? `${Math.round((part / total) * 100)}%` : "-");

/** One plain sentence on what the picture means. */
export function memoryVerdict(inUse: number, freeable: number, total: number): string {
  const used = total ? inUse / total : 0;
  if (used >= 0.9) return "Apps fill almost all memory. The computer may slow down: close apps you do not need, or add memory.";
  if (used >= 0.75) return "Apps use most of the memory. It is fine for now; it gets slow if they keep growing.";
  if (freeable > inUse) return "Most of it is cache: Linux keeps recently used files in spare memory and frees it the moment an app needs it. Nothing to worry about.";
  return "There is plenty of room for more apps.";
}

export function MemoryMap({ parts, total, detailed = false }: { parts: MemoryPart[]; total: number; detailed?: boolean }) {
  const inUse = parts.filter((p) => !p.freeable && p.slot !== "free").reduce((n, p) => n + p.bytes, 0);
  const freeable = parts.filter((p) => p.freeable).reduce((n, p) => n + p.bytes, 0);
  const free = parts.filter((p) => p.slot === "free").reduce((n, p) => n + p.bytes, 0);
  return (
    <div className={`mm ${detailed ? "mm-detailed" : ""}`}>
      {!detailed && (
        <div className="mm-sum">
          <div>
            <b>{bytes(inUse)}</b>
            <span>really in use · {pct(inUse, total)}</span>
          </div>
          <div>
            <b>{bytes(freeable)}</b>
            <span>can be freed · {pct(freeable, total)}</span>
          </div>
          <div>
            <b>{bytes(free)}</b>
            <span>free · {pct(free, total)}</span>
          </div>
        </div>
      )}
      <div className="mm-bar" role="img" aria-label={parts.map((p) => `${p.name} ${bytes(p.bytes)}`).join(", ")}>
        {parts.map((p) => (
          <span key={p.name} className={`mm-seg mm-${p.slot}${p.freeable ? " mm-freeable" : ""}`} style={{ width: `${(p.bytes / total) * 100}%` }} title={`${p.name}: ${bytes(p.bytes)} (${pct(p.bytes, total)})`} />
        ))}
      </div>
      <ul className="mm-legend">
        {parts.map((p) => (
          <li key={p.name}>
            <i className={`mm-seg mm-${p.slot}${p.freeable ? " mm-freeable" : ""}`} aria-hidden="true" />
            <span className="mm-name">
              {p.name}
              {detailed && p.means && <span className="mm-means">{p.means}</span>}
            </span>
            <span className="mm-size">
              {bytes(p.bytes)} <span className="muted">{pct(p.bytes, total)}</span>
            </span>
          </li>
        ))}
      </ul>
      {!detailed && <p className="mm-verdict">{memoryVerdict(inUse, freeable, total)}</p>}
    </div>
  );
}
