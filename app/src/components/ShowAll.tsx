// SPDX-License-Identifier: Apache-2.0
// The full table behind a short visual view (such as a top 10), folded away until asked for.
// Tables inside it show every row (no short preview), since the person asked for all of them.

import { type ReactNode, createContext } from "react";

/** True inside a ShowAll: tables there skip their short preview. */
export const FullList = createContext(false);

export function ShowAll({ label, children }: { label: string; children: ReactNode }) {
  return (
    <details className="show-all">
      <summary>{label}</summary>
      <FullList.Provider value={true}>{children}</FullList.Provider>
    </details>
  );
}
