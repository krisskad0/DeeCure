// SPDX-License-Identifier: Apache-2.0
// A sub-page with an open warning says what it is at the top and marks the panel that shows it.

import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Finding } from "../api";
import { FlaggedPanels, Panel } from "../components/Panel";
import { RULE_PANELS, RULE_TAB } from "../status";
import { FixContext } from "./FixDrawer";
import { HereFindings } from "./HereFindings";

afterEach(cleanup);

const hot: Finding = { rule: "hardware.part_hot", severity: "warning", state: "open", title: "The SSD is hot", explain: "It runs at 70 °C.", first_seen: 0, last_seen: 0, fixes: ["hardware.cool_down"] };

describe("warnings on a sub-page", () => {
  it("explain the warning at the top, with Fix and Show where", () => {
    const fix = vi.fn();
    render(
      <FixContext.Provider value={fix}>
        <HereFindings findings={[hot]} />
        <FlaggedPanels.Provider value={[["Temperatures", "warn"]]}>
          <Panel title="Temperatures">x</Panel>
          <Panel title="Temperature history">y</Panel>
        </FlaggedPanels.Provider>
      </FixContext.Provider>,
    );
    expect(screen.getByText("The SSD is hot")).toBeTruthy();
    expect(screen.getByText("It runs at 70 °C.")).toBeTruthy();
    // Only the panel that shows it is marked.
    expect(screen.getAllByText("Warning here")).toHaveLength(1);
    const scroll = vi.fn();
    Element.prototype.scrollIntoView = scroll;
    fireEvent.click(screen.getByRole("button", { name: "Show where" }));
    expect(scroll).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "Fix" }));
    expect(fix).toHaveBeenCalledWith("hardware.cool_down", undefined);
  });

  it("marks a panel whose title goes on after a dot", () => {
    render(
      <FlaggedPanels.Provider value={[["Battery details", "crit"]]}>
        <Panel title="Battery details · BAT0">x</Panel>
        <Panel title="Battery">y</Panel>
      </FlaggedPanels.Provider>,
    );
    expect(screen.getAllByText("Problem here")).toHaveLength(1);
  });

  it("only names panels for rules that have a sub-page", () => {
    for (const rule of Object.keys(RULE_PANELS)) expect(RULE_TAB[rule], rule).toBeDefined();
  });
});
