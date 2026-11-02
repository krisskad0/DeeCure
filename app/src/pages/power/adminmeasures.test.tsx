// SPDX-License-Identifier: Apache-2.0
// The admin-only power measurements, with answers shaped like the helper's.

import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { LowPowerView, PowerMetersView, WakeupsView, duration, zoneName } from "./AdminMeasures";

afterEach(cleanup);

describe("admin power measurements", () => {
  it("names power zones in plain words", () => {
    expect(zoneName("package-0")).toBe("Whole chip");
    expect(zoneName("package-1")).toBe("Whole chip 1");
    expect(zoneName("uncore")).toBe("Built-in graphics");
    expect(zoneName("odd")).toBe("odd");
  });

  it("formats microseconds", () => {
    expect(duration(40e6)).toBe("40 s");
    expect(duration(125 * 60e6)).toBe("2 h 5 min");
  });

  it("shows measured watts, or why there are none", () => {
    render(<PowerMetersView data={[{ zone: "intel-rapl:0", name: "package-0", watts: 7.5 }]} />);
    expect(screen.getByText("7.50 W")).toBeTruthy();
    cleanup();
    render(<PowerMetersView data={[]} />);
    expect(screen.getByText(/no power meters/)).toBeTruthy();
  });

  it("warns when the chip never reached its deepest sleep", () => {
    render(<LowPowerView data={[{ source: "pmc_core/slp_s0_residency_usec", name: "slp_s0_residency_usec", value: 0 }, { source: "pmc_core/package_cstate_show", name: "Package C10", value: 5000 }]} />);
    expect(screen.getByText(/Never reached/)).toBeTruthy();
    expect(screen.getByText("Package C10")).toBeTruthy();
  });

  it("lists wake-ups with kernel threads marked", () => {
    render(
      <WakeupsView
        data={[
          { pid: 812, name: "gnome-shell", per_second: 120, kernel: false },
          { pid: 15, name: "kworker/0:1", per_second: 10, kernel: true },
        ]}
      />,
    );
    expect(screen.getByText("130")).toBeTruthy();
    expect(screen.getByText("kernel")).toBeTruthy();
  });
});
