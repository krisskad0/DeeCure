// SPDX-License-Identifier: Apache-2.0
// One broken part shows a message and leaves the rest of the page working.

import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ErrorBoundary, Guard } from "./ErrorBoundary";
import { Panel } from "./Panel";

let broken = true;
function Fragile(): React.JSX.Element {
  if (broken) throw new Error("data had an unexpected shape");
  return <p>fine now</p>;
}

afterEach(() => {
  cleanup();
  broken = true;
});

describe("error boundary", () => {
  it("a failing panel body shows the reason; the panel next to it still draws", () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    vi.spyOn(console, "warn").mockImplementation(() => {});
    render(
      <>
        <Panel title="Broken">
          <Fragile />
        </Panel>
        <Panel title="Fine">
          <p>other panel</p>
        </Panel>
      </>,
    );
    expect(screen.getByRole("alert").textContent).toContain("Broken could not be shown (data had an unexpected shape)");
    expect(screen.getByText("other panel")).toBeTruthy();
    broken = false;
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(screen.getByText("fine now")).toBeTruthy();
  });

  it("an Expert section fails alone, and a new reset key clears the error", () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    vi.spyOn(console, "warn").mockImplementation(() => {});
    const { rerender } = render(
      <ErrorBoundary resetKey="a">
        <Guard>
          <Fragile />
        </Guard>
        <p>rest of the tab</p>
      </ErrorBoundary>,
    );
    expect(screen.getByText(/This Expert section could not be shown/)).toBeTruthy();
    expect(screen.getByText("rest of the tab")).toBeTruthy();
    broken = false;
    rerender(
      <ErrorBoundary resetKey="b">
        <p>new page</p>
      </ErrorBoundary>,
    );
    expect(screen.getByText("new page")).toBeTruthy();
  });
});
