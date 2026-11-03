// SPDX-License-Identifier: Apache-2.0
// After an update, the app says so once, with its main change and a link to everything new.

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { noteGroups } from "../components/ReleaseNotes";
import { UpdatedBanner } from "./UpdatedBanner";

let current = "0.1.0-alpha.9";
vi.mock("../api", () => ({
  getUpdateStatus: () =>
    Promise.resolve({
      settings: { check: "daily", channel: "prerelease", auto: false },
      current,
      available: false,
      from_apt: false,
      auto_ready: false,
      current_notes: "### Added\n\n- **Release notes** in the app.\n\n### How to update\n\n- Run it again.",
    }),
}));

afterEach(() => {
  cleanup();
  localStorage.clear();
});

describe("after an update", () => {
  it("the first start says nothing, a later version says it once", async () => {
    current = "0.1.0-alpha.8";
    const first = render(<UpdatedBanner />);
    await waitFor(() => expect(localStorage.getItem("deecure.seenVersion")).toBe('"0.1.0-alpha.8"'));
    expect(screen.queryByText(/was updated/)).toBeNull();
    first.unmount();

    current = "0.1.0-alpha.9";
    render(<UpdatedBanner />);
    await waitFor(() => expect(screen.getByText("DeeCure was updated to 0.1.0-alpha.9")).toBeTruthy());
    expect(screen.getByText(/Release notes in the app\. See everything/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
    expect(screen.queryByText(/was updated/)).toBeNull();
    expect(localStorage.getItem("deecure.seenVersion")).toBe('"0.1.0-alpha.9"');
  });

  it("reads notes into groups and stops at How to update", () => {
    const groups = noteGroups("### Added\n\n- One.\n- Two.\n\n### Fixed\n\n- Three.\n\n### How to update\n\n- Four.");
    expect(groups).toEqual([
      { title: "New", points: ["One.", "Two."] },
      { title: "Fixed", points: ["Three."] },
    ]);
    expect(noteGroups("See CHANGELOG.md for what changed.")).toEqual([]);
  });
});
