// SPDX-License-Identifier: Apache-2.0
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { UiContext } from "../ui";
import { type Column, DataTable, compareCells, sortNumber } from "./DataTable";

afterEach(cleanup);

describe("sorting values", () => {
  it("reads numbers with units, sizes in MB", () => {
    expect(sortNumber("22 MB")).toBe(22);
    expect(sortNumber("1.5 GB")).toBe(1500);
    expect(sortNumber("820 KB")).toBeCloseTo(0.82);
    expect(sortNumber("94%")).toBe(94);
    expect(sortNumber("1,240")).toBe(1240);
    expect(sortNumber("6.8.0-47-generic")).toBeNull();
    expect(sortNumber("—")).toBeNull();
  });

  it("compares numbers as numbers and text as text", () => {
    expect(["2.1 GB", "96 MB", "820 MB"].sort(compareCells)).toEqual(["96 MB", "820 MB", "2.1 GB"]);
    expect(["b.service", "A.service", "c.service"].sort(compareCells)).toEqual(["A.service", "b.service", "c.service"]);
  });
});

type Row = { name: string; state: string; mem: string };
const rows: Row[] = Array.from({ length: 15 }, (_, i) => ({ name: `unit${i}`, state: i === 3 ? "failed" : "active", mem: `${i * 10} MB` }));
const columns: Column<Row>[] = [
  { key: "name", label: "Unit", text: (r) => r.name },
  { key: "state", label: "Active", text: (r) => r.state },
  { key: "mem", label: "Memory", text: (r) => r.mem, numeric: true, expert: true },
];
const names = () => screen.getAllByRole("row").slice(1).map((tr) => tr.querySelector("td")?.textContent);

describe("data table", () => {
  it("pages, searches, and filters", () => {
    render(<DataTable rows={rows} columns={columns} rowKey={(r) => r.name} search={{ placeholder: "Filter units" }} filter={{ column: "state", options: ["active", "failed"] }} pageSize={10} total={181} />);
    expect(names()).toHaveLength(10);
    expect(screen.getByText("1–10 of 181")).toBeTruthy();
    fireEvent.click(screen.getByText("Next"));
    expect(names()).toHaveLength(5);

    fireEvent.click(screen.getByRole("button", { name: "failed" }));
    expect(names()).toEqual(["unit3"]);

    fireEvent.click(screen.getByRole("button", { name: "All" }));
    fireEvent.change(screen.getByLabelText("Filter units"), { target: { value: "UNIT1" } });
    expect(names()).toEqual(["unit1", "unit10", "unit11", "unit12", "unit13", "unit14"]);
  });

  it("sorts by a column, then reverses", () => {
    render(<DataTable rows={rows.slice(0, 3)} columns={columns} rowKey={(r) => r.name} />);
    fireEvent.click(screen.getByRole("button", { name: "Unit" }));
    fireEvent.click(screen.getByRole("button", { name: /Unit/ }));
    expect(names()).toEqual(["unit2", "unit1", "unit0"]);
  });

  it("hides Expert-only columns in Beginner mode", () => {
    const { rerender } = render(<DataTable rows={rows} columns={columns} rowKey={(r) => r.name} />);
    expect(screen.queryByText("Memory")).toBeNull();
    rerender(
      <UiContext.Provider value={{ mode: "expert", range: "24h" }}>
        <DataTable rows={rows} columns={columns} rowKey={(r) => r.name} />
      </UiContext.Provider>,
    );
    expect(screen.getByText("Memory")).toBeTruthy();
  });

  it("shows the empty text when nothing matches", () => {
    render(<DataTable rows={[]} columns={columns} rowKey={(r) => r.name} empty="No units" />);
    expect(screen.getByText("No units")).toBeTruthy();
  });
});
