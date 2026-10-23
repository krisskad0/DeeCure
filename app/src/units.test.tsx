// SPDX-License-Identifier: Apache-2.0
// Temperature unit and refresh interval (Settings › General).

import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { REFRESH_EVENT, refreshMs, temp, tempUnit, tempValue } from "./units";
import { useLive } from "./useLive";

const setPref = (key: string, value: string) => window.localStorage.setItem(`deecure.${key}`, JSON.stringify(value));

afterEach(() => {
  cleanup();
  window.localStorage.clear();
  vi.useRealTimers();
});

describe("units", () => {
  it("shows Celsius by default and Fahrenheit when chosen", () => {
    expect(temp(54)).toBe("54 °C");
    setPref("temp", "fahrenheit");
    expect(temp(54)).toBe("129 °F");
    expect(tempValue(100)).toBe(212);
    expect(tempUnit()).toBe("°F");
  });

  it("maps refresh choices", () => {
    expect(refreshMs("2s")).toBe(2000);
    expect(refreshMs("manual")).toBeNull();
  });
});

function Counter({ every }: { every: number }) {
  const { data } = useLive(() => Promise.resolve(Date.now()), every);
  return <span data-testid="v">{data === undefined ? "none" : "loaded"}</span>;
}

describe("live refresh", () => {
  it("manual mode loads once, and again on Refresh", async () => {
    setPref("refresh", "manual");
    vi.useFakeTimers();
    const load = vi.fn(() => Promise.resolve(1));
    function Probe() {
      useLive(load, 5_000);
      return null;
    }
    render(<Probe />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(60_000);
    });
    expect(load).toHaveBeenCalledTimes(1);
    await act(async () => {
      window.dispatchEvent(new Event(REFRESH_EVENT));
    });
    expect(load).toHaveBeenCalledTimes(2);
  });

  it("live panels follow the chosen interval; slow ones keep theirs", async () => {
    setPref("refresh", "2s");
    vi.useFakeTimers();
    const fast = vi.fn(() => Promise.resolve(1));
    const slow = vi.fn(() => Promise.resolve(1));
    function Probe() {
      useLive(fast, 10_000);
      useLive(slow, 300_000);
      return null;
    }
    render(<Probe />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(10_000);
    });
    expect(fast).toHaveBeenCalledTimes(6); // at once, then every 2 s
    expect(slow).toHaveBeenCalledTimes(1);
    render(<Counter every={5_000} />);
    expect(screen.getByTestId("v")).toBeTruthy();
  });
});
