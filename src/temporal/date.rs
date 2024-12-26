use std::str::FromStr;

use jiff::{civil::{date, Date}, ToSpan, Zoned};
use strum::IntoEnumIterator;

use crate::EventParseError;

pub trait AsDate {
    fn as_date(&self, now: Zoned) -> Result<Date, EventParseError>;
}

trait FromMultiword {
    /// usize is the number of words matched
    fn parse_multiword(words: &Vec<String>) -> Option<(Self, usize)> where Self: Sized;
}

#[derive(Debug, Clone, Copy, PartialEq, strum_macros::Display, strum_macros::EnumIter)]
pub enum DateRelativeLanguage {
    English,
    Finnish
}
impl DateRelativeLanguage {
    pub fn get_noun_prev(&self) -> &'static str {
        match self {
            DateRelativeLanguage::English => "last",
            DateRelativeLanguage::Finnish => "viime",
        }
    }
    pub fn get_noun_next(&self) -> &'static str {
        match self {
            DateRelativeLanguage::English => "next",
            DateRelativeLanguage::Finnish => "ensi",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, strum_macros::Display, strum_macros::EnumIter)]
pub enum DateRelativeWeekday {
    Monday,
    Tuesday,
    Wednesday,
    Thurdsday,
    Friday,
    Saturday,
    Sunday
}
impl Into<jiff::civil::Weekday> for DateRelativeWeekday {
    fn into(self) -> jiff::civil::Weekday {
        match self {
            DateRelativeWeekday::Monday     => jiff::civil::Weekday::Monday,
            DateRelativeWeekday::Tuesday    => jiff::civil::Weekday::Tuesday,
            DateRelativeWeekday::Wednesday  => jiff::civil::Weekday::Wednesday,
            DateRelativeWeekday::Thurdsday  => jiff::civil::Weekday::Thursday,
            DateRelativeWeekday::Friday     => jiff::civil::Weekday::Friday,
            DateRelativeWeekday::Saturday   => jiff::civil::Weekday::Saturday,
            DateRelativeWeekday::Sunday     => jiff::civil::Weekday::Sunday,
        }
    }
}
impl DateRelativeWeekday {
    pub fn to_locale_static_str(&self, lang: DateRelativeLanguage) -> &'static str {
        match (self, lang) {
            (DateRelativeWeekday::Monday, DateRelativeLanguage::English) => "monday",
            (DateRelativeWeekday::Monday, DateRelativeLanguage::Finnish) => "maanantaina",

            (DateRelativeWeekday::Tuesday, DateRelativeLanguage::English) => "tuesday",
            (DateRelativeWeekday::Tuesday, DateRelativeLanguage::Finnish) => "tiistaina",

            (DateRelativeWeekday::Wednesday, DateRelativeLanguage::English) => "wednesday",
            (DateRelativeWeekday::Wednesday, DateRelativeLanguage::Finnish) => "keskiviikkona",

            (DateRelativeWeekday::Thurdsday, DateRelativeLanguage::English) => "thursday",
            (DateRelativeWeekday::Thurdsday, DateRelativeLanguage::Finnish) => "torstaina",

            (DateRelativeWeekday::Friday, DateRelativeLanguage::English) => "friday",
            (DateRelativeWeekday::Friday, DateRelativeLanguage::Finnish) => "perjantaina",

            (DateRelativeWeekday::Saturday, DateRelativeLanguage::English) => "saturday",
            (DateRelativeWeekday::Saturday, DateRelativeLanguage::Finnish) => "lauantaina",

            (DateRelativeWeekday::Sunday, DateRelativeLanguage::English) => "sunday",
            (DateRelativeWeekday::Sunday, DateRelativeLanguage::Finnish) => "sunnuntaina",
        }
    }
}

/// "Natural language" date formats
#[derive(Debug, PartialEq)]
pub enum DateRelative {
    LastWeekday(DateRelativeLanguage, DateRelativeWeekday),
    Yesterday(DateRelativeLanguage),
    Today(DateRelativeLanguage),
    Tomorrow(DateRelativeLanguage),
    Overmorrow(DateRelativeLanguage),
    NextWeekday(DateRelativeLanguage, DateRelativeWeekday)
}
impl FromStr for DateRelative {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "yesterday" => Ok(Self::Yesterday(DateRelativeLanguage::English)),
            "eilen"     => Ok(Self::Yesterday(DateRelativeLanguage::Finnish)),

            "today" => Ok(Self::Today(DateRelativeLanguage::English)),
            "tänään" => Ok(Self::Today(DateRelativeLanguage::Finnish)),

            "tomorrow"  => Ok(Self::Tomorrow(DateRelativeLanguage::English)),
            "huomenna"  => Ok(Self::Tomorrow(DateRelativeLanguage::Finnish)),

            "overmorrow" | "day after tomorrow" => Ok(Self::Overmorrow(DateRelativeLanguage::English)),
            "ylihuomenna"                       => Ok(Self::Overmorrow(DateRelativeLanguage::Finnish)),

            _ => Err(())
        }
    }
}
impl FromMultiword for DateRelative {
    fn parse_multiword(words: &Vec<String>) -> Option<(Self, usize)> where Self: Sized {
        let check_sequence = |tokens: &[&'static str]| -> Option<()> {
            let mut iterator = words.iter().rev();
            let mut assume_next = |token: &'static str| -> Option<()> {
                let nxt = iterator.next()?;
                if nxt.as_str() == token.to_lowercase() {
                    return Some(());
                }
                None
            };
            for token in tokens.iter().rev() {
                assume_next(token)?;
            }
            Some(())
        };

        if check_sequence(&["day", "after", "tomorrow"]).is_some() {
            return Some((Self::Overmorrow(DateRelativeLanguage::English), 3));
        }

        for lang in DateRelativeLanguage::iter() {
            for weekday in DateRelativeWeekday::iter() {
                if check_sequence(&[lang.get_noun_next(), weekday.to_locale_static_str(lang)]).is_some() {
                    return Some((Self::NextWeekday(lang, weekday), 2));
                }
            }

            for weekday in DateRelativeWeekday::iter() {
                if check_sequence(&[lang.get_noun_prev(), weekday.to_locale_static_str(lang)]).is_some() {
                    return Some((Self::LastWeekday(lang, weekday), 2));
                }
            }
        }

        None
    }
}
impl AsDate for DateRelative {
    fn as_date(&self, now: Zoned) -> Result<Date, EventParseError> {
        match self {
            DateRelative::LastWeekday(_, weekday) => {
                let next_such_date = now.nth_weekday(-1, (*weekday).into()).map_err(|_e| EventParseError::AmbiguousTime)?;
                Ok(next_such_date.into())
            },
            DateRelative::Yesterday(_) => {
                let yesterday = now.checked_sub(1.day()).map_err(|_e| EventParseError::AmbiguousTime)?;
                Ok(yesterday.into())
            },
            DateRelative::Today(_) => {
                Ok(now.into())
            },
            DateRelative::Tomorrow(_) => {
                let tomorrow = now.checked_add(1.day()).map_err(|_e| EventParseError::AmbiguousTime)?;
                Ok(tomorrow.into())
            },
            DateRelative::Overmorrow(_) => {
                let overmorrow = now.checked_add(2.days()).map_err(|_e| EventParseError::AmbiguousTime)?;
                Ok(overmorrow.into())
            },
            DateRelative::NextWeekday(_, weekday) => {
                let next_such_date = now.nth_weekday(1, (*weekday).into()).map_err(|_e| EventParseError::AmbiguousTime)?;
                Ok(next_such_date.into())
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
    Ym(i8, i8)
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
        Ok(Self::Ym(month, date))
    }
}
impl AsDate for DateStructured {
    fn as_date(&self, now: Zoned) -> Result<Date, EventParseError> {
        match self {
            DateStructured::Ymd(year, month, day) => Ok(date(*year, *month, *day)),
            DateStructured::Ym(month, day) => {
                let current_year = now.year();
                let current_month = now.month();
                let current_day = now.day();
                if *month < current_month || *month == current_month && *day < current_day {
                    // That date has already passed this year, target next year instead
                    Ok(date(current_year + 1, *month, *day))
                } else {
                    Ok(date(current_year, *month, *day))
                }
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
    fn as_date(&self, now: Zoned) -> Result<Date, EventParseError> {
        match self {
            DateUnit::Structured(structured) => structured.as_date(now),
            DateUnit::Relative(relative) => relative.as_date(now),
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
///   - yesterday
///   - ("next"/"last") (weekday)
///   - (not implemented yet) ("next"/"last") (context event)
///   - (not implemented yet) (weekday/"day") ("after"/"before") (context event)
pub fn find_date(s: &str) -> Option<(DateUnit, usize, usize)> {
    let mut start = 0;
    let mut past_words = vec![];
    let mut past_words_start_positions = vec![];
    for word in s.split([' ', ',']) {
        let end = start + word.len();
        past_words.push(word.to_owned());
        past_words_start_positions.push(start);

        if let Some((unit, words_matched)) = DateRelative::parse_multiword(&past_words) {
            let start = past_words_start_positions[past_words_start_positions.len() - words_matched];
            return Some((DateUnit::Relative(unit), start, end));
        }
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
        assert_eq!(unit, DateUnit::Structured(DateStructured::Ym(11, 18)));
        assert_eq!(start, 16);
        assert_eq!(end, 22);
    }
    #[test]
    fn find_date_trivial_month_date_b() {
        let (unit, start, end) = find_date("Meet with Evelyn 1.12.").expect("parse failed");
        assert_eq!(unit, DateUnit::Structured(DateStructured::Ym(12, 1)));
        assert_eq!(start, 17);
        assert_eq!(end, 22);
    }
    #[test]
    fn find_date_trivial_month_date_c() {
        let (unit, start, end) = find_date("Meet with Evelyn 12.1.").expect("parse failed");
        assert_eq!(unit, DateUnit::Structured(DateStructured::Ym(1, 12)));
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
    fn find_date_relative_b() {
        let (unit, start, end) = find_date("John's birthday yesterday").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::Yesterday(DateRelativeLanguage::English)));
        assert_eq!(start, 16);
        assert_eq!(end, 25);
    }
    #[test]
    fn find_date_relative_overmorrow_a() {
        let (unit, start, end) = find_date("John's birthday overmorrow").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::Overmorrow(DateRelativeLanguage::English)));
        assert_eq!(start, 16);
        assert_eq!(end, 26);
    }
    #[test]
    fn find_date_relative_overmorrow_b() {
        let (unit, start, end) = find_date("John's birthday day after tomorrow").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::Overmorrow(DateRelativeLanguage::English)));
        assert_eq!(start, 16);
        assert_eq!(end, 34);
    }

    #[test]
    fn find_date_relative_weekday_a() {
        let (unit, start, end) = find_date("John's birthday next monday").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::NextWeekday(DateRelativeLanguage::English, DateRelativeWeekday::Monday)));
        assert_eq!(start, 16);
        assert_eq!(end, 27);
    }
    #[test]
    fn find_date_relative_weekday_b() {
        let (unit, start, end) = find_date("John's birthday next wednesday").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::NextWeekday(DateRelativeLanguage::English, DateRelativeWeekday::Wednesday)));
        assert_eq!(start, 16);
        assert_eq!(end, 30);
    }
    #[test]
    fn find_date_relative_weekday_c() {
        let (unit, start, end) = find_date("Marian synttärit ensi torstaina").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::NextWeekday(DateRelativeLanguage::Finnish, DateRelativeWeekday::Thurdsday)));
        assert_eq!(start, 18);
        assert_eq!(end, 32);
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
        let (unit, start, end) = find_date("John's birthday  yesterday ").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::Yesterday(DateRelativeLanguage::English)));
        assert_eq!(start, 17);
        assert_eq!(end, 26);
    }
    #[test]
    fn find_date_whitespace_d() {
        let (unit, start, end) = find_date(" John's  birthday   tomorrow ").expect("parse failed");
        assert_eq!(unit, DateUnit::Relative(DateRelative::Tomorrow(DateRelativeLanguage::English)));
        assert_eq!(start, 20);
        assert_eq!(end, 28);
    }
}
