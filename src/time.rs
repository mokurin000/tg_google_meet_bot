use crate::calendar3;

use calendar3::chrono::{self, Duration, FixedOffset, ParseError, TimeZone};
use chrono::{DateTime, Utc};
use chrono::{NaiveDate, NaiveTime};

#[derive(Debug, thiserror::Error)]
pub enum TimeParseError {
    #[error("{0}")]
    DurationError(#[from] duration_str::DError),
    #[error("{0}")]
    TimeError(#[from] ParseError),
    #[error("Duration must be non-negative!")]
    NegativeDuration,
}

pub fn parse_time_to_utc<Tz: TimeZone>(
    utc8_time: &str,
    duration: Option<&str>,
    now: DateTime<Tz>,
) -> Result<(DateTime<Utc>, DateTime<Utc>), TimeParseError>
where
    Tz::Offset: Send,
{
    let mut input = utc8_time.split_whitespace();
    let time = input.next().map_or(Ok(now.time()), |time| {
        NaiveTime::parse_from_str(time.trim(), "%H:%M")
    })?;
    let date = input.next().map_or_else(
        || {
            Ok(if time >= now.time() {
                // you can manually specify a past time for the meet
                now.date_naive()
            } else {
                now.date_naive().succ_opt().unwrap()
            })
        },
        |date| NaiveDate::parse_from_str(date, "%d/%m/%Y"),
    )?;

    let duration = match duration {
        Some(duration) => duration_str::parse_chrono(duration)?,
        None => Duration::hours(0),
    };
    if duration < Duration::hours(0) {
        return Err(TimeParseError::NegativeDuration);
    }

    let start_time = date
        .and_time(time)
        .and_utc()
        .checked_sub_signed(Duration::hours(8))
        .unwrap();
    let end_time = start_time + duration;
    Ok((start_time, end_time))
}

pub fn utc8_now() -> DateTime<FixedOffset> {
    FixedOffset::east_opt(8 * 3600)
        .unwrap()
        .from_utc_datetime(&Utc::now().naive_utc())
}

#[cfg(test)]
mod tests {
    use calendar3::chrono::format::ParseErrorKind;

    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn succ_day_on_previous_time() {
        assert_eq!(
            parse_time_to_utc(
                "12:00",
                None,
                DateTime::parse_from_rfc3339("2023-04-01T10:00:00+08:00").unwrap()
            )
            .map(|(dt, _)| dt.with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap()))
            .unwrap(),
            DateTime::parse_from_rfc3339("2023-04-01T12:00:00+08:00").unwrap()
        );
        assert_eq!(
            parse_time_to_utc(
                "8:00",
                None,
                DateTime::parse_from_rfc3339("2023-04-01T10:00:00+08:00").unwrap()
            )
            .map(|(dt, _)| dt.with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap()))
            .unwrap(),
            DateTime::parse_from_rfc3339("2023-04-02T08:00:00+08:00").unwrap()
        );
    }

    #[test]
    fn return_now_on_empty_time() {
        let now = utc8_now();
        assert_eq!(
            parse_time_to_utc("", None, now).unwrap().0,
            now.naive_utc().and_utc()
        )
    }

    #[test]
    fn test_parse_date() {
        let now = utc8_now();
        assert_eq!(
            parse_time_to_utc("08:00 01/06/2023", None, now)
                .map(|(dt, _)| dt.with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap()))
                .unwrap(),
            DateTime::parse_from_rfc3339("2023-06-01T08:00:00+08:00").unwrap()
        )
    }

    #[test]
    fn test_ill_formed_time() {
        let now = utc8_now();
        assert_eq!(
            parse_time_to_utc("05:12 1/20/1111", None, now)
                .map_err(|e| {
                    let TimeParseError::TimeError(e) = e else {unreachable!()};
                    e.kind()
                })
                .unwrap_err(),
            ParseErrorKind::OutOfRange
        )
    }

    #[test]
    fn test_duration() {
        let now = DateTime::parse_from_rfc3339("2023-06-01T08:00:00Z")
            .unwrap();
        let duration = "2h";

        let (start, end) = parse_time_to_utc("16:00 01/06/2023", Some(duration), now).unwrap();
        let utc8 = FixedOffset::east_opt(8*3600).unwrap();
        let start = start.with_timezone(&utc8);
        let end = end.with_timezone(&utc8);
        assert_eq!(
            (start, end),
            (now, now + Duration::hours(2))
        )
    }
}
