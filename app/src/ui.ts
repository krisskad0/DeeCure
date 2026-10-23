// SPDX-License-Identifier: Apache-2.0
// View settings every component can read: the view level and the time range.
// Levels (docs/01-product/view-levels.md): Beginner shows a short menu, Advanced every page,
// and Expert every page plus data sources, commands, raw columns, and the deep data.

import { createContext, useContext } from "react";

export type Range = "1h" | "24h" | "7d" | "30d";
export type Mode = "beginner" | "advanced" | "expert";

const ORDER: readonly Mode[] = ["beginner", "advanced", "expert"];

/** Whether `mode` shows at least what `level` shows. A saved value that is not a level counts as Beginner. */
export function atLeast(mode: Mode, level: Mode): boolean {
  return Math.max(0, ORDER.indexOf(mode)) >= ORDER.indexOf(level);
}

export const UiContext = createContext<{ mode: Mode; range: Range }>({ mode: "beginner", range: "24h" });

export function useUi() {
  return useContext(UiContext);
}
