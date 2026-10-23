// SPDX-License-Identifier: Apache-2.0
// Data the Hardware pages share.

import { getInventory } from "../../api";
import { useLive } from "../../useLive";

const loadHardware = () => getInventory("hardware");
const loadGpus = () => getInventory("gpu");
const loadBluetooth = () => getInventory("bluetooth");
const loadFirmware = () => getInventory("firmware");

export const useHardware = () => useLive(loadHardware, 30_000);
export const useGpus = () => useLive(loadGpus, 60_000).data;
export const useBluetooth = () => useLive(loadBluetooth, 15_000).data;
export const useFirmware = () => useLive(loadFirmware, 600_000).data;

/** Device classes that need a driver to work; others (bridges, SRAM) often have none. */
export const NEEDS_DRIVER = new Set(["Network", "Display", "Audio and video", "Storage", "Wireless", "USB controller", "Thunderbolt"]);

/** "05/29/2026" (how firmware writes dates) → a local date. */
export function biosDate(text?: string): string {
  const m = text?.match(/^(\d{2})\/(\d{2})\/(\d{4})$/);
  if (!m) return text ?? "-";
  return new Date(Number(m[3]), Number(m[1]) - 1, Number(m[2])).toLocaleDateString(undefined, { day: "numeric", month: "short", year: "numeric" });
}

/** "13th Gen Intel(R) Core(TM) i5-1335U" → "Intel Core i5-1335U". */
export function shortCpu(model?: string): string {
  if (!model) return "-";
  return model.replace(/\(R\)|\(TM\)|™|®/g, "").replace(/\d+th Gen /, "").replace(/\s+CPU.*$/, "").replace(/\s+/g, " ").trim();
}

export const GHZ = (hz?: number) => (hz === undefined ? "-" : `${(hz / 1e9).toFixed(1)} GHz`);
