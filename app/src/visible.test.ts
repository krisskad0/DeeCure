// SPDX-License-Identifier: Apache-2.0
// Features that are not built yet stay out of the app.

import { describe, expect, it } from "vitest";
import { parseHash } from "./route";
import { VISIBLE_PAGES } from "./visible";

describe("visible pages", () => {
  it("leaves out the assistant until it is built", () => {
    expect(VISIBLE_PAGES.some((p) => p.id === "assistant")).toBe(false);
    const settings = VISIBLE_PAGES.find((p) => p.id === "settings");
    expect(settings?.tabs.map((t) => t.id)).not.toContain("assistant");
    expect(settings?.tabs.map((t) => t.id)).toContain("notifications");
  });

  it("old links to hidden pages land on a real page", () => {
    expect(parseHash("#/assistant/chat").page.id).toBe("overview");
    const r = parseHash("#/settings/assistant");
    expect([r.page.id, r.tab.id]).toEqual(["settings", "general"]);
  });
});
