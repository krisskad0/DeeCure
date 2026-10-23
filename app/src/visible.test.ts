// SPDX-License-Identifier: Apache-2.0
// Features that are not built yet stay out of the app.

import { describe, expect, it } from "vitest";
import { MERGED, href, parseHash } from "./route";
import { PAGES } from "./spec/pages";
import { type Mode, atLeast } from "./ui";
import { VISIBLE_PAGES, inBeginner, menuPages } from "./visible";

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

  it("Beginner mode keeps the menus short, Expert mode shows everything", () => {
    expect(menuPages("expert")).toBe(VISIBLE_PAGES);
    expect(menuPages("advanced")).toBe(VISIBLE_PAGES);
    const beginner = menuPages("beginner");
    expect(beginner.map((p) => p.id)).toEqual(["overview", "performance", "activity", "system", "power", "storage", "security", "settings"]);
    expect(beginner.find((p) => p.id === "performance")?.tabs.map((t) => t.id)).toEqual(["cpu", "memory", "disk", "network", "gpu"]);
    expect(beginner.find((p) => p.id === "settings")?.tabs.map((t) => t.id)).not.toContain("assistant");
    expect(beginner.reduce((n, p) => n + p.tabs.length, 0)).toBeLessThan(25);
  });

  it("the page the user is on stays in the Beginner menus", () => {
    const onBoots = menuPages("beginner", { pageId: "logs", tabId: "boots" });
    expect(onBoots.find((p) => p.id === "logs")?.tabs.map((t) => t.id)).toEqual(["boots"]);
    const onPsi = menuPages("beginner", { pageId: "performance", tabId: "pressure" });
    expect(onPsi.find((p) => p.id === "performance")?.tabs.map((t) => t.id)).toEqual(["cpu", "memory", "disk", "network", "gpu", "pressure"]);
  });

  it("every Beginner sub-page is a real one (a typo would hide it)", () => {
    const shown = menuPages("beginner").flatMap((p) => p.tabs.map((t) => `${p.id}/${t.id}`));
    expect(shown).toEqual([
      "overview/summary", "overview/issues", "overview/score",
      "performance/cpu", "performance/memory", "performance/disk", "performance/network", "performance/gpu",
      "activity/timeline",
      "system/overview", "system/updates",
      "power/battery", "power/profile", "power/sleep",
      "storage/overview", "storage/cleanup",
      "security/overview",
      "settings/general", "settings/access", "settings/notifications", "settings/data", "settings/about",
    ]);
    expect(inBeginner("jobs", "cron")).toBe(false);
    expect(PAGES.some((p) => p.id === "jobs")).toBe(true);
  });

  it("levels are ordered, and a broken saved value counts as Beginner", () => {
    expect(atLeast("expert", "advanced")).toBe(true);
    expect(atLeast("advanced", "expert")).toBe(false);
    expect(atLeast("beginner", "beginner")).toBe(true);
    expect(atLeast("Expert" as Mode, "advanced")).toBe(false);
    expect(menuPages("nonsense" as Mode).length).toBe(menuPages("beginner").length);
  });

  it("old links to merged sub-pages open the merged one at the right part", () => {
    const r = parseHash("#/jobs/cron");
    expect([r.page.id, r.tab.id, r.section]).toEqual(["jobs", "all", "cron"]);
    expect(parseHash("#/logs/kernel").tab.id).toBe("journal");
    expect(parseHash("#/jobs/all/timers").section).toBe("timers");
    expect(parseHash("#/jobs/all").section).toBeUndefined();
    expect(href("logs", "kernel")).toBe("#/logs/journal/kernel");
    for (const [old, to] of Object.entries(MERGED)) {
      const [pageId] = old.split("/");
      const page = VISIBLE_PAGES.find((p) => p.id === pageId);
      expect(page?.tabs.map((t) => t.id)).toContain(to.tab);
      expect(page?.tabs.map((t) => t.id)).not.toContain(old.split("/")[1]);
    }
  });

  it("the full view has 77 sub-pages", () => {
    expect(VISIBLE_PAGES.reduce((n, p) => n + p.tabs.length, 0)).toBe(77);
  });
});
