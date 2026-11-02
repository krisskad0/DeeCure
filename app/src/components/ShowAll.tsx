// SPDX-License-Identifier: Apache-2.0
// The full table behind a short visual view (such as a top 10), folded away until asked for.

import type { ReactNode } from "react";

export function ShowAll({ label, children }: { label: string; children: ReactNode }) {
  return (
    <details className="show-all">
      <summary>{label}</summary>
      {children}
    </details>
  );
}
