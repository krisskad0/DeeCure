// SPDX-License-Identifier: Apache-2.0
// Cron schedules: when they run next (in local time) and what they mean in plain words.

const NICKNAMES: Record<string, string> = {
  "@yearly": "0 0 1 1 *",
  "@annually": "0 0 1 1 *",
  "@monthly": "0 0 1 * *",
  "@weekly": "0 0 * * 0",
  "@daily": "0 0 * * *",
  "@midnight": "0 0 * * *",
  "@hourly": "0 * * * *",
};
const MONTHS = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
const DAYS = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
const DAY_NAMES = ["Sundays", "Mondays", "Tuesdays", "Wednesdays", "Thursdays", "Fridays", "Saturdays"];

export interface Cron {
  minutes: Set<number>;
  hours: Set<number>;
  days: Set<number>;
  months: Set<number>;
  weekdays: Set<number>;
  /** Cron's rule: when both day fields are restricted, either one may match. */
  dayOr: boolean;
}

function field(text: string, min: number, max: number, names: string[] = [], offset = 0): Set<number> | null {
  const out = new Set<number>();
  const value = (v: string) => {
    const i = names.indexOf(v.toLowerCase());
    return i >= 0 ? i + offset : Number(v);
  };
  for (const part of text.split(",")) {
    const [range, stepText] = part.split("/");
    const step = stepText === undefined ? 1 : Number(stepText);
    let lo = min;
    let hi = max;
    if (range !== "*") {
      const [a, b] = range.split("-");
      lo = value(a);
      hi = b === undefined ? (stepText === undefined ? lo : max) : value(b);
    }
    if (![lo, hi, step].every(Number.isInteger) || step < 1 || lo < min || hi > max || lo > hi) return null;
    for (let v = lo; v <= hi; v += step) out.add(v);
  }
  return out;
}

/** `null` for `@reboot` and anything that is not a valid schedule. */
export function parseCron(schedule: string): Cron | null {
  const text = NICKNAMES[schedule.trim().toLowerCase()] ?? schedule.trim();
  const f = text.split(/\s+/);
  if (f.length !== 5) return null;
  const weekdays = field(f[4], 0, 7, DAYS);
  const parts = [field(f[0], 0, 59), field(f[1], 0, 23), field(f[2], 1, 31), field(f[3], 1, 12, MONTHS, 1), weekdays];
  if (parts.some((p) => p === null) || !weekdays) return null;
  if (weekdays.has(7)) weekdays.add(0); // 7 is Sunday too
  const [minutes, hours, days, months] = parts as Set<number>[];
  return { minutes, hours, days, months, weekdays, dayOr: f[2] !== "*" && f[4] !== "*" };
}

function dayMatches(c: Cron, d: Date): boolean {
  if (!c.months.has(d.getMonth() + 1)) return false;
  const dom = c.days.has(d.getDate());
  const dow = c.weekdays.has(d.getDay());
  return c.dayOr ? dom || dow : dom && dow;
}

/** Run times (ms) after `from` and before `until`, at most `limit`. */
export function runsBetween(c: Cron, from: number, until: number, limit = 500): number[] {
  const out: number[] = [];
  const day = new Date(from);
  day.setHours(0, 0, 0, 0);
  for (; day.getTime() <= until && out.length < limit; day.setDate(day.getDate() + 1)) {
    if (!dayMatches(c, day)) continue;
    for (const h of [...c.hours].sort((a, b) => a - b)) {
      for (const m of [...c.minutes].sort((a, b) => a - b)) {
        const t = new Date(day.getFullYear(), day.getMonth(), day.getDate(), h, m).getTime();
        if (t > from && t <= until) out.push(t);
        if (out.length >= limit) return out;
      }
    }
  }
  return out;
}

/** The next run within about 5 years, or `null`. */
export function nextRun(c: Cron, from: number): number | null {
  return runsBetween(c, from, from + 5 * 366 * 86_400_000, 1)[0] ?? null;
}

const two = (n: number) => String(n).padStart(2, "0");

/** "*\/30 * * * *" → "Every 30 minutes". Falls back to the schedule itself. */
export function describeCron(schedule: string): string {
  const s = schedule.trim().toLowerCase();
  if (s === "@reboot") return "At every start";
  const text = NICKNAMES[s] ?? schedule.trim();
  const [m, h, dom, mon, dow] = text.split(/\s+/);
  const every = /^\*\/(\d+)$/;
  if (text.split(/\s+/).length !== 5) return schedule;
  const rest = (a: string, b: string, c: string) => a === "*" && b === "*" && c === "*";
  if (every.test(m) && h === "*" && rest(dom, mon, dow)) return `Every ${m.match(every)?.[1]} minutes`;
  if (m === "*" && h === "*" && rest(dom, mon, dow)) return "Every minute";
  if (/^\d+$/.test(m) && h === "*" && rest(dom, mon, dow)) return `Hourly at :${two(Number(m))}`;
  const stepFrom = m.match(/^(\d+)-\d+\/(\d+)$/);
  if (stepFrom && h === "*" && rest(dom, mon, dow)) return `Every ${stepFrom[2]} minutes from :${two(Number(stepFrom[1]))}`;
  const hours = h.match(/^(\d+)-(\d+)$/);
  if (/^\d+$/.test(m) && hours && rest(dom, mon, dow)) return `Hourly at :${two(Number(m))}, ${two(Number(hours[1]))}:00 to ${two(Number(hours[2]))}:00`;
  if (/^\d+$/.test(m) && /^\d+$/.test(h)) {
    const time = `${two(Number(h))}:${two(Number(m))}`;
    if (rest(dom, mon, dow)) return `Every day at ${time}`;
    if (dom === "*" && mon === "*" && /^[0-7]$/.test(dow)) return `${DAY_NAMES[Number(dow) % 7]} at ${time}`;
    if (/^\d+$/.test(dom) && mon === "*" && dow === "*") return `Monthly on day ${dom} at ${time}`;
  }
  return schedule;
}

const WEEKDAYS: Record<string, string> = { mon: "Mondays", tue: "Tuesdays", wed: "Wednesdays", thu: "Thursdays", fri: "Fridays", sat: "Saturdays", sun: "Sundays" };

/**
 * One systemd timer setting in plain words, such as "*-*-* *:00/10:00" as "Every 10 minutes" or
 * "OnStartupUSec=1min" as "1min after start". Unusual forms come back as they are.
 */
export function describeCalendar(spec: string): string {
  const s = spec.trim();
  const after = s.match(/^On(?:Startup|Boot)USec=(.+)$/);
  if (after) return `${after[1]} after start`;
  const again = s.match(/^OnUnit(?:Active|Inactive)USec=(.+)$/);
  if (again) return `Every ${again[1]}`;
  const nick = s.toLowerCase();
  if (["hourly", "daily", "weekly", "monthly", "yearly"].includes(nick)) return nick[0].toUpperCase() + nick.slice(1);
  const m = s.match(/^(?:([A-Za-z]{3}(?:[.,]+[A-Za-z]{3})*) )?(?:\*-\*-(\*|\d+) )?(\S+)$/);
  if (!m) return spec;
  const [, days, dom = "*", time] = m;
  const [h, min = "00", sec = "00"] = time.split(":");
  if (!/^(\*|\d+)(\/\d+)?$/.test(min) || !/^\d+$/.test(sec)) return spec;
  const mm = (x: string) => two(Number(x.split("/")[0]));
  let when: string;
  if (h === "*") {
    const step = min.match(/\/(\d+)$/);
    when = step ? `Every ${Number(step[1])} minutes` : min === "*" ? "Every minute" : Number(min) === 0 ? "Every hour" : `Hourly at :${mm(min)}`;
  } else if (/^\d+$/.test(h)) when = `at ${two(Number(h))}:${mm(min)}`;
  else if (/^\d+(,\d+)+$/.test(h)) when = `at ${h.split(",").map((x) => `${two(Number(x))}:${mm(min)}`).join(", ")}`;
  else if (/^\d+\.\.\d+$/.test(h)) {
    const [a, b] = h.split("..").map(Number);
    when = `Hourly at :${mm(min)}, ${two(a)}:00 to ${two(b)}:00`;
  } else return spec;
  const daily = when.startsWith("at ");
  if (days) {
    const names = days.split(/[,.]+/).map((d) => WEEKDAYS[d.toLowerCase()] ?? d);
    return `${names.join(", ")} ${daily ? when : when.toLowerCase()}`;
  }
  if (dom !== "*") return `Monthly on day ${Number(dom)}${daily ? ` ${when}` : ""}`;
  return daily ? `Every day ${when}` : when;
}

/** All of a timer's settings in plain words; many daily times become "N times a day". */
export function describeTimer(schedule: string[]): string {
  const parts = schedule.map(describeCalendar);
  const daily = parts.filter((p) => p.startsWith("Every day at "));
  if (daily.length > 3) return [`${daily.length} times a day`, ...parts.filter((p) => !p.startsWith("Every day at "))].join(", ");
  return parts.join(", ") || "-";
}
