// SPDX-License-Identifier: Apache-2.0
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { UiContext } from "../ui";
import { Checklist } from "./Checklist";
import { InlineMeter } from "./InlineMeter";
import { KpiTile } from "./KpiTile";
import { Panel } from "./Panel";
import { StatusBadge } from "./StatusBadge";

afterEach(cleanup);

describe("components", () => {
  it("a status badge always has a word, not only a color", () => {
    render(<StatusBadge variant="crit" />);
    expect(screen.getByText("Critical")).toBeTruthy();
  });

  it("a KPI tile shows a badge only when its status is not ok", () => {
    const { container, rerender } = render(<KpiTile label="CPU" value="12" unit="%" />);
    expect(container.querySelector(".badge")).toBeNull();
    rerender(<KpiTile label="Disk" value="94" unit="%" status="crit" visual={{ kind: "bar", percent: 94 }} />);
    expect(screen.getByText("Critical")).toBeTruthy();
    expect(container.querySelector(".usage .fill-crit")).toBeTruthy();
  });

  it("the inline meter turns yellow at 80% and red at 90%", () => {
    const fill = (percent: number) => render(<InlineMeter percent={percent} />).container.querySelector(".meter-fill")?.className;
    expect(fill(79)).toContain("meter-ok");
    expect(fill(80)).toContain("meter-warn");
    expect(fill(90)).toContain("meter-crit");
  });

  it("checklist icons have an accessible word", () => {
    render(<Checklist checks={[{ state: "fail", title: "Firewall" }]} />);
    expect(screen.getByRole("img", { name: "Failed" })).toBeTruthy();
  });

  it("the panel source chip appears in Expert mode only", () => {
    const panel = <Panel title="CPU" source="/proc/stat">x</Panel>;
    const { container, rerender } = render(panel);
    expect(container.querySelector(".source")).toBeNull();
    rerender(<UiContext.Provider value={{ mode: "expert", range: "24h" }}>{panel}</UiContext.Provider>);
    expect(container.querySelector(".source")?.textContent).toBe("/proc/stat");
  });
});
