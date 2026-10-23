// SPDX-License-Identifier: Apache-2.0

//! Local time: the computer's time zone, for logs that write local times (apt, dpkg)
//! and for schedules such as quiet hours and the weekly summary.
//!
//! Every function has an `_in` twin that takes the time zone, so tests do not depend
//! on the zone of the machine they run on. If the system zone cannot be read, UTC is used.

use jiff::civil::DateTime;
pub use jiff::tz::TimeZone;

use crate::Timestamp;

/// The parts of a moment in local time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTime {
    /// Such as 2026.
    pub year: i16,
    /// 1 to 12.
    pub month: i8,
    /// 1 to 31.
    pub day: i8,
    /// 0 to 23.
    pub hour: i8,
    /// 0 to 59.
    pub minute: i8,
    /// 1 Monday … 7 Sunday.
    pub weekday: i8,
}

impl LocalTime {
    /// Minutes since local midnight, 0 to 1439.
    pub fn minute_of_day(&self) -> u16 {
        // Both parts are small and never negative.
        self.hour.unsigned_abs() as u16 * 60 + self.minute.unsigned_abs() as u16
    }
}

/// A zone `hours` ahead of UTC with no clock changes, for tests and fixed offsets.
pub fn fixed(hours: i8) -> TimeZone {
    TimeZone::fixed(jiff::tz::offset(hours))
}

/// The computer's time zone.
pub fn zone() -> TimeZone {
    TimeZone::system()
}

fn timestamp(at: Timestamp) -> Option<jiff::Timestamp> {
    jiff::Timestamp::from_millisecond(at.as_millis()).ok()
}

fn to_timestamp(t: jiff::Timestamp) -> Timestamp {
    Timestamp(t.as_millisecond())
}

/// `at` in the computer's time zone.
pub fn parts(at: Timestamp) -> Option<LocalTime> {
    parts_in(at, &zone())
}

/// `at` in time zone `tz`.
pub fn parts_in(at: Timestamp, tz: &TimeZone) -> Option<LocalTime> {
    let z = timestamp(at)?.to_zoned(tz.clone());
    Some(LocalTime {
        year: z.year(),
        month: z.month(),
        day: z.day(),
        hour: z.hour(),
        minute: z.minute(),
        weekday: z.weekday().to_monday_one_offset(),
    })
}

/// Reads a local date and time as logs write it, such as `2026-10-23  09:00:00`
/// (apt, two spaces) or `2026-10-23 09:00:00` (dpkg). A time that a clock change
/// skipped or repeated is read the way most programs do (the earlier one).
pub fn parse(text: &str) -> Option<Timestamp> {
    parse_in(text, &zone())
}

/// Like [`parse`], in time zone `tz`.
pub fn parse_in(text: &str, tz: &TimeZone) -> Option<Timestamp> {
    let mut words = text.split_whitespace();
    let (date, time) = (words.next()?, words.next()?);
    if words.next().is_some() {
        return None;
    }
    let civil: DateTime = format!("{date}T{time}").parse().ok()?;
    Some(to_timestamp(civil.to_zoned(tz.clone()).ok()?.timestamp()))
}

/// The moment of a local date and time, such as Sunday 18:00. `None` if the date
/// does not exist.
pub fn at_in(
    year: i16,
    month: i8,
    day: i8,
    hour: i8,
    minute: i8,
    tz: &TimeZone,
) -> Option<Timestamp> {
    let civil = jiff::civil::DateTime::new(year, month, day, hour, minute, 0, 0).ok()?;
    Some(to_timestamp(civil.to_zoned(tz.clone()).ok()?.timestamp()))
}

/// Local midnight at the start of the day `days` after the day of `at` (negative goes back).
pub fn day_start_in(at: Timestamp, days: i32, tz: &TimeZone) -> Option<Timestamp> {
    let z = timestamp(at)?.to_zoned(tz.clone());
    let date = z.date().checked_add(jiff::Span::new().days(days)).ok()?;
    Some(to_timestamp(date.to_zoned(tz.clone()).ok()?.timestamp()))
}

/// Formats `at` in local time with a `strftime` pattern, such as `%a %d %b %H:%M`.
pub fn format(at: Timestamp, pattern: &str) -> String {
    format_in(at, pattern, &zone())
}

/// Like [`format`], in time zone `tz`.
pub fn format_in(at: Timestamp, pattern: &str, tz: &TimeZone) -> String {
    match timestamp(at) {
        Some(t) => t.to_zoned(tz.clone()).strftime(pattern).to_string(),
        None => "unknown time".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn india() -> TimeZone {
        TimeZone::fixed(jiff::tz::offset(5))
    }

    #[test]
    fn reads_apt_and_dpkg_times() {
        let tz = india();
        // 2026-10-23 09:00:00 +05:00 = 04:00:00 UTC
        let want = Some(Timestamp(1_792_728_000_000));
        assert_eq!(parse_in("2026-10-23  09:00:00", &tz), want);
        assert_eq!(parse_in("2026-10-23 09:00:00", &tz), want);
        assert_eq!(parse_in("2026-10-23", &tz), None);
        assert_eq!(parse_in("yesterday at noon", &tz), None);
        assert_eq!(parse_in("2026-13-40 09:00:00", &tz), None);
    }

    #[test]
    fn splits_a_moment_into_local_parts() {
        let p = parts_in(Timestamp(1_792_728_000_000), &india()).unwrap();
        assert_eq!(
            (p.year, p.month, p.day, p.hour, p.minute),
            (2026, 10, 23, 9, 0)
        );
        assert_eq!(p.weekday, 5, "2026-10-23 is a Friday");
        assert_eq!(p.minute_of_day(), 540);
    }

    #[test]
    fn builds_and_formats_local_moments() {
        let tz = india();
        let sunday = at_in(2026, 10, 25, 18, 0, &tz).unwrap();
        assert_eq!(format_in(sunday, "%a %H:%M", &tz), "Sun 18:00");
        let midnight = day_start_in(sunday, 1, &tz).unwrap();
        assert_eq!(format_in(midnight, "%a %d %H:%M", &tz), "Mon 26 00:00");
        assert_eq!(at_in(2026, 2, 30, 0, 0, &tz), None);
    }

    #[test]
    fn clock_changes_are_handled() {
        // Europe/Berlin moves from 03:00 back to 02:00 on 2026-10-25, so 02:30 happens twice.
        let Ok(tz) = TimeZone::get("Europe/Berlin") else {
            return; // no time zone database on this machine
        };
        let first = parse_in("2026-10-25 02:30:00", &tz).unwrap();
        assert_eq!(format_in(first, "%H:%M %z", &tz), "02:30 +0200");
        // 02:30 on 2026-03-29 never happened; it is read as 03:30 summer time.
        let skipped = parse_in("2026-03-29 02:30:00", &tz).unwrap();
        assert_eq!(format_in(skipped, "%H:%M %z", &tz), "03:30 +0200");
    }
}
