use crate::calendar3;

use calendar3::chrono::{self, Duration, FixedOffset, TimeZone};
use chrono::{DateTime, Utc};
use chrono::{NaiveDate, NaiveTime};

/// input: UTC+8 time string
pub fn parse_time_to_utc<Tz: TimeZone>(input: &str, now: DateTime<Tz>) -> Option<DateTime<Utc>>
where
    Tz::Offset: Send,
{
    let mut input = input.trim().split_whitespace();
    let time = input.next().unwrap_or_default();
    let date = input.next().unwrap_or_default();

    let time = NaiveTime::parse_from_str(time.trim(), "%H:%M")
        .ok()
        .unwrap_or(now.time());
    let date = NaiveDate::parse_from_str(date, "%d/%m/%Y")
        .ok()
        .or(if time >= now.time() {
            // you can manually specify a past time for the meet
            Some(now.date_naive())
        } else {
            now.date_naive().succ_opt()
        })?;

    date.and_time(time)
        .and_utc()
        .checked_sub_signed(Duration::hours(8))
}

pub fn utc8_now() -> DateTime<FixedOffset> {
    FixedOffset::east_opt(8 * 3600)
        .unwrap()
        .from_utc_datetime(&Utc::now().naive_utc())
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn succ_day_on_previous_time() {
        assert_eq!(
            parse_time_to_utc(
                "12:00",
                DateTime::parse_from_rfc3339("2023-04-01T10:00:00+08:00").unwrap()
            )
            .map(|dt| dt.with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap())),
            Some(DateTime::parse_from_rfc3339("2023-04-01T12:00:00+08:00").unwrap())
        );
        assert_eq!(
            parse_time_to_utc(
                "8:00",
                DateTime::parse_from_rfc3339("2023-04-01T10:00:00+08:00").unwrap()
            )
            .map(|dt| dt.with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap())),
            Some(DateTime::parse_from_rfc3339("2023-04-02T08:00:00+08:00").unwrap())
        );
    }

    #[test]
    fn return_now_on_empty_time() {
        let now = utc8_now();
        assert_eq!(parse_time_to_utc("", now), Some(now.naive_utc().and_utc()))
    }
}
