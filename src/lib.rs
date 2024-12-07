pub mod temporal;

use std::str::FromStr;

use jiff::{civil::DateTime, Span};
use lazy_regex::regex;
use temporal::find_datetime;

#[derive(Debug, PartialEq)]
pub struct NewEvent {
    summary: String,
    time: DateTime,
    location: Option<String>,
    duration: Option<Span>
}


#[derive(Debug, PartialEq, thiserror::Error)]
pub enum EventParseError {
    #[error("Missing summary")]
    MissingSummary,
    #[error("Missing time")]
    MissingTime,
    #[error("Invalid time")]
    InvalidTime,
    #[error("Ambiguous time")]
    AmbiguousTime,
    #[error("Ambiguous duration")]
    AmbiguousDuration,
}
impl FromStr for NewEvent {
    type Err = EventParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut summary: Option<String> = None;
        let mut location: Option<String> = None;
        let (time, time_starts, time_ends) = find_datetime(s).ok_or(EventParseError::MissingTime)?;
        let (before_time, _) = s.split_at(time_starts);
        let (_, after_time) = s.split_at(time_ends);

        let before_time_trimmed = before_time.trim(); 
        if !before_time_trimmed.is_empty() {
            summary = Some(before_time_trimmed.to_owned());
        }

        let location_start_pattern = regex!(r"\s*[@ | ,]\s+.+");
        if location_start_pattern.is_match(after_time) {
            let trimmed_location = after_time.trim().trim_start_matches(['@', ',']).trim_start();
            location = Some(trimmed_location.to_owned());
        }

        Ok(Self {
            summary: summary.ok_or(EventParseError::MissingSummary)?,
            time,
            location,
            duration: None
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use jiff::{ToSpan, Zoned};

    #[test]
    fn fail_only_summary() {
        let event = "John's birthday".parse::<NewEvent>();
        assert_eq!(event, Err(EventParseError::MissingTime));
    }

    #[test]
    fn trivial_a() {
        let event = "John's birthday 18.11.".parse::<NewEvent>().unwrap();
        assert_eq!(event.summary, "John's birthday");
        assert_eq!(event.time.year(), Zoned::now().year());
        assert_eq!(event.time.day(), 18);
        assert_eq!(event.time.month(), 11);
        assert_eq!(event.time.hour(), 0);
        assert_eq!(event.location, None);
    }
    
    #[test]
    fn with_time_short() {
        let event = "John's birthday 18.11. 16".parse::<NewEvent>().unwrap();
        assert_eq!(event.summary, "John's birthday");
        assert_eq!(event.time.year(), Zoned::now().year());
        assert_eq!(event.time.day(), 18);
        assert_eq!(event.time.month(), 11);
        assert_eq!(event.time.hour(), 16);
        assert_eq!(event.time.minute(), 0);
        assert_eq!(event.location, None);
    }

    #[test]
    fn with_time_long_a() {
        let event = "John's birthday 18.11. 16:00".parse::<NewEvent>().unwrap();
        assert_eq!(event.summary, "John's birthday");
        assert_eq!(event.time.year(), Zoned::now().year());
        assert_eq!(event.time.day(), 18);
        assert_eq!(event.time.month(), 11);
        assert_eq!(event.time.hour(), 16);
        assert_eq!(event.time.minute(), 0);
        assert_eq!(event.location, None);
    }

    #[test]
    fn with_time_long_b() {
        let event = "John's birthday 18.11. 1:59".parse::<NewEvent>().unwrap();
        assert_eq!(event.summary, "John's birthday");
        assert_eq!(event.time.year(), Zoned::now().year());
        assert_eq!(event.time.day(), 18);
        assert_eq!(event.time.month(), 11);
        assert_eq!(event.time.hour(), 1);
        assert_eq!(event.time.minute(), 59);
        assert_eq!(event.location, None);
    }

    #[test]
    fn trivial_with_location_a() {
        let event = "John's birthday 18.11. @ Memory Plaza".parse::<NewEvent>().unwrap();
        assert_eq!(event.summary, "John's birthday");
        assert_eq!(event.time.year(), Zoned::now().year());
        assert_eq!(event.time.day(), 18);
        assert_eq!(event.time.month(), 11);
        assert_eq!(event.location, Some("Memory Plaza".to_owned()));
    }

    #[test]
    fn relative_a() {
        let event = "John's birthday tomorrow".parse::<NewEvent>().unwrap();
        assert_eq!(event.summary, "John's birthday");
        let tomorrow = Zoned::now().checked_add(1.day()).unwrap();
        assert_eq!(event.time.year(), tomorrow.year());
        assert_eq!(event.time.day(), tomorrow.day());
        assert_eq!(event.time.month(), tomorrow.month());
        assert_eq!(event.location, None);
    }

    #[test]
    fn relative_with_location_a() {
        let event = "John's birthday tomorrow @ Tuomiokirkko".parse::<NewEvent>().unwrap();
        assert_eq!(event.summary, "John's birthday");
        let tomorrow = Zoned::now().checked_add(1.day()).unwrap();
        assert_eq!(event.time.year(), tomorrow.year());
        assert_eq!(event.time.day(), tomorrow.day());
        assert_eq!(event.time.month(), tomorrow.month());
        assert_eq!(event.location, Some("Tuomiokirkko".to_owned()));
    }
    #[test]
    fn relative_with_location_b() {
        let event = "John's birthday tomorrow, Temppeliaukion Kirkko".parse::<NewEvent>().unwrap();
        assert_eq!(event.summary, "John's birthday");
        let tomorrow = Zoned::now().checked_add(1.day()).unwrap();
        assert_eq!(event.time.year(), tomorrow.year());
        assert_eq!(event.time.day(), tomorrow.day());
        assert_eq!(event.time.month(), tomorrow.month());
        assert_eq!(event.location, Some("Temppeliaukion Kirkko".to_owned()));
    }
}
