use std::str::FromStr;

use jiff::civil::Time;

use crate::EventParseError;

pub trait AsTime {
    fn as_time(&self) -> Result<Time, EventParseError>;
}

/// "Regularly formatted" time formats
#[derive(Debug, PartialEq)]
pub enum TimeStructured {
    /// Hours
    H(i8),
    /// Hours, Minutes
    Hm(i8, i8),
    /// Hours, Minutes, Seconds
    Hms(i8, i8, i8),
}
impl FromStr for TimeStructured {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut split_by_colon = string.split(':');
        let hours = split_by_colon.next().ok_or(())?.parse::<i8>().map_err(|_e| ())?;

        if let Some(minute_segment) = split_by_colon.next().filter(|s| !s.is_empty()) {
            let minutes = minute_segment.parse::<i8>().map_err(|_e| ())?;

            if let Some(second_segment) = split_by_colon.next().filter(|s| !s.is_empty()) {
                let seconds = second_segment.parse::<i8>().map_err(|_e| ())?;

                return Ok(Self::Hms(hours, minutes, seconds));
            };

            return Ok(Self::Hm(hours, minutes));
        };
        Ok(Self::H(hours))
    }
}
impl AsTime for TimeStructured {
    fn as_time(&self) -> Result<Time, EventParseError> {
        match self {
            TimeStructured::H(h) => Time::new(*h, 0, 0, 0).map_err(|_e| EventParseError::InvalidTime),
            TimeStructured::Hm(h, m) => Time::new(*h, *m, 0, 0).map_err(|_e| EventParseError::InvalidTime),
            TimeStructured::Hms(h, m, s) => Time::new(*h, *m, *s, 0).map_err(|_e| EventParseError::InvalidTime),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TimeUnit {
    Structured(TimeStructured),
}
impl AsTime for TimeUnit {
    fn as_time(&self) -> Result<Time, EventParseError> {
        match self {
            TimeUnit::Structured(structured) => structured.as_time(),
        }
    }
}

/// Tries to find a time from the supplied string.
/// The time can be expressed as
/// - a (H)H time: 12, 01, 8, ...
/// - a (H)H:(M)M time: 12:00, 01:30, 8:1, ...
/// - a (H)H:(M)M:(S)S time: 12:00:00, 01:30:1, 8:1:23, ...
pub fn find_time(s_after_date: &str) -> Option<(TimeUnit, usize, usize)> {
    let mut start: usize = 0;
    for c in s_after_date.chars() {
        match c {
            ' ' => start += 1,
            _ => break
        }
    }
    start = start.saturating_sub(1);
    for word in s_after_date.split([
        ' ',
        ',', // Might indicate that the next word is a location
        '@', // Might indicate that the next word is a location
        '-'  // Might indicate that the next word is a duration
    ]) {
        let end = start + word.len();
        if let Ok(unit) = word.parse::<TimeStructured>() {
            return Some((TimeUnit::Structured(unit), start, end));
        }

        start = end + 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_time_trivial_a() {
        let (unit, start, end) = find_time("18:11").expect("parse failed");
        assert_eq!(unit, TimeUnit::Structured(TimeStructured::Hm(18, 11)));
        assert_eq!(start, 0);
        assert_eq!(end, 5);
    }
    #[test]
    fn find_time_trivial_b() {
        let (unit, start, end) = find_time("3:03").expect("parse failed");
        assert_eq!(unit, TimeUnit::Structured(TimeStructured::Hm(3, 3)));
        assert_eq!(start, 0);
        assert_eq!(end, 4);
    }
    #[test]
    fn find_time_trivial_c() {
        let (unit, start, end) = find_time("0:1").expect("parse failed");
        assert_eq!(unit, TimeUnit::Structured(TimeStructured::Hm(0, 1)));
        assert_eq!(start, 0);
        assert_eq!(end, 3);
    }
    #[test]
    fn find_time_trivial_d() {
        let (unit, start, end) = find_time("18").expect("parse failed");
        assert_eq!(unit, TimeUnit::Structured(TimeStructured::H(18)));
        assert_eq!(start, 0);
        assert_eq!(end, 2);
    }

    #[test]
    fn find_time_whitespace_a() {
        let (unit, start, end) = find_time(" 4:01").expect("parse failed");
        assert_eq!(unit, TimeUnit::Structured(TimeStructured::Hm(4, 1)));
        assert_eq!(start, 1);
        assert_eq!(end, 5);
    }
    #[test]
    fn find_time_whitespace_b() {
        let (unit, start, end) = find_time(" 23:59  ").expect("parse failed");
        assert_eq!(unit, TimeUnit::Structured(TimeStructured::Hm(23, 59)));
        assert_eq!(start, 1);
        assert_eq!(end, 6);
    }

    #[test]
    fn find_time_junk_a() {
        let (unit, start, end) = find_time(" iaksjdk 13:30").expect("parse failed");
        assert_eq!(unit, TimeUnit::Structured(TimeStructured::Hm(13, 30)));
        assert_eq!(start, 9);
        assert_eq!(end, 14);
    }
    #[test]
    fn find_time_junk_b() {
        let (unit, start, end) = find_time("8:15 @ Annankatu 13").expect("parse failed");
        assert_eq!(unit, TimeUnit::Structured(TimeStructured::Hm(8, 15)));
        assert_eq!(start, 0);
        assert_eq!(end, 4);
    }
    #[test]
    fn find_time_junk_c() {
        let (unit, start, end) = find_time("ab123.23. 14:13 @ Taajamankatu 5").expect("parse failed");
        assert_eq!(unit, TimeUnit::Structured(TimeStructured::Hm(14, 13)));
        assert_eq!(start, 10);
        assert_eq!(end, 15);
    }
    #[test]
    fn find_time_junk_d() {
        let (unit, start, end) = find_time("ab123.23. 8 @ Taajamankatu 5").expect("parse failed");
        assert_eq!(unit, TimeUnit::Structured(TimeStructured::H(8)));
        assert_eq!(start, 10);
        assert_eq!(end, 11);
    }

    #[test]
    fn find_time_with_seconds_a() {
        let (unit, start, end) = find_time("19:59:00").expect("parse failed");
        assert_eq!(unit, TimeUnit::Structured(TimeStructured::Hms(19, 59, 0)));
        assert_eq!(start, 0);
        assert_eq!(end, 8);
    }
    #[test]
    fn find_time_with_seconds_b() {
        let (unit, start, end) = find_time("11:09:59").expect("parse failed");
        assert_eq!(unit, TimeUnit::Structured(TimeStructured::Hms(11, 9, 59)));
        assert_eq!(start, 0);
        assert_eq!(end, 8);
    }
    #[test]
    fn find_time_with_seconds_c() {
        let (unit, start, end) = find_time("8:0:1").expect("parse failed");
        assert_eq!(unit, TimeUnit::Structured(TimeStructured::Hms(8, 0, 1)));
        assert_eq!(start, 0);
        assert_eq!(end, 5);
    }
}
