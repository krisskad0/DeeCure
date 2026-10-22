// SPDX-License-Identifier: Apache-2.0
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { BarChart } from "./BarChart";
import { Donut } from "./Donut";
import { Gauge } from "./Gauge";
import { Heatmap } from "./Heatmap";
import { LineChart } from "./LineChart";
import { barLabelEvery, formatValue, heatLevel, labelArrayIndices, niceCeil, timeLabel, timeLabelIndices } from "./scale";

afterEach(cleanup);

describe("scale helpers", () => {
  it("niceCeil picks 1, 2, 2.5, 5, or 10 × 10ⁿ", () => {
    expect(niceCeil(0.8)).toBe(1);
    expect(niceCeil(13.8)).toBe(20);
    expect(niceCeil(21)).toBe(25);
    expect(niceCeil(47.4)).toBe(50);
    expect(niceCeil(69)).toBe(100);
    expect(niceCeil(100)).toBe(100);
    expect(niceCeil(0)).toBe(1);
  });

  it("formatValue rounds at 10 and above, keeps one decimal below", () => {
    expect(formatValue(94.24, "%")).toBe("94%");
    expect(formatValue(1.25, "MB/s")).toBe("1.3 MB/s");
    expect(formatValue(3, "s")).toBe("3 s");
    expect(formatValue(54, "°C")).toBe("54°C");
  });

  it("time labels follow the range", () => {
    const t = new Date(2026, 9, 21, 21, 15).getTime();
    expect(timeLabel(t, "1h", false)).toBe("21:15");
    expect(timeLabel(t, "24h", true)).toBe("Wed 21:15");
    expect(timeLabel(t, "7d", false)).toBe("Wed");
    expect(timeLabel(t, "30d", false)).toBe("21 Oct");
  });

  it("label positions", () => {
    expect(timeLabelIndices(96)).toEqual([0, 24, 48, 71, 95]);
    expect(labelArrayIndices(30)).toEqual([0, 5, 10, 15, 20, 25, 29]);
    expect(barLabelEvery(10, 600)).toBe(1);
    expect(barLabelEvery(30, 200)).toBe(8);
  });

  it("heat levels", () => {
    expect(heatLevel(0, 9)).toBe(1);
    expect(heatLevel(1, 9)).toBe(2);
    expect(heatLevel(9, 9)).toBe(5);
  });
});

describe("charts", () => {
  const times = Array.from({ length: 5 }, (_, i) => i * 60_000);

  it("a line chart shows a legend only with two or more series", () => {
    const { container, rerender } = render(<LineChart x={{ kind: "time", times, range: "1h" }} series={[{ name: "CPU", data: [1, 2, 3, 4, 5] }]} />);
    expect(container.querySelector(".legend")).toBeNull();
    rerender(
      <LineChart
        x={{ kind: "time", times, range: "1h" }}
        series={[
          { name: "CPU", data: [1, 2, 3, 4, 5] },
          { name: "Memory", data: [5, 4, 3, 2, 1] },
        ]}
      />,
    );
    expect(container.querySelector(".legend")?.textContent).toBe("CPUMemory");
    expect(screen.getByText("Now")).toBeTruthy();
  });

  it("a line chart with too little data says so", () => {
    render(<LineChart x={{ kind: "labels", labels: ["a"] }} series={[{ name: "x", data: [1] }]} />);
    expect(screen.getByText("Not enough data yet")).toBeTruthy();
  });

  it("highlighted bars show their value", () => {
    render(<BarChart unit="s" highlight={[1]} bars={[{ label: "a", value: 29 }, { label: "Today", value: 41.2 }]} />);
    expect(screen.getByText("41 s")).toBeTruthy();
  });

  it("the donut legend shows shares that add up", () => {
    const { container } = render(
      <Donut center={{ value: "50%" }} segments={[{ name: "Used", value: 1 }, { name: "Free", value: 1, tone: "neutral" }]} />,
    );
    const shares = [...container.querySelectorAll(".donut-legend tr td:last-child")].map((td) => td.textContent);
    expect(shares).toEqual(["50%", "50%"]);
  });

  it("the gauge clamps its value", () => {
    const { container } = render(<Gauge value={150} max={100} />);
    const arc = container.querySelectorAll("path")[1].getAttribute("d");
    expect(arc).toContain("180.00 96.00");
  });

  it("heatmap cells get a level and a readable tooltip", () => {
    render(<Heatmap unit="events" rows={[{ label: "Wed", values: [0, 9, ...Array(22).fill(0)] }]} />);
    expect(screen.getByLabelText("Wed 01:00 · 9 events").className).toContain("h5");
  });
});
