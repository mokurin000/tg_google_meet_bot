use crate::calendar3;

use calendar3::chrono;
use chrono::{DateTime, Duration, Utc};
use chrono::{NaiveDate, NaiveTime};

pub fn parse_time(input: &str) -> Option<DateTime<Utc>> {
    let now = utc8_now();
    let mut input = input.trim().split_whitespace();
    let time = input.next().unwrap_or_default();
    let date = input.next().unwrap_or_default();

    let time = NaiveTime::parse_from_str(time.trim(), "%H:%M")
        .ok()
        .unwrap_or(now.time());
    let date = NaiveDate::parse_from_str(date, "%d/%m/%Y")
        .ok()
        .unwrap_or(now.date_naive());
    Some(date.and_time(time).and_utc())
}

pub fn utc8_now() -> DateTime<Utc> {
    Utc::now()
        .checked_add_signed(Duration::hours(8))
        .expect("time overflows")
}
