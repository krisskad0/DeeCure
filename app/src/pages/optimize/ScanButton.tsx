// SPDX-License-Identifier: Apache-2.0
// The round scanner button of Optimize and the Overview card: a viewfinder with a scan line inside a
// ring. While a scan runs the line sweeps, a dashed ring turns, and the ring fills with the parts
// done. Large on the Optimize page (a button that starts the scan), small on Overview (a link to it).

import type { ReactNode } from "react";

const R = 46;
const LENGTH = 2 * Math.PI * R;

/** The ring and the viewfinder; `progress` (0 to 1) fills the ring. */
export function Scanner({ busy, progress, children }: { busy: boolean; progress?: number; children?: ReactNode }) {
  return (
    <span className={`scanner${busy ? " busy" : ""}`} aria-hidden="true">
      <svg className="scanner-ring" viewBox="0 0 100 100">
        <circle className="scanner-track" cx="50" cy="50" r={R} />
        <circle className="scanner-dash" cx="50" cy="50" r={R - 5} />
        {progress !== undefined && <circle className="scanner-fill" cx="50" cy="50" r={R} strokeDasharray={`${progress * LENGTH} ${LENGTH}`} transform="rotate(-90 50 50)" />}
      </svg>
      <span className="scanner-face">
        <svg className="scanner-glyph" viewBox="0 0 24 24">
          <path d="M3 8V5a2 2 0 0 1 2-2h3M16 3h3a2 2 0 0 1 2 2v3M21 16v3a2 2 0 0 1-2 2h-3M8 21H5a2 2 0 0 1-2-2v-3" />
          <line className="scanner-line" x1="6" y1="12" x2="18" y2="12" />
        </svg>
        {children}
      </span>
    </span>
  );
}

/** The large button on the Optimize page; the caption sits under the ring. */
export function ScanButton({ busy, progress, label, caption, onClick }: { busy: boolean; progress?: number; label: string; caption: string; onClick: () => void }) {
  return (
    <div className="scan-cta">
      <button type="button" className="scan-btn" onClick={onClick} disabled={busy} aria-busy={busy} aria-label={`${label}. ${caption}`}>
        <Scanner busy={busy} progress={progress}>
          <span className="scanner-label">{label}</span>
        </Scanner>
      </button>
      <span className="scan-caption" aria-hidden="true">
        {caption}
      </span>
    </div>
  );
}
