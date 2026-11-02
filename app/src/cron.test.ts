// SPDX-License-Identifier: Apache-2.0

import { describe, expect, it } from "vitest";
import { describeCalendar, describeCron, describeTimer, nextRun, parseCron, runsBetween } from "./cron";

const at = (y: number, mo: number, d: number, h = 0, mi = 0) => new Date(y, mo - 1, d, h, mi).getTime();

describe("cron", () => {
  it("describes common schedules", () => {
    expect(describeCron("*/30 * * * *")).toBe("Every 30 minutes");
    expect(describeCron("17 * * * *")).toBe("Hourly at :17");
    expect(describeCron("0 21 * * *")).toBe("Every day at 21:00");
    expect(describeCron("30 3 * * 0")).toBe("Sundays at 03:30");
    expect(describeCron("@daily")).toBe("Every day at 00:00");
    expect(describeCron("@reboot")).toBe("At every start");
    expect(describeCron("5 4 1-7 * 1")).toBe("5 4 1-7 * 1");
    expect(describeCron("30 7-23 * * *")).toBe("Hourly at :30, 07:00 to 23:00");
    expect(describeCron("5-55/10 * * * *")).toBe("Every 10 minutes from :05");
  });

  it("finds the next run in local time", () => {
    const from = at(2026, 10, 21, 21, 34);
    expect(nextRun(parseCron("*/30 * * * *")!, from)).toBe(at(2026, 10, 21, 22, 0));
    expect(nextRun(parseCron("0 21 * * *")!, from)).toBe(at(2026, 10, 22, 21, 0));
    // 21 Oct 2026 is a Wednesday; the next Sunday is the 25th.
    expect(nextRun(parseCron("30 3 * * sun")!, from)).toBe(at(2026, 10, 25, 3, 30));
    expect(nextRun(parseCron("0 0 29 2 *")!, from)).toBe(at(2028, 2, 29));
  });

  it("uses OR when both day fields are set", () => {
    const c = parseCron("0 12 1 * 1")!;
    const runs = runsBetween(c, at(2026, 10, 1), at(2026, 10, 14));
    expect(runs.map((t) => new Date(t).getDate())).toEqual([1, 5, 12]); // day 1, and Mondays 5 and 12
  });

  it("lists every run in the next 24 hours", () => {
    expect(runsBetween(parseCron("*/30 * * * *")!, at(2026, 10, 21, 21, 34), at(2026, 10, 22, 21, 34))).toHaveLength(48);
  });

  it("rejects what is not a schedule", () => {
    expect(parseCron("@reboot")).toBeNull();
    expect(parseCron("61 * * * *")).toBeNull();
    expect(parseCron("* * *")).toBeNull();
  });
});

describe("timer schedules in plain words", () => {
  it("reads common systemd calendar forms", () => {
    expect(describeCalendar("*-*-* *:00/10:00")).toBe("Every 10 minutes");
    expect(describeCalendar("*-*-* *:00:00")).toBe("Every hour");
    expect(describeCalendar("*-*-* 00:00:00")).toBe("Every day at 00:00");
    expect(describeCalendar("*-*-* 6,18:00")).toBe("Every day at 06:00, 18:00");
    expect(describeCalendar("*-*-* 07..23:30:00")).toBe("Hourly at :30, 07:00 to 23:00");
    expect(describeCalendar("Mon *-*-* 00:00:00")).toBe("Mondays at 00:00");
    expect(describeCalendar("*-*-01 03:00:00")).toBe("Monthly on day 1 at 03:00");
    expect(describeCalendar("OnStartupUSec=1min")).toBe("1min after start");
    expect(describeCalendar("weekly")).toBe("Weekly");
    expect(describeCalendar("*-*-* 00:00:00/3")).toBe("*-*-* 00:00:00/3");
  });

  it("folds many daily times", () => {
    expect(describeTimer(["*-*-* 03:00:00", "*-*-* 09:00:00", "*-*-* 15:00:00", "*-*-* 21:00:00"])).toBe("4 times a day");
    expect(describeTimer(["*-*-* 00,12:00:00", "OnStartupUSec=1min"])).toBe("Every day at 00:00, 12:00, 1min after start");
    expect(describeTimer([])).toBe("-");
  });
});
