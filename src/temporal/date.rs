use std::str::FromStr;

use jiff::{civil::{date, Date}, ToSpan, Zoned};

use crate::EventParseError;

pub trait AsDate {
    fn as_date(&self) -> Result<Date, EventParseError>;
}

#[derive(Debug, PartialEq)]
pub enum DateRelativeLanguage {
    English,
    Finnish
}

/// "Natural language" date formats
#[derive(Debug, PartialEq)]
pub enum DateRelative {
    Tomorrow(DateRelativeLanguage),
}
impl FromStr for DateRelative {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tomorrow" => Ok(Self::Tomorrow(DateRelativeLanguage::English)),
            "huomenna" => Ok(Self::Tomorrow(DateRelativeLanguage::Finnish)),
            _ => Err(())
        }
    }
}
impl AsDate for DateRelative {
    fn as_date(&self) -> Result<Date, EventParseError> {
        match self {
            DateRelative::Tomorrow(_) => {
                let today = Zoned::now();
                let tomorrow = today.checked_add(1.day()).map_err(|_e| EventParseError::AmbiguousTime)?;
                Ok(tomorrow.into())
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DateYMD(u16, u8, u8);
impl FromStr for DateYMD {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.splitn(3, '.');
        let date = split.next().ok_or(())?.parse::<u8>().map_err(|_e| ())?;
        let month = split.next().ok_or(())?.parse::<u8>().map_err(|_e| ())?;
        let year = split.next().ok_or(())?.parse::<u16>().map_err(|_e| ())?;
        Ok(Self(year, month, date))
    }
}
#[derive(Debug, PartialEq)]
pub struct DateMD(u8, u8);
impl FromStr for DateMD {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.splitn(3, '.');
        let date = split.next().ok_or(())?.parse::<u8>().map_err(|_e| ())?;
        let month = split.next().ok_or(())?.parse::<u8>().map_err(|_e| ())?;
        Ok(Self(month, date))
    }
}


#[derive(Debug, PartialEq)]
pub enum DateStructured {
    Ymd(i16, i8, i8),
    Hms(i8, i8)
}
impl FromStr for DateStructured {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut split_by_dots = string.split('.');
        let date = split_by_dots.next().ok_or(())?.parse::<i8>().map_err(|_e| ())?;
        let month = split_by_dots.next().ok_or(())?.parse::<i8>().map_err(|_e| ())?;
        if let Some(year_segment) = split_by_dots.next().filter(|s| !s.is_empty()) {
            let year = year_segment.parse::<i16>().map_err(|_e| ())?;
            return Ok(Self::Ymd(year, month, date));
        };
        Ok(Self::Hms(month, date))
    }
}
impl AsDate for DateStructured {
    fn as_date(&self) -> Result<Date, EventParseError> {
        match self {
            DateStructured::Ymd(year, month, day) => Ok(date(*year, *month, *day)),
            DateStructured::Hms(month, day) => {
                let current_year = Zoned::now().year();
                Ok(date(current_year, *month, *day))
            }
        }
    }
}


#[derive(Debug, PartialEq)]
pub enum DateUnit {
    Structured(DateStructured),
    Relative(DateRelative)
}
impl AsDate for DateUnit {
    fn as_date(&self) -> Result<Date, EventParseError> {
        match self {
            DateUnit::Structured(structured) => structured.as_date(),
            DateUnit::Relative(relative) => relative.as_date(),
        }
    }
}

/// Tries to find a date from the supplied string.
/// The date can be expressed as
/// - a full gregorian calendar date in (d)d.(m)m.(yyy)y: 8.12.2000, 13.04.2004, 1.1.0
/// - next matching (d)d.(m)m. gregorian calendar date: 8.12., 13.04., 1.1.
///   - If the date is currently 01.06.2019, the strings above will be parsed as: 8.12.2019,
///     13.04.2020, 1.1.2020
/// - a relative date, such as:
///   - tomorrow
///   - (not implemented yet) yesterday
///   - (not implemented yet) (next/last) (weekday/"context event")
///   - (not implemented yet) (weekday) (after/before) ("context event")
pub fn find_date(s: &str) -> Option<(DateUnit, usize, usize)> {
    let mut start = 0;
    for word in s.split([' ', ',']) {
        let end = start + word.len();
        if let Ok(unit) = word.parse::<DateRelative>() {
            return Some((DateUnit::Relative(unit), start, end));
        }
        if let Ok(unit) = word.parse::<DateStructured>() {
            return Some((DateUnit::Structured(unit), start, end));
        }

        start = end + 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_date_trivial_month_date_a() {
        let (unit, start, end) = find_date("John's birthday 18.11.").expect("parse failed");
        assert_eq!(unit, DateUnit::Structured(DateStructured::Hms(11, 18)));
        assert_eq!(start, 16);
        assert_eq!(end, 22);
    }
    #[test]
    fn find_date_trivial_month_date_b() {
        let (unit, start, end) = find_date("Meet with Evelyn 1.12.").expect("parse failed");
        assert_eq!(unit, DateUnit::Structured(DateStructured::Hms(12, 1)));
        assert_eq!(start, 17);
        assert_eq!(end, 22);
    }
    #[test]
    fn find_date_trivial_month_date_c() {
        let (unit, start, end) = find_date("Meet with Evelyn 12.1.").expect("parse failed");
        assert_eq!(unit, DateUnit::Structured(DateStructured::Hms(1, 12)));
        assert_eq!(start, 17);
        assert_eq!(end, 22);
    }
    #[test]
    fn find_date_trivial_year_month_date() {
        let (unit, start, end) = find_date("John's birthday 18.11.2004").expect("parse failed");
        assert_eq!(unit, DateUnit::Structured(DateStructured::Ymd(2004, 11, 18)));
        assert_eq!(start, 16);
        assert_eq!(end, 26);
    }
    #[test]
    fn find_date_relative_a() {
        let (unit, start, end) = find_date("John's birthday tomorrow").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::Tomorrow(DateRelativeLanguage::English)));
        assert_eq!(start, 16);
        assert_eq!(end, 24);
    }

    #[test]
    fn find_date_whitespace_a() {
        let (unit, start, end) = find_date(" John's birthday tomorrow").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::Tomorrow(DateRelativeLanguage::English)));
        assert_eq!(start, 17);
        assert_eq!(end, 25);
    }
    #[test]
    fn find_date_whitespace_b() {
        let (unit, start, end) = find_date("  John's birthday tomorrow ").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::Tomorrow(DateRelativeLanguage::English)));
        assert_eq!(start, 18);
        assert_eq!(end, 26);
    }
    #[test]
    fn find_date_whitespace_c() {
        let (unit, start, end) = find_date("John's birthday  tomorrow ").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::Tomorrow(DateRelativeLanguage::English)));
        assert_eq!(start, 17);
        assert_eq!(end, 25);
    }
    #[test]
    fn find_date_whitespace_d() {
        let (unit, start, end) = find_date(" John's  birthday   tomorrow ").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::Tomorrow(DateRelativeLanguage::English)));
        assert_eq!(start, 20);
        assert_eq!(end, 28);
    }
}
