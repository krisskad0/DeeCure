// SPDX-License-Identifier: Apache-2.0
// Panels stop reading while the window is hidden in the tray, and read at once when shown.

import { act, cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useLive } from "./useLive";
import { isWindowShown, setShownByApp } from "./windowShown";

vi.mock("./api", () => ({ toAppError: (e: unknown) => ({ kind: "other", message: String(e) }) }));

afterEach(() => {
  cleanup();
  setShownByApp(true);
  vi.useRealTimers();
});

function Panel({ load }: { load: () => Promise<number> }) {
  useLive(load, 60_000);
  return null;
}

describe("a hidden window reads nothing", () => {
  it("pauses while hidden and reads again when shown", async () => {
    vi.useFakeTimers();
    const load = vi.fn(() => Promise.resolve(1));
    render(<Panel load={load} />);
    expect(load).toHaveBeenCalledTimes(1);
    await act(async () => vi.advanceTimersByTime(60_000));
    expect(load).toHaveBeenCalledTimes(2);

    act(() => setShownByApp(false));
    expect(isWindowShown()).toBe(false);
    await act(async () => vi.advanceTimersByTime(10 * 60_000));
    expect(load).toHaveBeenCalledTimes(2);

    act(() => setShownByApp(true));
    expect(load).toHaveBeenCalledTimes(3);
    await act(async () => vi.advanceTimersByTime(60_000));
    expect(load).toHaveBeenCalledTimes(4);
  });

  it("follows a minimized window too", () => {
    const load = vi.fn(() => Promise.resolve(1));
    Object.defineProperty(document, "visibilityState", { value: "hidden", configurable: true });
    render(<Panel load={load} />);
    act(() => void document.dispatchEvent(new Event("visibilitychange")));
    expect(isWindowShown()).toBe(false);
    Object.defineProperty(document, "visibilityState", { value: "visible", configurable: true });
    act(() => void document.dispatchEvent(new Event("visibilitychange")));
    expect(isWindowShown()).toBe(true);
  });
});
