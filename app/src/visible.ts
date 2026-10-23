// SPDX-License-Identifier: Apache-2.0
// The pages and sub-pages this version shows. Features that are designed but not built
// yet stay out of the sidebar, the search palette, and the routes, so nothing in the app
// is a placeholder. Phase 4 brings back the Assistant (docs/05-roadmap/pending.md).

import { PAGES, type PageSpec } from "./spec/pages";

/** Pages that are not built yet. */
const LATER_PAGES = new Set(["assistant"]);
/** Sub-pages that are not built yet, as `page/tab`. */
const LATER_TABS = new Set(["settings/assistant"]);

export const VISIBLE_PAGES: PageSpec[] = PAGES.filter((p) => !LATER_PAGES.has(p.id)).map((p) => ({
  ...p,
  tabs: p.tabs.filter((t) => !LATER_TABS.has(`${p.id}/${t.id}`)),
}));
