use jiff::{tz::TimeZone, civil::DateTime, Timestamp, Zoned};
use js_sys::Date;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::{EventParseError, NewEvent};


#[derive(Debug, Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct EventResult(Result<NewEvent, EventParseError>);


#[wasm_bindgen]
pub fn parse(string: String) -> EventResult {
    EventResult(string.parse())
}

#[wasm_bindgen]
pub fn parse_at_time(string: String, at: Date) -> EventResult {
    let millis = at.get_milliseconds();
    let now = Zoned::new(Timestamp::from_millisecond(millis as i64).expect("failed to construct Zoned from js Date"), TimeZone::UTC);
    EventResult(NewEvent::parse_at_time(&string, now))
}

#[derive(Debug, Clone, Copy, Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct DateTimeWrapper(DateTime);
#[wasm_bindgen]
pub fn to_datetime(event: NewEvent) -> DateTimeWrapper {
    DateTimeWrapper(event.datetime())
}

#[wasm_bindgen]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
