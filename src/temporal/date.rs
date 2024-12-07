use std::str::FromStr;

use jiff::{civil::{date, Date}, ToSpan, Zoned};

pub trait AsDate {
    fn as_date(&self) -> Date;
}

#[derive(Debug, PartialEq)]
pub enum DateRelative {
    Tomorrow,
}
impl FromStr for DateRelative {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tomorrow" => Ok(Self::Tomorrow),
            _ => Err(())
        }
    }
}
impl AsDate for DateRelative {
    fn as_date(&self) -> Date {
        match self {
            DateRelative::Tomorrow => {
                let today = Zoned::now();
                today.checked_add(1.day()).expect("Tomorrow is ambiguous!??").into()
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
    YMD(i16, i8, i8),
    MD(i8, i8)
}
impl FromStr for DateStructured {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split_by_dots = s.split('.');
        let date = split_by_dots.next().ok_or(())?.parse::<i8>().map_err(|_e| ())?;
        let month = split_by_dots.next().ok_or(())?.parse::<i8>().map_err(|_e| ())?;
        if let Some(year_segment) = split_by_dots.next().filter(|s| !s.is_empty()) {
            let year = year_segment.parse::<i16>().map_err(|_e| ())?;
            return Ok(Self::YMD(year, month, date));
        };
        Ok(Self::MD(month, date))
    }
}
impl AsDate for DateStructured {
    fn as_date(&self) -> Date {
        match self {
            DateStructured::YMD(year, month, day) => date(*year, *month, *day),
            DateStructured::MD(month, day) => {
                let current_year = Zoned::now().year();
                date(current_year, *month, *day)
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
    fn as_date(&self) -> Date {
        match self {
            DateUnit::Structured(structured) => structured.as_date(),
            DateUnit::Relative(relative) => relative.as_date(),
        }
    }
}
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
        assert_eq!(unit, DateUnit::Structured(DateStructured::MD(11, 18)));
        assert_eq!(start, 16);
        assert_eq!(end, 22);
    }
    #[test]
    fn find_date_trivial_month_date_b() {
        let (unit, start, end) = find_date("Meet with Evelyn 1.12.").expect("parse failed");
        assert_eq!(unit, DateUnit::Structured(DateStructured::MD(12, 1)));
        assert_eq!(start, 17);
        assert_eq!(end, 22);
    }
    #[test]
    fn find_date_trivial_month_date_c() {
        let (unit, start, end) = find_date("Meet with Evelyn 12.1.").expect("parse failed");
        assert_eq!(unit, DateUnit::Structured(DateStructured::MD(1, 12)));
        assert_eq!(start, 17);
        assert_eq!(end, 22);
    }
    #[test]
    fn find_date_trivial_year_month_date() {
        let (unit, start, end) = find_date("John's birthday 18.11.2004").expect("parse failed");
        assert_eq!(unit, DateUnit::Structured(DateStructured::YMD(2004, 11, 18)));
        assert_eq!(start, 16);
        assert_eq!(end, 26);
    }
    #[test]
    fn find_date_relative_a() {
        let (unit, start, end) = find_date("John's birthday tomorrow").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::Tomorrow));
        assert_eq!(start, 16);
        assert_eq!(end, 24);
    }

    #[test]
    fn find_date_whitespace_a() {
        let (unit, start, end) = find_date(" John's birthday tomorrow").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::Tomorrow));
        assert_eq!(start, 17);
        assert_eq!(end, 25);
    }
    #[test]
    fn find_date_whitespace_b() {
        let (unit, start, end) = find_date("  John's birthday tomorrow ").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::Tomorrow));
        assert_eq!(start, 18);
        assert_eq!(end, 26);
    }
    #[test]
    fn find_date_whitespace_c() {
        let (unit, start, end) = find_date("John's birthday  tomorrow ").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::Tomorrow));
        assert_eq!(start, 17);
        assert_eq!(end, 25);
    }
    #[test]
    fn find_date_whitespace_d() {
        let (unit, start, end) = find_date(" John's  birthday   tomorrow ").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::Tomorrow));
        assert_eq!(start, 20);
        assert_eq!(end, 28);
    }
}
