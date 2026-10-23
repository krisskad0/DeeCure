// SPDX-License-Identifier: Apache-2.0
// Beginner mode keeps the sidebar, the page tabs, and the search short; Expert mode shows all.

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { parseHash } from "../route";
import { type Mode, UiContext } from "../ui";
import { CommandPalette } from "./CommandPalette";
import { PageHeader } from "./PageHeader";
import { Sidebar } from "./Sidebar";

vi.mock("../api", () => ({
  getSnapshot: () => Promise.resolve({ data: [{ name: "cups.service" }] }),
}));

// The test browser has no scrolling; the search palette scrolls its highlighted row.
Element.prototype.scrollIntoView = () => {};

afterEach(cleanup);

function inMode(mode: Mode, ui: ReactNode) {
  return render(<UiContext.Provider value={{ mode, range: "24h" }}>{ui}</UiContext.Provider>);
}

function sidebar(mode: Mode, hash: string) {
  inMode(mode, <Sidebar route={parseHash(hash)} statusOf={() => null} issueCount={0} rail={false} onRail={() => {}} drawerOpen={false} onNavigate={() => {}} />);
}

function header(mode: Mode, hash: string) {
  inMode(mode, <PageHeader route={parseHash(hash)} status={null} tabCounts={{}} onRefresh={() => {}} />);
}

const tabs = () => screen.getAllByRole("tab").map((t) => t.textContent);

describe("Beginner and Expert menus", () => {
  it("the Beginner sidebar shows only the pages most people need", () => {
    sidebar("beginner", "#/overview/summary");
    expect(screen.getByText("Overview")).toBeTruthy();
    expect(screen.getByText("Storage")).toBeTruthy();
    expect(screen.getByText("Settings")).toBeTruthy();
    for (const hidden of ["Scheduled Jobs", "Containers", "Logs", "Accounts", "Network"]) expect(screen.queryByText(hidden)).toBeNull();
  });

  it("the Expert sidebar shows every page", () => {
    sidebar("expert", "#/overview/summary");
    for (const page of ["Scheduled Jobs", "Containers", "Logs", "Accounts", "Diagnostics"]) expect(screen.getByText(page)).toBeTruthy();
  });

  it("a link to an Expert page keeps that page in the Beginner sidebar", () => {
    sidebar("beginner", "#/logs/kernel");
    expect(screen.getByText("Logs").closest("a")?.getAttribute("aria-current")).toBe("page");
    expect(screen.queryByText("Journal")).toBeNull();
  });

  it("page tabs follow the mode, and keep the open tab", () => {
    header("beginner", "#/performance/cpu");
    expect(tabs()).toEqual(["CPU", "Memory", "Disk I/O", "Network", "GPU"]);
    cleanup();
    header("beginner", "#/performance/pressure");
    expect(tabs()).toEqual(["CPU", "Memory", "Disk I/O", "Network", "GPU", "Pressure (PSI)"]);
    cleanup();
    header("expert", "#/performance/cpu");
    expect(tabs()).toContain("Pressure (PSI)");
  });

  it("search lists only Beginner pages, and says Expert mode finds more", async () => {
    inMode("beginner", <CommandPalette onClose={() => {}} />);
    const input = screen.getByRole("textbox", { name: "Search pages and services" });
    fireEvent.change(input, { target: { value: "storage" } });
    expect(screen.getByText("Storage › Overview")).toBeTruthy();
    expect(screen.queryByText("Storage › Filesystems")).toBeNull();
    fireEvent.change(input, { target: { value: "cups" } });
    expect(screen.getByText(/Expert mode shows every page and service/)).toBeTruthy();
    cleanup();
    inMode("expert", <CommandPalette onClose={() => {}} />);
    fireEvent.change(screen.getByRole("textbox", { name: "Search pages and services" }), { target: { value: "cups" } });
    await waitFor(() => expect(screen.getByText("cups.service")).toBeTruthy());
  });
});
