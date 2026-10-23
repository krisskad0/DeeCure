// SPDX-License-Identifier: Apache-2.0
// Where the user is: #/<page>/<sub-page>. Unknown parts fall back to the first page or tab.

import { useEffect, useState } from "react";
import type { PageSpec } from "./spec/pages";
import { VISIBLE_PAGES as PAGES } from "./visible";

export interface Route {
  page: PageSpec;
  tab: PageSpec["tabs"][number];
  /** The development-only component gallery (#/gallery). */
  gallery: boolean;
}

export function parseHash(hash: string): Route {
  const [pageId, tabId] = hash.replace(/^#\/?/, "").split("/");
  const page = PAGES.find((p) => p.id === pageId) ?? PAGES[0];
  const tab = page.tabs.find((t) => t.id === tabId) ?? page.tabs[0];
  return { page, tab, gallery: import.meta.env.DEV && pageId === "gallery" };
}

export function href(pageId: string, tabId?: string): string {
  return tabId ? `#/${pageId}/${tabId}` : `#/${pageId}`;
}

export function useRoute(): Route {
  const [hash, setHash] = useState(window.location.hash);
  useEffect(() => {
    const onChange = () => setHash(window.location.hash);
    window.addEventListener("hashchange", onChange);
    return () => window.removeEventListener("hashchange", onChange);
  }, []);
  return parseHash(hash);
}
