// SPDX-License-Identifier: Apache-2.0
// Helpers for tests: DeeCure's own dropdown is opened and picked like a person does.

import { fireEvent, screen } from "@testing-library/react";

/** Opens the dropdown with this label and picks the option with this value. */
export function choose(label: string, value: string) {
  fireEvent.click(screen.getByRole("combobox", { name: label }));
  const option = screen.getAllByRole("option").find((o) => o.getAttribute("data-value") === value);
  if (!option) throw new Error(`${label} has no option ${value}`);
  fireEvent.mouseDown(option);
}

/** The value the dropdown with this label shows. */
export function chosen(label: string): string | null {
  return screen.getByRole("combobox", { name: label }).getAttribute("data-value");
}

/** The option values of the dropdown with this label (it is opened and closed again). */
export function choices(label: string): string[] {
  const box = screen.getByRole("combobox", { name: label });
  fireEvent.click(box);
  const values = screen.getAllByRole("option").map((o) => o.getAttribute("data-value") ?? "");
  fireEvent.click(box);
  return values;
}
