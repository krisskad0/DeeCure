// SPDX-License-Identifier: Apache-2.0
// What is wrong on this sub-page, at its top (docs/03-ui-design/04-behavior/page-status.md):
// each open finding with its explanation, a Fix button when there is one, and "Show where",
// which scrolls to the panel that shows it (that panel is marked too).

import type { Finding } from "../api";
import { AlertBanner } from "../components/AlertBanner";
import { Button } from "../components/Button";
import { RULE_PANELS, panelMatches, severityVariant } from "../status";
import { useFix } from "./FixDrawer";

const TONE = { crit: "crit", warn: "warn", info: "info", ok: "ok", off: "info" } as const;

/** The first panel on the page that shows this finding, if it is there. */
function panelFor(f: Finding): HTMLElement | undefined {
  const wanted = RULE_PANELS[f.rule] ?? [];
  return [...document.querySelectorAll<HTMLElement>("section.panel[data-panel]")].find((el) => wanted.some((w) => panelMatches(el.dataset.panel ?? "", w)));
}

export function HereFindings({ findings }: { findings: Finding[] }) {
  const openFix = useFix();
  if (findings.length === 0) return null;
  return (
    <>
      {findings.map((f) => (
        <AlertBanner
          key={`${f.rule}|${f.subject ?? ""}`}
          tone={TONE[severityVariant(f.severity)]}
          title={f.title}
          text={f.explain}
          actions={
            <>
              {(RULE_PANELS[f.rule]?.length ?? 0) > 0 && (
                <Button
                  small
                  onClick={() => {
                    const el = panelFor(f);
                    if (el) el.scrollIntoView({ behavior: "smooth", block: "start" });
                  }}
                >
                  Show where
                </Button>
              )}
              {f.fixes && f.fixes.length > 0 && (
                <Button small variant="primary" onClick={() => openFix(f.fixes?.[0] ?? "", f.subject)}>
                  Fix
                </Button>
              )}
            </>
          }
        />
      ))}
    </>
  );
}
