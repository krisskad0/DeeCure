// SPDX-License-Identifier: Apache-2.0
// Measures an element's width, so SVG charts can be drawn at real pixel size.

import { useCallback, useRef, useState } from "react";

/**
 * A callback ref, so the element is measured whenever it appears: a chart that first shows
 * "Not enough data yet" and draws later would otherwise keep the fallback width.
 */
export function useWidth<T extends HTMLElement>(fallback = 600): [(element: T | null) => void, number] {
  const observer = useRef<ResizeObserver | null>(null);
  const [width, setWidth] = useState(fallback);
  const ref = useCallback((element: T | null) => {
    observer.current?.disconnect();
    observer.current = null;
    if (!element || typeof ResizeObserver === "undefined") return;
    observer.current = new ResizeObserver(([entry]) => setWidth(Math.max(120, Math.floor(entry.contentRect.width))));
    observer.current.observe(element);
  }, []);
  return [ref, width];
}
