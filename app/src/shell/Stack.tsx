// SPDX-License-Identifier: Apache-2.0
// Merged sub-pages (docs/01-product/view-levels.md): one sub-page made of former ones.

import { Fragment } from "react";
import { parseHash } from "../route";
import { type Mode, atLeast, useUi } from "../ui";

type Tab = () => React.JSX.Element;

/**
 * A sub-page made of former sub-pages, one after another. Each part starts with an anchor that
 * old links scroll to. A part with a level shows from that level up, or when a link asks for it.
 */
export function stack(...parts: [section: string, part: Tab, from?: Mode][]): Tab {
  return function Stacked() {
    const { mode } = useUi();
    const wanted = parseHash(window.location.hash).section;
    return (
      <>
        {parts
          .filter(([section, , from]) => !from || atLeast(mode, from) || section === wanted)
          .map(([section, Part]) => (
            <Fragment key={section}>
              <span id={`sec-${section}`} className="span-12 sec-anchor" aria-hidden="true" />
              <Part />
            </Fragment>
          ))}
      </>
    );
  };
}
