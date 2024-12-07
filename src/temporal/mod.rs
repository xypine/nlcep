//! Used internally by library for parsing date and time information from strings
#![allow(clippy::missing_docs_in_private_items)]

use date::find_date;
use jiff::civil::DateTime;

pub mod date;
pub mod time;

use date::AsDate;
use time::{find_time, AsTime};

/// Tries to find a datetime from the supplied string.
/// The date must be before the time.
/// See [`find_date`] and [`find_time`] for more information on accepted formatting of the date or
/// time.
///
/// Returns the interpreted [`DateTime`] and the start, end indices of
/// the original string where the datetime was parsed from.
pub fn find_datetime(s: &str) -> Option<(DateTime, usize, usize)> {
    let (date, date_start, date_end) = find_date(s)?;
    let (_, s_after_date) = s.split_at(date_end);

    let date = date.as_date();
    let mut end = date_end;
    let dt = if let Some((time, _time_start, time_end)) = find_time(s_after_date) {
        end += time_end;
        date.to_datetime(time.as_time())
    } else {
        date.into()
    };
    Some((dt, date_start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_a() {
        let (dt, start, end) = find_datetime("21.11.2004").expect("parse failed");
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
        let (dt, start, end) = find_datetime("22.9.1999 11:00").expect("parse failed");
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
        let (dt, start, end) = find_datetime("22.9.1999 11").expect("parse failed");
        assert_eq!(start, 0);
        assert_eq!(end, 12);
        assert_eq!(dt.year(), 1999);
        assert_eq!(dt.month(), 9);
        assert_eq!(dt.day(), 22);
        assert_eq!(dt.hour(), 11);
        assert_eq!(dt.minute(), 0);
    }
    #[test]
    fn datetime_c() {
        let (dt, start, end) = find_datetime("22.9. 11").expect("parse failed");
        assert_eq!(start, 0);
        assert_eq!(end, 8);
        assert_eq!(dt.month(), 9);
        assert_eq!(dt.day(), 22);
        assert_eq!(dt.hour(), 11);
        assert_eq!(dt.minute(), 0);
    }

    #[test]
    fn datetime_relative() {
        let (dt, start, end) = find_datetime("tomorrow 0:30:12").expect("parse failed");
        assert_eq!(start, 0);
        assert_eq!(end, 16);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 12);
    }
}
