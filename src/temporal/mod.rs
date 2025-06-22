//! Used internally by library for parsing date and time information from strings
#![allow(clippy::missing_docs_in_private_items)]

use date::find_date;
use jiff::{
    civil::{Date, Time},
    Zoned,
};

pub mod date;
pub mod time;

use date::AsDate;
use time::{find_time, AsTime};

use crate::EventParseError;

/// Tries to find a datetime from the supplied string.
/// The date must be before the time.
/// See [`find_date`] and [`find_time`] for more information on accepted formatting of the date or
/// time.
///
/// Returns the interpreted [`DateTime`], allday?, and the start, end indices of
/// the original string where the datetime was parsed from.
pub fn find_datetime(
    s: &str,
    now: Zoned,
) -> Result<Option<(Date, Option<Time>, usize, usize)>, EventParseError> {
    if let Some((date, date_start, date_end)) = find_date(s) {
        let (_, s_after_date) = s.split_at(date_end);

        let date = date.as_date(now)?;
        let mut end = date_end;
        let time = if let Some((time, _time_start, time_end)) = find_time(s_after_date) {
            end += time_end;
            Some(time.as_time()?)
        } else {
            None
        };
        return Ok(Some((date, time, date_start, end)));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_a() {
        let now = jiff::civil::date(2000, 1, 1).in_tz("UTC").unwrap();
        let (date, time, start, end) = find_datetime("21.11.2004", now)
            .expect("parse failed")
            .expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 10);
        assert_eq!(date.year(), 2004);
        assert_eq!(date.month(), 11);
        assert_eq!(date.day(), 21);
        assert!(time.is_none());
    }
    #[test]
    fn datetime_a() {
        let now = jiff::civil::date(2000, 1, 1).in_tz("UTC").unwrap();
        let (date, time, start, end) = find_datetime("22.9.1999 11:00", now)
            .expect("parse failed")
            .expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 15);
        assert_eq!(date.year(), 1999);
        assert_eq!(date.month(), 9);
        assert_eq!(date.day(), 22);
        let time = time.unwrap();
        assert_eq!(time.hour(), 11);
        assert_eq!(time.minute(), 0);
    }
    #[test]
    fn datetime_b() {
        let now = jiff::civil::date(2000, 1, 1).in_tz("UTC").unwrap();
        let (date, time, start, end) = find_datetime("22.9.1999 11", now)
            .expect("parse failed")
            .expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 12);
        assert_eq!(date.year(), 1999);
        assert_eq!(date.month(), 9);
        assert_eq!(date.day(), 22);
        let time = time.unwrap();
        assert_eq!(time.hour(), 11);
        assert_eq!(time.minute(), 0);
    }
    #[test]
    fn datetime_relative_year_a() {
        let now = jiff::civil::date(2000, 6, 1).in_tz("UTC").unwrap();
        let (date, time, start, end) = find_datetime("22.9. 11", now)
            .expect("parse failed")
            .expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 8);
        assert_eq!(date.year(), 2000);
        assert_eq!(date.month(), 9);
        assert_eq!(date.day(), 22);
        let time = time.unwrap();
        assert_eq!(time.hour(), 11);
        assert_eq!(time.minute(), 0);
    }
    #[test]
    fn datetime_relative_year_b() {
        let now = jiff::civil::date(2000, 6, 1).in_tz("UTC").unwrap();
        let (date, time, start, end) = find_datetime("22.1. 11", now)
            .expect("parse failed")
            .expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 8);
        assert_eq!(date.year(), 2001);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 22);
        let time = time.unwrap();
        assert_eq!(time.hour(), 11);
        assert_eq!(time.minute(), 0);
    }

    #[test]
    fn datetime_relative() {
        let now = jiff::civil::date(2000, 1, 2).in_tz("UTC").unwrap();
        let (date, time, start, end) = find_datetime("tomorrow 0:30:12", now)
            .expect("parse failed")
            .expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 16);
        assert_eq!(date.year(), 2000);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 3);
        let time = time.unwrap();
        assert_eq!(time.hour(), 0);
        assert_eq!(time.minute(), 30);
        assert_eq!(time.second(), 12);
    }

    #[test]
    fn datetime_relative_weekday_a() {
        let now = jiff::civil::date(2024, 12, 8).in_tz("UTC").unwrap();
        let (date, time, start, end) = find_datetime("next monday 0:30:12", now)
            .expect("parse failed")
            .expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 19);
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 9);
        let time = time.unwrap();
        assert_eq!(time.hour(), 0);
        assert_eq!(time.minute(), 30);
        assert_eq!(time.second(), 12);
    }
    #[test]
    fn datetime_relative_weekday_b() {
        let now = jiff::civil::date(2024, 12, 8).in_tz("UTC").unwrap();
        let (date, time, start, end) = find_datetime("last sunday 0:30:12", now)
            .expect("parse failed")
            .expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 19);
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 1);
        let time = time.unwrap();
        assert_eq!(time.hour(), 0);
        assert_eq!(time.minute(), 30);
        assert_eq!(time.second(), 12);
    }
    #[test]
    fn datetime_relative_weekday_c() {
        let now = jiff::civil::date(2024, 12, 8).in_tz("UTC").unwrap();
        let (date, time, start, end) = find_datetime("last wednesday 0:30:12", now)
            .expect("parse failed")
            .expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 22);
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 4);
        let time = time.unwrap();
        assert_eq!(time.hour(), 0);
        assert_eq!(time.minute(), 30);
        assert_eq!(time.second(), 12);
    }
}
