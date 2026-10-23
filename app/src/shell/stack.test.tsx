// SPDX-License-Identifier: Apache-2.0
// Merged sub-pages show their parts by level, and the part an old link asks for.

import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { type Mode, UiContext } from "../ui";
import { stack } from "./Stack";

const Timeline = () => <p>timeline part</p>;
const Packages = () => <p>packages part</p>;
const Merged = stack(["timeline", Timeline], ["packages", Packages, "advanced"]);

function show(mode: Mode, hash: string) {
  window.location.hash = hash;
  render(<UiContext.Provider value={{ mode, range: "24h" }}><Merged /></UiContext.Provider>);
}

afterEach(() => {
  cleanup();
  window.location.hash = "";
});

describe("merged sub-pages", () => {
  it("Beginner sees only the parts for Beginner", () => {
    show("beginner", "#/activity/timeline");
    expect(screen.getByText("timeline part")).toBeTruthy();
    expect(screen.queryByText("packages part")).toBeNull();
  });

  it("an old link shows the part it asks for, even in Beginner, with its anchor", () => {
    show("beginner", "#/activity/packages");
    expect(screen.getByText("packages part")).toBeTruthy();
    expect(document.getElementById("sec-packages")).toBeTruthy();
  });

  it("Advanced sees every part", () => {
    show("advanced", "#/activity/timeline");
    expect(screen.getByText("packages part")).toBeTruthy();
  });
});
