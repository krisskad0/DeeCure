// SPDX-License-Identifier: Apache-2.0
// DeeCure's own dropdown: mouse, keyboard, and closing work like a system one.

import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { afterEach, describe, expect, it } from "vitest";
import { Select } from "./FormControls";

afterEach(cleanup);

const OPTIONS = [
  { value: "2s", label: "2 seconds" },
  { value: "5s", label: "5 seconds" },
  { value: "15s", label: "15 seconds" },
  { value: "manual", label: "Manual" },
] as const;

function Demo() {
  const [v, setV] = useState<(typeof OPTIONS)[number]["value"]>("5s");
  return (
    <>
      <Select value={v} options={OPTIONS} onChange={setV} label="Refresh interval" />
      <span data-testid="out">{v}</span>
    </>
  );
}

const box = () => screen.getByRole("combobox", { name: "Refresh interval" });
const value = () => screen.getByTestId("out").textContent;

describe("dropdown", () => {
  it("opens on click, picks with the mouse, and closes", () => {
    render(<Demo />);
    expect(box().textContent).toBe("5 seconds");
    expect(screen.queryByRole("listbox")).toBeNull();
    fireEvent.click(box());
    expect(box().getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByRole("option", { name: "5 seconds" }).getAttribute("aria-selected")).toBe("true");
    fireEvent.mouseDown(screen.getByRole("option", { name: "Manual" }));
    expect(value()).toBe("manual");
    expect(screen.queryByRole("listbox")).toBeNull();
  });

  it("works with the keyboard: arrows, Enter, Escape, and a letter", () => {
    render(<Demo />);
    fireEvent.keyDown(box(), { key: "ArrowDown" });
    expect(screen.getByRole("listbox")).toBeTruthy();
    fireEvent.keyDown(box(), { key: "ArrowDown" });
    fireEvent.keyDown(box(), { key: "Enter" });
    expect(value()).toBe("15s");
    fireEvent.keyDown(box(), { key: "Enter" });
    fireEvent.keyDown(box(), { key: "Escape" });
    expect(screen.queryByRole("listbox")).toBeNull();
    expect(value()).toBe("15s");
    fireEvent.keyDown(box(), { key: "m" });
    fireEvent.keyDown(box(), { key: "Enter" });
    expect(value()).toBe("manual");
  });

  it("closes on a click outside without changing anything", () => {
    render(<Demo />);
    fireEvent.click(box());
    fireEvent.mouseDown(document.body);
    expect(screen.queryByRole("listbox")).toBeNull();
    expect(value()).toBe("5s");
  });
});
