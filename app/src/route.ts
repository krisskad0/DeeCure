// SPDX-License-Identifier: Apache-2.0
// Where the user is: #/<page>/<sub-page>[/<part>]. Unknown parts fall back to the first page or
// tab. Sub-pages merged into another (docs/01-product/view-levels.md) keep working: an old link
// opens the merged sub-page and scrolls to that part.

import { useEffect, useState } from "react";
import type { PageSpec } from "./spec/pages";
import { VISIBLE_PAGES as PAGES } from "./visible";

export interface Route {
  page: PageSpec;
  tab: PageSpec["tabs"][number];
  /** The development-only component gallery (#/gallery). */
  gallery: boolean;
  /** The part of a merged sub-page to scroll to. */
  section?: string;
}

/** Old sub-pages, as `page/tab`, and where they live now. */
export const MERGED: Record<string, { tab: string; section: string }> = {
  "logs/kernel": { tab: "journal", section: "kernel" },
  "logs/crashes": { tab: "errors", section: "crashes" },
  "activity/packages": { tab: "timeline", section: "packages" },
  "activity/devices": { tab: "timeline", section: "devices" },
  "activity/deecure": { tab: "timeline", section: "deecure" },
  "jobs/cron": { tab: "all", section: "cron" },
  "jobs/timers": { tab: "all", section: "timers" },
  "jobs/anacron": { tab: "all", section: "anacron" },
  "jobs/at": { tab: "all", section: "at" },
  "containers/volumes": { tab: "images", section: "volumes" },
  "backups/snapshots": { tab: "jobs", section: "snapshots" },
  "accounts/groups": { tab: "users", section: "groups" },
  "accounts/logins": { tab: "sessions", section: "logins" },
};

export function parseHash(hash: string): Route {
  const [pageId, rawTab, rawSection] = hash.replace(/^#\/?/, "").split("/");
  const merged = MERGED[`${pageId}/${rawTab}`];
  const tabId = merged?.tab ?? rawTab;
  const page = PAGES.find((p) => p.id === pageId) ?? PAGES[0];
  const tab = page.tabs.find((t) => t.id === tabId) ?? page.tabs[0];
  const section = merged?.section ?? (rawSection || undefined);
  return { page, tab, gallery: import.meta.env.DEV && pageId === "gallery", section };
}

export function href(pageId: string, tabId?: string): string {
  const merged = tabId ? MERGED[`${pageId}/${tabId}`] : undefined;
  if (merged) return `#/${pageId}/${merged.tab}/${merged.section}`;
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
