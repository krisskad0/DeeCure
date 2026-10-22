// SPDX-License-Identifier: Apache-2.0
// View settings every component can read: Beginner/Expert mode and the time range.

import { createContext, useContext } from "react";

export type Range = "1h" | "24h" | "7d" | "30d";
export type Mode = "beginner" | "expert";

export const UiContext = createContext<{ mode: Mode; range: Range }>({ mode: "beginner", range: "24h" });

export function useUi() {
  return useContext(UiContext);
}
