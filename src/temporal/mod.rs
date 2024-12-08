//! Used internally by library for parsing date and time information from strings
#![allow(clippy::missing_docs_in_private_items)]

use date::find_date;
use jiff::{civil::DateTime, Zoned};

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
/// Returns the interpreted [`DateTime`] and the start, end indices of
/// the original string where the datetime was parsed from.
pub fn find_datetime(s: &str, now: Zoned) -> Result<Option<(DateTime, usize, usize)>, EventParseError> {
    if let Some((date, date_start, date_end)) = find_date(s) {
        let (_, s_after_date) = s.split_at(date_end);

        let date = date.as_date(now)?;
        let mut end = date_end;
        let dt = if let Some((time, _time_start, time_end)) = find_time(s_after_date) {
            end += time_end;
            date.to_datetime(time.as_time()?)
        } else {
            date.into()
        };
        return Ok(Some((dt, date_start, end)));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_a() {
        let now = jiff::civil::date(2000, 1, 1).intz("UTC").unwrap();
        let (dt, start, end) = find_datetime("21.11.2004", now).expect("parse failed").expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 10);
        assert_eq!(dt.year(), 2004);
        assert_eq!(dt.month(), 11);
        assert_eq!(dt.day(), 21);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
    }
    #[test]
    fn datetime_a() {
        let now = jiff::civil::date(2000, 1, 1).intz("UTC").unwrap();
        let (dt, start, end) = find_datetime("22.9.1999 11:00", now).expect("parse failed").expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 15);
        assert_eq!(dt.year(), 1999);
        assert_eq!(dt.month(), 9);
        assert_eq!(dt.day(), 22);
        assert_eq!(dt.hour(), 11);
        assert_eq!(dt.minute(), 0);
    }
    #[test]
    fn datetime_b() {
        let now = jiff::civil::date(2000, 1, 1).intz("UTC").unwrap();
        let (dt, start, end) = find_datetime("22.9.1999 11", now).expect("parse failed").expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 12);
        assert_eq!(dt.year(), 1999);
        assert_eq!(dt.month(), 9);
        assert_eq!(dt.day(), 22);
        assert_eq!(dt.hour(), 11);
        assert_eq!(dt.minute(), 0);
    }
    #[test]
    fn datetime_relative_year_a() {
        let now = jiff::civil::date(2000, 6, 1).intz("UTC").unwrap();
        let (dt, start, end) = find_datetime("22.9. 11", now).expect("parse failed").expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 8);
        assert_eq!(dt.year(), 2000);
        assert_eq!(dt.month(), 9);
        assert_eq!(dt.day(), 22);
        assert_eq!(dt.hour(), 11);
        assert_eq!(dt.minute(), 0);
    }
    #[test]
    fn datetime_relative_year_b() {
        let now = jiff::civil::date(2000, 6, 1).intz("UTC").unwrap();
        let (dt, start, end) = find_datetime("22.1. 11", now).expect("parse failed").expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 8);
        assert_eq!(dt.year(), 2001);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 22);
        assert_eq!(dt.hour(), 11);
        assert_eq!(dt.minute(), 0);
    }

    #[test]
    fn datetime_relative() {
        let now = jiff::civil::date(2000, 1, 2).intz("UTC").unwrap();
        let (dt, start, end) = find_datetime("tomorrow 0:30:12", now).expect("parse failed").expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 16);
        assert_eq!(dt.year(), 2000);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 3);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 12);
    }

    #[test]
    fn datetime_relative_weekday_a() {
        let now = jiff::civil::date(2024, 12, 8).intz("UTC").unwrap();
        let (dt, start, end) = find_datetime("next monday 0:30:12", now).expect("parse failed").expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 19);
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 12);
        assert_eq!(dt.day(), 9);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 12);
    }
    #[test]
    fn datetime_relative_weekday_b() {
        let now = jiff::civil::date(2024, 12, 8).intz("UTC").unwrap();
        let (dt, start, end) = find_datetime("last sunday 0:30:12", now).expect("parse failed").expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 19);
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 12);
        assert_eq!(dt.day(), 1);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 12);
    }
    #[test]
    fn datetime_relative_weekday_c() {
        let now = jiff::civil::date(2024, 12, 8).intz("UTC").unwrap();
        let (dt, start, end) = find_datetime("last wednesday 0:30:12", now).expect("parse failed").expect("no parse result");
        assert_eq!(start, 0);
        assert_eq!(end, 22);
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 12);
        assert_eq!(dt.day(), 4);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 12);
    }
}
