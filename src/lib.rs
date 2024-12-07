//! ## Natural Language Calendar Event Parser
//! ### About
//! A library for parsing strings such as "John's birthday 18.11." or "Meeting about new duck quotas tomorrow 11:00 @ A769" into a machine readable format.
//! 
//! Copyright (C) 2024 Elias Eskelinen <elias.eskelinen@pm.me>
//! 
//! This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//! 
//! This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//! 
//! You should have received a copy of the GNU Affero General Public License along with this program (LICENSE.txt). If not, see <https://www.gnu.org/licenses/>. 
//!
//! ### Usage
//! The main logic can be accessed by constructing a [NewEvent] from a string. For example:
//! ```rust
//! // Parse event
//! let event: nlcep::NewEvent = 
//!     "Meeting about Q3 duckling quotas tomorrow 11:00, A769"
//!     .parse()
//!     .expect("Parsing event failed");
//!
//!
//! // Basic details should be correct
//! assert_eq!(event.summary, "Meeting about Q3 duckling quotas");
//! assert_eq!(event.location, Some("A769".to_owned()));
//!
//! // Let's check that the meeting has been parsed as tomorrow 11:00!
//! use jiff::{ Zoned, ToSpan }; // nlcep uses jiff for storing dates
//! let tomorrow = Zoned::now().checked_add(1.day()).unwrap();
//!
//! assert_eq!(event.time.year(), tomorrow.year());
//! assert_eq!(event.time.day(), tomorrow.day());
//! assert_eq!(event.time.month(), tomorrow.month());
//! assert_eq!(event.time.hour(), 11);
//! assert_eq!(event.time.minute(), 0);
//! 
//! ```


pub(crate) mod temporal;

use std::str::FromStr;

use jiff::{civil::DateTime, Span};
use lazy_regex::regex;
use temporal::find_datetime;

/// Represents a parsed event
#[derive(Debug, PartialEq)]
pub struct NewEvent {
    /// Summary of the parsed event
    pub summary: String,
    /// When the event takes place, stored without a timezone (constructed from user input as-is)
    pub time: DateTime,
    /// Where the event takes place, not mandatory
    pub location: Option<String>,
    /// For how long the event goes on, not mandatory
    pub duration: Option<Span>
}


/// Contains all possible error variants that may occur while parsing a new event.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum EventParseError {
    /// No valid datetime could be parsed, other details might be valid.
    /// For example:
    /// ```rust
    /// use nlcep::{ NewEvent, EventParseError };
    /// let err = "Meet Saara @ Local Library".parse::<NewEvent>();
    /// assert_eq!(err, Err(EventParseError::MissingTime));
    /// ```
    #[error("Missing time")]
    MissingTime,
    /// Reserved for future use
    #[error("Invalid time")]
    InvalidTime,
    /// Reserved for future use
    #[error("Ambiguous time")]
    AmbiguousTime,
    /// The event contains a valid time, but a summary couldn't be found.
    /// For example:
    /// ```rust
    /// use nlcep::{ NewEvent, EventParseError };
    /// let err = "tomorrow 11:00".parse::<NewEvent>();
    /// assert_eq!(err, Err(EventParseError::MissingSummary));
    /// ```
    #[error("Missing summary")]
    MissingSummary,
    /// Reserved for future use
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
