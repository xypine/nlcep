use date::find_date;
use jiff::civil::DateTime;

pub mod date;
pub mod time;

use date::AsDate;
use time::{find_time, AsTime};

pub fn find_datetime(s: &str) -> Option<(DateTime, usize, usize)> {
    let (date, date_start, date_end) = find_date(s)?;
    let (_, s_after_date) = s.split_at(date_end);

    let date = date.as_date();
    let mut end = date_end;
    let dt = if let Some((time, _time_start, time_end)) = find_time(s_after_date) {
        end = time_end;
        date.to_datetime(time.as_time())
    } else {
        date.into()
    };
    Some((dt, date_start, end))
}
