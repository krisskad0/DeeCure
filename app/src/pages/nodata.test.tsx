// SPDX-License-Identifier: Apache-2.0
// The shared empty states, including an agent older than the app.

import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { NoData } from "./NoData";

afterEach(cleanup);

describe("NoData", () => {
  it("asks for an agent restart when the agent is older than the app", () => {
    render(<NoData error={{ kind: "agent_outdated", message: "older" }} waiting="Loading…" />);
    expect(screen.getByText("Agent needs a restart")).toBeTruthy();
    expect(screen.getByText("systemctl --user restart deecure-agent.service")).toBeTruthy();
  });

  it("shows other errors and the waiting text", () => {
    render(<NoData error={{ kind: "other", message: "database: locked" }} waiting="Loading…" />);
    expect(screen.getByText("database: locked")).toBeTruthy();
    cleanup();
    render(<NoData error={null} waiting="Loading…" />);
    expect(screen.getByText("Loading…")).toBeTruthy();
  });
});
