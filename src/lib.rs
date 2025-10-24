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
//! The main logic can be accessed by constructing a [`NewEvent`] from a string. For example:
//! ```rust
//! # use jiff::{ Zoned, ToSpan }; // nlcep uses jiff for storing dates
//! # let tomorrow = Zoned::now().checked_add(1.day()).unwrap();
//! #
//! // Parse event
//! let event: nlcep::NewEvent =
//!     "Meeting about Q3 duckling quotas tomorrow 11:00, A769"
//!     .parse()
//!     .expect("Parsing event failed");
//!
//! // Check that the details have been parsed correctly :)
//! assert_eq!(event.summary, "Meeting about Q3 duckling quotas");
//! assert_eq!(event.location, Some("A769".to_owned()));
//! assert_eq!(event.date.year(), tomorrow.year());
//! assert_eq!(event.date.day(), tomorrow.day());
//! assert_eq!(event.date.month(), tomorrow.month());
//! let time = event.time.unwrap();
//! assert_eq!(time.hour(), 11);
//! assert_eq!(time.minute(), 0);
//! ```
//!
//! You can also use [`NewEvent::parse_at_time`] to supply an alternative basis for relative time
//! formats such as "tomorrow" or "yesterday":
//! ```rust
//! use jiff::{ Timestamp, Zoned };
//! let now: Zoned = "2024-07-11T13:14:00Z"
//!     .parse::<Timestamp>().expect("Invalid timestamp")
//!     .in_tz("America/New_York").expect("Invalid timezone");
//! // Parse event with an alternative "now"
//! let event =
//!     nlcep::NewEvent::parse_at_time("water the plants tomorrow", now)
//!     .expect("Parsing event failed");
//! assert_eq!(event.date.year(), 2024);
//! assert_eq!(event.date.month(), 7);
//! assert_eq!(event.date.day(), 11 + 1);
//! ```
#![deny(unsafe_code)]
#![warn(
    clippy::cognitive_complexity,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_link_with_quotes,
    clippy::doc_markdown,
    clippy::empty_line_after_outer_attr,
    clippy::empty_structs_with_brackets,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::float_equality_without_abs,
    keyword_idents,
    clippy::missing_const_for_fn,
    missing_copy_implementations,
    missing_debug_implementations,
    clippy::missing_docs_in_private_items,
    clippy::missing_panics_doc,
    non_ascii_idents,
    noop_method_call,
    clippy::option_if_let_else,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::semicolon_if_nothing_returned,
    clippy::unseparated_literal_suffix,
    clippy::shadow_unrelated,
    clippy::similar_names,
    clippy::suspicious_operation_groupings,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    clippy::unused_self,
    clippy::use_debug,
    clippy::used_underscore_binding,
    clippy::useless_let_if_seq,
    clippy::wildcard_dependencies,
    clippy::wildcard_imports
)]

pub(crate) mod temporal;
pub use temporal::find_datetime;

#[cfg(feature = "wasm")]
pub mod wasm;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use std::str::FromStr;

use jiff::{
    civil::{Date, DateTime, Time},
    Span, Zoned,
};
use lazy_regex::regex;
use serde::{Deserialize, Serialize};

use crate::temporal::DateTimeMatch;

/// Represents a parsed event
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct NewEvent {
    /// Summary of the parsed event
    pub summary: String,
    pub date: Date,
    pub time: Option<Time>,
    /// Where the event takes place, not mandatory
    pub location: Option<String>,
    /// For how long the event goes on, not mandatory
    pub duration: Option<Span>,
}

impl PartialEq for NewEvent {
    fn eq(&self, other: &Self) -> bool {
        let duration_same = match (self.duration, other.duration) {
            (None, None) => true,
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (Some(a), Some(b)) => a
                .compare(b)
                .map(|ord| matches!(ord, std::cmp::Ordering::Equal))
                .unwrap_or(false),
        };
        self.summary == other.summary
            && self.date == other.date
            && self.time == other.time
            && self.location == other.location
            && duration_same
    }
}

impl NewEvent {
    pub fn parse_at_time(s: &str, now: Zoned) -> Result<Self, EventParseError> {
        let mut summary: Option<String> = None;
        let mut location: Option<String> = None;
        let DateTimeMatch {
            date,
            time,
            start_char: time_starts,
            end_char: time_ends,
        } = find_datetime(s, now, false)?.ok_or(EventParseError::MissingTime)?;
        let (before_time, _) = s.split_at(time_starts);
        let (_, after_time) = s.split_at(time_ends);

        let before_time_trimmed = before_time.trim();
        if !before_time_trimmed.is_empty() {
            summary = Some(before_time_trimmed.to_owned());
        }

        let location_start_pattern = regex!(r"\s*[@ | ,]\s+.+");
        if location_start_pattern.is_match(after_time) {
            let trimmed_location = after_time
                .trim()
                .trim_start_matches(['@', ','])
                .trim_start();
            location = Some(trimmed_location.to_owned());
        }

        Ok(Self {
            summary: summary.ok_or(EventParseError::MissingSummary)?,
            date,
            time,
            location,
            duration: None,
        })
    }

    pub fn datetime(&self) -> DateTime {
        self.time
            .map_or_else(|| self.date.into(), |time| self.date.to_datetime(time))
    }
}

/// Contains all possible error variants that may occur while parsing a new event.
#[derive(Debug, PartialEq, Clone, Copy, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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
        let now = Zoned::now();
        Self::parse_at_time(s, now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use jiff::civil::date;

    #[test]
    fn fail_only_summary() {
        let event = "John's birthday".parse::<NewEvent>();
        assert_eq!(event, Err(EventParseError::MissingTime));
    }

    #[test]
    fn trivial_a() {
        let now = date(2024, 6, 1).in_tz("UTC").unwrap();
        let event = NewEvent::parse_at_time("John's birthday 18.11.", now).unwrap();
        assert_eq!(event.summary, "John's birthday");
        assert_eq!(event.datetime().year(), 2024);
        assert_eq!(event.datetime().day(), 18);
        assert_eq!(event.datetime().month(), 11);
        assert_eq!(event.datetime().hour(), 0);
        assert_eq!(event.location, None);
    }

    #[test]
    fn with_time_short() {
        let now = date(2024, 6, 1).in_tz("UTC").unwrap();
        let event = NewEvent::parse_at_time("John's birthday 18.11. 16", now).unwrap();
        assert_eq!(event.summary, "John's birthday");
        assert_eq!(event.datetime().year(), 2024);
        assert_eq!(event.datetime().day(), 18);
        assert_eq!(event.datetime().month(), 11);
        assert_eq!(event.datetime().hour(), 16);
        assert_eq!(event.datetime().minute(), 0);
        assert_eq!(event.location, None);
    }

    #[test]
    fn with_time_long_a() {
        let now = date(2024, 6, 1).in_tz("UTC").unwrap();
        let event = NewEvent::parse_at_time("John's birthday 18.11. 16:00", now).unwrap();
        assert_eq!(event.summary, "John's birthday");
        assert_eq!(event.datetime().year(), 2024);
        assert_eq!(event.datetime().day(), 18);
        assert_eq!(event.datetime().month(), 11);
        assert_eq!(event.datetime().hour(), 16);
        assert_eq!(event.datetime().minute(), 0);
        assert_eq!(event.location, None);
    }

    #[test]
    fn with_time_long_b() {
        let now = date(2024, 6, 1).in_tz("UTC").unwrap();
        let event = NewEvent::parse_at_time("John's birthday 18.11. 1:59", now).unwrap();
        assert_eq!(event.summary, "John's birthday");
        assert_eq!(event.datetime().year(), 2024);
        assert_eq!(event.datetime().day(), 18);
        assert_eq!(event.datetime().month(), 11);
        assert_eq!(event.datetime().hour(), 1);
        assert_eq!(event.datetime().minute(), 59);
        assert_eq!(event.location, None);
    }

    #[test]
    fn trivial_with_location_a() {
        let now = date(2024, 6, 1).in_tz("UTC").unwrap();
        let event = NewEvent::parse_at_time("John's birthday 18.11. @ Memory Plaza", now).unwrap();
        assert_eq!(event.summary, "John's birthday");
        assert_eq!(event.date.year(), 2024);
        assert_eq!(event.date.day(), 18);
        assert_eq!(event.date.month(), 11);
        assert_eq!(event.location, Some("Memory Plaza".to_owned()));
    }

    #[test]
    fn relative_a() {
        let now = date(2024, 6, 1).in_tz("UTC").unwrap();
        let event = NewEvent::parse_at_time("John's birthday tomorrow", now).unwrap();
        assert_eq!(event.summary, "John's birthday");
        assert_eq!(event.date.year(), 2024);
        assert_eq!(event.date.month(), 6);
        assert_eq!(event.date.day(), 2);
        assert_eq!(event.location, None);
    }

    #[test]
    fn relative_with_location_a() {
        let now = date(2024, 6, 1).in_tz("UTC").unwrap();
        let event =
            NewEvent::parse_at_time("John's birthday tomorrow @ Tuomiokirkko", now).unwrap();
        assert_eq!(event.summary, "John's birthday");
        assert_eq!(event.date.year(), 2024);
        assert_eq!(event.date.month(), 6);
        assert_eq!(event.date.day(), 2);
        assert_eq!(event.location, Some("Tuomiokirkko".to_owned()));
    }
    #[test]
    fn relative_with_location_b() {
        let now = date(2024, 6, 1).in_tz("UTC").unwrap();
        let event = NewEvent::parse_at_time("John's birthday tomorrow, Temppeliaukion Kirkko", now)
            .unwrap();
        assert_eq!(event.summary, "John's birthday");
        assert_eq!(event.date.year(), 2024);
        assert_eq!(event.date.month(), 6);
        assert_eq!(event.date.day(), 2);
        assert_eq!(event.location, Some("Temppeliaukion Kirkko".to_owned()));
    }
}
