// SPDX-License-Identifier: Apache-2.0
// The pages and sub-pages this version shows. Features that are designed but not built
// yet stay out of the sidebar, the search palette, and the routes, so nothing in the app
// is a placeholder. Phase 4 brings back the Assistant (docs/05-roadmap/pending.md).
//
// Beginner mode keeps the menus short: the sidebar, the page tabs, and the search palette
// show only BEGINNER. Links inside pages still open any visible page, and the page the
// user is on stays in the menus, so no link ever lands on a blank screen.

import { PAGES, type PageSpec } from "./spec/pages";
import type { Mode } from "./ui";

/** Pages that are not built yet. */
const LATER_PAGES = new Set(["assistant"]);
/** Sub-pages that are not built yet, as `page/tab`. */
const LATER_TABS = new Set(["settings/assistant"]);

export const VISIBLE_PAGES: PageSpec[] = PAGES.filter((p) => !LATER_PAGES.has(p.id)).map((p) => ({
  ...p,
  tabs: p.tabs.filter((t) => !LATER_TABS.has(`${p.id}/${t.id}`)),
}));

/** Beginner mode: what a newcomer needs, by page ID; "all" keeps every sub-page. */
const BEGINNER: Record<string, "all" | readonly string[]> = {
  overview: "all",
  performance: ["cpu", "memory"],
  activity: ["timeline"],
  system: ["overview", "updates"],
  power: "all",
  storage: ["overview", "cleanup"],
  security: ["overview"],
  settings: "all",
};

/** Whether a sub-page is in Beginner mode's menus. */
export function inBeginner(pageId: string, tabId: string): boolean {
  const tabs = BEGINNER[pageId];
  return tabs === "all" || (tabs?.includes(tabId) ?? false);
}

/**
 * The pages and sub-pages the menus show in a mode. Expert mode shows every visible page.
 * Beginner mode shows BEGINNER, plus the page and sub-page the user is on.
 */
export function menuPages(mode: Mode, current?: { pageId: string; tabId: string }): PageSpec[] {
  if (mode === "expert") return VISIBLE_PAGES;
  const shown = (pageId: string, tabId: string) => inBeginner(pageId, tabId) || (current?.pageId === pageId && current.tabId === tabId);
  return VISIBLE_PAGES.map((p) => ({ ...p, tabs: p.tabs.filter((t) => shown(p.id, t.id)) })).filter((p) => p.tabs.length > 0);
}
